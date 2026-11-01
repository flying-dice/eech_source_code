/*
 * The campaign's half of host-owned terrain (EEDCS-47, D3): see mb_hostterrain.h.  This file holds no terrain arithmetic and no terrain geometry.  It
 * implements the public queries of the 3d/terrain module (the ones terrelev.c and the get function of terrsel.c implement in the original game) by
 * carrying each one to the host and keeping, in the caller's record, what the host answered.  The order of the campaign's queries, and which record each
 * one reads and writes (a shared record when the caller passes none), are exactly the original's: only the computation is the host's.
 */

#include "project.h"

#include <math.h>
#include <string.h>

#include "mb_hostterrain.h"

#ifdef EECH_CAMPAIGN_HOST_TERRAIN

static campaign_terrain_host current_host;
static int host_installed;

/* the record of a query that passes none, as the original kept one for all of them */
static terrain_3d_point_data
	system_point_data;

void campaign_terrain_set_host (const campaign_terrain_host *host)
{
	if (host && host->terrain)
	{
		current_host = *host;
		host_installed = TRUE;
	}
}

// the season the terrain was loaded in: the global season is read where the original read it (terrdata.c, when it chose the sector and colour files), which
// is before the first question of any kind, and kept: the campaign file's own season tag is parsed later and must not change which map the host serves
static int season_captured;
static int season_winter;

static void ask (campaign_terrain_request *request, campaign_terrain_reply *reply, const char *what)
{
	char
		error[CAMPAIGN_TERRAIN_ERROR_MAX + 1];

	if (!host_installed)
	{
		debug_fatal ("host terrain: [terrain-provider:missing] no terrain provider is installed (%s)", what);
	}

	if (!season_captured)
	{
		season_winter = (get_global_season () == SESSION_SEASON_WINTER);
		season_captured = TRUE;
	}

	request->winter = season_winter;

	memset (reply, 0, sizeof (*reply));

	memset (error, 0, sizeof (error));

	if (current_host.terrain (request, reply, error, CAMPAIGN_TERRAIN_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host terrain: %s", error[0] ? error : "[terrain-provider:point.callback] the host terrain provider failed without a message");
	}
}

static int all_finite (const float *values, int count)
{
	int i;

	for (i = 0; i < count; i++)
	{
		if (!isfinite (values[i]))
		{
			return FALSE;
		}
	}

	return TRUE;
}

////////////////////////////////////////
//
// the map facts
//
////////////////////////////////////////

void campaign_terrain_install_map_facts (void)
{
	campaign_terrain_request
		request;

	campaign_terrain_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_TERRAIN_MAP;

	ask (&request, &reply, "map facts");

	if ((reply.sector_x_max < 1) || (reply.sector_z_max < 1) || (reply.fast_width < 0) || (reply.fast_height < 0) ||
		!all_finite (&reply.min_x, 4) || !all_finite (&reply.min_height, 3) || !(reply.max_x > reply.min_x) || !(reply.max_z > reply.min_z) ||
		!(reply.max_height >= reply.min_height))
	{
		debug_fatal ("host terrain: [terrain-provider:map.facts] the host's map facts are malformed");
	}

	// the sector-level users that are still the campaign's own (trees, per-sector surface types, ground impact: audit groups G2 and G3) read the terrain the
	// campaign loaded, so it must be the host's map
	if ((reply.sector_x_max != terrain_3d_sector_x_max) || (reply.sector_z_max != terrain_3d_sector_z_max))
	{
		debug_fatal ("host terrain: [terrain-provider:map.facts] the host's map is %d by %d sectors but the terrain the campaign loaded is %d by %d", reply.sector_x_max, reply.sector_z_max,
			terrain_3d_sector_x_max, terrain_3d_sector_z_max);
	}

	// the extents and heights the core reads are the host's
	terrain_3d_min_map_x = reply.min_x;
	terrain_3d_min_map_z = reply.min_z;
	terrain_3d_max_map_x = reply.max_x;
	terrain_3d_max_map_z = reply.max_z;
	terrain_3d_map_minimum_height = reply.min_height;
	terrain_3d_map_maximum_height = reply.max_height;
	terrain_3d_map_scaled_height_difference = reply.scaled_height_difference;
}

////////////////////////////////////////
//
// the point surface query and the record
//
////////////////////////////////////////

void initialise_3d_terrain_elevation (void)
{
	// the original forgets the shared record's face hint when a map is loaded
	system_point_data.hint[0] = '\0';
}

#if ( TERRAIN_CALL_SITES )
float get_3d_terrain_point_data_ex ( float x, float z, terrain_3d_point_data *point_data, const char *file, int line )
#else
float get_3d_terrain_point_data ( float x, float z, terrain_3d_point_data *point_data )
#endif
{
	campaign_terrain_request
		request;

	campaign_terrain_reply
		reply;

#ifdef EECH_TERRAIN_FIXTURE
	int
		hint_null = ( point_data == NULL );
#endif

	if ( ( x != x ) || ( z != z ) )
	{
		// the original's build patch N1: a NaN position is reported with its callers; the host answers it as the map corner
		extern void eech_report_nan_position (const char *what, void *caller0, void *caller1, void *caller2);

		eech_report_nan_position ("terrain elevation", __builtin_return_address (0), NULL, NULL);
	}

	if ( !point_data )
	{
		point_data = &system_point_data;
	}

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_TERRAIN_POINT;
	request.x = x;
	request.z = z;

	strncpy (request.hint, point_data->hint, CAMPAIGN_TERRAIN_HINT_MAX);

	ask (&request, &reply, "point");

	if (!all_finite (&reply.elevation, 2) || !all_finite (reply.normal, 3) || (reply.type < 0) || (reply.type >= TERRAIN_TYPE_LAST) ||
		((reply.stored != FALSE) && (reply.stored != TRUE)) || (memchr (reply.hint, '\0', sizeof (reply.hint)) == NULL))
	{
		debug_fatal ("host terrain: [terrain-provider:point.answer] the host's point answer is malformed");
	}

	point_data->last_elevation = reply.last_elevation;
	point_data->face_normal.x = reply.normal[0];
	point_data->face_normal.y = reply.normal[1];
	point_data->face_normal.z = reply.normal[2];
	point_data->terrain_type = reply.type;

	// an answer without a hint leaves the record without one, as a query that found no face did; the hint is the host's own business and says nothing about
	// whether a surface is stored: the host says that, by value, and the record remembers that a face was once stored (until it is cleared) as the original's
	// stale face pointer did
	strcpy (point_data->hint, reply.hint);

	if (reply.stored)
	{
		point_data->face_stored = TRUE;
	}

#ifdef EECH_TERRAIN_FIXTURE
	// the client's view of the query: what the host's kernel knows beyond its answer (the hint attempt, a fast hit, no face found) is not visible here
	eech_terrain_record ( file, line, x, z, hint_null, request.hint[0] != '\0', FALSE, TRUE, FALSE, FALSE, reply.elevation, point_data );
#endif

	return ( reply.elevation );
}

int get_3d_terrain_point_data_has_face ( terrain_3d_point_data *point_data )
{
	if ( !point_data )
	{
		point_data = &system_point_data;
	}

	return ( point_data->face_stored );
}

vec3d *get_3d_terrain_point_data_normal ( terrain_3d_point_data *point_data )
{
	if ( !point_data )
	{
		point_data = &system_point_data;
	}

	return ( &point_data->face_normal );
}

float get_3d_terrain_point_data_elevation ( terrain_3d_point_data *point_data )
{
	if ( !point_data )
	{
		point_data = &system_point_data;
	}

	return ( point_data->last_elevation );
}

terrain_types get_3d_terrain_point_data_type ( terrain_3d_point_data *point_data )
{
	if ( !point_data )
	{
		point_data = &system_point_data;
	}

	return ( point_data->terrain_type );
}

void clear_3d_terrain_point_data ( terrain_3d_point_data *point_data )
{
	if ( !point_data )
	{
		point_data = &system_point_data;
	}

	point_data->hint[0] = '\0';
	point_data->face_stored = FALSE;
	point_data->terrain_type = TERRAIN_TYPE_INVALID;
	point_data->face_normal.x = 0;
	point_data->face_normal.y = 0;
	point_data->face_normal.z = 0;
}

float get_3d_terrain_elevation_and_face_normal ( float x, float z, vec3d *normal, terrain_3d_point_data *point_data )
{
	float
		height;

	ASSERT( ( x > terrain_3d_min_map_x ) && ( x < terrain_3d_max_map_x ) && ( z > terrain_3d_min_map_z ) && ( z < terrain_3d_max_map_z ) );

	height = get_3d_terrain_point_data ( x, z, NULL );

	if ( normal )
	{
		get_3d_terrain_last_normal ( normal );
	}

	return ( height );
}

void get_3d_terrain_face_normal ( vec3d *normal, float x, float z )
{
	ASSERT( ( x > terrain_3d_min_map_x ) && ( x < terrain_3d_max_map_x ) && ( z > terrain_3d_min_map_z ) && ( z < terrain_3d_max_map_z ) );

	get_3d_terrain_point_data ( x, z, NULL );

	get_3d_terrain_last_normal ( normal );
}

void get_3d_terrain_last_normal ( vec3d *normal )
{
	*normal = system_point_data.face_normal;
}

// the original's debugging aids of the solver: nothing of the solver is compiled here, so there is nothing to report or to soak
void terrain_elevation_soak_test ( int samples_per_sector_side )
{
}

void display_terrain_elevation_statistics ( void )
{
}

////////////////////////////////////////
//
// the ground-contact queries (EEDCS-48, D3 G2)
//
// point_above_ground and point_below_ground (en_world.h) and get_ground_impact_point (en_cllsn.c) are, with the host's terrain, physical questions: the host
// answers them with the original's expressions, short circuits and bisection (see mb_hostterrain.h).  Nothing here compares an elevation, reads a sector maximum or
// bisects.  What is here is the campaign's half: the world-map area it supplies, the shared record the original's queries wrote (the one a NULL point record means),
// the caller's buffers and their aliasing, and the terrain type -> class mapping.  A reply is checked completely before anything is written.
//
////////////////////////////////////////

static void ground_request (campaign_terrain_request *request, int op)
{
	memset (request, 0, sizeof (*request));

	request->op = op;

	// the campaign's world-map area (point_inside_map_area), as it is when the question is asked
	request->world[0] = MIN_MAP_X;
	request->world[1] = MIN_MAP_Z;
	request->world[2] = MAX_MAP_X;
	request->world[3] = MAX_MAP_Z;

	// the shared record's hint: the host looks at it only if it reads the terrain
	strncpy (request->hint, system_point_data.hint, CAMPAIGN_TERRAIN_HINT_MAX);
}

static int ground_record_valid (const campaign_terrain_reply *reply)
{
	return all_finite (&reply->last_elevation, 1) && all_finite (reply->normal, 3) && (reply->type >= 0) && (reply->type < TERRAIN_TYPE_LAST) &&
		((reply->stored == FALSE) || (reply->stored == TRUE)) && (memchr (reply->hint, '\0', sizeof (reply->hint)) != NULL);
}

// what the original's terrain reads left in the shared record: all of it, as a point query leaves it
static void ground_record_commit (const campaign_terrain_reply *reply)
{
	system_point_data.last_elevation = reply->last_elevation;
	system_point_data.face_normal.x = reply->normal[0];
	system_point_data.face_normal.y = reply->normal[1];
	system_point_data.face_normal.z = reply->normal[2];
	system_point_data.terrain_type = reply->type;

	strcpy (system_point_data.hint, reply->hint);

	if (reply->stored)
	{
		system_point_data.face_stored = TRUE;
	}
}

static int ground_predicate (int op, const vec3d *position, const char *what)
{
	campaign_terrain_request
		request;

	campaign_terrain_reply
		reply;

	ASSERT (position);

	ground_request (&request, op);

	request.x = position->x;
	request.y = position->y;
	request.z = position->z;

	ask (&request, &reply, what);

	if (((reply.answer != FALSE) && (reply.answer != TRUE)) || ((reply.queried != FALSE) && (reply.queried != TRUE)) || (reply.queried && !ground_record_valid (&reply)))
	{
		debug_fatal ("host terrain: [terrain-provider:ground.answer] the host's %s answer is malformed", what);
	}

	// an answer that read no terrain leaves the record exactly as it was, as the original's short circuits did
	if (reply.queried)
	{
		ground_record_commit (&reply);
	}

	return reply.answer;
}

int campaign_terrain_ground_above (const vec3d *position)
{
	return ground_predicate (CAMPAIGN_TERRAIN_GROUND_ABOVE, position, "ground above");
}

int campaign_terrain_ground_below (const vec3d *position)
{
	return ground_predicate (CAMPAIGN_TERRAIN_GROUND_BELOW, position, "ground below");
}

terrain_classes campaign_terrain_ground_impact (vec3d *above_ground, vec3d *below_ground, vec3d *ground_impact_point)
{
	campaign_terrain_request
		request;

	campaign_terrain_reply
		reply;

	ASSERT (above_ground);

	ASSERT (below_ground);

	ASSERT (ground_impact_point);

	// the endpoints cross by value, so the original's reads of one through the other cannot be reproduced: the callers pass distinct endpoints (the impact may alias the
	// below endpoint, which is a write)
	if (above_ground == below_ground)
	{
		debug_fatal ("host terrain: the ground impact endpoints are one buffer");
	}

	ground_request (&request, CAMPAIGN_TERRAIN_GROUND_IMPACT);

	request.x = above_ground->x;
	request.y = above_ground->y;
	request.z = above_ground->z;
	request.below[0] = below_ground->x;
	request.below[1] = below_ground->y;
	request.below[2] = below_ground->z;

	ask (&request, &reply, "ground impact");

	if (!all_finite (reply.above, 3) || !all_finite (reply.below, 3) || !all_finite (reply.impact, 3) || !ground_record_valid (&reply))
	{
		debug_fatal ("host terrain: [terrain-provider:ground.answer] the host's ground impact answer is malformed");
	}

	// the writes, in the original's order: the corrected endpoints, the record, then the impact point last (it may be the below endpoint's buffer)
	above_ground->x = reply.above[0];
	above_ground->y = reply.above[1];
	above_ground->z = reply.above[2];

	below_ground->x = reply.below[0];
	below_ground->y = reply.below[1];
	below_ground->z = reply.below[2];

	ground_record_commit (&reply);

	ground_impact_point->x = reply.impact[0];
	ground_impact_point->y = reply.impact[1];
	ground_impact_point->z = reply.impact[2];

	return (get_terrain_type_class (get_3d_terrain_point_data_type (NULL)));
}

#ifdef EECH_TERRAIN_FIXTURE

terrain_3d_point_data *eech_terrain_system_point_data ( void )
{
	return ( &system_point_data );
}

#endif

////////////////////////////////////////
//
// the fast gridpost elevation
//
////////////////////////////////////////

float get_3d_terrain_fast_elevation ( float x, float z )
{
	campaign_terrain_request
		request;

	campaign_terrain_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_TERRAIN_FAST;
	request.x = x;
	request.z = z;

	ask (&request, &reply, "fast elevation");

	if (!isfinite (reply.elevation))
	{
		debug_fatal ("host terrain: [terrain-provider:fast.elevation] the host's fast elevation answer is not a finite number");
	}

	return ( reply.elevation );
}

#endif // EECH_CAMPAIGN_HOST_TERRAIN

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

static void ask (const campaign_terrain_request *request, campaign_terrain_reply *reply, const char *what)
{
	char
		error[CAMPAIGN_TERRAIN_ERROR_MAX + 1];

	if (!host_installed)
	{
		debug_fatal ("host terrain: [terrain-provider:missing] no terrain provider is installed (%s)", what);
	}

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
		(memchr (reply.hint, '\0', sizeof (reply.hint)) == NULL))
	{
		debug_fatal ("host terrain: [terrain-provider:point.answer] the host's point answer is malformed");
	}

	point_data->last_elevation = reply.last_elevation;
	point_data->face_normal.x = reply.normal[0];
	point_data->face_normal.y = reply.normal[1];
	point_data->face_normal.z = reply.normal[2];
	point_data->terrain_type = reply.type;

	// an answer without a hint leaves the record without one, as a query that found no face did; face_stored remembers that a face was once stored
	strcpy (point_data->hint, reply.hint);

	if (reply.hint[0])
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

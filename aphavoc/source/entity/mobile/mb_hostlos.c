/*
 * The campaign's half of host-owned line of sight (EEDCS-42): see mb_hostlos.h.  This file holds no line-of-sight arithmetic.  It
 * carries a query to the host, answers the host's environmental questions from the campaign's own terrain and collision services in
 * the order the host asks them, and returns the host's visibility.  It validates every host answer.
 */

#include "project.h"

#include <math.h>
#include <string.h>

#include "mb_hostlos.h"

#ifdef EECH_LOS_FIXTURE

//
// TEST-ONLY characterisation hooks (feature `los-fixture`, never in the shipping module): the same lookup transcript the
// original solver reported, taken here at the boundary where the campaign answers the host, and the same query record.
//

extern void eech_los_lookup_terrain (int phase, vec3d *point, float elevation);

extern void eech_los_lookup_objects (int phase, vec3d *start, vec3d *end, int hit, vec3d *collision_point);

extern void eech_los_trace_record (entity *source, entity *target, vec3d *source_position, vec3d *target_position, mobile_los_check_criteria criteria, int result);

#endif

// a host that answers every question never needs more than about range / 200 + 16 requests, so this is far above any real query
#define CAMPAIGN_LOS_MAX_REQUESTS 65536

static campaign_los_host current_host;
static int query_open;

void campaign_los_set_host (const campaign_los_host *host)
{
	if (host)
	{
		current_host = *host;
	}
	else
	{
		memset (&current_host, 0, sizeof (current_host));
	}
	query_open = FALSE;
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

static void fill_host_id (char *out, entity *en)
{
	const char *host_id = campaign_roster_host_id (en);

	out[0] = '\0';

	if (host_id && host_id[0])
	{
		if (strlen (host_id) > CAMPAIGN_LOS_HOST_ID_MAX)
		{
			debug_fatal ("host line of sight: entity %d has a host identity longer than %d bytes", get_local_entity_index (en), CAMPAIGN_LOS_HOST_ID_MAX);
		}

		memcpy (out, host_id, strlen (host_id) + 1);
	}
}

static campaign_los_result ask_host (const campaign_los_step *step)
{
	campaign_los_result
		result;

	char
		error[CAMPAIGN_LOS_ERROR_MAX + 1];

	memset (error, 0, sizeof (error));

	memset (&result, 0, sizeof (result));

	result.visible = -1;

	if (current_host.line_of_sight (step, &result, error, CAMPAIGN_LOS_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host line of sight: host line_of_sight failed: %s", error[0] ? error : "unspecified error");
	}

	switch (result.state)
	{
		case CAMPAIGN_LOS_DONE:
		{
			if ((result.visible != FALSE) && (result.visible != TRUE))
			{
				debug_fatal ("host line of sight: host line_of_sight answered DONE without a boolean visibility");
			}

			break;
		}
		case CAMPAIGN_LOS_NEED_TERRAIN:
		{
			if (!all_finite (result.point, 3))
			{
				debug_fatal ("host line of sight: host line_of_sight asked for a non-finite terrain point");
			}

			if ((result.phase < CAMPAIGN_LOS_PHASE_COARSE) || (result.phase > CAMPAIGN_LOS_PHASE_TARGET_FINE))
			{
				debug_fatal ("host line of sight: host line_of_sight asked for terrain in phase %d", result.phase);
			}

			break;
		}
		case CAMPAIGN_LOS_NEED_OBJECTS:
		{
			if (!all_finite (result.start, 3) || !all_finite (result.end, 3))
			{
				debug_fatal ("host line of sight: host line_of_sight asked for a non-finite object ray");
			}

			if ((result.phase != CAMPAIGN_LOS_PHASE_SOURCE_OBJECTS) && (result.phase != CAMPAIGN_LOS_PHASE_TARGET_OBJECTS))
			{
				debug_fatal ("host line of sight: host line_of_sight asked for objects in phase %d", result.phase);
			}

			break;
		}
		default:
		{
			debug_fatal ("host line of sight: host line_of_sight returned an unknown state %d", result.state);
		}
	}

	return result;
}

int campaign_los_query
(
	entity *source,
	entity *target,
	const vec3d *source_position,
	const vec3d *target_position,
	int criteria,
	int entity_form,
	int source_eye,
	int target_eye
)
{
	campaign_los_step
		step;

	campaign_los_result
		result;

	terrain_3d_point_data
		terrain_info;

	vec3d
		point,
		start,
		end,
		collision_point,
		normal;

	entity
		*collision_en;

	int
		requests,
		visible;

	ASSERT (source);

	ASSERT (target);

	ASSERT (source_position);

	ASSERT (target_position);

	if (!current_host.line_of_sight)
	{
		debug_fatal ("host line of sight: a line-of-sight query needs a host line_of_sight callback");
	}

	if (query_open)
	{
		debug_fatal ("host line of sight: a line-of-sight query was started inside another");
	}

	if (!isfinite (source_position->x) || !isfinite (source_position->y) || !isfinite (source_position->z) ||
		!isfinite (target_position->x) || !isfinite (target_position->y) || !isfinite (target_position->z))
	{
		debug_fatal ("host line of sight: a line-of-sight query with a non-finite end point");
	}

	query_open = TRUE;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_LOS_BEGIN;
	step.entity_form = entity_form;
	step.criteria = criteria;
	step.source_eye = source_eye;
	step.target_eye = target_eye;
	fill_host_id (step.source_host_id, source);
	fill_host_id (step.target_host_id, target);
	step.source_position[0] = source_position->x;
	step.source_position[1] = source_position->y;
	step.source_position[2] = source_position->z;
	step.target_position[0] = target_position->x;
	step.target_position[1] = target_position->y;
	step.target_position[2] = target_position->z;

	result = ask_host (&step);

	//
	// the campaign's terrain hint, kept for the whole query: the original solver zeroed it before its first sample
	//

	memset (&terrain_info, 0, sizeof (terrain_3d_point_data));

	requests = 0;

	while (result.state != CAMPAIGN_LOS_DONE)
	{
		requests++;

		if (requests > CAMPAIGN_LOS_MAX_REQUESTS)
		{
			debug_fatal ("host line of sight: host line_of_sight asked for more than %d values in one query", CAMPAIGN_LOS_MAX_REQUESTS);
		}

		memset (&step, 0, sizeof (step));

		if (result.state == CAMPAIGN_LOS_NEED_TERRAIN)
		{
			point.x = result.point[0];
			point.y = result.point[1];
			point.z = result.point[2];

			ASSERT (point_inside_map_area (&point));

			get_3d_terrain_point_data (point.x, point.z, &terrain_info);

			step.mode = CAMPAIGN_LOS_TERRAIN;
			step.elevation = get_3d_terrain_point_data_elevation (&terrain_info);

			#ifdef EECH_LOS_FIXTURE

			eech_los_lookup_terrain (result.phase, &point, step.elevation);

			#endif
		}
		else
		{
			start.x = result.start[0];
			start.y = result.start[1];
			start.z = result.start[2];

			end.x = result.end[0];
			end.y = result.end[1];
			end.z = result.end[2];

			memset (&collision_point, 0, sizeof (collision_point));

			collision_en = get_line_of_sight_collision_entity (source, target, &start, &end, &collision_point, &normal);

			step.mode = CAMPAIGN_LOS_OBJECTS;
			step.hit = (collision_en != NULL);

			if (collision_en)
			{
				step.collision_point[0] = collision_point.x;
				step.collision_point[1] = collision_point.y;
				step.collision_point[2] = collision_point.z;
			}

			#ifdef EECH_LOS_FIXTURE

			eech_los_lookup_objects (result.phase, &start, &end, step.hit, &collision_point);

			#endif
		}

		result = ask_host (&step);
	}

	visible = result.visible;

	query_open = FALSE;

	#ifdef EECH_LOS_FIXTURE
	{
		//
		// the recorder folds the effective end points: the pose lifted by the eye height, as the original entity check passed them
		//

		vec3d
			effective_source,
			effective_target;

		effective_source = *source_position;

		effective_target = *target_position;

		if (source_eye)
		{
			effective_source.y += 1.5;
		}

		if (target_eye)
		{
			effective_target.y += 1.5;
		}

		eech_los_trace_record (source, target, &effective_source, &effective_target, (mobile_los_check_criteria) criteria, visible);
	}
	#endif

	return visible;
}

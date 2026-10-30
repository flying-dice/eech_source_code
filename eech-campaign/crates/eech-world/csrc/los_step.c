/*
 * The physical calculation of line of sight (EEDCS-42, D2): whether one point sees another over the terrain and past the objects.
 * It is EECH's own check_position_line_of_sight (aphavoc/source/entity/mobile/mb_tgt.c), moved here expression for expression so the
 * host and the original reference produce the same bits, and turned into a resumable state machine because the host does not own the
 * terrain heights (D3) or the object collision scan (D5): it asks the campaign for one value at a time and goes on with the answer.
 *
 * What is kept exactly:
 *   - the range is the horizontal distance (x and z), while the step increments come from the normalised 3D direction;
 *   - float and double types: `sqrt` returns a double that is narrowed to float, the step counter is a float compared with the double 1.0
 *     and decremented as a float, `min` is the engine's macro;
 *   - the order: coarse terrain from the source, source-end fine terrain from the source, target-end fine terrain walking back
 *     from the target, the source-end object ray, then the target-end object ray; the first failure ends the query;
 *   - the target-end hit counts only when the squared range from the collision point to the target exceeds 12 m squared;
 *   - the entity form's eye height: `y += 1.5` in the engine's own double expression, applied here from the two facts the campaign sends.
 *
 * What is not here: which two points to ask about, and what the answer decides (target selection, weapon choice, guide placement),
 * which stay in the campaign. The terrain sample is the campaign's `get_3d_terrain_point_data_elevation` read after its lookup, and the
 * object answer its collision scan with the two entities excluded; both are declared transitional coupling.
 *
 * The state is one open query. A new `eech_los_begin` replaces whatever was open; an answer with nothing pending is refused.
 */
#include <stddef.h>

#include "eech_arith.h"

#define LOS_CHECK_COARSE_TERRAIN (1 << 0)
#define LOS_CHECK_SOURCE_END_TERRAIN (1 << 1)
#define LOS_CHECK_TARGET_END_TERRAIN (1 << 2)
#define LOS_CHECK_SOURCE_END_OBJECTS (1 << 3)
#define LOS_CHECK_TARGET_END_OBJECTS (1 << 4)

#define LOS_COARSE_CHECK_DISTANCE 200.0f

#define LOS_FINE_CHECK_DISTANCE_SOURCE_END 40.0f
#define LOS_NUMBER_OF_FINE_CHECKS_SOURCE_END 8.0f

#define LOS_FINE_CHECK_DISTANCE_TARGET_END 24.0f
#define LOS_NUMBER_OF_FINE_CHECKS_TARGET_END 8.0f

#define LOS_OBJECT_CHECK_DISTANCE_SOURCE_END 200.0f

#define LOS_OBJECT_CHECK_DISTANCE_TARGET_END 200.0f

/* ignore any obstacles within a certain radius of the target */
#define LOS_OBJECT_EXCLUDE_DISTANCE_TARGET_END 12.0f

/* the states of eech_los_request: the numbers are the campaign's campaign_los_state values */
#define LOS_DONE 1
#define LOS_NEED_TERRAIN 2
#define LOS_NEED_OBJECTS 3

/* the phases a request belongs to: the campaign's campaign_los_phase values */
#define LOS_PHASE_COARSE 1
#define LOS_PHASE_SOURCE_FINE 2
#define LOS_PHASE_TARGET_FINE 3
#define LOS_PHASE_SOURCE_OBJECTS 4
#define LOS_PHASE_TARGET_OBJECTS 5

enum
{
	STAGE_COARSE_INIT,
	STAGE_COARSE_LOOP,
	STAGE_SOURCE_FINE_INIT,
	STAGE_SOURCE_FINE_LOOP,
	STAGE_TARGET_FINE_INIT,
	STAGE_TARGET_FINE_LOOP,
	STAGE_SOURCE_OBJECTS,
	STAGE_SOURCE_OBJECTS_WAIT,
	STAGE_TARGET_OBJECTS,
	STAGE_TARGET_OBJECTS_WAIT,
	STAGE_FINISHED
};

enum
{
	AWAIT_NONE,
	AWAIT_TERRAIN,
	AWAIT_OBJECTS
};

typedef struct eech_los_request
{
	int state;
	int phase;
	float point[3];
	float start[3];
	float end[3];
	int visible;
} eech_los_request;

typedef struct eech_los_query
{
	int stage;
	int awaiting;
	int criteria;
	vec3d source_position;
	vec3d target_position;
	vec3d direction;
	vec3d increment;
	vec3d check_position;
	float target_range;
	float number_of_terrain_checks;
} eech_los_query;

size_t eech_los_query_size(void)
{
	return sizeof(eech_los_query);
}

static void set_vector(float *out, const vec3d *v)
{
	out[0] = v->x;
	out[1] = v->y;
	out[2] = v->z;
}

static void finish(eech_los_query *q, int visible, eech_los_request *out)
{
	q->stage = STAGE_FINISHED;
	q->awaiting = AWAIT_NONE;
	out->state = LOS_DONE;
	out->phase = 0;
	out->visible = visible;
}

static void ask_terrain(eech_los_query *q, int phase, eech_los_request *out)
{
	q->awaiting = AWAIT_TERRAIN;
	out->state = LOS_NEED_TERRAIN;
	out->phase = phase;
	set_vector(out->point, &q->check_position);
}

static void ask_objects(eech_los_query *q, int phase, const vec3d *start, eech_los_request *out)
{
	q->awaiting = AWAIT_OBJECTS;
	out->state = LOS_NEED_OBJECTS;
	out->phase = phase;
	set_vector(out->start, start);
	set_vector(out->end, &q->check_position);
}

/* runs the original solver from the current stage until it needs a value from the campaign, or has an answer */
static void advance(eech_los_query *q, eech_los_request *out)
{
	for (;;)
	{
		switch (q->stage)
		{
		case STAGE_COARSE_INIT:
			if (q->criteria & LOS_CHECK_COARSE_TERRAIN)
			{
				q->number_of_terrain_checks = q->target_range / LOS_COARSE_CHECK_DISTANCE;

				q->increment.x = q->direction.x * LOS_COARSE_CHECK_DISTANCE;
				q->increment.y = q->direction.y * LOS_COARSE_CHECK_DISTANCE;
				q->increment.z = q->direction.z * LOS_COARSE_CHECK_DISTANCE;

				q->check_position = q->source_position;

				q->stage = STAGE_COARSE_LOOP;
			}
			else
			{
				q->stage = STAGE_SOURCE_FINE_INIT;
			}
			break;

		case STAGE_COARSE_LOOP:
			if (q->number_of_terrain_checks > 1.0)
			{
				q->check_position.x += q->increment.x;
				q->check_position.y += q->increment.y;
				q->check_position.z += q->increment.z;

				ask_terrain(q, LOS_PHASE_COARSE, out);
				return;
			}
			q->stage = STAGE_SOURCE_FINE_INIT;
			break;

		case STAGE_SOURCE_FINE_INIT:
			if (q->criteria & LOS_CHECK_SOURCE_END_TERRAIN)
			{
				q->number_of_terrain_checks = q->target_range / LOS_FINE_CHECK_DISTANCE_SOURCE_END;

				q->number_of_terrain_checks = min(q->number_of_terrain_checks, LOS_NUMBER_OF_FINE_CHECKS_SOURCE_END);

				q->increment.x = q->direction.x * LOS_FINE_CHECK_DISTANCE_SOURCE_END;
				q->increment.y = q->direction.y * LOS_FINE_CHECK_DISTANCE_SOURCE_END;
				q->increment.z = q->direction.z * LOS_FINE_CHECK_DISTANCE_SOURCE_END;

				q->check_position = q->source_position;

				q->stage = STAGE_SOURCE_FINE_LOOP;
			}
			else
			{
				q->stage = STAGE_TARGET_FINE_INIT;
			}
			break;

		case STAGE_SOURCE_FINE_LOOP:
			if (q->number_of_terrain_checks > 1.0)
			{
				q->check_position.x += q->increment.x;
				q->check_position.y += q->increment.y;
				q->check_position.z += q->increment.z;

				ask_terrain(q, LOS_PHASE_SOURCE_FINE, out);
				return;
			}
			q->stage = STAGE_TARGET_FINE_INIT;
			break;

		case STAGE_TARGET_FINE_INIT:
			if (q->criteria & LOS_CHECK_TARGET_END_TERRAIN)
			{
				q->number_of_terrain_checks = q->target_range / LOS_FINE_CHECK_DISTANCE_TARGET_END;

				q->number_of_terrain_checks = min(q->number_of_terrain_checks, LOS_NUMBER_OF_FINE_CHECKS_TARGET_END);

				q->increment.x = q->direction.x * LOS_FINE_CHECK_DISTANCE_TARGET_END;
				q->increment.y = q->direction.y * LOS_FINE_CHECK_DISTANCE_TARGET_END;
				q->increment.z = q->direction.z * LOS_FINE_CHECK_DISTANCE_TARGET_END;

				q->check_position = q->target_position;

				q->stage = STAGE_TARGET_FINE_LOOP;
			}
			else
			{
				q->stage = STAGE_SOURCE_OBJECTS;
			}
			break;

		case STAGE_TARGET_FINE_LOOP:
			if (q->number_of_terrain_checks > 1.0)
			{
				q->check_position.x -= q->increment.x;
				q->check_position.y -= q->increment.y;
				q->check_position.z -= q->increment.z;

				ask_terrain(q, LOS_PHASE_TARGET_FINE, out);
				return;
			}
			q->stage = STAGE_SOURCE_OBJECTS;
			break;

		case STAGE_SOURCE_OBJECTS:
			if (q->criteria & LOS_CHECK_SOURCE_END_OBJECTS)
			{
				q->check_position.x = q->source_position.x + (q->direction.x * min(q->target_range, LOS_OBJECT_CHECK_DISTANCE_SOURCE_END));
				q->check_position.y = q->source_position.y + (q->direction.y * min(q->target_range, LOS_OBJECT_CHECK_DISTANCE_SOURCE_END));
				q->check_position.z = q->source_position.z + (q->direction.z * min(q->target_range, LOS_OBJECT_CHECK_DISTANCE_SOURCE_END));

				q->stage = STAGE_SOURCE_OBJECTS_WAIT;
				ask_objects(q, LOS_PHASE_SOURCE_OBJECTS, &q->source_position, out);
				return;
			}
			q->stage = STAGE_TARGET_OBJECTS;
			break;

		case STAGE_TARGET_OBJECTS:
			if (q->criteria & LOS_CHECK_TARGET_END_OBJECTS)
			{
				q->check_position.x = q->target_position.x - (q->direction.x * min(q->target_range, LOS_OBJECT_CHECK_DISTANCE_TARGET_END));
				q->check_position.y = q->target_position.y - (q->direction.y * min(q->target_range, LOS_OBJECT_CHECK_DISTANCE_TARGET_END));
				q->check_position.z = q->target_position.z - (q->direction.z * min(q->target_range, LOS_OBJECT_CHECK_DISTANCE_TARGET_END));

				q->stage = STAGE_TARGET_OBJECTS_WAIT;
				ask_objects(q, LOS_PHASE_TARGET_OBJECTS, &q->target_position, out);
				return;
			}
			finish(q, 1, out);
			return;

		default:
			finish(q, 1, out);
			return;
		}
	}
}

/*
 * Starts a query: `source` and `target` are the ray's end points as the campaign supplied them, `criteria` the five check bits,
 * and `source_eye` / `target_eye` the facts that lift that end 1.5 m (the entity form's vehicle base raise).
 */
void eech_los_begin(eech_los_query *q, const float *source, const float *target, int criteria, int source_eye, int target_eye, eech_los_request *out)
{
	memset(q, 0, sizeof(*q));
	memset(out, 0, sizeof(*out));

	q->criteria = criteria;

	q->source_position.x = source[0];
	q->source_position.y = source[1];
	q->source_position.z = source[2];
	q->target_position.x = target[0];
	q->target_position.y = target[1];
	q->target_position.z = target[2];

	/* raise vehicle's base to avoid targeting problems */
	if (source_eye)
	{
		q->source_position.y += 1.5;
	}

	if (target_eye)
	{
		q->target_position.y += 1.5;
	}

	q->direction.x = q->target_position.x - q->source_position.x;
	q->direction.y = q->target_position.y - q->source_position.y;
	q->direction.z = q->target_position.z - q->source_position.z;

	q->target_range = sqrt((q->direction.x * q->direction.x) + (q->direction.z * q->direction.z));

	normalise_any_3d_vector(&q->direction);

	q->stage = STAGE_COARSE_INIT;
	q->awaiting = AWAIT_NONE;

	advance(q, out);
}

/* The campaign's elevation for the sample the host asked about. Returns 0, or -1 when no terrain answer is pending (nothing changes). */
int eech_los_terrain(eech_los_query *q, float elevation, eech_los_request *out)
{
	memset(out, 0, sizeof(*out));

	if (q->awaiting != AWAIT_TERRAIN)
	{
		return -1;
	}

	q->awaiting = AWAIT_NONE;

	if (elevation > q->check_position.y)
	{
		finish(q, 0, out);
		return 0;
	}

	q->number_of_terrain_checks--;

	advance(q, out);
	return 0;
}

/*
 * The campaign's answer to an object ray: whether an entity blocks it and, only if it does, the collision point.  Returns 0, or -1
 * when no object answer is pending (nothing changes).
 */
int eech_los_objects(eech_los_query *q, int hit, const float *collision_point, eech_los_request *out)
{
	memset(out, 0, sizeof(*out));

	if (q->awaiting != AWAIT_OBJECTS)
	{
		return -1;
	}

	q->awaiting = AWAIT_NONE;

	if (q->stage == STAGE_SOURCE_OBJECTS_WAIT)
	{
		if (hit)
		{
			finish(q, 0, out);
			return 0;
		}

		q->stage = STAGE_TARGET_OBJECTS;
	}
	else
	{
		if (hit)
		{
			vec3d point;
			float collision_distance;

			point.x = collision_point[0];
			point.y = collision_point[1];
			point.z = collision_point[2];

			collision_distance = get_sqr_3d_range(&point, &q->target_position);

			if (collision_distance > (LOS_OBJECT_EXCLUDE_DISTANCE_TARGET_END * LOS_OBJECT_EXCLUDE_DISTANCE_TARGET_END))
			{
				finish(q, 0, out);
				return 0;
			}
		}

		finish(q, 1, out);
		return 0;
	}

	advance(q, out);
	return 0;
}

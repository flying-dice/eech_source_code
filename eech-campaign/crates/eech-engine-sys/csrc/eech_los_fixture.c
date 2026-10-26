/*
 * TEST-ONLY line-of-sight characterisation (EEDCS-42). Compiled only with the `los-fixture` feature; never part of the
 * shipping module.
 *
 * Two things live here:
 *
 *   1. The run-wide trace. mb_tgt.c, built with EECH_LOS_FIXTURE, forwards every line-of-sight query (entity or position
 *      check) to its unchanged solver and calls eech_los_trace_record with the query's inputs and its answer. The trace
 *      folds them into a call count, a visible count, per-criteria counts and an order-dependent FNV-1a hash over the
 *      inputs' bit patterns and the answers. It only observes: nothing it reads is altered.
 *
 *   2. The probe sweep. A deterministic set of queries over the loaded map from two live mobiles' positions: segments of
 *      25 m to 15 km whose lengths bracket the 200 m coarse step (150, 199.5, 200, 201, 399, 450 m among them; the 40 m and
 *      24 m fine steps are not bracketed), at five heights over the real terrain, each with one of eight chosen criteria masks
 *      (ALL, each of the five bits alone, the three terrain bits together, and NONE; not every subset of the five bits),
 *      then the entity form for each of those masks, both ways round. The points come from a
 *      private xorshift generator, so the sweep never touches the game's random stream, and the terrain heights it uses to
 *      place points come from a private point-data record, so it leaves EECH's shared terrain state alone.
 *
 * The same sweep and the same hash are used before and after the extraction, so a difference is a difference in answers.
 */

#ifdef EECH_LOS_FIXTURE

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

static struct eech_los_trace run_trace;
static struct eech_los_trace *active_trace = &run_trace;

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

static void fold (struct eech_los_trace *trace, uint32_t word)
{
	int byte;
	for (byte = 0; byte < 4; byte++)
	{
		trace->hash ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		trace->hash *= FNV_PRIME;
	}
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void trace_reset (struct eech_los_trace *trace)
{
	memset (trace, 0, sizeof (*trace));
	trace->hash = FNV_OFFSET;
}

/*
 * Which kinds of entity ask and are asked: a small table of (source type, target type, source has a host id, target has a host
 * id) -> count for the run-wide trace only (the probe's own sweep is not counted), read-only like the rest of the recorder.
 * It is logged when the run-wide trace is read, one `los-kinds` line per row, and is not part of the hash.
 */
#define KIND_ROWS 256
static struct { int source_type, target_type, source_hosted, target_hosted; uint64_t count; } kind_rows[KIND_ROWS];
static int kind_row_count, kind_overflow;

static void note_kinds (entity *source, entity *target)
{
	int st = (int) get_local_entity_type (source), tt = (int) get_local_entity_type (target);
	int sh = campaign_roster_host_id (source) != NULL, th = campaign_roster_host_id (target) != NULL;
	int i;
	for (i = 0; i < kind_row_count; i++)
	{
		if (kind_rows[i].source_type == st && kind_rows[i].target_type == tt && kind_rows[i].source_hosted == sh && kind_rows[i].target_hosted == th)
		{
			kind_rows[i].count++;
			return;
		}
	}
	if (kind_row_count < KIND_ROWS)
	{
		kind_rows[kind_row_count].source_type = st;
		kind_rows[kind_row_count].target_type = tt;
		kind_rows[kind_row_count].source_hosted = sh;
		kind_rows[kind_row_count].target_hosted = th;
		kind_rows[kind_row_count].count = 1;
		kind_row_count++;
	}
	else
	{
		kind_overflow++;
	}
}

void eech_los_trace_log_kinds (void)
{
	int i;
	for (i = 0; i < kind_row_count; i++)
	{
		eech_log (1, "los-kinds source=%s target=%s source_host_id=%d target_host_id=%d count=%llu",
			get_entity_type_name ((entity_types) kind_rows[i].source_type), get_entity_type_name ((entity_types) kind_rows[i].target_type),
			kind_rows[i].source_hosted, kind_rows[i].target_hosted, (unsigned long long) kind_rows[i].count);
	}
	eech_log (1, "los-kinds rows=%d overflow=%d", kind_row_count, kind_overflow);
}

void eech_los_trace_record (entity *source, entity *target, vec3d *source_position, vec3d *target_position, mobile_los_check_criteria criteria, int result)
{
	struct eech_los_trace *trace = active_trace;
	if (trace == &run_trace)
	{
		note_kinds (source, target);
	}
	unsigned slot = ((unsigned) criteria) % EECH_LOS_CRITERIA;
	if (trace->calls == 0 && trace->hash == 0)
	{
		trace_reset (trace);
	}
	trace->calls++;
	trace->by_criteria_calls[slot]++;
	if (result)
	{
		trace->visible++;
		trace->by_criteria_visible[slot]++;
	}
	fold (trace, (uint32_t) criteria);
	fold (trace, float_bits (source_position->x));
	fold (trace, float_bits (source_position->y));
	fold (trace, float_bits (source_position->z));
	fold (trace, float_bits (target_position->x));
	fold (trace, float_bits (target_position->y));
	fold (trace, float_bits (target_position->z));
	fold (trace, (uint32_t) (result ? 1 : 0));
}

void eech_los_trace_get (struct eech_los_trace *out)
{
	eech_los_trace_log_kinds ();
	if (run_trace.calls == 0 && run_trace.hash == 0)
	{
		trace_reset (&run_trace);
	}
	*out = run_trace;
}

void eech_los_trace_clear (void)
{
	trace_reset (&run_trace);
}

static uint32_t next_random (uint32_t *state)
{
	uint32_t x = *state;
	x ^= x << 13;
	x ^= x >> 17;
	x ^= x << 5;
	*state = x;
	return x;
}

static float unit (uint32_t *state)
{
	return (float) (next_random (state) & 0xffffffu) / 16777216.0f;
}

/*
 * A live mobile the sweep can query: the index lies within the heap actually allocated (number_of_entities, which is less
 * than MAX_NUM_ENTITIES), and the entity is one of the mobile families whose position the campaign reads for a line-of-sight
 * check (the six hosted mobile families, so no weapon, keysite or free slot). Anything else is refused (NULL), and the caller
 * reports a bad argument; nothing is dereferenced for an unsupported shape.
 */
static entity *live_entity (int index)
{
	entity *en;
	if (!entities || (index < 0) || (index >= number_of_entities))
	{
		return NULL;
	}
	en = get_local_entity_ptr (index);
	switch (get_local_entity_type (en))
	{
	case ENTITY_TYPE_HELICOPTER:
	case ENTITY_TYPE_FIXED_WING:
	case ENTITY_TYPE_ROUTED_VEHICLE:
	case ENTITY_TYPE_ANTI_AIRCRAFT:
	case ENTITY_TYPE_PERSON:
	case ENTITY_TYPE_SHIP_VEHICLE:
		return en;
	default:
		return NULL;
	}
}

static float terrain_height (float x, float z)
{
	terrain_3d_point_data point;
	memset (&point, 0, sizeof (point));
	return get_3d_terrain_point_data (x, z, &point);
}

static const float probe_ranges[] = { 25.0f, 150.0f, 199.5f, 200.0f, 201.0f, 399.0f, 450.0f, 1000.0f, 2500.0f, 6000.0f, 15000.0f };
static const float probe_heights[] = { 0.0f, 1.5f, 12.0f, 60.0f, 300.0f };
static const int probe_criteria[] = {
	MOBILE_LOS_CHECK_ALL,
	MOBILE_LOS_CHECK_COURSE_TERRAIN,
	MOBILE_LOS_CHECK_SOURCE_END_TERRAIN,
	MOBILE_LOS_CHECK_TARGET_END_TERRAIN,
	MOBILE_LOS_CHECK_SOURCE_END_OBJECTS,
	MOBILE_LOS_CHECK_TARGET_END_OBJECTS,
	MOBILE_LOS_CHECK_COURSE_TERRAIN | MOBILE_LOS_CHECK_SOURCE_END_TERRAIN | MOBILE_LOS_CHECK_TARGET_END_TERRAIN,
	MOBILE_LOS_CHECK_NONE,
};

#define COUNT_OF(a) ((int) (sizeof (a) / sizeof ((a)[0])))

/*
 * Runs the sweep and the entity-form queries between two live entities. Returns 0, or -1 when either entity is not live.
 * `count` position queries are generated; one whose end points fall within 500 m of the map edge is skipped and counted, not
 * adjusted, so the set stays a pure function of the seed and the map.
 */
int eech_los_probe_run (int source_id, int target_id, uint32_t seed, int count, struct eech_los_trace *out)
{
	entity *source = live_entity (source_id);
	entity *target = live_entity (target_id);
	struct eech_los_trace *saved = active_trace;
	uint32_t state = seed ? seed : 1u;
	int i, c;
	vec3d centre;

	if (!source || !target || (source == target))
	{
		return -1;
	}
	trace_reset (out);
	active_trace = out;

	get_local_entity_vec3d (source, VEC3D_TYPE_POSITION, &centre);

	for (i = 0; i < count; i++)
	{
		vec3d s, t;
		float bearing, range, margin = 500.0f;
		int ri = (int) (next_random (&state) % COUNT_OF (probe_ranges));
		int hs = (int) (next_random (&state) % COUNT_OF (probe_heights));
		int ht = (int) (next_random (&state) % COUNT_OF (probe_heights));
		s.x = centre.x + (unit (&state) - 0.5f) * 3000.0f;
		s.z = centre.z + (unit (&state) - 0.5f) * 3000.0f;
		bearing = unit (&state) * 6.2831853f;
		range = probe_ranges[ri];
		t.x = s.x + range * (float) sin (bearing);
		t.z = s.z + range * (float) cos (bearing);
		c = (int) (next_random (&state) % COUNT_OF (probe_criteria));
		if ((s.x < MIN_MAP_X + margin) || (s.x > MAX_MAP_X - margin) || (s.z < MIN_MAP_Z + margin) || (s.z > MAX_MAP_Z - margin)
			|| (t.x < MIN_MAP_X + margin) || (t.x > MAX_MAP_X - margin) || (t.z < MIN_MAP_Z + margin) || (t.z > MAX_MAP_Z - margin))
		{
			out->skipped++;
			continue;
		}
		s.y = terrain_height (s.x, s.z) + probe_heights[hs];
		t.y = terrain_height (t.x, t.z) + probe_heights[ht];
		(void) check_position_line_of_sight (source, target, &s, &t, (mobile_los_check_criteria) probe_criteria[c]);
	}

	for (c = 0; c < COUNT_OF (probe_criteria); c++)
	{
		(void) check_entity_line_of_sight (source, target, (mobile_los_check_criteria) probe_criteria[c]);
		(void) check_entity_line_of_sight (target, source, (mobile_los_check_criteria) probe_criteria[c]);
	}

	active_trace = saved;
	return 0;
}

#endif

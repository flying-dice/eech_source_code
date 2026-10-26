/*
 * TEST-ONLY sensor-detection characterisation (EEDCS-43). Compiled only with the `sensor-fixture` feature; never part of the
 * shipping module.
 *
 * ac_tgt.c and vh_tgt.c, built with EECH_SENSOR_FIXTURE, call the eech_sensor_* hooks declared in entity/mobile/mb_sensorfx.h from
 * inside the four AI target-scan group functions (air-to-air, air-to-surface, surface-to-air, surface-to-surface). The hooks only
 * observe. This file folds what they report into two hashes and a set of counters:
 *
 *   physical_hash  the physical facts, in the order the scan computed them: the source's position and scan envelope, the sector
 *                  rectangle it enumerates, and for every candidate that reaches the physical predicates its measured range,
 *                  scan floor (after the original's cumulative mutation) and the floor an independent per-target threshold
 *                  would have used, radar altitude, the tolerance draw and limit, the emitter flags actually read, the stealth
 *                  factor and the stealth-scaled limit, and the verdict (accepted, or the predicate that rejected it).
 *   decision_hash  what the campaign layer decides from them: each eligibility skip (by reason), the raw ranking inputs
 *                  (cos_angle, target_result, target_select_criteria and whether the candidate displaced the best), and the
 *                  call's outcome (target_count, best group position, best score).
 *
 * Neither hash carries an entity index, a heap address or a seed. The tolerance draw is recomputed on a local copy of the
 * original's source-derived seed, so the game's random stream and the original's own seed variable are not touched; only the
 * draw and the limit built from it are folded.
 *
 * The probe walks the engine's own used-entity list (never an index supplied by a caller) and runs the two scans of each eligible
 * source family, writing nothing and engaging nothing; its trace is separate from the run-wide one.
 */

#ifdef EECH_SENSOR_FIXTURE

#include <stdarg.h>
#include <stdint.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"
#include "entity/mobile/mb_sensorfx.h"

#define F EECH_SENSOR_FUNCTIONS
#define V EECH_SENSOR_VERDICTS
#define S EECH_SENSOR_SKIPS
#define SAMPLE_SLOTS 2
#define SAMPLE_COUNT 48
#define SAMPLE_TEXT 1300
#define KIND_ROWS 64

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

static const char *const function_names[F] = { "air_to_air", "air_to_surface", "surface_to_air", "surface_to_surface" };
static const char *const verdict_names[V] = { "accept", "range", "range_adjusted", "altitude", "tolerance", "line_of_sight" };
static const char *const skip_names[S] = { "self", "side", "identify", "airborne", "target_type", "no_group", "best_group", "dead" };
static const char *const tag_names[EECH_SENSOR_TAGS] = {
	"source_x", "source_y", "source_z", "scan_range", "scan_range_squared", "scan_ceiling", "scan_floor_initial",
	"sector_x_min", "sector_z_min", "sector_x_max", "sector_z_max", "target_range", "target_player", "radar_on",
	"radar_jammer_on", "infra_red_jammer_on", "adjusted_range", "stealth_factor", "stealth_limit", "floor",
	"floor_independent", "difficulty_factor", "radar_altitude", "tolerance_draw", "tolerance_limit", "cos_angle", "result",
	"los_criteria", "replaced"
};

struct sample
{
	int used;
	char text[SAMPLE_TEXT];
};

struct kind_row
{
	int source_type, function;
	uint64_t calls, scanned;
};

struct sensor_trace
{
	uint64_t physical_hash, decision_hash;
	uint64_t calls[F], scanned[F], nonnull[F], count_sum[F], count_positive_null[F], candidates[F];
	uint64_t verdicts[F][V];
	uint64_t skips[F][S];
	/* edges: what the run actually exercised */
	uint64_t player_candidates[F], emitter_off[F], stealth_not_one[F], adjusted_differs[F], replaced[F];
	uint64_t sector_cells_sum[F], sector_whole_map[F], cos_seen[F], cos_outside_unit[F];
	double cos_min[F], cos_max[F];
	uint64_t floor_cumulative_differs, floor_multi_candidate_calls, altitude_below_floor, altitude_above_ceiling;
	uint64_t tolerance_draws, tolerance_mismatch, tolerance_varies_in_call, tolerance_rejects_after_floor_pass;
	uint64_t difficulty_factors;
	double tolerance_min, tolerance_max;
	float floor_ratio_min;
	int kind_count, kind_overflow, sample_overflow;
	struct kind_row kinds[KIND_ROWS];
	int sample_count;
	char sample_key[SAMPLE_COUNT][40];
	struct sample samples[SAMPLE_COUNT];
};

static struct sensor_trace run_trace;
static struct sensor_trace *active_trace = &run_trace;

/* the one scan in flight (scans never nest) */
static struct
{
	int active, function, scanned;
	int candidate_open, candidates, floor_evals, tolerance_pending, draw_valid;
	double draw_first;
	float scan_range, floor_initial, ceiling, floor_after, radar_altitude, target_range, factor;
	int factor_applied, radar_on, jammer_on, ir_on, player;
	int sector_x_min, sector_z_min, sector_x_max, sector_z_max;
	float stealth;
	float adjusted;
	char header[400];
	char text[SAMPLE_TEXT - 440];
} cur;

static void text_append (char *buffer, size_t capacity, const char *format, ...) __attribute__ ((format (printf, 3, 4)));

static void text_append (char *buffer, size_t capacity, const char *format, ...)
{
	size_t used = strlen (buffer);
	va_list args;
	if (used + 1 >= capacity)
	{
		return;
	}
	va_start (args, format);
	vsnprintf (buffer + used, capacity - used, format, args);
	va_end (args);
}

static void fold (uint64_t *hash, uint32_t word)
{
	int byte;
	for (byte = 0; byte < 4; byte++)
	{
		*hash ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		*hash *= FNV_PRIME;
	}
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void fold_double (uint64_t *hash, double value)
{
	uint64_t bits;
	memcpy (&bits, &value, sizeof (bits));
	fold (hash, (uint32_t) (bits & 0xffffffffu));
	fold (hash, (uint32_t) (bits >> 32));
}

static void trace_reset (struct sensor_trace *trace)
{
	memset (trace, 0, sizeof (*trace));
	trace->physical_hash = FNV_OFFSET;
	trace->decision_hash = FNV_OFFSET;
	trace->tolerance_min = 2.0;
	trace->tolerance_max = -1.0;
	trace->floor_ratio_min = 2.0f;
	{
		int f;
		for (f = 0; f < F; f++)
		{
			trace->cos_min[f] = 1e30;
			trace->cos_max[f] = -1e30;
		}
	}
}

static struct sensor_trace *ready (void)
{
	struct sensor_trace *trace = active_trace;
	if (!trace->physical_hash)
	{
		trace_reset (trace);
	}
	return trace;
}

static void keep_sample (struct sensor_trace *trace, const char *key)
{
	int i, matches = 0;
	for (i = 0; i < trace->sample_count; i++)
	{
		if (strcmp (trace->sample_key[i], key) == 0)
		{
			matches++;
		}
	}
	if (matches >= SAMPLE_SLOTS)
	{
		return;
	}
	if (trace->sample_count >= SAMPLE_COUNT)
	{
		trace->sample_overflow++;
		return;
	}
	snprintf (trace->sample_key[trace->sample_count], sizeof (trace->sample_key[0]), "%s", key);
	snprintf (trace->samples[trace->sample_count].text, SAMPLE_TEXT, "%s | %s", cur.header, cur.text);
	trace->samples[trace->sample_count].used = 1;
	trace->sample_count++;
}

static void note_kind (struct sensor_trace *trace, entity *source, int function)
{
	int type = (int) get_local_entity_type (source);
	int i;
	for (i = 0; i < trace->kind_count; i++)
	{
		if (trace->kinds[i].source_type == type && trace->kinds[i].function == function)
		{
			trace->kinds[i].calls++;
			return;
		}
	}
	if (trace->kind_count < KIND_ROWS)
	{
		trace->kinds[trace->kind_count].source_type = type;
		trace->kinds[trace->kind_count].function = function;
		trace->kinds[trace->kind_count].calls = 1;
		trace->kind_count++;
	}
	else
	{
		trace->kind_overflow++;
	}
}

static void note_scanned (struct sensor_trace *trace, entity *source, int function)
{
	int type = (int) get_local_entity_type (source);
	int i;
	for (i = 0; i < trace->kind_count; i++)
	{
		if (trace->kinds[i].source_type == type && trace->kinds[i].function == function)
		{
			trace->kinds[i].scanned++;
			return;
		}
	}
}

static entity *cur_source;

void eech_sensor_begin (int function, entity *source)
{
	struct sensor_trace *trace = ready ();
	trace->calls[function]++;
	note_kind (trace, source, function);
	memset (&cur, 0, sizeof (cur));
	cur.active = 1;
	cur.function = function;
	cur.tolerance_pending = -1;
	cur_source = source;
	fold (&trace->physical_hash, 0xB0000000u | (uint32_t) function);
	fold (&trace->decision_hash, 0xB0000000u | (uint32_t) function);
}

void eech_sensor_skip (int reason)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	trace->skips[cur.function][reason]++;
	fold (&trace->decision_hash, 0x50000000u | (uint32_t) reason);
}

void eech_sensor_candidate (void)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	cur.candidate_open = 1;
	cur.factor_applied = 0;
	cur.factor = 1.0f;
	cur.tolerance_pending = -1;
	cur.radar_on = cur.jammer_on = cur.ir_on = -1;
	cur.player = 0;
	cur.stealth = 1.0f;
	cur.adjusted = 0.0f;
	cur.text[0] = 0;
	fold (&trace->physical_hash, 0xC0000000u | (uint32_t) cur.function);
}

static void describe_f (int tag, float value)
{
	if (cur.candidate_open)
	{
		text_append (cur.text, sizeof (cur.text), "%s%s=%.9g", cur.text[0] ? " " : "", tag_names[tag], (double) value);
	}
	else
	{
		text_append (cur.header, sizeof (cur.header), "%s%s=%.9g", cur.header[0] ? " " : "", tag_names[tag], (double) value);
	}
}

static void describe_i (int tag, int value)
{
	if (cur.candidate_open)
	{
		text_append (cur.text, sizeof (cur.text), "%s%s=%d", cur.text[0] ? " " : "", tag_names[tag], value);
	}
	else
	{
		text_append (cur.header, sizeof (cur.header), "%s%s=%d", cur.header[0] ? " " : "", tag_names[tag], value);
	}
}

void eech_sensor_f (int tag, float value)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	fold (&trace->physical_hash, 0xD0000000u | (uint32_t) tag);
	fold (&trace->physical_hash, float_bits (value));
	describe_f (tag, value);
	switch (tag)
	{
	case EECH_SENSOR_TAG_SOURCE_X:
		cur.scanned = 1;
		break;
	case EECH_SENSOR_TAG_SCAN_RANGE:
		cur.scan_range = value;
		break;
	case EECH_SENSOR_TAG_SCAN_FLOOR_INITIAL:
		cur.floor_initial = value;
		break;
	case EECH_SENSOR_TAG_SCAN_CEILING:
		cur.ceiling = value;
		break;
	case EECH_SENSOR_TAG_TARGET_RANGE:
		cur.target_range = value;
		break;
	case EECH_SENSOR_TAG_RADAR_ALTITUDE:
		cur.radar_altitude = value;
		break;
	case EECH_SENSOR_TAG_ADJUSTED_RANGE:
		cur.adjusted = value;
		break;
	case EECH_SENSOR_TAG_STEALTH_FACTOR:
		cur.stealth = value;
		break;
	default:
		break;
	}
}

void eech_sensor_i (int tag, int value)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	fold (&trace->physical_hash, 0xD8000000u | (uint32_t) tag);
	fold (&trace->physical_hash, (uint32_t) value);
	describe_i (tag, value);
	switch (tag)
	{
	case EECH_SENSOR_TAG_SECTOR_X_MIN:
		cur.sector_x_min = value;
		break;
	case EECH_SENSOR_TAG_SECTOR_Z_MIN:
		cur.sector_z_min = value;
		break;
	case EECH_SENSOR_TAG_SECTOR_X_MAX:
		cur.sector_x_max = value;
		break;
	case EECH_SENSOR_TAG_SECTOR_Z_MAX:
		cur.sector_z_max = value;
		break;
	case EECH_SENSOR_TAG_TARGET_PLAYER:
		cur.player = value;
		break;
	case EECH_SENSOR_TAG_RADAR_ON:
		cur.radar_on = value;
		break;
	case EECH_SENSOR_TAG_RADAR_JAMMER_ON:
		cur.jammer_on = value;
		break;
	case EECH_SENSOR_TAG_INFRA_RED_JAMMER_ON:
		cur.ir_on = value;
		break;
	default:
		break;
	}
}

void eech_sensor_dec_f (int tag, float value)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	fold (&trace->decision_hash, 0xE0000000u | (uint32_t) tag);
	fold (&trace->decision_hash, float_bits (value));
	describe_f (tag, value);
	if (tag == EECH_SENSOR_TAG_COS_ANGLE)
	{
		/* the original's cos_angle is a dot product with an unnormalised vector: a length in metres, not a cosine */
		trace->cos_seen[cur.function]++;
		if ((double) value < trace->cos_min[cur.function])
		{
			trace->cos_min[cur.function] = value;
		}
		if ((double) value > trace->cos_max[cur.function])
		{
			trace->cos_max[cur.function] = value;
		}
		if (value < -1.0f || value > 1.0f)
		{
			trace->cos_outside_unit[cur.function]++;
		}
	}
}

void eech_sensor_dec_i (int tag, int value)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	fold (&trace->decision_hash, 0xE8000000u | (uint32_t) tag);
	fold (&trace->decision_hash, (uint32_t) value);
	describe_i (tag, value);
	if (tag == EECH_SENSOR_TAG_REPLACED)
	{
		char key[40];
		if (value)
		{
			trace->replaced[cur.function]++;
		}
		snprintf (key, sizeof (key), "%s/ranked", function_names[cur.function]);
		keep_sample (trace, key);
	}
}

/*
 * The stateful scan floor of vh_tgt.c: `source_air_scan_floor` is multiplied in place for every candidate, so each candidate's floor
 * depends on the ones before it. `floor_independent` is what a per-target threshold would have been (the group's initial floor times
 * this target's own range ratio, and the same player factor), recomputed with the original's expressions; the two differ from the
 * second candidate on. Both are folded, so a change to either shows in the hash.
 */
void eech_sensor_floor (float floor_after, float scan_range, float target_range)
{
	struct sensor_trace *trace = ready ();
	float independent;
	float ratio;
	if (!cur.active)
	{
		return;
	}
	independent = cur.floor_initial;
	independent *= bound (target_range / scan_range, 0.1, 1.0);
	if (cur.factor_applied)
	{
		independent *= cur.factor;
	}
	cur.floor_after = floor_after;
	cur.floor_evals++;
	eech_sensor_f (EECH_SENSOR_TAG_FLOOR, floor_after);
	eech_sensor_f (EECH_SENSOR_TAG_FLOOR_INDEPENDENT, independent);
	if (float_bits (independent) != float_bits (floor_after))
	{
		trace->floor_cumulative_differs++;
		keep_sample (trace, "floor_cumulative");
	}
	if (cur.floor_initial > 0.0f)
	{
		ratio = floor_after / cur.floor_initial;
		if (ratio < trace->floor_ratio_min)
		{
			trace->floor_ratio_min = ratio;
		}
	}
}

void eech_sensor_floor_factor (float factor)
{
	struct sensor_trace *trace = ready ();
	if (!cur.active)
	{
		return;
	}
	cur.factor = factor;
	cur.factor_applied = 1;
	trace->difficulty_factors++;
	eech_sensor_f (EECH_SENSOR_TAG_DIFFICULTY_FACTOR, factor);
}

/*
 * The random tolerance of vh_tgt.c: the original draws frand1x (&seed) from a local seed that is the source's heap slot, so the draw is
 * a fixed value per source, the same for every candidate of every scan by that source. The hook repeats the draw on its own copy
 * and folds the draw and the limit; it never reports the seed.
 */
void eech_sensor_tolerance (entity *source, float floor, float radar_altitude)
{
	struct sensor_trace *trace = ready ();
	int seed;
	double draw, limit;
	if (!cur.active)
	{
		return;
	}
	seed = get_client_server_entity_random_number_seed (source);
	draw = frand1x (&seed);
	limit = floor * (1.0 + (0.5 * draw));
	fold_double (&trace->physical_hash, draw);
	fold_double (&trace->physical_hash, limit);
	text_append (cur.text, sizeof (cur.text), " tolerance_draw=%.17g tolerance_limit=%.17g", draw, limit);
	cur.tolerance_pending = radar_altitude < limit;
	trace->tolerance_draws++;
	if (draw < trace->tolerance_min)
	{
		trace->tolerance_min = draw;
	}
	if (draw > trace->tolerance_max)
	{
		trace->tolerance_max = draw;
	}
	if (!cur.draw_valid)
	{
		cur.draw_valid = 1;
		cur.draw_first = draw;
	}
	else if (draw != cur.draw_first)
	{
		trace->tolerance_varies_in_call++;
	}
}

void eech_sensor_verdict (int verdict)
{
	struct sensor_trace *trace = ready ();
	char key[40];
	if (!cur.active)
	{
		return;
	}
	trace->candidates[cur.function]++;
	trace->verdicts[cur.function][verdict]++;
	fold (&trace->physical_hash, 0xF0000000u | (uint32_t) verdict);
	cur.candidates++;
	if (cur.tolerance_pending >= 0)
	{
		/* the recomputed tolerance outcome must be the original's: it rejects exactly when the verdict says so */
		if ((verdict == EECH_SENSOR_VERDICT_TOLERANCE) != (cur.tolerance_pending == 1))
		{
			trace->tolerance_mismatch++;
		}
		if (verdict == EECH_SENSOR_VERDICT_TOLERANCE)
		{
			trace->tolerance_rejects_after_floor_pass++;
		}
	}
	if (verdict == EECH_SENSOR_VERDICT_ALTITUDE)
	{
		if (cur.radar_altitude < cur.floor_after)
		{
			trace->altitude_below_floor++;
		}
		else
		{
			trace->altitude_above_ceiling++;
		}
	}
	if (cur.player)
	{
		trace->player_candidates[cur.function]++;
	}
	if (cur.radar_on == 0 || cur.jammer_on == 0 || cur.ir_on == 0)
	{
		trace->emitter_off[cur.function]++;
	}
	if (cur.stealth != 1.0f)
	{
		trace->stealth_not_one[cur.function]++;
	}
	if (cur.adjusted != 0.0f && float_bits (cur.adjusted) != float_bits (cur.target_range))
	{
		trace->adjusted_differs[cur.function]++;
	}
	snprintf (key, sizeof (key), "%s/%s", function_names[cur.function], verdict_names[verdict]);
	keep_sample (trace, key);
	cur.tolerance_pending = -1;
}

void eech_sensor_end (int target_count, entity *best_group, float best_score)
{
	struct sensor_trace *trace = ready ();
	char key[40];
	if (!cur.active)
	{
		return;
	}
	trace->count_sum[cur.function] += (uint64_t) target_count;
	fold (&trace->decision_hash, 0xA0000000u | (uint32_t) target_count);
	fold (&trace->decision_hash, float_bits (best_score));
	if (cur.scanned)
	{
		trace->scanned[cur.function]++;
		note_scanned (trace, cur_source, cur.function);
		trace->sector_cells_sum[cur.function] += (uint64_t) (cur.sector_x_max - cur.sector_x_min + 1) * (uint64_t) (cur.sector_z_max - cur.sector_z_min + 1);
		if (cur.sector_x_min == MIN_MAP_X_SECTOR && cur.sector_x_max == MAX_MAP_X_SECTOR && cur.sector_z_min == MIN_MAP_Z_SECTOR
			&& cur.sector_z_max == MAX_MAP_Z_SECTOR)
		{
			trace->sector_whole_map[cur.function]++;
		}
	}
	if (cur.function == EECH_SENSOR_SURFACE_TO_AIR && cur.floor_evals >= 2)
	{
		trace->floor_multi_candidate_calls++;
	}
	if (best_group)
	{
		vec3d *position = get_local_entity_vec3d_ptr (best_group, VEC3D_TYPE_POSITION);
		trace->nonnull[cur.function]++;
		fold (&trace->decision_hash, 1u);
		fold (&trace->decision_hash, float_bits (position->x));
		fold (&trace->decision_hash, float_bits (position->y));
		fold (&trace->decision_hash, float_bits (position->z));
	}
	else
	{
		fold (&trace->decision_hash, 0u);
		if (target_count > 0)
		{
			trace->count_positive_null[cur.function]++;
			cur.candidate_open = 0;
			cur.text[0] = 0;
			text_append (cur.text, sizeof (cur.text), "target_count=%d best_group=none", target_count);
			snprintf (key, sizeof (key), "%s/count_without_group", function_names[cur.function]);
			keep_sample (trace, key);
		}
	}
	cur.active = 0;
	cur_source = NULL;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* report */

struct out
{
	char *text;
	int capacity, used, overflow;
};

static void emit (struct out *out, const char *format, ...) __attribute__ ((format (printf, 2, 3)));

static void emit (struct out *out, const char *format, ...)
{
	va_list args;
	int n;
	if (out->overflow)
	{
		return;
	}
	va_start (args, format);
	n = vsnprintf (out->text + out->used, (size_t) (out->capacity - out->used), format, args);
	va_end (args);
	if (n < 0 || n >= out->capacity - out->used)
	{
		out->overflow = 1;
		return;
	}
	out->used += n;
}

static int report (const struct sensor_trace *trace, char *text, int capacity)
{
	struct out out;
	int f, v, s, i;
	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	text[0] = 0;
	emit (&out, "sensor-hash physical=%016llx decision=%016llx\n", (unsigned long long) trace->physical_hash,
		(unsigned long long) trace->decision_hash);
	for (f = 0; f < F; f++)
	{
		emit (&out, "sensor-fn name=%s calls=%llu scanned=%llu nonnull=%llu target_count_sum=%llu count_positive_null_group=%llu candidates=%llu replaced=%llu\n",
			function_names[f], (unsigned long long) trace->calls[f], (unsigned long long) trace->scanned[f],
			(unsigned long long) trace->nonnull[f], (unsigned long long) trace->count_sum[f],
			(unsigned long long) trace->count_positive_null[f], (unsigned long long) trace->candidates[f],
			(unsigned long long) trace->replaced[f]);
		emit (&out, "sensor-verdicts name=%s", function_names[f]);
		for (v = 0; v < V; v++)
		{
			emit (&out, " %s=%llu", verdict_names[v], (unsigned long long) trace->verdicts[f][v]);
		}
		emit (&out, "\n");
		emit (&out, "sensor-skips name=%s", function_names[f]);
		for (s = 0; s < S; s++)
		{
			emit (&out, " %s=%llu", skip_names[s], (unsigned long long) trace->skips[f][s]);
		}
		emit (&out, "\n");
		emit (&out, "sensor-edges name=%s player_candidates=%llu emitter_off=%llu stealth_not_one=%llu adjusted_differs=%llu\n",
			function_names[f], (unsigned long long) trace->player_candidates[f], (unsigned long long) trace->emitter_off[f],
			(unsigned long long) trace->stealth_not_one[f], (unsigned long long) trace->adjusted_differs[f]);
		emit (&out, "sensor-geometry name=%s sector_cells_sum=%llu sector_whole_map=%llu cos_values=%llu cos_outside_unit=%llu cos_min=%.9g cos_max=%.9g\n",
			function_names[f], (unsigned long long) trace->sector_cells_sum[f], (unsigned long long) trace->sector_whole_map[f],
			(unsigned long long) trace->cos_seen[f], (unsigned long long) trace->cos_outside_unit[f],
			trace->cos_seen[f] ? trace->cos_min[f] : 0.0, trace->cos_seen[f] ? trace->cos_max[f] : 0.0);
	}
	emit (&out, "sensor-surface-to-air floor_cumulative_differs=%llu multi_candidate_calls=%llu altitude_below_floor=%llu altitude_above_ceiling=%llu "
		"difficulty_factors=%llu floor_ratio_min=%.9g\n",
		(unsigned long long) trace->floor_cumulative_differs, (unsigned long long) trace->floor_multi_candidate_calls,
		(unsigned long long) trace->altitude_below_floor, (unsigned long long) trace->altitude_above_ceiling,
		(unsigned long long) trace->difficulty_factors, (double) trace->floor_ratio_min);
	emit (&out, "sensor-tolerance draws=%llu recompute_mismatch=%llu varies_within_call=%llu rejects=%llu draw_min=%.9g draw_max=%.9g\n",
		(unsigned long long) trace->tolerance_draws, (unsigned long long) trace->tolerance_mismatch,
		(unsigned long long) trace->tolerance_varies_in_call, (unsigned long long) trace->tolerance_rejects_after_floor_pass,
		trace->tolerance_draws ? trace->tolerance_min : 0.0, trace->tolerance_draws ? trace->tolerance_max : 0.0);
	for (i = 0; i < trace->kind_count; i++)
	{
		emit (&out, "sensor-kind source=%s function=%s calls=%llu scanned=%llu\n",
			get_entity_type_name ((entity_types) trace->kinds[i].source_type), function_names[trace->kinds[i].function],
			(unsigned long long) trace->kinds[i].calls, (unsigned long long) trace->kinds[i].scanned);
	}
	emit (&out, "sensor-kinds rows=%d overflow=%d\n", trace->kind_count, trace->kind_overflow);
	for (i = 0; i < trace->sample_count; i++)
	{
		emit (&out, "sensor-sample key=%s %s\n", trace->sample_key[i], trace->samples[i].text);
	}
	emit (&out, "sensor-samples count=%d overflow=%d\n", trace->sample_count, trace->sample_overflow);
	return out.overflow ? -1 : 0;
}

int eech_sensor_trace_get (char *text, int capacity)
{
	ready ();
	return report (&run_trace, text, capacity);
}

void eech_sensor_trace_clear (void)
{
	trace_reset (&run_trace);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* probe */

extern entity *eech_sensor_probe_aircraft_scan (entity *source, int air_to_air, int *target_count);

static int allocated_entity (entity *en)
{
	return entities && en >= entities && en < entities + number_of_entities;
}

static int aircraft_source (entity *en)
{
	if (!allocated_entity (en))
	{
		return 0;
	}
	if (get_local_entity_type (en) != ENTITY_TYPE_HELICOPTER && get_local_entity_type (en) != ENTITY_TYPE_FIXED_WING)
	{
		return 0;
	}
	return get_local_entity_int_value (en, INT_TYPE_ALIVE) && get_local_entity_int_value (en, INT_TYPE_AIRBORNE_AIRCRAFT)
		&& get_local_entity_int_value (en, INT_TYPE_OFFENSIVE_CAPABILITY) && get_local_entity_float_value (en, FLOAT_TYPE_TARGET_SCAN_DELAY) != 0.0
		&& get_local_entity_parent (en, LIST_TYPE_MEMBER);
}

static int vehicle_source (entity *en)
{
	if (!allocated_entity (en))
	{
		return 0;
	}
	switch (get_local_entity_type (en))
	{
	case ENTITY_TYPE_ROUTED_VEHICLE:
	case ENTITY_TYPE_ANTI_AIRCRAFT:
	case ENTITY_TYPE_PERSON:
	case ENTITY_TYPE_SHIP_VEHICLE:
		break;
	default:
		return 0;
	}
	/* the caller lets only a group's first member scan */
	return get_local_entity_int_value (en, INT_TYPE_ALIVE) && get_local_entity_int_value (en, INT_TYPE_OFFENSIVE_CAPABILITY)
		&& get_local_entity_float_value (en, FLOAT_TYPE_TARGET_SCAN_DELAY) != 0.0 && get_local_entity_parent (en, LIST_TYPE_MEMBER)
		&& !get_local_entity_child_pred (en, LIST_TYPE_MEMBER);
}

/*
 * Runs, for every eligible live source on the engine's used-entity list, the two scan group functions its family's update calls.
 * Read-only: the scans write nothing, and nothing here engages, speaks or changes a timer. The result goes to a trace of its own and
 * the run-wide one is untouched. Returns 0 and writes the report, or -1 when it does not fit.
 */
int eech_sensor_probe_run (char *text, int capacity)
{
	static struct sensor_trace probe_trace;
	struct sensor_trace *saved = active_trace;
	entity *en;
	int sources = 0, aircraft = 0, vehicles = 0, count, rc;
	trace_reset (&probe_trace);
	active_trace = &probe_trace;
	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		if (!allocated_entity (en))
		{
			break;
		}
		if (aircraft_source (en))
		{
			aircraft++;
			(void) eech_sensor_probe_aircraft_scan (en, 1, &count);
			(void) eech_sensor_probe_aircraft_scan (en, 0, &count);
		}
		else if (vehicle_source (en))
		{
			vehicles++;
			(void) get_vehicle_surface_to_air_scan_group_targets (en, &count);
			(void) get_vehicle_surface_to_surface_scan_group_targets (en, &count);
		}
	}
	sources = aircraft + vehicles;
	active_trace = saved;
	rc = report (&probe_trace, text, capacity);
	if (rc == 0)
	{
		int used = (int) strlen (text);
		int n = snprintf (text + used, (size_t) (capacity - used), "sensor-probe sources=%d aircraft=%d vehicles=%d\n", sources, aircraft, vehicles);
		if (n < 0 || n >= capacity - used)
		{
			rc = -1;
		}
	}
	return rc;
}

#endif

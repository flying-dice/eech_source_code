/*
 * TEST-ONLY physical weapon-execution characterisation (EEDCS-45). Compiled only with the `weapon-fixture` feature; never part of the
 * shipping module.
 *
 * weapon/wn_creat.c, wn_move.c, wn_updt.c and wn_cllsn.c, built with EECH_WEAPON_FIXTURE, call the eech_wpn_* hooks declared in
 * entity/mobile/mb_wpnfx.h from inside the original weapon-execution code. The hooks only observe: they read arguments, results and
 * entity fields the original has already produced and fold them into five hashes, a set of counters, an ordered event log and a few
 * sample records, in the order the game computed them. Nothing here writes campaign state, draws from the game's random stream or
 * changes a branch.
 *
 *   table_hash     the boot-time projectile simulation that generates the ballistic tables (every simulated flight's pitch, its range-mark
 *                  fill and end time) and the tables themselves, as the exact float bits handed to the host (campaign_aim_supply_table),
 *                  and the end state of each guided-missile range simulation
 *   launch_hash    the physical birth of each weapon: velocity (parent contribution, muzzle velocity and its error), lifetime, decoy/fuze
 *                  timer, position and attitude (with the dispersion rotation), the burst, lock-on-after-launch and phase flags, the number
 *                  of draws it took from the shared random stream, and the parent that launched it
 *   flight_hash    every weapon_movement call: the branch it took (path mask), the guidance type and whether a target position was
 *                  produced, the intercept prediction and guided turn terms, the contact queries, and the position, velocity, motion
 *                  vector, attitude, lifetime, kill code and phase it left behind
 *   terminal_hash  each weapon's terminal fact: the physical kill code with the whole trajectory's own hash, step count, final state and
 *                  the parent/target kinds the combat code will consume next
 *   decision_hash  what the campaign does around the physics: salvo size, launch attempts and their outcome (no ammunition, rate-of-fire
 *                  gating, created, fired-at notification, ammunition accounting), guidance source and validity decisions, and the
 *                  per-update stage order
 *
 * No hash, event or sample carries an entity pointer, index, heap address or seed. The fixture numbers weapons itself, in birth order
 * (w=), from a private table keyed by the entity address; the key never leaves this file. The game's shared random stream is observed only
 * by the NUMBER of draws between two points (stepping a local copy of the generator), never its value.
 *
 * Outputs the original leaves unwritten on the path taken are never read: a guided target position is read only when get_target_position
 * reported it valid.
 */

#ifdef EECH_WEAPON_FIXTURE

#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"
#include "entity/mobile/mb_wpnfx.h"
#include "entity/mobile/mb_hostaim.h"

#define SAMPLE_SLOTS 2
#define SAMPLE_COUNT 256
#define SAMPLE_KEY 56
#define SAMPLE_TEXT 560
#define TRACKS 2048
#define TABLE_ROWS 160
#define RANGES 20
#define LOG_EVENTS 16384
#define LOG_TEXT 640

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

#define COUNTER_LIST(X) \
	X (table_steps) X (table_flights) X (table_flights_full) X (table_flights_short) X (table_tail_cells) X (table_supplies) X (table_cells) \
	X (table_cells_nonfinite) X (range_types) X (range_types_no_file) X (range_flights) X (range_flights_zero_pitch) X (range_flights_aimed) \
	X (salvo_calls) X (salvo_ai) X (salvo_not_ai) X (salvo_pods_over_one) \
	X (launch_calls) X (launch_no_ammo) X (launch_gated) X (launch_created) X (launch_create_failed) X (launch_fired_at) X (launch_attempts) \
	X (launch_with_target) X (launch_decoy_cargo_debris) X (launch_ammo_now_empty) \
	X (birth_calls) X (birth_submunition) X (birth_fuze_entered) X (birth_fuze_solved) X (birth_dispersion) X (birth_draws_0) X (birth_draws_1) \
	X (birth_draws_more) X (birth_track_overflow) \
	X (move_calls) X (move_unregistered) X (move_draws_0) X (move_draws_1) X (move_draws_more) X (move_target_position_valid) \
	X (move_target_position_invalid) X (tracking_point_valid) X (precision_high) X (precision_normal) X (ground_tests) X (ground_hits) \
	X (impact_water) X (impact_land) X (impact_other) X (impact_survive) X (submunition_spawns) X (guidance_source_calls) \
	X (guidance_source_no_launcher) X (guidance_source_target_changes) X (guidance_source_target_dropped) \
	X (tgt_valid) X (tgt_invalid) X (tgt_tracking_point) X (tgt_mobile_intercept) X (tgt_jammer_chaff) X (tgt_jammer_ir) X (tgt_smoke_distract) \
	X (tgt_decoy_intercept) X (intercept_calls) X (intercept_iterated) X (intercept_weapon_not_moving) X (intercept_target_not_moving) \
	X (intercept_target_not_slower) X (guided_too_close) X (guided_overshot) X (guided_target_lost_true) X (guided_target_lost_false) \
	X (guided_turned) X (contact_any_tests) X (contact_any_hits) X (contact_trees_tests) X (contact_trees_hits) X (decoy_updates) \
	X (decoy_restarts) X (update_stage_1) X (update_stage_2) X (update_stage_3) X (update_stage_4) X (update_stage_5) X (update_order_violations) \
	X (terminal_calls) X (terminal_unregistered) X (terminal_with_target) X (terminal_without_target) X (destroy_calls) \
	X (destroy_without_terminal) X (destroy_unregistered) X (table_diagnostics_absent) X (range_diagnostics_absent) X (log_events)

enum counter
{
#define X(n) C_##n,
	COUNTER_LIST (X)
#undef X
	COUNTERS
};

static const char *const counter_names[COUNTERS] = {
#define X(n) #n,
	COUNTER_LIST (X)
#undef X
};

enum range_id
{
	R_BIRTH_VELOCITY,
	R_BIRTH_LIFETIME,
	R_BIRTH_DECOY_TIMER,
	R_BIRTH_DISPERSION_HEADING,
	R_BIRTH_DISPERSION_PITCH,
	R_MOVE_VELOCITY,
	R_MOVE_LIFETIME,
	R_MOVE_STEP_DISTANCE,
	R_GUIDED_TURN_DEMAND,
	R_GUIDED_TURN_RATE,
	R_GUIDED_LENGTH,
	R_INTERCEPT_VELOCITY_RATIO,
	R_TERMINAL_STEPS,
	R_TERMINAL_VELOCITY,
	R_TERMINAL_DISTANCE,
	R_TABLE_DROP_ANGLE,
	R_TABLE_FLIGHT_TIME,
	R_RANGE_END_Z,
	R_FUZE_DECOY_TIMER,
	R_CONTACT_SEGMENT,
	RANGE_IDS
};

static const char *const range_names[RANGES] = {
	"birth_velocity", "birth_lifetime", "birth_decoy_timer", "birth_dispersion_heading", "birth_dispersion_pitch", "move_velocity", "move_lifetime",
	"move_step_distance", "guided_turn_demand", "guided_turn_rate", "guided_length", "intercept_velocity_ratio", "terminal_steps", "terminal_velocity",
	"terminal_distance", "table_drop_angle", "table_flight_time", "range_end_z", "fuze_decoy_timer", "contact_segment"
};

static const char *const path_names[EECH_WPN_PATHS] = {
	"fuze_timer", "exhausted", "on_ground", "guided", "unguided", "loal_climb_dive", "killed_by_guidance", "contact_target", "contact_trees",
	"out_of_map_area", "above_ceiling", "ground_water", "ground_survived", "ground_smoke_grenade", "ground_land"
};

static const char *const guidance_names[NUM_WEAPON_GUIDANCE_TYPES] = {
	"none", "passive_infra_red", "semi_active_radar", "active_radar", "semi_active_laser", "beam_riding", "active_laser", "radio_command", "wire_guided"
};

static const char *const guidance_keys[NUM_WEAPON_GUIDANCE_TYPES] = { "none", "ir", "sar", "ar", "sal", "br", "al", "rc", "wire" };

static const char *const kill_names[NUM_WEAPON_KILL_CODES] = {
	"ok", "exhausted", "out_of_bounds", "hit_land", "hit_water", "hit_target", "overshot_target", "self_destruct"
};

static const char *const outcome_names[4] = { "created", "no_ammo", "gated", "create_failed" };

struct sample
{
	char key[SAMPLE_KEY];
	char text[SAMPLE_TEXT];
};

struct track
{
	const entity *en;
	int used, ordinal, sub_type, steps, terminal_seen, launcher_type, target_type, path_union;
	uint64_t hash;
	double distance;
	float max_velocity, last_x, last_y, last_z, last_lifetime;
	int on_ground;
};

struct table_row
{
	int sub_type, velocity_tests, pitch_indices, num_range_values;
	char name[48];
	float max_range;
	uint64_t hash, cells, nonzero, nonfinite, flights, flights_full, flights_short, steps, supplied;
};

struct weapon_row
{
	uint64_t births, submunitions, moves, moves_guided, moves_unguided, moves_loal, kills[NUM_WEAPON_KILL_CODES];
};

struct fixture_trace
{
	uint64_t table_hash, launch_hash, flight_hash, terminal_hash, decision_hash;
	uint64_t counters[COUNTERS];
	uint64_t paths[EECH_WPN_PATHS];
	uint64_t guidance[NUM_WEAPON_GUIDANCE_TYPES][2];
	uint64_t outcomes[4];
	uint64_t kills[NUM_WEAPON_KILL_CODES];
	uint64_t terminal_by_launcher[NUM_ENTITY_TYPES];
	uint64_t birth_by_launcher[NUM_ENTITY_TYPES];
	double range_min[RANGES], range_max[RANGES];
	uint64_t range_n[RANGES];
	int table_count, table_overflow, sample_count, sample_overflow, next_ordinal;
	struct table_row tables[TABLE_ROWS];
	struct weapon_row weapons[NUM_ENTITY_SUB_TYPE_WEAPONS];
	struct sample samples[SAMPLE_COUNT];
};

static struct fixture_trace trace;
static struct track tracks[TRACKS];
static int track_high;

/* the ordered event log: births, terminals, destroys, launch outcomes, table summaries; drained incrementally (eech_wpn_events_get) */
static char log_pool[LOG_EVENTS][LOG_TEXT];
static int log_count, log_drained, log_overflow;
static uint64_t log_seq;

/* the calls in flight */
static struct
{
	int active, sub_type, launcher_type, attempted, created, no_ammo, fired_at, current_count, new_count, burst_size, has_target, decoy, births;
} cur_launch;

static struct
{
	int active, rng0, fuze_entered, fuze_rng0, fuze_solved, fuze_draws, dispersion;
	float fuze_timer, disp_heading, disp_pitch;
} cur_birth;

static struct
{
	int active, rng0, sub_type, guidance, valid, tracking, precision, ground, impact_class, path_mask, tgt_mask, contact_hits;
	float old_x, old_y, old_z, old_velocity, old_lifetime;
	struct track *track;
} cur_move;

static int last_stage;
static int draws_total;

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

static void fold_f (uint64_t *hash, float value)
{
	fold (hash, float_bits (value));
}

static void fold_vec (uint64_t *hash, const vec3d *v)
{
	fold_f (hash, v->x);
	fold_f (hash, v->y);
	fold_f (hash, v->z);
}

static void fold_matrix (uint64_t *hash, const float (*m)[3])
{
	int row, column;
	for (row = 0; row < 3; row++)
	{
		for (column = 0; column < 3; column++)
		{
			fold_f (hash, m[row][column]);
		}
	}
}

static void trace_reset (void)
{
	int i;
	memset (&trace, 0, sizeof (trace));
	memset (tracks, 0, sizeof (tracks));
	memset (&cur_launch, 0, sizeof (cur_launch));
	memset (&cur_birth, 0, sizeof (cur_birth));
	memset (&cur_move, 0, sizeof (cur_move));
	track_high = 0;
	log_count = log_drained = log_overflow = 0;
	log_seq = 0;
	last_stage = 0;
	trace.table_hash = trace.launch_hash = trace.flight_hash = trace.terminal_hash = trace.decision_hash = FNV_OFFSET;
	for (i = 0; i < RANGES; i++)
	{
		trace.range_min[i] = 1e30;
		trace.range_max[i] = -1e30;
	}
}

static void ready (void)
{
	if (!trace.table_hash)
	{
		trace_reset ();
	}
}

static void bump (int counter)
{
	trace.counters[counter]++;
}

static void note_range (int id, double value)
{
	if (!(value == value) || value > 1e300 || value < -1e300)
	{
		return;
	}
	trace.range_n[id]++;
	if (value < trace.range_min[id])
	{
		trace.range_min[id] = value;
	}
	if (value > trace.range_max[id])
	{
		trace.range_max[id] = value;
	}
}

static void keep_sample (const char *key, const char *text)
{
	int i, matches = 0;
	for (i = 0; i < trace.sample_count; i++)
	{
		if (strcmp (trace.samples[i].key, key) == 0)
		{
			matches++;
		}
	}
	if (matches >= SAMPLE_SLOTS)
	{
		return;
	}
	if (trace.sample_count >= SAMPLE_COUNT)
	{
		trace.sample_overflow++;
		return;
	}
	snprintf (trace.samples[trace.sample_count].key, SAMPLE_KEY, "%s", key);
	snprintf (trace.samples[trace.sample_count].text, SAMPLE_TEXT, "%s", text);
	trace.sample_count++;
}

static void log_event (const char *format, ...) __attribute__ ((format (printf, 1, 2)));

static void log_event (const char *format, ...)
{
	va_list args;
	int used, n;
	log_seq++;
	bump (C_log_events);
	if (log_count >= LOG_EVENTS)
	{
		log_overflow++;
		return;
	}
	used = snprintf (log_pool[log_count], LOG_TEXT, "wpn-ev seq=%llu ", (unsigned long long) log_seq);
	va_start (args, format);
	n = vsnprintf (log_pool[log_count] + used, (size_t) (LOG_TEXT - used), format, args);
	va_end (args);
	if (n < 0 || n >= LOG_TEXT - used)
	{
		/* an event that does not fit is a defect of the fixture, never silently cut: mark it so the comparator refuses the trace */
		log_overflow++;
		snprintf (log_pool[log_count], LOG_TEXT, "wpn-ev seq=%llu kind=truncated", (unsigned long long) log_seq);
	}
	log_count++;
}

static const char *type_name (int type)
{
	if (type < 0)
	{
		return "none";
	}
	return get_entity_type_name ((entity_types) type);
}

static const char *weapon_name (int sub_type, char *buffer, size_t capacity)
{
	size_t n;
	const char *name = (sub_type > 0 && sub_type < NUM_ENTITY_SUB_TYPE_WEAPONS && weapon_database[sub_type].full_name) ? weapon_database[sub_type].full_name : "none";
	snprintf (buffer, capacity, "%s", name);
	for (n = 0; buffer[n]; n++)
	{
		if (buffer[n] == ' ' || buffer[n] == '=')
		{
			buffer[n] = '_';
		}
	}
	return buffer;
}

static int valid_sub_type (int sub_type)
{
	return sub_type > 0 && sub_type < NUM_ENTITY_SUB_TYPE_WEAPONS;
}

/* how many draws the shared stream made between two of its states (never the states themselves) */
static int draws_between (int before, int after)
{
	uint32_t s = (uint32_t) before;
	int k;
	if (before == after)
	{
		return 0;
	}
	for (k = 1; k <= 64; k++)
	{
		s = s * 1103515245u + 12345u;
		if ((int) s == after)
		{
			return k;
		}
	}
	return 65;
}

static int guidance_of (int sub_type)
{
	return valid_sub_type (sub_type) ? (int) weapon_database[sub_type].guidance_type : 0;
}

static struct track *track_find (const entity *en)
{
	int i;
	for (i = 0; i < track_high; i++)
	{
		if (tracks[i].used && tracks[i].en == en)
		{
			return &tracks[i];
		}
	}
	return NULL;
}

static struct track *track_new (const entity *en)
{
	int i;
	for (i = 0; i < TRACKS; i++)
	{
		if (!tracks[i].used)
		{
			memset (&tracks[i], 0, sizeof (tracks[i]));
			tracks[i].used = 1;
			tracks[i].en = en;
			tracks[i].ordinal = ++trace.next_ordinal;
			tracks[i].hash = FNV_OFFSET;
			if (i >= track_high)
			{
				track_high = i + 1;
			}
			return &tracks[i];
		}
	}
	return NULL;
}

static struct table_row *table_row_for (int sub_type)
{
	int i;
	for (i = 0; i < trace.table_count; i++)
	{
		if (trace.tables[i].sub_type == sub_type)
		{
			return &trace.tables[i];
		}
	}
	if (trace.table_count < TABLE_ROWS)
	{
		struct table_row *row = &trace.tables[trace.table_count++];
		memset (row, 0, sizeof (*row));
		row->sub_type = sub_type;
		row->hash = FNV_OFFSET;
		weapon_name (sub_type, row->name, sizeof (row->name));
		return row;
	}
	trace.table_overflow++;
	return NULL;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* ballistic table generation (boot) */

static int table_flight_steps;

void eech_wpn_table_flight (int sub_type, int velocity_test, int pitch_index, int range_mark, int num_range_values, float end_time)
{
	struct table_row *row;
	int full = range_mark == num_range_values;
	ready ();
	bump (C_table_flights);
	bump (full ? C_table_flights_full : C_table_flights_short);
	trace.counters[C_table_tail_cells] += (uint64_t) (num_range_values - range_mark);
	fold (&trace.table_hash, 0xA1000000u | (uint32_t) (velocity_test & 0xff));
	fold (&trace.table_hash, (uint32_t) sub_type);
	fold (&trace.table_hash, (uint32_t) pitch_index);
	fold (&trace.table_hash, (uint32_t) range_mark);
	fold (&trace.table_hash, (uint32_t) table_flight_steps);
	fold_f (&trace.table_hash, end_time);
	row = table_row_for (sub_type);
	if (row)
	{
		row->flights++;
		row->steps += (uint64_t) table_flight_steps;
		if (full)
		{
			row->flights_full++;
		}
		else
		{
			row->flights_short++;
		}
	}
	table_flight_steps = 0;
}

void eech_wpn_table_supply (int sub_type, float max_range, int velocity_tests, int pitch_indices, int num_range_values, const float *cells, unsigned long long count)
{
	struct table_row *row = table_row_for (sub_type);
	unsigned long long i;
	uint64_t hash = FNV_OFFSET;
	uint64_t nonzero = 0, nonfinite = 0;
	char name[48], text[LOG_TEXT];
	ready ();
	bump (C_table_supplies);
	for (i = 0; i < count; i++)
	{
		float value = cells[i];
		fold_f (&hash, value);
		if (value != 0.0f)
		{
			nonzero++;
		}
		if (!(value == value) || value > 3.0e38f || value < -3.0e38f)
		{
			nonfinite++;
		}
		else if ((i & 1u) == 0)
		{
			note_range (R_TABLE_DROP_ANGLE, (double) value);
		}
		else
		{
			note_range (R_TABLE_FLIGHT_TIME, (double) value);
		}
	}
	trace.counters[C_table_cells] += count;
	trace.counters[C_table_cells_nonfinite] += nonfinite;
	fold (&trace.table_hash, 0xA2000000u | (uint32_t) sub_type);
	fold_f (&trace.table_hash, max_range);
	fold (&trace.table_hash, (uint32_t) velocity_tests);
	fold (&trace.table_hash, (uint32_t) pitch_indices);
	fold (&trace.table_hash, (uint32_t) num_range_values);
	fold (&trace.table_hash, (uint32_t) (hash & 0xffffffffu));
	fold (&trace.table_hash, (uint32_t) (hash >> 32));
	if (row)
	{
		row->velocity_tests = velocity_tests;
		row->pitch_indices = pitch_indices;
		row->num_range_values = num_range_values;
		row->max_range = max_range;
		row->cells = count;
		row->hash = hash;
		row->nonzero = nonzero;
		row->nonfinite = nonfinite;
		row->supplied++;
	}
	/* representative cells: velocity test 0 and the last, the level-pitch row, at a quarter, half and three quarters of the range */
	text[0] = 0;
	text_append (text, sizeof (text), "kind=table type=%s max_range=%.9g dims=%d,%d,%d cells=%llu hash=%016llx nonzero=%llu nonfinite=%llu", weapon_name (sub_type, name, sizeof (name)),
		(double) max_range, velocity_tests, pitch_indices, num_range_values, count, (unsigned long long) hash, (unsigned long long) nonzero, (unsigned long long) nonfinite);
	if (velocity_tests > 0 && pitch_indices > 2 && num_range_values > 4)
	{
		int tests[2], t, q;
		tests[0] = 0;
		tests[1] = velocity_tests - 1;
		for (t = 0; t < 2; t++)
		{
			for (q = 1; q <= 3; q++)
			{
				unsigned long long at = 2ull * (((unsigned long long) tests[t] * pitch_indices + (unsigned long long) (pitch_indices / 2)) * num_range_values
					+ (unsigned long long) (num_range_values * q / 4));
				if (at + 1 < count)
				{
					text_append (text, sizeof (text), " c%d_%d=%.9g,%.9g", tests[t], q, (double) cells[at], (double) cells[at + 1]);
				}
			}
		}
	}
	log_event ("%s", text);
}

void eech_wpn_range_type (int sub_type, int output_open)
{
	(void) sub_type;
	ready ();
	bump (C_range_types);
	if (!output_open)
	{
		bump (C_range_types_no_file);
	}
}

void eech_wpn_range_flight (int sub_type, int zero_pitch, float end_z, float end_y, float end_velocity, float end_lifetime)
{
	char name[48];
	ready ();
	bump (C_range_flights);
	bump (zero_pitch ? C_range_flights_zero_pitch : C_range_flights_aimed);
	fold (&trace.table_hash, 0xA3000000u | (uint32_t) (zero_pitch ? 1 : 0));
	fold (&trace.table_hash, (uint32_t) sub_type);
	fold_f (&trace.table_hash, end_z);
	fold_f (&trace.table_hash, end_y);
	fold_f (&trace.table_hash, end_velocity);
	fold_f (&trace.table_hash, end_lifetime);
	note_range (R_RANGE_END_Z, (double) end_z);
	log_event ("kind=range type=%s zero_pitch=%d end_z=%.9g end_y=%.9g end_velocity=%.9g end_lifetime=%.9g", weapon_name (sub_type, name, sizeof (name)), zero_pitch,
		(double) end_z, (double) end_y, (double) end_velocity, (double) end_lifetime);
}

/* The host generated the tables (EEDCS-45): it echoes one record per simulated flight and the table's cells; they are replayed here into the folds the original made while
 * it simulated (every flight, with the steps it took, as the hooks inside the simulation saw them), and the table itself is folded after the pass, as the original handed
 * it to the host after the pass. The cells are kept (a copy) until then. */
static float *saved_cells[NUM_ENTITY_SUB_TYPE_WEAPONS];
static unsigned long long saved_count[NUM_ENTITY_SUB_TYPE_WEAPONS];
static float saved_max_range[NUM_ENTITY_SUB_TYPE_WEAPONS];

void eech_wpn_table_host (int sub_type, float max_range, const struct campaign_aim_event *events, int event_count, const float *cells, unsigned long long cell_count)
{
	int i;
	ready ();
	if (event_count == 0 && cell_count == 0)
	{
		/* a host with a model of its own echoes nothing: that is recorded as absent, not invented, and the reference-evidence checker refuses such a trace */
		bump (C_table_diagnostics_absent);
		return;
	}
	for (i = 0; i < event_count; i++)
	{
		const struct campaign_aim_event *e = &events[i];
		if (e->kind == CAMPAIGN_AIM_EVENT_TABLE_FLIGHT)
		{
			trace.counters[C_table_steps] += (uint64_t) e->d;
			table_flight_steps = e->d;
			eech_wpn_table_flight (sub_type, e->a, e->b, e->c, (int) e->f[1], e->f[0]);
		}
	}
	if (valid_sub_type (sub_type))
	{
		free (saved_cells[sub_type]);
		saved_cells[sub_type] = (float *) malloc ((size_t) cell_count * sizeof (float));
		if (saved_cells[sub_type])
		{
			memcpy (saved_cells[sub_type], cells, (size_t) cell_count * sizeof (float));
		}
		saved_count[sub_type] = saved_cells[sub_type] ? cell_count : 0;
		saved_max_range[sub_type] = max_range;
	}
}

void eech_wpn_table_supplied (int sub_type)
{
	if (valid_sub_type (sub_type) && saved_cells[sub_type])
	{
		unsigned long long range_values = saved_count[sub_type] / (2ull * 5ull * 85ull);
		eech_wpn_table_supply (sub_type, saved_max_range[sub_type], 5, 85, (int) range_values, saved_cells[sub_type], saved_count[sub_type]);
		free (saved_cells[sub_type]);
		saved_cells[sub_type] = NULL;
		saved_count[sub_type] = 0;
	}
}

void eech_wpn_range_host (int sub_type, const struct campaign_aim_event *events, int event_count)
{
	int i;
	ready ();
	if (event_count == 0)
	{
		bump (C_range_diagnostics_absent);
		return;
	}
	for (i = 0; i < event_count; i++)
	{
		const struct campaign_aim_event *e = &events[i];
		if (e->kind == CAMPAIGN_AIM_EVENT_RANGE_RESULT)
		{
			if (i == 0)
			{
				eech_wpn_range_type (sub_type, e->c);
			}
			if (e->b)
			{
				eech_wpn_range_flight (sub_type, e->a, e->f[0], e->f[1], e->f[2], e->f[3]);
			}
		}
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* launch: the campaign's salvo and burst policy around the physical attempt */

void eech_wpn_salvo (entity *launcher, int sub_type, int salvo_size, unsigned pods_firing, int ai_launcher)
{
	(void) launcher;
	ready ();
	bump (C_salvo_calls);
	bump (ai_launcher ? C_salvo_ai : C_salvo_not_ai);
	if (pods_firing > 1)
	{
		bump (C_salvo_pods_over_one);
	}
	fold (&trace.decision_hash, 0xC1000000u | (uint32_t) (ai_launcher ? 1 : 0));
	fold (&trace.decision_hash, (uint32_t) sub_type);
	fold (&trace.decision_hash, (uint32_t) salvo_size);
	fold (&trace.decision_hash, (uint32_t) pods_firing);
}

void eech_wpn_launch_begin (entity *launcher, int sub_type)
{
	ready ();
	bump (C_launch_calls);
	memset (&cur_launch, 0, sizeof (cur_launch));
	cur_launch.active = 1;
	cur_launch.sub_type = sub_type;
	cur_launch.launcher_type = launcher ? (int) launcher->type : -1;
}

void eech_wpn_launch_no_ammo (void)
{
	if (cur_launch.active)
	{
		cur_launch.no_ammo = 1;
	}
}

void eech_wpn_launch_attempt (int current_count, int burst_size, int has_target)
{
	if (!cur_launch.active)
	{
		return;
	}
	cur_launch.attempted = 1;
	cur_launch.current_count = current_count;
	cur_launch.burst_size = burst_size;
	cur_launch.has_target = has_target;
	bump (C_launch_attempts);
	if (has_target)
	{
		bump (C_launch_with_target);
	}
}

void eech_wpn_launch_created (int created)
{
	if (cur_launch.active)
	{
		cur_launch.created = created ? 1 : 0;
	}
}

void eech_wpn_launch_fired_at (void)
{
	if (cur_launch.active)
	{
		cur_launch.fired_at = 1;
		bump (C_launch_fired_at);
	}
}

void eech_wpn_launch_ammo (int current_count, int new_count, int decoy)
{
	if (cur_launch.active)
	{
		cur_launch.current_count = current_count;
		cur_launch.new_count = new_count;
		cur_launch.decoy = decoy ? 1 : 0;
		if (decoy)
		{
			bump (C_launch_decoy_cargo_debris);
		}
		if (new_count <= 0)
		{
			bump (C_launch_ammo_now_empty);
		}
	}
}

void eech_wpn_launch_end (void)
{
	int outcome;
	char name[48];
	if (!cur_launch.active)
	{
		return;
	}
	if (cur_launch.no_ammo)
	{
		outcome = 1;
		bump (C_launch_no_ammo);
	}
	else if (!cur_launch.attempted)
	{
		outcome = 2;
		bump (C_launch_gated);
	}
	else if (cur_launch.created)
	{
		outcome = 0;
		bump (C_launch_created);
	}
	else
	{
		outcome = 3;
		bump (C_launch_create_failed);
	}
	trace.outcomes[outcome]++;
	fold (&trace.decision_hash, 0xC2000000u | (uint32_t) outcome);
	fold (&trace.decision_hash, (uint32_t) cur_launch.sub_type);
	fold (&trace.decision_hash, (uint32_t) cur_launch.burst_size);
	fold (&trace.decision_hash, (uint32_t) cur_launch.has_target);
	fold (&trace.decision_hash, (uint32_t) cur_launch.fired_at);
	fold (&trace.decision_hash, (uint32_t) cur_launch.current_count);
	fold (&trace.decision_hash, (uint32_t) cur_launch.new_count);
	if (outcome != 2)
	{
		/* a rate-of-fire gate is a hash fold and a counter only; every other outcome is an ordered event */
		log_event ("kind=launch outcome=%s launcher=%s type=%s burst=%d has_target=%d fired_at=%d ammo_before=%d ammo_after=%d decoy_cargo_debris=%d",
			outcome_names[outcome], type_name (cur_launch.launcher_type), weapon_name (cur_launch.sub_type, name, sizeof (name)), cur_launch.burst_size,
			cur_launch.has_target, cur_launch.fired_at, cur_launch.current_count, cur_launch.new_count, cur_launch.decoy);
	}
	cur_launch.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* birth (wn_creat.c create_local) */

void eech_wpn_birth_begin (void)
{
	ready ();
	memset (&cur_birth, 0, sizeof (cur_birth));
	cur_birth.active = 1;
	cur_birth.rng0 = random_number_seed;
}

void eech_wpn_fuze_begin (void)
{
	if (cur_birth.active)
	{
		cur_birth.fuze_entered = 1;
		cur_birth.fuze_rng0 = random_number_seed;
		bump (C_birth_fuze_entered);
	}
}

void eech_wpn_fuze_solved (float decoy_timer)
{
	if (cur_birth.active)
	{
		cur_birth.fuze_solved = 1;
		cur_birth.fuze_timer = decoy_timer;
		cur_birth.fuze_draws = draws_between (cur_birth.fuze_rng0, random_number_seed);
		bump (C_birth_fuze_solved);
		note_range (R_FUZE_DECOY_TIMER, (double) decoy_timer);
	}
}

void eech_wpn_dispersion (float heading, float pitch)
{
	if (cur_birth.active)
	{
		cur_birth.dispersion = 1;
		cur_birth.disp_heading = heading;
		cur_birth.disp_pitch = pitch;
		bump (C_birth_dispersion);
		note_range (R_BIRTH_DISPERSION_HEADING, (double) heading);
		note_range (R_BIRTH_DISPERSION_PITCH, (double) pitch);
	}
}

void eech_wpn_birth_end (entity *en, const weapon *raw, int submunition)
{
	struct track *t;
	char text[LOG_TEXT], name[48], key[SAMPLE_KEY];
	int draws, sub = (int) raw->mob.sub_type;
	int launcher_type = raw->launched_weapon_link.parent ? (int) raw->launched_weapon_link.parent->type : -1;
	int target_type = raw->mob.target_link.parent ? (int) raw->mob.target_link.parent->type : -1;
	ready ();
	if (!cur_birth.active)
	{
		return;
	}
	draws = draws_between (cur_birth.rng0, random_number_seed);
	cur_birth.active = 0;
	bump (C_birth_calls);
	if (submunition)
	{
		bump (C_birth_submunition);
	}
	bump (draws == 0 ? C_birth_draws_0 : (draws == 1 ? C_birth_draws_1 : C_birth_draws_more));
	draws_total += draws;
	cur_launch.births += cur_launch.active ? 1 : 0;
	if (valid_sub_type (sub))
	{
		trace.weapons[sub].births++;
		if (submunition)
		{
			trace.weapons[sub].submunitions++;
		}
	}
	if (launcher_type >= 0 && launcher_type < NUM_ENTITY_TYPES)
	{
		trace.birth_by_launcher[launcher_type]++;
	}
	t = track_new (en);
	if (!t)
	{
		bump (C_birth_track_overflow);
	}
	else
	{
		t->sub_type = sub;
		t->launcher_type = launcher_type;
		t->target_type = target_type;
		t->last_x = raw->mob.position.x;
		t->last_y = raw->mob.position.y;
		t->last_z = raw->mob.position.z;
		t->max_velocity = raw->mob.velocity;
		/* a weapon that has not moved yet reports the state it was born with, not zero */
		t->last_lifetime = raw->weapon_lifetime;
		t->on_ground = raw->weapon_on_ground ? 1 : 0;
	}
	fold (&trace.launch_hash, 0xB1000000u | (uint32_t) (submunition ? 1 : 0));
	fold (&trace.launch_hash, (uint32_t) sub);
	fold (&trace.launch_hash, (uint32_t) launcher_type);
	fold (&trace.launch_hash, (uint32_t) (target_type >= 0 ? 1 : 0));
	fold_f (&trace.launch_hash, raw->mob.velocity);
	fold_f (&trace.launch_hash, raw->weapon_lifetime);
	fold_f (&trace.launch_hash, raw->decoy_timer);
	fold_vec (&trace.launch_hash, &raw->mob.position);
	fold_matrix (&trace.launch_hash, raw->mob.attitude);
	fold (&trace.launch_hash, (uint32_t) raw->burst_size);
	fold (&trace.launch_hash, (uint32_t) raw->loal_mode);
	fold (&trace.launch_hash, (uint32_t) raw->missile_phase);
	fold (&trace.launch_hash, (uint32_t) raw->weapon_on_ground);
	fold (&trace.launch_hash, (uint32_t) raw->kill_code);
	fold (&trace.launch_hash, (uint32_t) draws);
	fold (&trace.launch_hash, (uint32_t) (cur_birth.fuze_solved | (cur_birth.fuze_entered << 1) | (cur_birth.dispersion << 2)));
	fold_f (&trace.launch_hash, cur_birth.fuze_timer);
	fold_f (&trace.launch_hash, cur_birth.disp_heading);
	fold_f (&trace.launch_hash, cur_birth.disp_pitch);
	note_range (R_BIRTH_VELOCITY, (double) raw->mob.velocity);
	note_range (R_BIRTH_LIFETIME, (double) raw->weapon_lifetime);
	note_range (R_BIRTH_DECOY_TIMER, (double) raw->decoy_timer);
	text[0] = 0;
	text_append (text, sizeof (text),
		"kind=birth w=%d type=%s launcher=%s target=%s submunition=%d velocity=%.9g lifetime=%.9g decoy_timer=%.9g pos=%.9g,%.9g,%.9g "
		"att=%.9g,%.9g,%.9g,%.9g,%.9g,%.9g,%.9g,%.9g,%.9g burst=%d loal=%d phase=%d fuze=%d fuze_solved=%d fuze_timer=%.9g fuze_draws=%d dispersion=%d disp=%.9g,%.9g draws=%d cruise_time=%.9g cruise_error=%.9g survive_ground=%d",
		t ? t->ordinal : -1, weapon_name (sub, name, sizeof (name)), type_name (launcher_type), type_name (target_type), submunition, (double) raw->mob.velocity,
		(double) raw->weapon_lifetime, (double) raw->decoy_timer, (double) raw->mob.position.x, (double) raw->mob.position.y, (double) raw->mob.position.z,
		(double) raw->mob.attitude[0][0], (double) raw->mob.attitude[0][1], (double) raw->mob.attitude[0][2], (double) raw->mob.attitude[1][0],
		(double) raw->mob.attitude[1][1], (double) raw->mob.attitude[1][2], (double) raw->mob.attitude[2][0], (double) raw->mob.attitude[2][1],
		(double) raw->mob.attitude[2][2], (int) raw->burst_size, (int) raw->loal_mode, (int) raw->missile_phase, cur_birth.fuze_entered, cur_birth.fuze_solved,
		(double) cur_birth.fuze_timer, cur_birth.fuze_draws, cur_birth.dispersion, (double) cur_birth.disp_heading, (double) cur_birth.disp_pitch, draws,
		valid_sub_type (sub) ? (double) weapon_database[sub].cruise_time : -1.0, valid_sub_type (sub) ? (double) weapon_database[sub].cruise_time_max_error : -1.0,
		valid_sub_type (sub) ? (int) weapon_database[sub].survive_ground_impact : -1);
	log_event ("%s", text);
	snprintf (key, sizeof (key), "birth/%s", submunition ? "submunition" : (cur_birth.fuze_solved ? "fuze_solved" : (cur_birth.dispersion ? "dispersed" : "plain")));
	keep_sample (key, text + 5);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* flight: weapon_movement (wrapped) and its branches */

void eech_wpn_move_begin (entity *en)
{
	ready ();
	bump (C_move_calls);
	memset (&cur_move, 0, sizeof (cur_move));
	cur_move.active = 1;
	cur_move.rng0 = random_number_seed;
	cur_move.track = track_find (en);
	cur_move.guidance = -1;
	cur_move.valid = -1;
	cur_move.precision = -1;
	cur_move.ground = -1;
	last_stage = 0;
	fold (&trace.decision_hash, 0xD0000000u);
	if (cur_move.track)
	{
		cur_move.sub_type = cur_move.track->sub_type;
	}
	else
	{
		bump (C_move_unregistered);
		cur_move.sub_type = (int) ((const weapon *) get_local_entity_data (en))->mob.sub_type;
	}
}

void eech_wpn_path (int path)
{
	if (!cur_move.active)
	{
		return;
	}
	cur_move.path_mask |= 1 << path;
	trace.paths[path]++;
}

void eech_wpn_submunitions (int count)
{
	if (cur_move.active)
	{
		bump (C_submunition_spawns);
		trace.counters[C_submunition_spawns] += (uint64_t) (count > 0 ? count - 1 : 0);
		fold (&trace.flight_hash, 0xF9000000u | (uint32_t) count);
	}
}

void eech_wpn_tracking (int tracking_point)
{
	if (cur_move.active)
	{
		cur_move.tracking = tracking_point ? 1 : 0;
		if (tracking_point)
		{
			bump (C_tracking_point_valid);
		}
	}
}

void eech_wpn_guidance (int guidance_type, int intercept_point_valid)
{
	if (!cur_move.active)
	{
		return;
	}
	cur_move.guidance = guidance_type;
	cur_move.valid = intercept_point_valid ? 1 : 0;
	if (guidance_type >= 0 && guidance_type < NUM_WEAPON_GUIDANCE_TYPES)
	{
		trace.guidance[guidance_type][cur_move.valid]++;
	}
	if (guidance_type != WEAPON_GUIDANCE_TYPE_NONE)
	{
		bump (cur_move.valid ? C_move_target_position_valid : C_move_target_position_invalid);
	}
	fold (&trace.decision_hash, 0xD1000000u | (uint32_t) (guidance_type << 4) | (uint32_t) cur_move.valid);
}

void eech_wpn_guidance_source (int has_launcher, int target_changes, int new_target_present)
{
	ready ();
	bump (C_guidance_source_calls);
	if (!has_launcher)
	{
		bump (C_guidance_source_no_launcher);
	}
	if (target_changes)
	{
		bump (C_guidance_source_target_changes);
		if (!new_target_present)
		{
			bump (C_guidance_source_target_dropped);
		}
	}
	fold (&trace.decision_hash, 0xD2000000u | (uint32_t) (has_launcher ? 4 : 0) | (uint32_t) (target_changes ? 2 : 0) | (uint32_t) (new_target_present ? 1 : 0));
}

void eech_wpn_tgt_branch (int branch, int taken)
{
	if (!taken || !cur_move.active)
	{
		return;
	}
	cur_move.tgt_mask |= 1 << branch;
	bump (C_tgt_tracking_point + branch);
}

void eech_wpn_tgt_end (int valid, const vec3d *position)
{
	if (!cur_move.active)
	{
		return;
	}
	bump (valid ? C_tgt_valid : C_tgt_invalid);
	fold (&trace.flight_hash, 0xF1000000u | (uint32_t) (valid ? 1 : 0) | (uint32_t) (cur_move.tgt_mask << 4));
	if (valid)
	{
		fold_vec (&trace.flight_hash, position);
	}
}

void eech_wpn_intercept (float weapon_velocity, float target_true_velocity, const vec3d *intercept_point)
{
	int moving = weapon_velocity > 0.001f && target_true_velocity > 0.001f;
	bump (C_intercept_calls);
	if (!moving)
	{
		bump (weapon_velocity > 0.001f ? C_intercept_target_not_moving : C_intercept_weapon_not_moving);
	}
	else if (target_true_velocity < weapon_velocity)
	{
		bump (C_intercept_iterated);
		note_range (R_INTERCEPT_VELOCITY_RATIO, (double) (target_true_velocity / weapon_velocity));
	}
	else
	{
		bump (C_intercept_target_not_slower);
	}
	fold (&trace.flight_hash, 0xF2000000u | (uint32_t) (moving ? 1 : 0));
	fold_f (&trace.flight_hash, weapon_velocity);
	fold_f (&trace.flight_hash, target_true_velocity);
	fold_vec (&trace.flight_hash, intercept_point);
}

void eech_wpn_guided (int kind, float a, float b, float c, float d)
{
	ready ();
	fold (&trace.flight_hash, 0xF3000000u | (uint32_t) kind);
	fold_f (&trace.flight_hash, a);
	fold_f (&trace.flight_hash, b);
	fold_f (&trace.flight_hash, c);
	fold_f (&trace.flight_hash, d);
	switch (kind)
	{
		case EECH_WPN_GUIDED_TOO_CLOSE:
			bump (C_guided_too_close);
			note_range (R_GUIDED_LENGTH, (double) a);
			break;
		case EECH_WPN_GUIDED_OVERSHOT:
			bump (C_guided_overshot);
			note_range (R_GUIDED_LENGTH, (double) a);
			break;
		case EECH_WPN_GUIDED_TARGET_LOST:
			bump (a != 0.0f ? C_guided_target_lost_true : C_guided_target_lost_false);
			note_range (R_GUIDED_LENGTH, (double) b);
			break;
		case EECH_WPN_GUIDED_TURNED:
			bump (C_guided_turned);
			note_range (R_GUIDED_TURN_DEMAND, (double) a);
			note_range (R_GUIDED_TURN_RATE, (double) c);
			break;
	}
}

void eech_wpn_precision (int high_precision_collision)
{
	if (cur_move.active)
	{
		cur_move.precision = high_precision_collision ? 1 : 0;
		bump (high_precision_collision ? C_precision_high : C_precision_normal);
	}
}

void eech_wpn_ground (int hit_ground, int high_precision_collision)
{
	(void) high_precision_collision;
	if (cur_move.active)
	{
		cur_move.ground = hit_ground ? 1 : 0;
		bump (C_ground_tests);
		if (hit_ground)
		{
			bump (C_ground_hits);
		}
	}
}

void eech_wpn_impact (int terrain_class, int survive_ground_impact)
{
	if (!cur_move.active)
	{
		return;
	}
	cur_move.impact_class = terrain_class;
	bump (terrain_class == TERRAIN_CLASS_WATER ? C_impact_water : (terrain_class == TERRAIN_CLASS_LAND ? C_impact_land : C_impact_other));
	if (survive_ground_impact)
	{
		bump (C_impact_survive);
	}
	fold (&trace.flight_hash, 0xF4000000u | (uint32_t) (terrain_class & 0xff) | (uint32_t) (survive_ground_impact ? 0x100 : 0));
}

void eech_wpn_contact_any (const entity *hit, const vec3d *old_position, const vec3d *new_position)
{
	ready ();
	bump (C_contact_any_tests);
	fold (&trace.flight_hash, 0xF5000000u | (uint32_t) (hit ? 1 : 0));
	fold_vec (&trace.flight_hash, old_position);
	fold_vec (&trace.flight_hash, new_position);
	note_range (R_CONTACT_SEGMENT, (double) sqrt ((double) ((new_position->x - old_position->x) * (new_position->x - old_position->x)
		+ (new_position->y - old_position->y) * (new_position->y - old_position->y) + (new_position->z - old_position->z) * (new_position->z - old_position->z))));
	if (hit)
	{
		char text[SAMPLE_TEXT];
		bump (C_contact_any_hits);
		cur_move.contact_hits++;
		fold (&trace.flight_hash, (uint32_t) hit->type);
		text[0] = 0;
		text_append (text, sizeof (text), "hit=%s old=%.9g,%.9g,%.9g at=%.9g,%.9g,%.9g", type_name ((int) hit->type), (double) old_position->x, (double) old_position->y,
			(double) old_position->z, (double) new_position->x, (double) new_position->y, (double) new_position->z);
		keep_sample ("contact/any_target", text);
	}
}

void eech_wpn_contact_trees (int hit, const vec3d *old_position, const vec3d *new_position)
{
	ready ();
	bump (C_contact_trees_tests);
	fold (&trace.flight_hash, 0xF6000000u | (uint32_t) (hit ? 1 : 0));
	fold_vec (&trace.flight_hash, old_position);
	fold_vec (&trace.flight_hash, new_position);
	if (hit)
	{
		char text[SAMPLE_TEXT];
		bump (C_contact_trees_hits);
		text[0] = 0;
		text_append (text, sizeof (text), "old=%.9g,%.9g,%.9g at=%.9g,%.9g,%.9g", (double) old_position->x, (double) old_position->y, (double) old_position->z,
			(double) new_position->x, (double) new_position->y, (double) new_position->z);
		keep_sample ("contact/trees", text);
	}
}

void eech_wpn_move_end (entity *en)
{
	const weapon *raw = (const weapon *) get_local_entity_data (en);
	struct track *t = cur_move.track;
	char text[SAMPLE_TEXT], key[SAMPLE_KEY];
	int draws, sub = (int) raw->mob.sub_type, code = (int) raw->kill_code;
	float dx, dy, dz, step;
	ready ();
	if (!cur_move.active)
	{
		return;
	}
	draws = draws_between (cur_move.rng0, random_number_seed);
	cur_move.active = 0;
	bump (draws == 0 ? C_move_draws_0 : (draws == 1 ? C_move_draws_1 : C_move_draws_more));
	if (valid_sub_type (sub))
	{
		trace.weapons[sub].moves++;
		if (cur_move.path_mask & (1 << EECH_WPN_PATH_GUIDED))
		{
			trace.weapons[sub].moves_guided++;
		}
		if (cur_move.path_mask & (1 << EECH_WPN_PATH_UNGUIDED))
		{
			trace.weapons[sub].moves_unguided++;
		}
		if (cur_move.path_mask & (1 << EECH_WPN_PATH_LOAL_CLIMB_DIVE))
		{
			trace.weapons[sub].moves_loal++;
		}
	}
	dx = raw->mob.position.x - (t ? t->last_x : raw->mob.position.x);
	dy = raw->mob.position.y - (t ? t->last_y : raw->mob.position.y);
	dz = raw->mob.position.z - (t ? t->last_z : raw->mob.position.z);
	step = (float) sqrt ((double) (dx * dx + dy * dy + dz * dz));
	fold (&trace.flight_hash, 0xF0000000u | (uint32_t) cur_move.path_mask);
	fold (&trace.flight_hash, (uint32_t) sub);
	fold (&trace.flight_hash, (uint32_t) ((cur_move.guidance + 1) | ((cur_move.valid + 1) << 8) | ((cur_move.precision + 1) << 12) | ((cur_move.ground + 1) << 16)));
	fold (&trace.flight_hash, (uint32_t) cur_move.tgt_mask);
	fold (&trace.flight_hash, (uint32_t) code);
	fold (&trace.flight_hash, (uint32_t) raw->missile_phase);
	fold (&trace.flight_hash, (uint32_t) raw->weapon_on_ground);
	fold (&trace.flight_hash, (uint32_t) draws);
	fold_vec (&trace.flight_hash, &raw->mob.position);
	fold_f (&trace.flight_hash, raw->mob.velocity);
	fold_vec (&trace.flight_hash, &raw->mob.motion_vector);
	fold_matrix (&trace.flight_hash, raw->mob.attitude);
	fold_f (&trace.flight_hash, raw->weapon_lifetime);
	fold_f (&trace.flight_hash, raw->decoy_timer);
	note_range (R_MOVE_VELOCITY, (double) raw->mob.velocity);
	note_range (R_MOVE_LIFETIME, (double) raw->weapon_lifetime);
	note_range (R_MOVE_STEP_DISTANCE, (double) step);
	if (t)
	{
		t->steps++;
		t->distance += (double) step;
		t->path_union |= cur_move.path_mask;
		if (raw->mob.velocity > t->max_velocity)
		{
			t->max_velocity = raw->mob.velocity;
		}
		fold_vec (&t->hash, &raw->mob.position);
		fold_f (&t->hash, raw->mob.velocity);
		fold (&t->hash, (uint32_t) code);
		fold (&t->hash, (uint32_t) cur_move.path_mask);
		t->last_x = raw->mob.position.x;
		t->last_y = raw->mob.position.y;
		t->last_z = raw->mob.position.z;
		t->last_lifetime = raw->weapon_lifetime;
		t->on_ground = raw->weapon_on_ground ? 1 : 0;
	}
	snprintf (key, sizeof (key), "move/%s/mask%x", (cur_move.guidance >= 0 && cur_move.guidance < NUM_WEAPON_GUIDANCE_TYPES) ? guidance_keys[cur_move.guidance] : "unk",
		(unsigned) cur_move.path_mask);
	text[0] = 0;
	text_append (text, sizeof (text), "type=%d guidance=%d valid=%d tgt_mask=%x precision=%d ground=%d code=%s pos=%.9g,%.9g,%.9g velocity=%.9g lifetime=%.9g phase=%d draws=%d step=%.9g",
		sub, cur_move.guidance, cur_move.valid, (unsigned) cur_move.tgt_mask, cur_move.precision, cur_move.ground, (code >= 0 && code < NUM_WEAPON_KILL_CODES) ? kill_names[code] : "invalid",
		(double) raw->mob.position.x, (double) raw->mob.position.y, (double) raw->mob.position.z, (double) raw->mob.velocity, (double) raw->weapon_lifetime,
		(int) raw->missile_phase, draws, (double) step);
	keep_sample (key, text);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the per-update order (wn_updt.c) and the terminal fact */

static void stage_order (int stage)
{
	if (stage <= last_stage)
	{
		bump (C_update_order_violations);
	}
	last_stage = stage;
	fold (&trace.decision_hash, 0xD3000000u | (uint32_t) stage);
}

void eech_wpn_update_stage (entity *en, int stage)
{
	(void) en;
	ready ();
	if (stage >= 1 && stage <= 5)
	{
		bump (C_update_stage_1 + stage - 1);
	}
	stage_order (stage);
}

/* update_decoy counts the weapon's timer down and, once it is below zero, sets it to the restart value. The hooks bracket ONE weapon's own call, so a restart is a timer
 * that went up across that call; no state is carried from one weapon to the next. */
static float decoy_before;

void eech_wpn_decoy_begin (float decoy_timer)
{
	ready ();
	decoy_before = decoy_timer;
}

void eech_wpn_decoy (float decoy_timer)
{
	ready ();
	bump (C_decoy_updates);
	bump (C_update_stage_3);
	stage_order (3);
	if (decoy_timer > decoy_before)
	{
		bump (C_decoy_restarts);
	}
	fold_f (&trace.flight_hash, decoy_before);
	fold_f (&trace.flight_hash, decoy_timer);
}

void eech_wpn_terminal (entity *en, const weapon *raw)
{
	struct track *t = track_find (en);
	char text[LOG_TEXT], name[48], key[SAMPLE_KEY];
	int code = (int) raw->kill_code, sub = (int) raw->mob.sub_type;
	int launcher_type = raw->launched_weapon_link.parent ? (int) raw->launched_weapon_link.parent->type : -1;
	int target_type = raw->mob.target_link.parent ? (int) raw->mob.target_link.parent->type : -1;
	ready ();
	bump (C_terminal_calls);
	bump (C_update_stage_4);
	stage_order (4);
	bump (target_type >= 0 ? C_terminal_with_target : C_terminal_without_target);
	if (code >= 0 && code < NUM_WEAPON_KILL_CODES)
	{
		trace.kills[code]++;
		if (valid_sub_type (sub))
		{
			trace.weapons[sub].kills[code]++;
		}
	}
	if (launcher_type >= 0 && launcher_type < NUM_ENTITY_TYPES)
	{
		trace.terminal_by_launcher[launcher_type]++;
	}
	fold (&trace.terminal_hash, 0xE1000000u | (uint32_t) code);
	fold (&trace.terminal_hash, (uint32_t) sub);
	fold (&trace.terminal_hash, (uint32_t) launcher_type);
	fold (&trace.terminal_hash, (uint32_t) target_type);
	fold_vec (&trace.terminal_hash, &raw->mob.position);
	fold_f (&trace.terminal_hash, raw->mob.velocity);
	fold_f (&trace.terminal_hash, raw->weapon_lifetime);
	if (!t)
	{
		bump (C_terminal_unregistered);
	}
	else
	{
		t->terminal_seen = 1;
		fold (&trace.terminal_hash, (uint32_t) t->steps);
		fold (&trace.terminal_hash, (uint32_t) (t->hash & 0xffffffffu));
		fold (&trace.terminal_hash, (uint32_t) (t->hash >> 32));
		fold (&trace.terminal_hash, (uint32_t) t->path_union);
		note_range (R_TERMINAL_STEPS, (double) t->steps);
		note_range (R_TERMINAL_DISTANCE, t->distance);
	}
	note_range (R_TERMINAL_VELOCITY, (double) raw->mob.velocity);
	text[0] = 0;
	text_append (text, sizeof (text),
		"kind=terminal w=%d type=%s code=%s steps=%d distance=%.9g launcher=%s target=%s pos=%.9g,%.9g,%.9g velocity=%.9g lifetime=%.9g max_velocity=%.9g "
		"paths=%x warhead=%d class=%d phase=%d on_ground=%d track=%016llx",
		t ? t->ordinal : -1, weapon_name (sub, name, sizeof (name)), (code >= 0 && code < NUM_WEAPON_KILL_CODES) ? kill_names[code] : "invalid", t ? t->steps : -1,
		t ? t->distance : -1.0, type_name (launcher_type), type_name (target_type), (double) raw->mob.position.x, (double) raw->mob.position.y,
		(double) raw->mob.position.z, (double) raw->mob.velocity, (double) raw->weapon_lifetime, t ? (double) t->max_velocity : -1.0, t ? t->path_union : -1,
		valid_sub_type (sub) ? (int) weapon_database[sub].warhead_type : -1, valid_sub_type (sub) ? (int) weapon_database[sub].weapon_class : -1, (int) raw->missile_phase,
		(int) raw->weapon_on_ground, t ? (unsigned long long) t->hash : 0ull);
	log_event ("%s", text);
	snprintf (key, sizeof (key), "terminal/%s", (code >= 0 && code < NUM_WEAPON_KILL_CODES) ? kill_names[code] : "invalid");
	keep_sample (key, text + 5);
}

void eech_wpn_destroy (entity *en)
{
	struct track *t = track_find (en);
	ready ();
	bump (C_destroy_calls);
	if (!t)
	{
		bump (C_destroy_unregistered);
		return;
	}
	if (!t->terminal_seen)
	{
		bump (C_destroy_without_terminal);
	}
	fold (&trace.decision_hash, 0xD4000000u | (uint32_t) (t->terminal_seen ? 1 : 0));
	{
		char name[48];
		log_event ("kind=destroy w=%d type=%s after_terminal=%d steps=%d", t->ordinal, weapon_name (t->sub_type, name, sizeof (name)), t->terminal_seen, t->steps);
	}
	t->used = 0;
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

static int report (char *text, int capacity)
{
	struct out out;
	int i, j, live = 0;
	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	text[0] = 0;
	for (i = 0; i < track_high; i++)
	{
		if (tracks[i].used)
		{
			live++;
		}
	}
	emit (&out, "wpn-hash table=%016llx launch=%016llx flight=%016llx terminal=%016llx decision=%016llx\n", (unsigned long long) trace.table_hash,
		(unsigned long long) trace.launch_hash, (unsigned long long) trace.flight_hash, (unsigned long long) trace.terminal_hash, (unsigned long long) trace.decision_hash);
	for (i = 0; i < COUNTERS; i++)
	{
		emit (&out, "wpn-counter name=%s value=%llu\n", counter_names[i], (unsigned long long) trace.counters[i]);
	}
	emit (&out, "wpn-counter name=draws_total value=%d\n", draws_total);
	emit (&out, "wpn-counter name=live_weapons value=%d\n", live);
	emit (&out, "wpn-counter name=next_ordinal value=%d\n", trace.next_ordinal);
	for (i = 0; i < RANGES; i++)
	{
		emit (&out, "wpn-range name=%s n=%llu min=%.9g max=%.9g\n", range_names[i], (unsigned long long) trace.range_n[i], trace.range_n[i] ? trace.range_min[i] : 0.0,
			trace.range_n[i] ? trace.range_max[i] : 0.0);
	}
	for (i = 0; i < EECH_WPN_PATHS; i++)
	{
		emit (&out, "wpn-path name=%s count=%llu\n", path_names[i], (unsigned long long) trace.paths[i]);
	}
	for (i = 0; i < NUM_WEAPON_GUIDANCE_TYPES; i++)
	{
		emit (&out, "wpn-guidance name=%s target_position_invalid=%llu target_position_valid=%llu\n", guidance_names[i], (unsigned long long) trace.guidance[i][0],
			(unsigned long long) trace.guidance[i][1]);
	}
	for (i = 0; i < NUM_WEAPON_KILL_CODES; i++)
	{
		emit (&out, "wpn-kill code=%s count=%llu\n", kill_names[i], (unsigned long long) trace.kills[i]);
	}
	for (i = 0; i < 4; i++)
	{
		emit (&out, "wpn-outcome name=%s count=%llu\n", outcome_names[i], (unsigned long long) trace.outcomes[i]);
	}
	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		if (trace.birth_by_launcher[i] || trace.terminal_by_launcher[i])
		{
			emit (&out, "wpn-launcher kind=%s births=%llu terminals=%llu\n", get_entity_type_name ((entity_types) i), (unsigned long long) trace.birth_by_launcher[i],
				(unsigned long long) trace.terminal_by_launcher[i]);
		}
	}
	for (i = 0; i < NUM_ENTITY_SUB_TYPE_WEAPONS; i++)
	{
		const struct weapon_row *w = &trace.weapons[i];
		if (w->births || w->moves)
		{
			char name[48];
			emit (&out, "wpn-weapon sub_type=%d name=%s births=%llu submunitions=%llu moves=%llu guided=%llu unguided=%llu loal=%llu", i, weapon_name (i, name, sizeof (name)),
				(unsigned long long) w->births, (unsigned long long) w->submunitions, (unsigned long long) w->moves, (unsigned long long) w->moves_guided,
				(unsigned long long) w->moves_unguided, (unsigned long long) w->moves_loal);
			for (j = 0; j < NUM_WEAPON_KILL_CODES; j++)
			{
				emit (&out, " kill_%s=%llu", kill_names[j], (unsigned long long) w->kills[j]);
			}
			emit (&out, "\n");
		}
	}
	for (i = 0; i < trace.table_count; i++)
	{
		const struct table_row *r = &trace.tables[i];
		emit (&out, "wpn-table sub_type=%d name=%s dims=%d,%d,%d max_range=%.9g cells=%llu hash=%016llx nonzero=%llu nonfinite=%llu flights=%llu full=%llu short=%llu steps=%llu supplied=%llu\n",
			r->sub_type, r->name, r->velocity_tests, r->pitch_indices, r->num_range_values, (double) r->max_range, (unsigned long long) r->cells, (unsigned long long) r->hash,
			(unsigned long long) r->nonzero, (unsigned long long) r->nonfinite, (unsigned long long) r->flights, (unsigned long long) r->flights_full,
			(unsigned long long) r->flights_short, (unsigned long long) r->steps, (unsigned long long) r->supplied);
	}
	emit (&out, "wpn-tables rows=%d overflow=%d\n", trace.table_count, trace.table_overflow);
	/* every weapon still in flight or resting, with the state weapon_movement left: a weapon that survived ground impact is not moved again (on_ground=1) and ends only when its
	 * lifetime reaches -(cruise_time + a draw of at most 0.5 * cruise_time_max_error), i.e. in at most `expires_in` + error more seconds */
	for (i = 0; i < track_high; i++)
	{
		const struct track *t = &tracks[i];
		if (t->used)
		{
			char name[48];
			double cruise = valid_sub_type (t->sub_type) ? (double) weapon_database[t->sub_type].cruise_time : -1.0;
			double error = valid_sub_type (t->sub_type) ? (double) weapon_database[t->sub_type].cruise_time_max_error : -1.0;
			emit (&out, "wpn-live w=%d type=%s steps=%d on_ground=%d lifetime=%.9g cruise_time=%.9g cruise_error=%.9g expires_in_at_least=%.9g terminal_seen=%d\n", t->ordinal,
				weapon_name (t->sub_type, name, sizeof (name)), t->steps, t->on_ground, (double) t->last_lifetime, cruise, error, (double) t->last_lifetime + cruise, t->terminal_seen);
		}
	}
	for (i = 0; i < trace.sample_count; i++)
	{
		emit (&out, "wpn-sample key=%s %s\n", trace.samples[i].key, trace.samples[i].text);
	}
	emit (&out, "wpn-samples count=%d overflow=%d\n", trace.sample_count, trace.sample_overflow);
	emit (&out, "wpn-log events=%llu stored=%d drained=%d overflow=%d\n", (unsigned long long) log_seq, log_count, log_drained, log_overflow);
	return out.overflow ? -1 : 0;
}

int eech_wpn_trace_get (char *text, int capacity)
{
	ready ();
	return report (text, capacity);
}

/* drains the ordered event log from the cursor: whole events only; returns 0 when everything stored has been drained and 1 when more remains */
int eech_wpn_events_get (char *text, int capacity)
{
	int used = 0;
	ready ();
	text[0] = 0;
	while (log_drained < log_count)
	{
		int n = (int) strlen (log_pool[log_drained]);
		if (used + n + 2 > capacity)
		{
			break;
		}
		memcpy (text + used, log_pool[log_drained], (size_t) n);
		used += n;
		text[used++] = '\n';
		text[used] = 0;
		log_drained++;
	}
	return log_drained < log_count ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* controlled launch case (EEDCS-45): one real host-identified launcher is given the campaign's own physical launch attempt for one weapon type, through
 * launch_client_server_weapon (the call the family fire code makes), optionally with one host-identified target link set first through the campaign's own
 * set_client_server_entity_parent. Only host ids cross. The precondition (launcher, weapon type, target) is the test's choice; the creation, the birth and the flight
 * that follow are the original's and are observed by the hooks above. */

static int case_entity_live (entity *en)
{
	return entities && en >= entities && en < entities + number_of_entities;
}

static entity *case_find (const char *host_id)
{
	entity *en;
	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		const char *id;
		if (!case_entity_live (en))
		{
			break;
		}
		id = campaign_roster_host_id (en);
		if (id && strcmp (id, host_id) == 0)
		{
			return en;
		}
	}
	return NULL;
}

/* The roster holds kinds with different abilities, so the guards are per ROLE, from the operations each role needs:
 *   launcher / query subject: the weapon-package count, the alive flag and the target link as a child (LIST_TYPE_TARGET): the six mobile families. A SITE has a roster id and
 *                             none of the launcher operations; it is rejected here.
 *   target: the target list ROOT (LIST_TYPE_TARGET_ROOT), alive, side, position and target point. The six mobile families have them (mobile base) and so do the fixed kinds
 *           with roster ids, SITE and SITE_UPDATABLE (fixed base: fx_list.c, fx_int.c, fx_vec3d.c; st_funcs.c overload_fixed_functions): a SITE is a valid static target.
 * Anything else is rejected BEFORE any getter or setter runs on it. */
static int launcher_kind (const entity *en)
{
	switch (en->type)
	{
		case ENTITY_TYPE_HELICOPTER:
		case ENTITY_TYPE_FIXED_WING:
		case ENTITY_TYPE_ANTI_AIRCRAFT:
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SHIP_VEHICLE:
		case ENTITY_TYPE_PERSON:
			return 1;
		default:
			return 0;
	}
}

static int target_kind (const entity *en)
{
	switch (en->type)
	{
		case ENTITY_TYPE_SITE:
		case ENTITY_TYPE_SITE_UPDATABLE:
			return 1;
		default:
			return launcher_kind (en);
	}
}

/* The kind of a roster host id, from the entity's type field alone (no getter runs on it): found=0 for an id that names nothing. Never an error: it is how a driver learns
 * which roster ids the other case operations will reject. */
int eech_wpn_case_kind_run (const char *host_id, char *text, int capacity)
{
	entity *en = case_find (host_id);
	int n = snprintf (text, (size_t) capacity, "wpn-case-kind host_id=%s found=%d kind=%s launcher=%d target=%d\n", host_id, en ? 1 : 0, en ? get_entity_type_name ((entity_types) en->type) : "none",
		en ? launcher_kind (en) : 0, en ? target_kind (en) : 0);
	return (n < 0 || n >= capacity) ? -1 : 0;
}

int eech_wpn_case_query_run (const char *host_id, int weapon, char *text, int capacity)
{
	entity *en = case_find (host_id);
	vec3d position;
	entity *target;
	int count = -1, n;
	text[0] = 0;
	if (!en)
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=no_such_host_id host_id=%s\n", host_id);
		return -1;
	}
	if (!launcher_kind (en))
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=subject_kind_unsupported kind=%s\n", get_entity_type_name ((entity_types) en->type));
		return -1;
	}
	get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &position);
	target = get_local_entity_parent (en, LIST_TYPE_TARGET);
	if (valid_sub_type (weapon))
	{
		count = get_local_entity_weapon_count (en, (entity_sub_types) weapon);
	}
	n = snprintf (text, (size_t) capacity,
		"wpn-case-state host_id=%s type=%s alive=%d position=%.9g,%.9g,%.9g has_target=%d target_type=%s weapon=%d weapon_count=%d warhead=%d guidance=%d aiming=%d decoy=%d\n", host_id,
		get_entity_type_name ((entity_types) en->type), get_local_entity_int_value (en, INT_TYPE_ALIVE), (double) position.x, (double) position.y, (double) position.z,
		target ? 1 : 0, target ? get_entity_type_name ((entity_types) target->type) : "none", weapon, count,
		valid_sub_type (weapon) ? (int) weapon_database[weapon].warhead_type : -1, valid_sub_type (weapon) ? (int) weapon_database[weapon].guidance_type : -1,
		valid_sub_type (weapon) ? (int) weapon_database[weapon].aiming_type : -1, valid_sub_type (weapon) ? (int) weapon_database[weapon].decoy_type : -1);
	return (n < 0 || n >= capacity) ? -1 : 0;
}

int eech_wpn_case_launch_run (const char *launcher_id, const char *target_id, int weapon, int salvo_count, char *text, int capacity)
{
	entity *launcher = case_find (launcher_id);
	entity *target = target_id[0] ? case_find (target_id) : NULL;
	entity *previous;
	uint64_t births_before, created_before;
	int count_before, count_after, n, link_before, link_after, restored = -1;
	text[0] = 0;
	if (!launcher || (target_id[0] && !target))
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=%s\n", !launcher ? "no_such_launcher_host_id" : "no_such_target_host_id");
		return -1;
	}
	/* kinds first: no getter or setter below runs on an entity that cannot launch or be linked as a weapon target; nothing has been changed when this rejects */
	if (!launcher_kind (launcher))
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=launcher_kind_unsupported kind=%s\n", get_entity_type_name ((entity_types) launcher->type));
		return -1;
	}
	if (target && !target_kind (target))
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=target_kind_unsupported kind=%s\n", get_entity_type_name ((entity_types) target->type));
		return -1;
	}
	if (launcher == target || !valid_sub_type (weapon) || salvo_count < 0 || !get_local_entity_int_value (launcher, INT_TYPE_ALIVE)
		|| (target && !get_local_entity_int_value (target, INT_TYPE_ALIVE)))
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=launcher_target_or_weapon_not_valid\n");
		return -1;
	}
	ready ();
	previous = get_local_entity_parent (launcher, LIST_TYPE_TARGET);
	link_before = previous ? 1 : 0;
	/* the requested link, empty or not, is established through the campaign's own setter (NULL is the campaign's own "no target") and then read back */
	if (previous != target)
	{
		set_client_server_entity_parent (launcher, LIST_TYPE_TARGET, target);
	}
	if (get_local_entity_parent (launcher, LIST_TYPE_TARGET) != target)
	{
		snprintf (text, (size_t) capacity, "wpn-case-error reason=target_link_not_set\n");
		return -1;
	}
	link_after = target ? 1 : 0;
	count_before = get_local_entity_weapon_count (launcher, (entity_sub_types) weapon);
	births_before = trace.counters[C_birth_calls];
	created_before = trace.counters[C_launch_created];
	launch_client_server_weapon (launcher, (entity_sub_types) weapon, salvo_count);
	count_after = get_local_entity_weapon_count (launcher, (entity_sub_types) weapon);
	/* restore the launcher's previous link through the same setter, when that entity is still a live roster member */
	if (previous != target)
	{
		if (!previous)
		{
			set_client_server_entity_parent (launcher, LIST_TYPE_TARGET, NULL);
			restored = get_local_entity_parent (launcher, LIST_TYPE_TARGET) == NULL;
		}
		else if (case_entity_live (previous) && get_local_entity_int_value (previous, INT_TYPE_ALIVE))
		{
			set_client_server_entity_parent (launcher, LIST_TYPE_TARGET, previous);
			restored = get_local_entity_parent (launcher, LIST_TYPE_TARGET) == previous;
		}
		else
		{
			restored = 0;
		}
	}
	n = snprintf (text, (size_t) capacity,
		"wpn-case-launch launcher=%s weapon=%d target_link_before=%d target_link=%d count_before=%d count_after=%d births=%llu creates=%llu restored=%d\n",
		get_entity_type_name ((entity_types) launcher->type), weapon, link_before, link_after, count_before, count_after,
		(unsigned long long) (trace.counters[C_birth_calls] - births_before), (unsigned long long) (trace.counters[C_launch_created] - created_before), restored);
	return (n < 0 || n >= capacity) ? -1 : 0;
}

#endif

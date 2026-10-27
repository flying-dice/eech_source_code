/*
 * TEST-ONLY physical weapon-aiming characterisation (EEDCS-44). Compiled only with the `aim-fixture` feature; never part of the
 * shipping module.
 *
 * weapon/weapon.c, weapon/wn_move.c, aircraft/ac_wpn.c, vehicle/vh_wpn.c and vehicle/person/ps_anim.c, built with EECH_AIM_FIXTURE,
 * call the eech_aim_* hooks declared in entity/mobile/mb_aimfx.h from inside the original aiming code. The hooks only observe:
 * they read arguments, results and entity fields the original has already produced and fold them into two hashes and a set of
 * counters, in the order the game computed them.
 *
 *   physical_hash  the physical facts: every pitch-device-to-target solve (the target and intercept vectors and range it produced), every
 *                  guided-weapon intercept prediction (target point, speeds, predicted point), every lead-and-ballistic solve (its branch,
 *                  iterations, iteration points, range before and after the original's random range jitter, intercept point, projection
 *                  angle, time of flight), every ballistic-table lookup, every weapon-system device slew (required and current heading and
 *                  pitch, the rate-limited step, the shared-device flags) and the weapon and target vectors the update leaves on the
 *                  launcher, the final device angles, the measured launch-angle error at each fire decision, and a person's target-facing
 *                  heading.
 *   decision_hash  what the campaign does with them: how far the vector update got (the gate reached), whether the vectors were valid,
 *                  the weapon-system-ready verdict, the early exits and every fire-decision rejection (not ready, vectors invalid,
 *                  launch angle).
 *
 * Neither hash carries an entity index, a heap address or a seed. The game's shared random stream is observed only by the NUMBER of
 * draws a solver made (stepping a local copy of the generator to the stream's later state); its value and the seed never leave
 * this file. The fixture evaluates no campaign getter the original did not and draws nothing.
 *
 * There is no probe. The pitch-device solver draws from the shared random stream and the weapon-system update turns devices, so a
 * sweep that called them outside the game's own order would need the stream saved and restored; the natural calls during stepping
 * are numerous, so this fixture observes those and nothing else.
 */

#ifdef EECH_AIM_FIXTURE

#include <stdarg.h>
#include <stdint.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"
#include "entity/mobile/mb_aimfx.h"
#include "entity/mobile/mb_hostaim.h"

#define SAMPLE_SLOTS 2
#define SAMPLE_COUNT 64
#define SAMPLE_TEXT 700
#define KIND_ROWS 48
#define WEAPON_ROWS 96
#define STAGES 8
#define RANGES 13

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

enum counter
{
	C_VEC_CALLS,
	C_VEC_TARGETED,
	C_VEC_PLAYER_WITHOUT_TARGET,
	C_VEC_TRACKING_VALID,
	C_VEC_VALID,
	C_VEC_INVALID,
	C_VEC_STAGE_0,
	C_VEC_STAGE_1,
	C_VEC_STAGE_2,
	C_VEC_STAGE_3,
	C_VEC_STAGE_4,
	C_VEC_STAGE_5,
	C_VEC_STAGE_6,
	C_SOLVE_CALLS,
	C_SOLVE_OUTSIDE_VECTOR_UPDATE,
	C_SOLVE_GUIDED_MOBILE,
	C_SOLVE_GUIDED_STATIC,
	C_SOLVE_ANGLE_OF_PROJECTION,
	C_SOLVE_LEAD_AND_BALLISTIC,
	C_SOLVE_TRUE,
	C_SOLVE_FALSE,
	C_INTERCEPT_CALLS,
	C_INTERCEPT_OUTSIDE_VECTOR_UPDATE,
	C_INTERCEPT_ITERATED,
	C_INTERCEPT_WEAPON_NOT_MOVING,
	C_INTERCEPT_TARGET_NOT_MOVING,
	C_INTERCEPT_TARGET_NOT_SLOWER,
	C_LEAD_CALLS,
	C_LEAD_OUTSIDE_VECTOR_UPDATE,
	C_LEAD_TRUE,
	C_LEAD_FALSE,
	C_LEAD_PATH_TINY_RANGE,
	C_LEAD_PATH_OFF_MAP,
	C_LEAD_PATH_STATIONARY,
	C_LEAD_PATH_MOVING,
	C_LEAD_MOVING_5_ITERATIONS,
	C_LEAD_MOVING_3_ITERATIONS,
	C_LEAD_ITERATIONS_RUN,
	C_LEAD_JITTER_APPLIED,
	C_LEAD_TRIANGULATED,
	C_LEAD_RNG_DRAWS_0,
	C_LEAD_RNG_DRAWS_1,
	C_LEAD_RNG_DRAWS_MORE,
	C_BALLISTIC_CALLS,
	C_BALLISTIC_OUTSIDE_VECTOR_UPDATE,
	C_BALLISTIC_FROM_LEAD,
	C_BALLISTIC_TRUE,
	C_BALLISTIC_FALSE,
	C_BALLISTIC_SIMPLIFIED,
	C_BALLISTIC_FIXED_PITCH,
	C_BALLISTIC_POINT_BLANK_ZERO_TIME,
	C_WS_CALLS,
	C_WS_NO_PACKAGE_STATUS,
	C_WS_UNARMED,
	C_WS_ARMED,
	C_WS_EXIT_NO_VIEWPOINT,
	C_WS_EXIT_NO_HEADING_DEVICE,
	C_WS_EXIT_NO_PITCH_DEVICE,
	C_WS_READY_DEVICE_FOUND,
	C_WS_MAKE_SELECTED_READY,
	C_WS_READY_TRUE,
	C_WS_READY_FALSE,
	C_WS_PACKAGES_ROTATING,
	C_WS_REQUIRED_LOCATED,
	C_WS_REQUIRED_DIFFERS,
	C_WS_STEP_HEADING_SHORT,
	C_WS_STEP_PITCH_SHORT,
	C_WS_STEP_HEADING_SHARED,
	C_WS_STEP_PITCH_SHARED,
	C_WS_EDGE_CLIMB_BUMP,
	C_WS_EDGE_HOLD_NOT_READY,
	C_WS_EDGE_NON_AI_PLAYER,
	C_WS_EDGE_PLAYER_LOCAL,
	C_WS_EDGE_POINT_LOCK,
	C_ANGLE_EVAL_AIRCRAFT_FIRE,
	C_ANGLE_EVAL_AIRCRAFT_UPDATE,
	C_ANGLE_EVAL_VEHICLE_UPDATE,
	C_REJECT_AIRCRAFT_FIRE_NOT_READY,
	C_REJECT_AIRCRAFT_FIRE_VECTORS_INVALID,
	C_REJECT_AIRCRAFT_FIRE_LAUNCH_ANGLE,
	C_REJECT_AIRCRAFT_UPDATE_NOT_READY,
	C_REJECT_AIRCRAFT_UPDATE_VECTORS_INVALID,
	C_REJECT_AIRCRAFT_UPDATE_LAUNCH_ANGLE,
	C_REJECT_VEHICLE_UPDATE_NOT_READY,
	C_REJECT_VEHICLE_UPDATE_VECTORS_INVALID,
	C_REJECT_VEHICLE_UPDATE_LAUNCH_ANGLE,
	C_READY_UPDATE_AIRCRAFT,
	C_READY_UPDATE_VEHICLE,
	C_READY_STATE_START,
	C_READY_STATE_OPENING,
	C_READY_STATE_OPEN,
	C_READY_STATE_CLOSING,
	C_READY_COMMAND_OPEN,
	C_READY_COMMAND_CLOSE,
	C_PERSON_BRANCH_ENTERED,
	C_PERSON_BRANCH_WITH_TARGET_LINK,
	C_PERSON_FACING,
	COUNTERS
};

static const char *const counter_names[COUNTERS] = {
	"vec_calls", "vec_targeted", "vec_player_without_target", "vec_tracking_valid", "vec_valid", "vec_invalid",
	"vec_stage_0", "vec_stage_1", "vec_stage_2", "vec_stage_3", "vec_stage_4", "vec_stage_5", "vec_stage_6",
	"solve_calls", "solve_outside_vector_update", "solve_guided_mobile", "solve_guided_static", "solve_angle_of_projection",
	"solve_lead_and_ballistic", "solve_true", "solve_false",
	"intercept_calls", "intercept_outside_vector_update", "intercept_iterated", "intercept_weapon_not_moving", "intercept_target_not_moving",
	"intercept_target_not_slower",
	"lead_calls", "lead_outside_vector_update", "lead_true", "lead_false", "lead_path_tiny_range", "lead_path_off_map", "lead_path_stationary",
	"lead_path_moving", "lead_moving_5_iterations", "lead_moving_3_iterations", "lead_iterations_run", "lead_jitter_applied",
	"lead_triangulated", "lead_rng_draws_0", "lead_rng_draws_1", "lead_rng_draws_more",
	"ballistic_calls", "ballistic_outside_vector_update", "ballistic_from_lead", "ballistic_true", "ballistic_false", "ballistic_simplified",
	"ballistic_fixed_pitch", "ballistic_point_blank_zero_time",
	"ws_calls", "ws_no_package_status", "ws_unarmed", "ws_armed", "ws_exit_no_viewpoint", "ws_exit_no_heading_device", "ws_exit_no_pitch_device",
	"ws_ready_device_found", "ws_make_selected_ready", "ws_ready_true", "ws_ready_false", "ws_packages_rotating", "ws_required_located",
	"ws_required_differs", "ws_step_heading_short", "ws_step_pitch_short", "ws_step_heading_shared", "ws_step_pitch_shared",
	"ws_edge_climb_bump", "ws_edge_hold_not_ready", "ws_edge_non_ai_player", "ws_edge_player_local", "ws_edge_point_lock",
	"angle_eval_aircraft_fire", "angle_eval_aircraft_update", "angle_eval_vehicle_update",
	"reject_aircraft_fire_not_ready", "reject_aircraft_fire_vectors_invalid", "reject_aircraft_fire_launch_angle",
	"reject_aircraft_update_not_ready", "reject_aircraft_update_vectors_invalid", "reject_aircraft_update_launch_angle",
	"reject_vehicle_update_not_ready", "reject_vehicle_update_vectors_invalid", "reject_vehicle_update_launch_angle",
	"ready_update_aircraft", "ready_update_vehicle", "ready_state_start", "ready_state_opening", "ready_state_open", "ready_state_closing",
	"ready_command_open", "ready_command_close",
	"person_branch_entered", "person_branch_with_target_link", "person_facing"
};

/* named value ranges: min and max of what the run measured */
enum range_id
{
	R_VEC_INTERCEPT_RANGE,
	R_SOLVE_INTERCEPT_RANGE,
	R_LEAD_RANGE,
	R_LEAD_TIME_OF_FLIGHT,
	R_LEAD_ANGLE,
	R_BALLISTIC_RANGE,
	R_BALLISTIC_PARENT_VELOCITY,
	R_BALLISTIC_TIME_OF_FLIGHT,
	R_LAUNCH_ANGLE_ERROR,
	R_LAUNCH_ANGLE_LIMIT,
	R_WS_STEP_HEADING,
	R_PERSON_HEADING,
	R_READY_STATE_AFTER,
	RANGE_IDS
};

static const char *const range_names[RANGES] = {
	"vec_intercept_range", "solve_intercept_range", "lead_range", "lead_time_of_flight", "lead_angle", "ballistic_range",
	"ballistic_parent_velocity", "ballistic_time_of_flight", "launch_angle_error", "launch_angle_limit", "ws_step_heading", "person_heading", "ready_state_after"
};

static const char *const function_names[4] = { "vector_update", "weapon_system_update", "person_facing", "ready_update" };
static const char *const aiming_names[4] = { "none", "guided_intercept", "angle_of_projection", "lead_and_ballistic" };

struct sample
{
	char key[40];
	char text[SAMPLE_TEXT];
};

struct kind_row
{
	int source_type, function;
	uint64_t calls, reached_devices, valid, steps;
};

struct weapon_row
{
	int aiming, sub_type;
	char name[48];
	uint64_t solves, solves_true, leads, leads_moving, ballistic;
};

struct aim_trace
{
	uint64_t physical_hash, decision_hash;
	uint64_t counters[COUNTERS];
	double range_min[RANGES], range_max[RANGES];
	uint64_t range_n[RANGES];
	int kind_count, kind_overflow, weapon_count, weapon_overflow, sample_count, sample_overflow;
	struct kind_row kinds[KIND_ROWS];
	struct weapon_row weapons[WEAPON_ROWS];
	struct sample samples[SAMPLE_COUNT];
};

static struct aim_trace trace;

/* the calls in flight (the vector update, the weapon-system update, the pitch-device solve and the lead solve nest as the original nests them) */
static struct
{
	int active, stage, targeted, tracking, solve_result, solve_seen, source_type;
	int guided_mobile, aiming, sub_type;
} cur_vec;

static struct
{
	int active, stage, config_type, source_type;
	float required_heading, required_pitch;
} cur_ws;

static struct
{
	int active, path, iterations, jitter, triangulated;
	float tov, range_initial, range_pre, range_post;
	int rng_before;
} cur_lead;

static struct
{
	int active, intercept_called, aiming, sub_type;
} cur_solve;

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

static void trace_reset (void)
{
	int i;
	memset (&trace, 0, sizeof (trace));
	trace.physical_hash = FNV_OFFSET;
	trace.decision_hash = FNV_OFFSET;
	for (i = 0; i < RANGES; i++)
	{
		trace.range_min[i] = 1e30;
		trace.range_max[i] = -1e30;
	}
	memset (&cur_vec, 0, sizeof (cur_vec));
	memset (&cur_ws, 0, sizeof (cur_ws));
	memset (&cur_lead, 0, sizeof (cur_lead));
	memset (&cur_solve, 0, sizeof (cur_solve));
}

static void ready (void)
{
	if (!trace.physical_hash)
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
	snprintf (trace.samples[trace.sample_count].key, sizeof (trace.samples[0].key), "%s", key);
	snprintf (trace.samples[trace.sample_count].text, SAMPLE_TEXT, "%s", text);
	trace.sample_count++;
}

static struct kind_row *kind_row_for (int type, int function)
{
	int i;
	for (i = 0; i < trace.kind_count; i++)
	{
		if (trace.kinds[i].source_type == type && trace.kinds[i].function == function)
		{
			return &trace.kinds[i];
		}
	}
	return NULL;
}

static void note_kind (entity *source, int function)
{
	int type = (int) source->type;
	int i;
	for (i = 0; i < trace.kind_count; i++)
	{
		if (trace.kinds[i].source_type == type && trace.kinds[i].function == function)
		{
			trace.kinds[i].calls++;
			return;
		}
	}
	if (trace.kind_count < KIND_ROWS)
	{
		trace.kinds[trace.kind_count].source_type = type;
		trace.kinds[trace.kind_count].function = function;
		trace.kinds[trace.kind_count].calls = 1;
		trace.kind_count++;
	}
	else
	{
		trace.kind_overflow++;
	}
}

static struct weapon_row *find_weapon_row (int aiming, int sub_type)
{
	int i;
	for (i = 0; i < trace.weapon_count; i++)
	{
		if (trace.weapons[i].aiming == aiming && trace.weapons[i].sub_type == sub_type)
		{
			return &trace.weapons[i];
		}
	}
	if (trace.weapon_count < WEAPON_ROWS)
	{
		struct weapon_row *row = &trace.weapons[trace.weapon_count++];
		const char *name = (sub_type > 0 && sub_type < NUM_ENTITY_SUB_TYPE_WEAPONS && weapon_database[sub_type].full_name) ? weapon_database[sub_type].full_name : "none";
		size_t n;
		memset (row, 0, sizeof (*row));
		row->aiming = aiming;
		row->sub_type = sub_type;
		snprintf (row->name, sizeof (row->name), "%s", name);
		for (n = 0; row->name[n]; n++)
		{
			if (row->name[n] == ' ' || row->name[n] == '=')
			{
				row->name[n] = '_';
			}
		}
		return row;
	}
	trace.weapon_overflow++;
	return NULL;
}

static int aiming_of (int sub_type)
{
	if (sub_type > 0 && sub_type < NUM_ENTITY_SUB_TYPE_WEAPONS)
	{
		return (int) weapon_database[sub_type].aiming_type;
	}
	return 0;
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
	for (k = 1; k <= 16; k++)
	{
		s = s * 1103515245u + 12345u;
		if ((int) s == after)
		{
			return k;
		}
	}
	return 17;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* vector update */

void eech_aim_vec_begin (entity *launcher)
{
	ready ();
	bump (C_VEC_CALLS);
	note_kind (launcher, 0);
	memset (&cur_vec, 0, sizeof (cur_vec));
	cur_vec.active = 1;
	cur_vec.solve_result = -1;
	cur_vec.source_type = (int) launcher->type;
	fold (&trace.physical_hash, 0xB1000000u | (uint32_t) launcher->type);
	fold (&trace.decision_hash, 0xB1000000u | (uint32_t) launcher->type);
}

void eech_aim_vec_stage (int stage)
{
	if (!cur_vec.active)
	{
		return;
	}
	if (stage > cur_vec.stage)
	{
		cur_vec.stage = stage;
	}
}

void eech_aim_vec_targeted (int has_target)
{
	if (!cur_vec.active)
	{
		return;
	}
	cur_vec.targeted = has_target ? 1 : 0;
	bump (has_target ? C_VEC_TARGETED : C_VEC_PLAYER_WITHOUT_TARGET);
}

void eech_aim_vec_tracking (void)
{
	if (!cur_vec.active)
	{
		return;
	}
	cur_vec.tracking = 1;
	bump (C_VEC_TRACKING_VALID);
}

void eech_aim_vec_end (entity *launcher, const vec3d *weapon_vector, const vec3d *weapon_to_target_vector,
	const vec3d *weapon_to_intercept_point_vector, float weapon_to_intercept_point_range)
{
	int valid;
	char text[SAMPLE_TEXT], key[40];
	ready ();
	if (!cur_vec.active)
	{
		return;
	}
	valid = (cur_vec.tracking == 1) || (cur_vec.solve_result == 1);
	bump (C_VEC_STAGE_0 + cur_vec.stage);
	{
		struct kind_row *row = kind_row_for (cur_vec.source_type, 0);
		if (row)
		{
			if (cur_vec.stage >= 6)
			{
				row->reached_devices++;
			}
			if (valid)
			{
				row->valid++;
			}
		}
	}
	bump (valid ? C_VEC_VALID : C_VEC_INVALID);
	fold (&trace.decision_hash, 0xB2000000u | (uint32_t) (cur_vec.stage << 4) | (uint32_t) (valid ? 1 : 0));
	fold (&trace.physical_hash, 0xB3000000u | (uint32_t) cur_vec.stage);
	fold_vec (&trace.physical_hash, weapon_vector);
	fold_vec (&trace.physical_hash, weapon_to_target_vector);
	fold_vec (&trace.physical_hash, weapon_to_intercept_point_vector);
	fold_f (&trace.physical_hash, weapon_to_intercept_point_range);
	if (cur_vec.stage >= 6)
	{
		note_range (R_VEC_INTERCEPT_RANGE, (double) weapon_to_intercept_point_range);
		text[0] = 0;
		text_append (text, sizeof (text), "source=%s stage=%d valid=%d targeted=%d aiming=%s weapon_vec=(%.9g,%.9g,%.9g) target_vec=(%.9g,%.9g,%.9g) "
			"intercept_vec=(%.9g,%.9g,%.9g) range=%.9g", get_entity_type_name ((entity_types) launcher->type), cur_vec.stage, valid, cur_vec.targeted,
			aiming_names[cur_vec.aiming], (double) weapon_vector->x, (double) weapon_vector->y, (double) weapon_vector->z,
			(double) weapon_to_target_vector->x, (double) weapon_to_target_vector->y, (double) weapon_to_target_vector->z,
			(double) weapon_to_intercept_point_vector->x, (double) weapon_to_intercept_point_vector->y,
			(double) weapon_to_intercept_point_vector->z, (double) weapon_to_intercept_point_range);
		snprintf (key, sizeof (key), "vec/%s/%s", aiming_names[cur_vec.aiming], valid ? "valid" : "invalid");
		keep_sample (key, text);
	}
	else
	{
		text[0] = 0;
		text_append (text, sizeof (text), "source=%s stage=%d valid=%d", get_entity_type_name ((entity_types) launcher->type), cur_vec.stage, valid);
		snprintf (key, sizeof (key), "vec/stage_%d", cur_vec.stage);
		keep_sample (key, text);
	}
	cur_vec.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* pitch-device-to-target solve (static in weapon.c) */

void eech_aim_pitch_solve_begin (void)
{
	ready ();
	memset (&cur_solve, 0, sizeof (cur_solve));
	cur_solve.active = 1;
}

void eech_aim_pitch_solve (entity *source, entity *target, int weapon_sub_type, const vec3d *pitch_device_position, const vec3d *target_vector,
	const vec3d *intercept_point_vector, float intercept_point_range, int result)
{
	int aiming = aiming_of (weapon_sub_type);
	struct weapon_row *row = find_weapon_row (aiming, weapon_sub_type);
	char text[SAMPLE_TEXT], key[40];
	const char *branch = "none";
	(void) target;
	ready ();
	bump (C_SOLVE_CALLS);
	if (!cur_vec.active)
	{
		bump (C_SOLVE_OUTSIDE_VECTOR_UPDATE);
	}
	if (aiming == WEAPON_AIMING_TYPE_CALC_INTERCEPT_POINT)
	{
		bump (cur_solve.intercept_called ? C_SOLVE_GUIDED_MOBILE : C_SOLVE_GUIDED_STATIC);
		branch = cur_solve.intercept_called ? "guided_mobile" : "guided_static";
	}
	else if (aiming == WEAPON_AIMING_TYPE_CALC_ANGLE_OF_PROJECTION)
	{
		bump (C_SOLVE_ANGLE_OF_PROJECTION);
		branch = "angle_of_projection";
	}
	else if (aiming == WEAPON_AIMING_TYPE_CALC_LEAD_AND_BALLISTIC)
	{
		bump (C_SOLVE_LEAD_AND_BALLISTIC);
		branch = "lead_and_ballistic";
	}
	bump (result ? C_SOLVE_TRUE : C_SOLVE_FALSE);
	if (row)
	{
		row->solves++;
		if (result)
		{
			row->solves_true++;
		}
	}
	fold (&trace.physical_hash, 0xC1000000u | (uint32_t) (aiming << 8) | (uint32_t) (result ? 1 : 0) | (uint32_t) (cur_solve.intercept_called ? 2 : 0));
	fold_vec (&trace.physical_hash, pitch_device_position);
	fold_vec (&trace.physical_hash, target_vector);
	fold_vec (&trace.physical_hash, intercept_point_vector);
	fold_f (&trace.physical_hash, intercept_point_range);
	note_range (R_SOLVE_INTERCEPT_RANGE, (double) intercept_point_range);
	cur_vec.solve_result = result ? 1 : 0;
	cur_vec.solve_seen = 1;
	cur_vec.aiming = aiming;
	cur_vec.sub_type = weapon_sub_type;
	text[0] = 0;
	text_append (text, sizeof (text), "source=%s weapon=%d branch=%s result=%d device=(%.9g,%.9g,%.9g) target_vec=(%.9g,%.9g,%.9g) intercept_vec=(%.9g,%.9g,%.9g) range=%.9g",
		get_entity_type_name ((entity_types) source->type), weapon_sub_type, branch, result, (double) pitch_device_position->x,
		(double) pitch_device_position->y, (double) pitch_device_position->z, (double) target_vector->x, (double) target_vector->y,
		(double) target_vector->z, (double) intercept_point_vector->x, (double) intercept_point_vector->y, (double) intercept_point_vector->z,
		(double) intercept_point_range);
	snprintf (key, sizeof (key), "solve/%s/%s", branch, result ? "true" : "false");
	keep_sample (key, text);
	cur_solve.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* guided-weapon intercept prediction (static get_intercept_point in weapon.c) */

static float intercept_weapon_velocity, intercept_target_velocity;
static vec3d intercept_device, intercept_target_point;

void eech_aim_intercept_begin (const vec3d *pitch_device_position, float weapon_velocity, float target_true_velocity, const vec3d *target_point)
{
	ready ();
	bump (C_INTERCEPT_CALLS);
	if (!cur_vec.active)
	{
		bump (C_INTERCEPT_OUTSIDE_VECTOR_UPDATE);
	}
	cur_solve.intercept_called = 1;
	intercept_weapon_velocity = weapon_velocity;
	intercept_target_velocity = target_true_velocity;
	intercept_device = *pitch_device_position;
	intercept_target_point = *target_point;
}

void eech_aim_intercept_end (const vec3d *intercept_point)
{
	int iterated = 0;
	char text[SAMPLE_TEXT];
	ready ();
	if ((intercept_weapon_velocity > 0.001) && (intercept_target_velocity > 0.001))
	{
		if (intercept_target_velocity < intercept_weapon_velocity)
		{
			iterated = 1;
		}
		else
		{
			bump (C_INTERCEPT_TARGET_NOT_SLOWER);
		}
	}
	else if (!(intercept_weapon_velocity > 0.001))
	{
		bump (C_INTERCEPT_WEAPON_NOT_MOVING);
	}
	else
	{
		bump (C_INTERCEPT_TARGET_NOT_MOVING);
	}
	if (iterated)
	{
		bump (C_INTERCEPT_ITERATED);
	}
	fold (&trace.physical_hash, 0xC2000000u | (uint32_t) iterated);
	fold_vec (&trace.physical_hash, &intercept_device);
	fold_f (&trace.physical_hash, intercept_weapon_velocity);
	fold_f (&trace.physical_hash, intercept_target_velocity);
	fold_vec (&trace.physical_hash, &intercept_target_point);
	fold_vec (&trace.physical_hash, intercept_point);
	text[0] = 0;
	text_append (text, sizeof (text), "iterated=%d weapon_velocity=%.9g target_velocity=%.9g target_point=(%.9g,%.9g,%.9g) predicted=(%.9g,%.9g,%.9g)", iterated,
		(double) intercept_weapon_velocity, (double) intercept_target_velocity, (double) intercept_target_point.x, (double) intercept_target_point.y,
		(double) intercept_target_point.z, (double) intercept_point->x, (double) intercept_point->y, (double) intercept_point->z);
	keep_sample (iterated ? "intercept/iterated" : "intercept/not_iterated", text);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* lead-and-ballistic solve */

void eech_aim_lead_begin (void)
{
	ready ();
	memset (&cur_lead, 0, sizeof (cur_lead));
	cur_lead.active = 1;
	cur_lead.rng_before = random_number_seed;
}

void eech_aim_lead_path (int path, float target_true_velocity, int iterations)
{
	if (!cur_lead.active)
	{
		return;
	}
	cur_lead.path = path;
	cur_lead.tov = target_true_velocity;
	cur_lead.iterations = iterations;
	fold (&trace.physical_hash, 0xC3000000u | (uint32_t) path);
	fold_f (&trace.physical_hash, target_true_velocity);
	fold (&trace.physical_hash, (uint32_t) iterations);
}

void eech_aim_lead_range (int phase, int iteration, float range)
{
	if (!cur_lead.active)
	{
		return;
	}
	if (phase == 0)
	{
		cur_lead.range_initial = range;
	}
	else if (phase == 2)
	{
		cur_lead.range_post = range;
		if (iteration == 1)
		{
			bump (C_LEAD_JITTER_APPLIED);
		}
	}
	else
	{
		cur_lead.range_pre = range;
	}
	fold (&trace.physical_hash, 0xC4000000u | (uint32_t) (phase << 8) | (uint32_t) (iteration & 0xff));
	fold_f (&trace.physical_hash, range);
}

void eech_aim_lead_iteration (int iteration, float target_move_distance, const vec3d *new_intercept_point)
{
	if (!cur_lead.active)
	{
		return;
	}
	bump (C_LEAD_ITERATIONS_RUN);
	fold (&trace.physical_hash, 0xC5000000u | (uint32_t) iteration);
	fold_f (&trace.physical_hash, target_move_distance);
	fold_vec (&trace.physical_hash, new_intercept_point);
}

void eech_aim_lead_triangulated (void)
{
	if (!cur_lead.active)
	{
		return;
	}
	cur_lead.triangulated = 1;
	bump (C_LEAD_TRIANGULATED);
}

/* The outputs arrive as pointers: the solver writes the angle before any exit but the time of flight only on its success paths (never on the range
 * check or off-map exits), so a value is read here only where the original wrote it. */
void eech_aim_lead_end (const vec3d *pitch_device_position, int weapon_sub_type, float parent_velocity, entity *source, entity *target,
	const vec3d *intercept_point, const float *angle_of_projection_out, const float *time_of_flight_out, int result)
{
	int draws, have_time_of_flight;
	float angle_of_projection = 0.0f, time_of_flight = 0.0f;
	char text[SAMPLE_TEXT], key[40];
	static const char *const path_names[EECH_AIM_LEAD_PATHS] = { "none", "tiny_range", "off_map", "stationary", "moving" };
	struct weapon_row *row = find_weapon_row (aiming_of (weapon_sub_type), weapon_sub_type);
	(void) target;
	ready ();
	if (!cur_lead.active)
	{
		return;
	}
	draws = draws_between (cur_lead.rng_before, random_number_seed);
	if (cur_lead.path == EECH_AIM_LEAD_PATH_NONE)
	{
		/* every other exit sets its branch first: the solver returned at its range check */
		cur_lead.path = EECH_AIM_LEAD_PATH_TINY_RANGE;
	}
	angle_of_projection = *angle_of_projection_out;
	/* a time of flight the host did not produce is not read (no pointer is given) and not folded */
	have_time_of_flight = result && cur_lead.path != EECH_AIM_LEAD_PATH_OFF_MAP && time_of_flight_out != NULL;
	if (have_time_of_flight)
	{
		time_of_flight = *time_of_flight_out;
	}
	bump (C_LEAD_CALLS);
	if (!cur_vec.active)
	{
		bump (C_LEAD_OUTSIDE_VECTOR_UPDATE);
	}
	bump (result ? C_LEAD_TRUE : C_LEAD_FALSE);
	switch (cur_lead.path)
	{
	case EECH_AIM_LEAD_PATH_TINY_RANGE:
		bump (C_LEAD_PATH_TINY_RANGE);
		break;
	case EECH_AIM_LEAD_PATH_OFF_MAP:
		bump (C_LEAD_PATH_OFF_MAP);
		break;
	case EECH_AIM_LEAD_PATH_STATIONARY:
		bump (C_LEAD_PATH_STATIONARY);
		break;
	case EECH_AIM_LEAD_PATH_MOVING:
		bump (C_LEAD_PATH_MOVING);
		bump (cur_lead.iterations == 5 ? C_LEAD_MOVING_5_ITERATIONS : C_LEAD_MOVING_3_ITERATIONS);
		break;
	default:
		break;
	}
	bump (draws == 0 ? C_LEAD_RNG_DRAWS_0 : (draws == 1 ? C_LEAD_RNG_DRAWS_1 : C_LEAD_RNG_DRAWS_MORE));
	if (row)
	{
		row->leads++;
		if (cur_lead.path == EECH_AIM_LEAD_PATH_MOVING)
		{
			row->leads_moving++;
		}
	}
	/* the draw count is physical behaviour (the solver consumes the shared stream); the value never is */
	fold (&trace.physical_hash, 0xC6000000u | (uint32_t) (result ? 1 : 0) | (uint32_t) (draws << 4) | (uint32_t) (cur_lead.path << 12));
	fold_vec (&trace.physical_hash, pitch_device_position);
	fold (&trace.physical_hash, (uint32_t) weapon_sub_type);
	fold_f (&trace.physical_hash, parent_velocity);
	fold_vec (&trace.physical_hash, intercept_point);
	fold_f (&trace.physical_hash, angle_of_projection);
	/* time of flight is written only on the solver's success paths (the off-map path leaves the caller's value) */
	if (have_time_of_flight)
	{
		fold_f (&trace.physical_hash, time_of_flight);
		note_range (R_LEAD_TIME_OF_FLIGHT, (double) time_of_flight);
	}
	if (cur_lead.path != EECH_AIM_LEAD_PATH_TINY_RANGE)
	{
		note_range (R_LEAD_ANGLE, (double) angle_of_projection);
	}
	note_range (R_LEAD_RANGE, (double) cur_lead.range_initial);
	text[0] = 0;
	text_append (text, sizeof (text), "source=%s weapon=%d path=%s result=%d iterations=%d tov=%.9g parent_velocity=%.9g rng_draws=%d range_initial=%.9g range_pre=%.9g range_post=%.9g "
		"intercept=(%.9g,%.9g,%.9g) angle=%.9g", get_entity_type_name ((entity_types) source->type), weapon_sub_type, path_names[cur_lead.path], result,
		cur_lead.iterations, (double) cur_lead.tov, (double) parent_velocity, draws, (double) cur_lead.range_initial, (double) cur_lead.range_pre, (double) cur_lead.range_post,
		(double) intercept_point->x, (double) intercept_point->y, (double) intercept_point->z, (double) angle_of_projection);
	if (have_time_of_flight)
	{
		text_append (text, sizeof (text), " tof=%.9g", (double) time_of_flight);
	}
	snprintf (key, sizeof (key), "lead/%s/%s", path_names[cur_lead.path], result ? "true" : "false");
	keep_sample (key, text);
	cur_lead.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* ballistic table lookup (wn_move.c) */

void eech_aim_ballistic (int weapon_sub_type, float range, float height_diff_or_pitch, int simplified, int fixed_pitch, float parent_velocity, int result,
	const float *aiming_pitch_out, const float *time_of_flight_out)
{
	/* the table lookup writes both outputs on every success path and neither on a failure: read them only after a success */
	float aiming_pitch = 0.0f, time_of_flight = 0.0f;
	char text[SAMPLE_TEXT], key[40];
	struct weapon_row *row = find_weapon_row (aiming_of (weapon_sub_type), weapon_sub_type);
	ready ();
	if (result)
	{
		aiming_pitch = *aiming_pitch_out;
		time_of_flight = *time_of_flight_out;
	}
	bump (C_BALLISTIC_CALLS);
	if (!cur_vec.active)
	{
		bump (C_BALLISTIC_OUTSIDE_VECTOR_UPDATE);
	}
	if (cur_lead.active)
	{
		bump (C_BALLISTIC_FROM_LEAD);
	}
	bump (result ? C_BALLISTIC_TRUE : C_BALLISTIC_FALSE);
	if (simplified)
	{
		bump (C_BALLISTIC_SIMPLIFIED);
	}
	if (fixed_pitch)
	{
		bump (C_BALLISTIC_FIXED_PITCH);
	}
	if (row)
	{
		row->ballistic++;
	}
	fold (&trace.physical_hash, 0xC7000000u | (uint32_t) (result ? 1 : 0) | (uint32_t) (simplified ? 2 : 0) | (uint32_t) (fixed_pitch ? 4 : 0));
	fold (&trace.physical_hash, (uint32_t) weapon_sub_type);
	fold_f (&trace.physical_hash, range);
	fold_f (&trace.physical_hash, height_diff_or_pitch);
	fold_f (&trace.physical_hash, parent_velocity);
	note_range (R_BALLISTIC_RANGE, (double) range);
	note_range (R_BALLISTIC_PARENT_VELOCITY, (double) parent_velocity);
	if (result)
	{
		/* both outputs are written on every success path */
		fold_f (&trace.physical_hash, aiming_pitch);
		fold_f (&trace.physical_hash, time_of_flight);
		note_range (R_BALLISTIC_TIME_OF_FLIGHT, (double) time_of_flight);
		if (time_of_flight == 0.0f)
		{
			bump (C_BALLISTIC_POINT_BLANK_ZERO_TIME);
		}
	}
	text[0] = 0;
	text_append (text, sizeof (text), "weapon=%d range=%.9g height_or_pitch=%.9g simplified=%d fixed_pitch=%d parent_velocity=%.9g result=%d", weapon_sub_type,
		(double) range, (double) height_diff_or_pitch, simplified, fixed_pitch, (double) parent_velocity, result);
	if (result)
	{
		text_append (text, sizeof (text), " pitch=%.9g tof=%.9g", (double) aiming_pitch, (double) time_of_flight);
	}
	snprintf (key, sizeof (key), "ballistic/%s/%s", fixed_pitch ? "fixed" : "free", result ? "true" : "false");
	keep_sample (key, text);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the reference host's echo of its predictions, replayed in its own order into the hooks the original solver bodies called */

void eech_aim_replay_echo (struct campaign_aim_result *result)
{
	int i;
	if (!result->has_echo)
	{
		return;
	}
	for (i = 0; i < result->echo_count; i++)
	{
		const campaign_aim_event *event = &result->echo[i];
		switch (event->kind)
		{
		case CAMPAIGN_AIM_EVENT_INTERCEPT:
		{
			vec3d device, target_point, predicted;
			device.x = event->f[0]; device.y = event->f[1]; device.z = event->f[2];
			target_point.x = event->f[5]; target_point.y = event->f[6]; target_point.z = event->f[7];
			predicted.x = event->f[8]; predicted.y = event->f[9]; predicted.z = event->f[10];
			eech_aim_intercept_begin (&device, event->f[3], event->f[4], &target_point);
			eech_aim_intercept_end (&predicted);
			break;
		}
		case CAMPAIGN_AIM_EVENT_LEAD_PATH:
			eech_aim_lead_path (event->a, event->f[0], event->b);
			break;
		case CAMPAIGN_AIM_EVENT_LEAD_RANGE:
			eech_aim_lead_range (event->a, event->b, event->f[0]);
			break;
		case CAMPAIGN_AIM_EVENT_LEAD_ITERATION:
		{
			vec3d point;
			point.x = event->f[1]; point.y = event->f[2]; point.z = event->f[3];
			eech_aim_lead_iteration (event->a, event->f[0], &point);
			break;
		}
		case CAMPAIGN_AIM_EVENT_BALLISTIC:
			eech_aim_ballistic (event->a, event->f[0], event->f[1], event->b, event->c, event->f[2], event->d, &event->f[3], &event->f[4]);
			break;
		case CAMPAIGN_AIM_EVENT_LEAD_RESULT:
			/* the prediction's own outputs inside a vectors answer (the answer itself carries only the vectors): the trace of the prediction reads them here */
			result->intercept_point[0] = event->f[0];
			result->intercept_point[1] = event->f[1];
			result->intercept_point[2] = event->f[2];
			result->angle = event->f[3];
			/* the time of flight only when the prediction wrote one (event->a); an absent one is 0 and the hook below is given no pointer to read */
			result->time_of_flight = event->a ? event->f[4] : 0.0f;
			result->has_time_of_flight = event->a ? 1 : 0;
			break;
		}
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* weapon-system update: readiness and device slewing */

void eech_aim_ws_begin (entity *source)
{
	ready ();
	bump (C_WS_CALLS);
	note_kind (source, 1);
	memset (&cur_ws, 0, sizeof (cur_ws));
	cur_ws.active = 1;
	cur_ws.source_type = (int) source->type;
	fold (&trace.physical_hash, 0xD1000000u | (uint32_t) source->type);
	fold (&trace.decision_hash, 0xD1000000u | (uint32_t) source->type);
}

void eech_aim_ws_stage (int stage)
{
	if (!cur_ws.active)
	{
		return;
	}
	if (stage > cur_ws.stage)
	{
		cur_ws.stage = stage;
	}
}

void eech_aim_ws_ready (int found_device, int make_selected_ready, int selected_weapon_system_ready)
{
	if (!cur_ws.active)
	{
		return;
	}
	if (found_device)
	{
		bump (C_WS_READY_DEVICE_FOUND);
	}
	if (make_selected_ready)
	{
		bump (C_WS_MAKE_SELECTED_READY);
	}
	fold (&trace.decision_hash, 0xD2000000u | (uint32_t) (found_device ? 1 : 0) | (uint32_t) (make_selected_ready ? 2 : 0)
		| (uint32_t) (selected_weapon_system_ready ? 4 : 0));
}

void eech_aim_ws_required (int package, int located, float required_heading, float required_pitch, float current_heading, float current_pitch)
{
	if (!cur_ws.active)
	{
		return;
	}
	bump (C_WS_PACKAGES_ROTATING);
	if (located)
	{
		bump (C_WS_REQUIRED_LOCATED);
	}
	if (float_bits (required_heading) != float_bits (current_heading) || float_bits (required_pitch) != float_bits (current_pitch))
	{
		bump (C_WS_REQUIRED_DIFFERS);
	}
	fold (&trace.physical_hash, 0xD3000000u | (uint32_t) package | (uint32_t) (located ? 0x100 : 0));
	fold_f (&trace.physical_hash, required_heading);
	fold_f (&trace.physical_hash, required_pitch);
	fold_f (&trace.physical_hash, current_heading);
	fold_f (&trace.physical_hash, current_pitch);
	cur_ws.required_heading = required_heading;
	cur_ws.required_pitch = required_pitch;
}

void eech_aim_ws_step (int package, float delta_heading, float delta_pitch, float new_heading, float new_pitch, int heading_shared, int pitch_shared)
{
	char text[SAMPLE_TEXT];
	if (!cur_ws.active)
	{
		return;
	}
	/* the step stopped short of the required offset: the turn rate limited it, or the heading wrapped */
	if (float_bits (new_heading) != float_bits (cur_ws.required_heading))
	{
		bump (C_WS_STEP_HEADING_SHORT);
	}
	if (float_bits (new_pitch) != float_bits (cur_ws.required_pitch))
	{
		bump (C_WS_STEP_PITCH_SHORT);
	}
	if (heading_shared)
	{
		bump (C_WS_STEP_HEADING_SHARED);
	}
	if (pitch_shared)
	{
		bump (C_WS_STEP_PITCH_SHARED);
	}
	fold (&trace.physical_hash, 0xD4000000u | (uint32_t) package | (uint32_t) (heading_shared ? 0x100 : 0) | (uint32_t) (pitch_shared ? 0x200 : 0));
	fold_f (&trace.physical_hash, delta_heading);
	fold_f (&trace.physical_hash, delta_pitch);
	fold_f (&trace.physical_hash, new_heading);
	fold_f (&trace.physical_hash, new_pitch);
	note_range (R_WS_STEP_HEADING, (double) delta_heading);
	{
		struct kind_row *row = kind_row_for (cur_ws.source_type, 1);
		if (row)
		{
			row->steps++;
		}
	}
	text[0] = 0;
	text_append (text, sizeof (text), "package=%d delta_heading=%.9g delta_pitch=%.9g new_heading=%.9g new_pitch=%.9g heading_shared=%d pitch_shared=%d", package,
		(double) delta_heading, (double) delta_pitch, (double) new_heading, (double) new_pitch, heading_shared, pitch_shared);
	keep_sample (heading_shared || pitch_shared ? "ws/step/shared" : "ws/step", text);
}

void eech_aim_ws_edge (int edge)
{
	static const int counters[EECH_AIM_WS_EDGES] = { C_WS_EDGE_CLIMB_BUMP, C_WS_EDGE_HOLD_NOT_READY, C_WS_EDGE_NON_AI_PLAYER, C_WS_EDGE_PLAYER_LOCAL,
		C_WS_EDGE_POINT_LOCK };
	if (!cur_ws.active)
	{
		return;
	}
	bump (counters[edge]);
	fold (&trace.decision_hash, 0xD5000000u | (uint32_t) edge);
}

static void fold_devices (const weapon_package_status *package_status, int config_type)
{
	int package;
	if (!package_status || !weapon_config_type_valid (config_type))
	{
		return;
	}
	for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
	{
		if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			fold_f (&trace.physical_hash, package_status[package].weapon_system_heading);
			fold_f (&trace.physical_hash, package_status[package].weapon_system_pitch);
		}
	}
}

void eech_aim_ws_exit (entity *source, int reason, const weapon_package_status *package_status, int config_type)
{
	static const int counters[EECH_AIM_WS_EXITS] = { C_WS_EXIT_NO_VIEWPOINT, C_WS_EXIT_NO_HEADING_DEVICE, C_WS_EXIT_NO_PITCH_DEVICE };
	char text[SAMPLE_TEXT], key[40];
	ready ();
	if (!cur_ws.active)
	{
		return;
	}
	bump (counters[reason]);
	bump (C_WS_ARMED);
	fold (&trace.decision_hash, 0xD6000000u | (uint32_t) reason);
	fold (&trace.physical_hash, 0xD6000000u | (uint32_t) reason);
	fold_devices (package_status, config_type);
	text[0] = 0;
	text_append (text, sizeof (text), "source=%s exit=%d", get_entity_type_name ((entity_types) source->type), reason);
	snprintf (key, sizeof (key), "ws/exit/%d", reason);
	keep_sample (key, text);
	cur_ws.active = 0;
}

void eech_aim_ws_end (entity *source, int selected_weapon_system_ready, const weapon_package_status *package_status, int config_type)
{
	char text[SAMPLE_TEXT];
	ready ();
	if (!cur_ws.active)
	{
		return;
	}
	if (!package_status)
	{
		bump (C_WS_NO_PACKAGE_STATUS);
	}
	else if (!weapon_config_type_valid (config_type) || config_type == WEAPON_CONFIG_TYPE_UNARMED)
	{
		bump (C_WS_UNARMED);
	}
	else
	{
		bump (C_WS_ARMED);
		bump (selected_weapon_system_ready ? C_WS_READY_TRUE : C_WS_READY_FALSE);
		fold (&trace.decision_hash, 0xD7000000u | (uint32_t) (selected_weapon_system_ready ? 1 : 0));
		fold (&trace.physical_hash, 0xD7000000u);
		fold_devices (package_status, config_type);
		text[0] = 0;
		text_append (text, sizeof (text), "source=%s ready=%d", get_entity_type_name ((entity_types) source->type), selected_weapon_system_ready);
		keep_sample (selected_weapon_system_ready ? "ws/end/ready" : "ws/end/not_ready", text);
	}
	cur_ws.active = 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the fire decision's reads of the physical result */

/* the readiness state machine (aircraft and vehicle producers, advanced every update whether or not the entity is drawn) */
void eech_aim_ready_update (int vehicle, entity *en, float state_after, int state_before)
{
	char text[SAMPLE_TEXT], key[40];
	static const char *const state_names[4] = { "closed", "opening", "open", "closing" };
	struct kind_row *row;
	ready ();
	bump (vehicle ? C_READY_UPDATE_VEHICLE : C_READY_UPDATE_AIRCRAFT);
	if (state_before >= 0 && state_before <= 3)
	{
		bump (C_READY_STATE_START + state_before);
	}
	note_kind (en, 3);
	row = kind_row_for ((int) en->type, 3);
	if (row && (state_before == 1 || state_before == 3))
	{
		row->steps++;
	}
	fold (&trace.physical_hash, 0xE4000000u | (uint32_t) (vehicle ? 0x100 : 0) | (uint32_t) (state_before & 0xff));
	fold_f (&trace.physical_hash, state_after);
	note_range (R_READY_STATE_AFTER, (double) state_after);
	if (state_before == 1 || state_before == 3)
	{
		text[0] = 0;
		text_append (text, sizeof (text), "source=%s state_before=%s state_after=%.9g", get_entity_type_name ((entity_types) en->type),
			state_before >= 0 && state_before <= 3 ? state_names[state_before] : "invalid", (double) state_after);
		snprintf (key, sizeof (key), "ready/update/%s", state_before == 1 ? "opening" : "closing");
		keep_sample (key, text);
	}
}

void eech_aim_ready_command (entity *en, int open, float new_state)
{
	char text[SAMPLE_TEXT];
	ready ();
	bump (open ? C_READY_COMMAND_OPEN : C_READY_COMMAND_CLOSE);
	fold (&trace.physical_hash, 0xE5000000u | (uint32_t) (open ? 1 : 0));
	fold_f (&trace.physical_hash, new_state);
	text[0] = 0;
	text_append (text, sizeof (text), "source=%s command=%s new_state=%.9g", get_entity_type_name ((entity_types) en->type), open ? "open" : "close", (double) new_state);
	keep_sample (open ? "ready/command/open" : "ready/command/close", text);
}

void eech_aim_angle (int site, entity *launcher, float launch_angle_error, float limit)
{
	char text[SAMPLE_TEXT], key[40];
	static const char *const site_names[EECH_AIM_SITES] = { "aircraft_fire", "aircraft_update", "vehicle_update" };
	ready ();
	bump (C_ANGLE_EVAL_AIRCRAFT_FIRE + site);
	fold (&trace.physical_hash, 0xE1000000u | (uint32_t) site);
	fold_f (&trace.physical_hash, launch_angle_error);
	fold_f (&trace.physical_hash, limit);
	note_range (R_LAUNCH_ANGLE_ERROR, (double) launch_angle_error);
	note_range (R_LAUNCH_ANGLE_LIMIT, (double) limit);
	text[0] = 0;
	text_append (text, sizeof (text), "source=%s error=%.9g limit=%.9g", get_entity_type_name ((entity_types) launcher->type), (double) launch_angle_error, (double) limit);
	snprintf (key, sizeof (key), "angle/%s", site_names[site]);
	keep_sample (key, text);
}

void eech_aim_reject (int site, int reason)
{
	ready ();
	bump (C_REJECT_AIRCRAFT_FIRE_NOT_READY + site * EECH_AIM_REJECTS + reason);
	fold (&trace.decision_hash, 0xE2000000u | (uint32_t) (site << 4) | (uint32_t) reason);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* person target-facing (ps_anim.c) */

void eech_aim_person_branch (entity *person, int has_target_link)
{
	ready ();
	bump (C_PERSON_BRANCH_ENTERED);
	if (has_target_link)
	{
		bump (C_PERSON_BRANCH_WITH_TARGET_LINK);
	}
	note_kind (person, 2);
}

void eech_aim_person_facing (entity *person, const vec3d *position, const vec3d *target_position, float heading)
{
	char text[SAMPLE_TEXT];
	ready ();
	bump (C_PERSON_FACING);
	fold (&trace.physical_hash, 0xE3000000u);
	fold_vec (&trace.physical_hash, position);
	fold_vec (&trace.physical_hash, target_position);
	fold_f (&trace.physical_hash, heading);
	note_range (R_PERSON_HEADING, (double) heading);
	text[0] = 0;
	text_append (text, sizeof (text), "position=(%.9g,%.9g,%.9g) target=(%.9g,%.9g,%.9g) heading=%.9g", (double) position->x, (double) position->y,
		(double) position->z, (double) target_position->x, (double) target_position->y, (double) target_position->z, (double) heading);
	keep_sample ("person/facing", text);
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
	int i;
	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	text[0] = 0;
	emit (&out, "aim-hash physical=%016llx decision=%016llx\n", (unsigned long long) trace.physical_hash, (unsigned long long) trace.decision_hash);
	for (i = 0; i < COUNTERS; i++)
	{
		emit (&out, "aim-counter name=%s value=%llu\n", counter_names[i], (unsigned long long) trace.counters[i]);
	}
	for (i = 0; i < RANGES; i++)
	{
		emit (&out, "aim-range name=%s n=%llu min=%.9g max=%.9g\n", range_names[i], (unsigned long long) trace.range_n[i],
			trace.range_n[i] ? trace.range_min[i] : 0.0, trace.range_n[i] ? trace.range_max[i] : 0.0);
	}
	for (i = 0; i < trace.kind_count; i++)
	{
		emit (&out, "aim-kind source=%s function=%s calls=%llu reached_devices=%llu valid=%llu steps=%llu\n",
			get_entity_type_name ((entity_types) trace.kinds[i].source_type), function_names[trace.kinds[i].function],
			(unsigned long long) trace.kinds[i].calls, (unsigned long long) trace.kinds[i].reached_devices,
			(unsigned long long) trace.kinds[i].valid, (unsigned long long) trace.kinds[i].steps);
	}
	emit (&out, "aim-kinds rows=%d overflow=%d\n", trace.kind_count, trace.kind_overflow);
	for (i = 0; i < trace.weapon_count; i++)
	{
		emit (&out, "aim-weapon aiming=%s sub_type=%d name=%s solves=%llu solves_true=%llu leads=%llu leads_moving=%llu ballistic=%llu\n",
			aiming_names[trace.weapons[i].aiming], trace.weapons[i].sub_type, trace.weapons[i].name, (unsigned long long) trace.weapons[i].solves,
			(unsigned long long) trace.weapons[i].solves_true, (unsigned long long) trace.weapons[i].leads,
			(unsigned long long) trace.weapons[i].leads_moving, (unsigned long long) trace.weapons[i].ballistic);
	}
	emit (&out, "aim-weapons rows=%d overflow=%d\n", trace.weapon_count, trace.weapon_overflow);
	for (i = 0; i < trace.sample_count; i++)
	{
		emit (&out, "aim-sample key=%s %s\n", trace.samples[i].key, trace.samples[i].text);
	}
	emit (&out, "aim-samples count=%d overflow=%d\n", trace.sample_count, trace.sample_overflow);
	return out.overflow ? -1 : 0;
}

int eech_aim_trace_get (char *text, int capacity)
{
	ready ();
	return report (text, capacity);
}

void eech_aim_trace_clear (void)
{
	trace_reset ();
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* controlled person case (EEDCS-44): one real host-identified person is given a real host-identified target through the campaign's own target-link
 * API, then the game steps normally. Only host ids cross; no pointer, index or seed leaves this file. */

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

static int case_state (const char *label, const char *host_id, entity *en, char *text, int capacity)
{
	vec3d position;
	matrix3x3 attitude;
	entity *target = get_local_entity_parent (en, LIST_TYPE_TARGET);
	vec3d target_position;
	const char *target_id = target ? campaign_roster_host_id (target) : NULL;
	int n;
	get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &position);
	get_local_entity_attitude_matrix (en, attitude);
	target_position.x = target_position.y = target_position.z = 0.0f;
	if (target)
	{
		get_local_entity_vec3d (target, VEC3D_TYPE_POSITION, &target_position);
	}
	n = snprintf (text, (size_t) capacity,
		"aim-case-state label=%s host_id=%s type=%s alive=%d position=%.9g,%.9g,%.9g attitude=%.9g,%.9g,%.9g,%.9g,%.9g,%.9g,%.9g,%.9g,%.9g has_target=%d target_host_id=%s "
		"target_position=%.9g,%.9g,%.9g\n",
		label, host_id, get_entity_type_name ((entity_types) en->type), get_local_entity_int_value (en, INT_TYPE_ALIVE), (double) position.x, (double) position.y,
		(double) position.z, (double) attitude[0][0], (double) attitude[0][1], (double) attitude[0][2], (double) attitude[1][0], (double) attitude[1][1],
		(double) attitude[1][2], (double) attitude[2][0], (double) attitude[2][1], (double) attitude[2][2], target ? 1 : 0, target_id ? target_id : "none",
		(double) target_position.x, (double) target_position.y, (double) target_position.z);
	return (n < 0 || n >= capacity) ? -1 : n;
}

int eech_aim_person_query_run (const char *host_id, char *text, int capacity)
{
	entity *en = case_find (host_id);
	text[0] = 0;
	if (!en)
	{
		snprintf (text, (size_t) capacity, "aim-case-error reason=no_such_host_id host_id=%s\n", host_id);
		return -1;
	}
	return case_state ("query", host_id, en, text, capacity) < 0 ? -1 : 0;
}

/* gives `person` the target through the campaign's own set_client_server_entity_parent (the call the family target code makes) and reports before and after;
 * an empty target id releases the link through the same call (target NULL), so the person returns to its own movement */
/* TEST-ONLY: the campaign's own lead-and-ballistic function (weapon.c, the one weapon launch calls for the decoy and fuze timer) asked once for two host-identified live
 * entities, with the caller's time-of-flight variable set to `sentinel`. The host answers through the real `aim` callback; the report says what the function returned and
 * whether the caller's variable still holds the sentinel (the partial-output contract: it is stored only when the host produced a value). */
int eech_aim_lead_probe_run (const char *source_id, const char *target_id, int weapon, float sentinel, char *text, int capacity)
{
	entity *source = case_find (source_id);
	entity *target = case_find (target_id);
	vec3d device, intercept_point;
	float angle_of_projection, time_of_flight;
	int result, n;
	text[0] = 0;
	if (!source || !target || source == target || !get_local_entity_int_value (source, INT_TYPE_ALIVE) || !get_local_entity_int_value (target, INT_TYPE_ALIVE)
		|| weapon <= 0 || weapon >= NUM_ENTITY_SUB_TYPE_WEAPONS)
	{
		snprintf (text, (size_t) capacity, "aim-lead-probe-error reason=entities_or_weapon_not_valid\n");
		return -1;
	}
	get_local_entity_vec3d (source, VEC3D_TYPE_POSITION, &device);
	angle_of_projection = -7.0f;
	time_of_flight = sentinel;
	result = get_lead_and_ballistic_intercept_point_and_angle_of_projection (&device, (entity_sub_types) weapon, 0.0f, source, target, &intercept_point, &angle_of_projection,
		&time_of_flight);
	n = snprintf (text, (size_t) capacity, "aim-lead-probe result=%d time_of_flight_bits=%08x sentinel_bits=%08x kept_sentinel=%d time_of_flight=%.9g angle=%.9g\n", result,
		float_bits (time_of_flight), float_bits (sentinel), float_bits (time_of_flight) == float_bits (sentinel), (double) time_of_flight, (double) angle_of_projection);
	return (n < 0 || n >= capacity) ? -1 : 0;
}

int eech_aim_person_case_run (const char *person_id, const char *target_id, char *text, int capacity)
{
	entity *person = case_find (person_id);
	int release = !target_id[0];
	entity *target = release ? NULL : case_find (target_id);
	int used, n;
	text[0] = 0;
	if (!person || (!release && !target))
	{
		snprintf (text, (size_t) capacity, "aim-case-error reason=%s\n", !person ? "no_such_person_host_id" : "no_such_target_host_id");
		return -1;
	}
	if (person == target || get_local_entity_type (person) != ENTITY_TYPE_PERSON || !get_local_entity_int_value (person, INT_TYPE_ALIVE)
		|| (!release && !get_local_entity_int_value (target, INT_TYPE_ALIVE)))
	{
		snprintf (text, (size_t) capacity, "aim-case-error reason=person_or_target_not_valid\n");
		return -1;
	}
	used = case_state ("before", person_id, person, text, capacity);
	if (used < 0)
	{
		return -1;
	}
	set_client_server_entity_parent (person, LIST_TYPE_TARGET, target);
	if (get_local_entity_parent (person, LIST_TYPE_TARGET) != target)
	{
		snprintf (text + used, (size_t) (capacity - used), "aim-case-error reason=target_link_not_set\n");
		return -1;
	}
	n = case_state (release ? "after_release" : "after_setup", person_id, person, text + used, capacity - used);
	return n < 0 ? -1 : 0;
}

#endif

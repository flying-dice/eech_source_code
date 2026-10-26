/*
 * The physical aiming and weapon-system readiness of EECH's weapon systems (EEDCS-44, D2): what a launcher's weapon-system devices do and where
 * they point. It is EECH's own arithmetic, moved here expression for expression from
 *   - aphavoc/source/entity/mobile/weapon/weapon.c: get_pitch_device_to_target_vector, the guided get_intercept_point,
 *     get_lead_and_ballistic_intercept_point_and_angle_of_projection, and the required offsets and the rate-limited step of the device slew in
 *     update_entity_weapon_systems;
 *   - aphavoc/source/entity/mobile/weapon/wn_move.c: get_ballistic_pitch_deflection (the table interpolation and its iteration);
 *   - modules/maths/projtile.c: get_angle_of_projection (the off-map fall-back of the lead solver);
 *   - aphavoc/source/entity/en_misc/en_anim/key_anim.c and en_anim.c, aircraft/ac_anim.c, vehicle/vh_anim.c: the keyframed weapon-system-ready
 *     state, advanced every update and commanded open or closed;
 * so the host and the original reference produce the same bits. Neither a rounding mode nor a type is chosen here: the engine runs a host
 * callback under round-toward-zero and every float and double expression below is the engine's own, in its own order.
 *
 * What is kept exactly: the intercept iteration (three passes, in float), the lead iteration (three passes, five for a fast target or the
 * player's gunship) with the time-of-flight term in double and the final range jitter `5 * sfrand1 ()` (the campaign's draw, passed as the
 * raw 15-bit sample and turned into the double here by the macro's own expression), the table interpolation (the iterations, the weighting
 * and the 45 degree drop limit), the required offsets (`atan2`, the bounds and the ten degree climb for a ground-launched guided missile),
 * the rate-limited step with its wrap at plus and minus 180 degrees, and the keyframe state machine.
 *
 * What is not here: which weapon is selected and which target it is aimed at, which package's devices are used, the ammunition and damage
 * state, whether the weapon system should be made ready, the fire decision (the launch-angle threshold, burst and salvo timers, range, altitude
 * and line of sight), the random generator and its seed, and the projectile flight that produced the ballistic tables. The tables are raw
 * weapon data the campaign supplies once (they are the product of the still-core projectile simulation); every use of them is here.
 * The pitch device's world viewpoint, the target point, the entity positions, speeds and motion vectors and the weapon and package figures are
 * transitional mirrors of what a world-owning host already knows, supplied as inputs of this reference model: they do not make the campaign the
 * owner of a physical solver, and a host with another model need not use them.
 *
 * Every function is a pure function of its arguments and the tables: the host keeps nothing between messages. A lead solve that reaches its
 * random jitter without a draw reports `need_draw` and is asked again with the draw.
 */
#include <stddef.h>

#include "eech_arith.h"

#define G (9.81f)
#define deg(RAD) ((RAD) * (180.0f / PI))

#define RANGE_STEP 20.0
#define PITCH_STEP 5
#define VELOCITY_TESTS 1 + 4
#define VELOCITY_TEST_STEP 50.0
#define NUM_SINGLE_DEGREE_VALUES 61
#define NUM_NEGATIVE_PITCH_VALUES (((90 - (NUM_SINGLE_DEGREE_VALUES - 1) / 2) / PITCH_STEP))
#define TOTAL_PITCH_INDICES (2*NUM_NEGATIVE_PITCH_VALUES + NUM_SINGLE_DEGREE_VALUES)

#define TRUE 1
#define FALSE 0

#define AIM_ECHO_MAX 48

/* the echo event kinds: the numbers are the campaign's campaign_aim_event_kind values */
#define AIM_EVENT_INTERCEPT 1
#define AIM_EVENT_LEAD_PATH 2
#define AIM_EVENT_LEAD_RANGE 3
#define AIM_EVENT_LEAD_ITERATION 4
#define AIM_EVENT_BALLISTIC 5

#define AIM_LEAD_PATH_TINY_RANGE 1
#define AIM_LEAD_PATH_OFF_MAP 2
#define AIM_LEAD_PATH_STATIONARY 3
#define AIM_LEAD_PATH_MOVING 4

#define AIM_AIMING_TYPE_NONE 0
#define AIM_AIMING_TYPE_CALC_INTERCEPT_POINT 1
#define AIM_AIMING_TYPE_CALC_ANGLE_OF_PROJECTION 2
#define AIM_AIMING_TYPE_CALC_LEAD_AND_BALLISTIC 3

#define KEYFRAME_STATE_START_FLOAT_VALUE 0.0
#define KEYFRAME_STATE_ANIMATING_FORWARDS_FLOAT_VALUE 1.0
#define KEYFRAME_STATE_END_FLOAT_VALUE 2.0
#define KEYFRAME_STATE_ANIMATING_REVERSE_FLOAT_VALUE 3.0
#define KEYFRAME_STATE_WRAP_FLOAT_VALUE 4.0

enum
{
	KEYFRAME_STATE_START,
	KEYFRAME_STATE_ANIMATING_FORWARDS,
	KEYFRAME_STATE_END,
	KEYFRAME_STATE_ANIMATING_REVERSE
};

typedef struct eech_aim_event
{
	int kind;
	int a;
	int b;
	int c;
	int d;
	float f[12];
} eech_aim_event;

typedef struct eech_aim_echo
{
	int count;
	int overflow;
	eech_aim_event events[AIM_ECHO_MAX];
} eech_aim_echo;

/* the ballistic table of one weapon: cells[(((k * pitch_indices) + pitch) * range_values + range) * 2 + (0 drop angle, 1 flight time)] */
typedef struct eech_aim_table
{
	int present;
	int velocity_tests;
	int pitch_indices;
	int range_values;
	float max_range;
	const float *cells;
} eech_aim_table;

static eech_aim_event *echo_next(eech_aim_echo *echo, int kind)
{
	eech_aim_event *event;

	if (!echo)
	{
		return NULL;
	}

	if (echo->count >= AIM_ECHO_MAX)
	{
		echo->overflow = TRUE;

		return NULL;
	}

	event = &echo->events[echo->count++];

	memset(event, 0, sizeof(*event));

	event->kind = kind;

	return event;
}

static float get_3d_range(const vec3d *v1, const vec3d *v2)
{
	float dx, dy, dz, range;

	dx = v1->x - v2->x;
	dy = v1->y - v2->y;
	dz = v1->z - v2->z;

	range = sqrt((dx * dx) + (dy * dy) + (dz * dz));

	return (range);
}

static float get_2d_range(const vec3d *v1, const vec3d *v2)
{
	float dx, dz, range;

	dx = v1->x - v2->x;
	dz = v1->z - v2->z;

	range = sqrt((dx * dx) + (dz * dz));

	return (range);
}

/* maths/convert.h, patched for 64 bits (docs/patches.md X1): a C conversion */
static void convert_float_to_int(float value, int *integer)
{
	*integer = (int) value;
}

/* maths/miscmath.c */
static float fast_modf(float value, float *iptr)
{
	int whole_value;

	convert_float_to_int(value, &whole_value);

	value -= whole_value;

	*iptr = (float) whole_value;

	return (value);
}

/* key_anim.c */
static void get_keyframed_animation_state(float value, int *state, float *remainder)
{
	float state_float_part, state_int_part;

	state_float_part = fast_modf(value, &state_int_part);

	if (state)
	{
		convert_float_to_int(state_int_part, state);
	}

	if (remainder)
	{
		*remainder = state_float_part;
	}
}

/* ac_anim.c update_aircraft_weapon_system_ready, vh_anim.c update_vehicle_weapon_system_ready: update_entity_reversable_keyframed_value.
   Returns the state the keyframe was in BEFORE the update (0 closed, 1 opening, 2 open, 3 closing) and writes the advanced state. */
int eech_aim_ready_update(float state, float animation_rate, float delta_time, float *state_out)
{
	int int_state;
	float anim_scale, delta_state, animation_state;

	animation_state = state;

	get_keyframed_animation_state(animation_state, &int_state, &anim_scale);

	switch (int_state)
	{
		case KEYFRAME_STATE_START:
		case KEYFRAME_STATE_END:
		{
			break;
		}
		case KEYFRAME_STATE_ANIMATING_FORWARDS:
		case KEYFRAME_STATE_ANIMATING_REVERSE:
		{
			delta_state = animation_rate * delta_time;

			animation_state += delta_state;

			if (int_state <= KEYFRAME_STATE_END_FLOAT_VALUE)
			{
				if (animation_state >= KEYFRAME_STATE_END_FLOAT_VALUE)
				{
					animation_state = KEYFRAME_STATE_END_FLOAT_VALUE;
				}
			}
			else
			{
				if (animation_state >= KEYFRAME_STATE_WRAP_FLOAT_VALUE)
				{
					animation_state = KEYFRAME_STATE_START_FLOAT_VALUE;
				}
			}
		}
	}

	*state_out = animation_state;

	return int_state;
}

/* en_anim.c open_entity_weapon_system_ready / close_entity_weapon_system_ready: set_entity_keyframe_value_animating_forwards / _reverse */
float eech_aim_ready_command(float animation_state, int open)
{
	int int_state;
	float anim_scale, new_state;

	new_state = 0.0f;

	get_keyframed_animation_state(animation_state, &int_state, &anim_scale);

	if (open)
	{
		switch (int_state)
		{
			case KEYFRAME_STATE_ANIMATING_FORWARDS:
			case KEYFRAME_STATE_END:
			{
				new_state = animation_state;

				break;
			}
			case KEYFRAME_STATE_START:
			{
				new_state = KEYFRAME_STATE_ANIMATING_FORWARDS_FLOAT_VALUE;

				break;
			}
			case KEYFRAME_STATE_ANIMATING_REVERSE:
			{
				new_state = KEYFRAME_STATE_END_FLOAT_VALUE - anim_scale;

				break;
			}
		}
	}
	else
	{
		switch (int_state)
		{
			case KEYFRAME_STATE_ANIMATING_REVERSE:
			case KEYFRAME_STATE_START:
			{
				new_state = animation_state;

				break;
			}
			case KEYFRAME_STATE_END:
			{
				new_state = KEYFRAME_STATE_ANIMATING_REVERSE_FLOAT_VALUE;

				break;
			}
			case KEYFRAME_STATE_ANIMATING_FORWARDS:
			{
				new_state = KEYFRAME_STATE_WRAP_FLOAT_VALUE - anim_scale;

				if (new_state >= KEYFRAME_STATE_WRAP_FLOAT_VALUE)
				{
					new_state = KEYFRAME_STATE_START_FLOAT_VALUE;
				}

				break;
			}
		}
	}

	return new_state;
}

/* weapon.c update_entity_weapon_systems: the offsets the selected, ready, targeted package's devices are asked to reach.
   `attitude` is the pitch device's unrotated world attitude, `intercept_vector` the launcher's weapon-to-intercept-point vector. */
void eech_aim_required(const float *attitude, const float *intercept_vector, float min_heading, float max_heading, float min_pitch, float max_pitch_limit, int climb,
	float *required_heading, float *required_pitch)
{
	matrix3x3 m;
	vec3d offset_vector, intercept;
	float required_heading_offset, required_pitch_offset, flat_range;

	memcpy(m, attitude, sizeof(m));

	intercept.x = intercept_vector[0];
	intercept.y = intercept_vector[1];
	intercept.z = intercept_vector[2];

	multiply_transpose_matrix3x3_vec3d(&offset_vector, m, &intercept);

	required_heading_offset = atan2(offset_vector.x, offset_vector.z);
	required_heading_offset = bound(required_heading_offset, min_heading, max_heading);

	flat_range = sqrt((offset_vector.x * offset_vector.x) + (offset_vector.z * offset_vector.z));

	required_pitch_offset = atan2(offset_vector.y, flat_range);

	// fire ground launched missiles with slight climb angle to avoid missile hitting ground as often
	if (climb)
	{
		min_pitch = max(rad(10.0f), min_pitch);
	}

	required_pitch_offset = bound(required_pitch_offset, min_pitch, max_pitch_limit);

	*required_heading = required_heading_offset;
	*required_pitch = required_pitch_offset;
}

/* weapon.c update_entity_weapon_systems: one rate-limited step of a package's devices toward the required offsets.
   out: delta heading, delta pitch, new heading, new pitch */
void eech_aim_step(float current_heading, float current_pitch, float required_heading, float required_pitch, float min_heading, float max_heading, float heading_rate,
	float pitch_rate, float delta_time, float *out)
{
	float delta_heading_offset, delta_pitch_offset, heading_rate_delta_time, pitch_rate_delta_time, new_heading_offset, new_pitch_offset;

	delta_heading_offset = required_heading - current_heading;

	if ((min_heading == (float) rad(-180.0)) && (max_heading == (float) rad(180.0)))
	{
		if (delta_heading_offset > rad(180.0))
		{
			delta_heading_offset -= rad(360.0);
		}
		else if (delta_heading_offset < rad(-180.0))
		{
			delta_heading_offset += rad(360.0);
		}
	}

	heading_rate_delta_time = heading_rate * delta_time;

	delta_heading_offset = bound(delta_heading_offset, -heading_rate_delta_time, heading_rate_delta_time);

	new_heading_offset = current_heading + delta_heading_offset;

	if (new_heading_offset > rad(180.0))
	{
		new_heading_offset -= rad(360);
	}
	else if (new_heading_offset < rad(-180.0))
	{
		new_heading_offset += rad(360);
	}

	delta_pitch_offset = required_pitch - current_pitch;

	pitch_rate_delta_time = pitch_rate * delta_time;

	delta_pitch_offset = bound(delta_pitch_offset, -pitch_rate_delta_time, pitch_rate_delta_time);

	new_pitch_offset = current_pitch + delta_pitch_offset;

	out[0] = delta_heading_offset;
	out[1] = delta_pitch_offset;
	out[2] = new_heading_offset;
	out[3] = new_pitch_offset;
}

/* wn_move.c get_floor_pitch_index. Returns the index, or -1 when it falls outside the table (the original asserted). */
static int get_floor_pitch_index(float pitch, float *closeness)
{
	float deg_pitch = deg(pitch);

	int int_pitch, i_elevation = (NUM_SINGLE_DEGREE_VALUES - 1) / 2;
	float f_elevation = (NUM_SINGLE_DEGREE_VALUES - 1) / 2;

	if (deg_pitch < -f_elevation)
	{
		int_pitch = (int) (floor(deg_pitch / PITCH_STEP));

		*closeness = 1.0 - ((deg_pitch - (int_pitch * PITCH_STEP)) / PITCH_STEP);

		return int_pitch + (i_elevation / PITCH_STEP) + NUM_NEGATIVE_PITCH_VALUES;
	}
	else if (deg_pitch < (f_elevation + PITCH_STEP))
	{
		int_pitch = (int) (floor(deg_pitch));

		*closeness = 1.0 - (deg_pitch - int_pitch);

		return int_pitch + NUM_NEGATIVE_PITCH_VALUES + i_elevation;
	}
	else
	{
		int_pitch = (int) (floor(deg_pitch / PITCH_STEP));

		*closeness = 1.0 - ((deg_pitch - (int_pitch * PITCH_STEP)) / PITCH_STEP);

		return int_pitch - (i_elevation / PITCH_STEP) + NUM_NEGATIVE_PITCH_VALUES + NUM_SINGLE_DEGREE_VALUES - 1;
	}
}

typedef struct ballistics_data
{
	float drop_angle;
	float flight_time;
} ballistics_data;

static ballistics_data table_cell(const eech_aim_table *table, int k, int pitch_index, int range_index, int *fault)
{
	ballistics_data data;

	data.drop_angle = 0.0f;
	data.flight_time = 0.0f;

	if (k < 0 || k >= table->velocity_tests || pitch_index < 0 || pitch_index >= table->pitch_indices || range_index < 0 || range_index >= table->range_values)
	{
		*fault = TRUE;

		return data;
	}

	data.drop_angle = table->cells[((((size_t) k * table->pitch_indices) + pitch_index) * table->range_values + range_index) * 2];
	data.flight_time = table->cells[((((size_t) k * table->pitch_indices) + pitch_index) * table->range_values + range_index) * 2 + 1];

	return data;
}

/* wn_move.c get_ballistic_pitch_deflection, with the point-blank patch W1 (docs/patches.md). A weapon without a table, or an invalid weapon, has no
   solution. *fault is set when the table does not cover an index the original would have read. */
static int ballistic_pitch_deflection(const eech_aim_table *table, float range, float height_diff_or_pitch, float *aiming_pitch, float *time_of_flight, int simplified,
	int fixed_pitch, float parent_velocity, int *fault)
{
	float pitch_delta, range_error, range_delta, drop_compensation[2] = {0, 0}, time[2] = {0, 0}, straight_pitch, velocity_multiplier = 1;

	int iterations, i, k, l = 0, pitch_index, range_index, velocity_test = 0;

	if (!table || !table->present)
	{
		return FALSE;
	}

	// we need 2d range

	if (!fixed_pitch)
	{
		if (!(fabs(height_diff_or_pitch) < range)) return FALSE; /* EECH headless (W1) */
		straight_pitch = -asin(height_diff_or_pitch / range);
		range = sqrt(range * range - height_diff_or_pitch * height_diff_or_pitch);
	}
	else
	{
		straight_pitch = height_diff_or_pitch;
		range = range * cos(height_diff_or_pitch);
	}

	if (fixed_pitch)
	{
		iterations = 2;
	}
	else
	{
		iterations = 3;
	}

	// prepare velocity test

	parent_velocity = bound(parent_velocity, 0, ((float) VELOCITY_TESTS - 1) * VELOCITY_TEST_STEP - 1);

	if (parent_velocity > 1)
	{
		velocity_test = (int) (parent_velocity / VELOCITY_TEST_STEP);
		velocity_multiplier = 1 - (parent_velocity / VELOCITY_TEST_STEP - velocity_test);
	}

	if (range <= 0.1)
	{
		if (!fixed_pitch)
		{
			*aiming_pitch = straight_pitch;
		}
		else
		{
			*aiming_pitch = 0;
		}

		*time_of_flight = 0.0;

		return TRUE;
	}

	range = bound(range, 0.01, table->max_range);
	range_index = (int) (range / RANGE_STEP);
	range_error = range - (range_index * RANGE_STEP);
	range_delta = 1.0 - (range_error / RANGE_STEP); // normalize to [0..1]

	// refine drop_compensation - do it several times because as we adjust
	// cannon pitch we have to use a different ballistics table.  Do it a few
	// times so that it stabalizes somewhat

	pitch_index = 0;
	pitch_delta = 0.0f;

	for (k = velocity_test; k <= velocity_test + (parent_velocity > 1); k++)
	{
		for (i = 0; i < iterations; i++)
		{
			float use_pitch = min(straight_pitch + drop_compensation[l], rad(90.0f));
			ballistics_data cell;

			pitch_index = get_floor_pitch_index(use_pitch, &pitch_delta);

			if (!(pitch_index >= 0 && pitch_index < TOTAL_PITCH_INDICES))
			{
				*fault = TRUE;

				return FALSE;
			}

			cell = table_cell(table, k, pitch_index, range_index, fault);

			if (*fault)
			{
				return FALSE;
			}

			drop_compensation[l] = cell.drop_angle;

			if (drop_compensation[l] > rad(45.0))
			{
				return FALSE;
			}

			time[l] = cell.flight_time;
		}

		// average between next pitch and range:
		if (!simplified)
		{
			ballistics_data pitch_compensation[2], compensation_grid[2][2], cell;

			int next_pitch_index = min(pitch_index + 1, TOTAL_PITCH_INDICES - 1);

			// first average the next ranges
			compensation_grid[0][0].drop_angle = drop_compensation[l];
			compensation_grid[0][0].flight_time = time[l];

			cell = table_cell(table, k, pitch_index, range_index + 1, fault);
			compensation_grid[0][1].drop_angle = cell.drop_angle;
			compensation_grid[0][1].flight_time = cell.flight_time;

			cell = table_cell(table, k, next_pitch_index, range_index, fault);
			compensation_grid[1][0].drop_angle = cell.drop_angle;
			compensation_grid[1][0].flight_time = cell.flight_time;

			cell = table_cell(table, k, next_pitch_index, range_index + 1, fault);
			compensation_grid[1][1].drop_angle = cell.drop_angle;
			compensation_grid[1][1].flight_time = cell.flight_time;

			if (*fault)
			{
				return FALSE;
			}

			// then average oover next pitch
			pitch_compensation[0].drop_angle = (pitch_delta * compensation_grid[0][0].drop_angle) + ((1 - pitch_delta) * compensation_grid[1][0].drop_angle);
			pitch_compensation[0].flight_time = (pitch_delta * compensation_grid[0][0].flight_time) + ((1 - pitch_delta) * compensation_grid[1][0].flight_time);

			pitch_compensation[1].drop_angle = (pitch_delta * compensation_grid[0][1].drop_angle) + ((1 - pitch_delta) * compensation_grid[1][1].drop_angle);
			pitch_compensation[1].flight_time = (pitch_delta * compensation_grid[0][1].flight_time) + ((1 - pitch_delta) * compensation_grid[1][1].flight_time);

			// and finally get a total
			drop_compensation[l] = (range_delta * pitch_compensation[0].drop_angle) + ((1 - range_delta) * pitch_compensation[1].drop_angle);
			time[l] = (range_delta * pitch_compensation[0].flight_time) + ((1 - range_delta) * pitch_compensation[1].flight_time);
		}

		l++;
	}

	*aiming_pitch = drop_compensation[0] * velocity_multiplier + drop_compensation[1] * (1 - velocity_multiplier);
	*time_of_flight = time[0] * velocity_multiplier + time[1] * (1 - velocity_multiplier);

	if (!fixed_pitch)
	{
		*aiming_pitch += straight_pitch; // get absolute angle only if straight_pitch wasn't given
	}

	return TRUE;
}

static void echo_ballistic(eech_aim_echo *echo, int weapon, float range, float height_diff_or_pitch, int simplified, int fixed_pitch, float parent_velocity, int result,
	float aiming_pitch, float time_of_flight)
{
	eech_aim_event *event = echo_next(echo, AIM_EVENT_BALLISTIC);

	if (event)
	{
		event->a = weapon;
		event->b = simplified;
		event->c = fixed_pitch;
		event->d = result;
		event->f[0] = range;
		event->f[1] = height_diff_or_pitch;
		event->f[2] = parent_velocity;
		event->f[3] = aiming_pitch;
		event->f[4] = time_of_flight;
	}
}

/* wn_move.c get_ballistic_pitch_deflection as the campaign asks for it directly (the AI salvo aim, the player's pipper). Returns the result; *fault set
   when the table does not cover an index. */
int eech_aim_ballistic(const eech_aim_table *table, float range, float height_diff_or_pitch, int simplified, int fixed_pitch, float parent_velocity, float *aiming_pitch,
	float *time_of_flight, int *fault)
{
	*fault = FALSE;

	return ballistic_pitch_deflection(table, range, height_diff_or_pitch, aiming_pitch, time_of_flight, simplified, fixed_pitch, parent_velocity, fault);
}

/* modules/maths/projtile.c */
static int get_angle_of_projection_with_range(const vec3d *source, const vec3d *target, float velocity, float *angle_of_projection, float range)
{
	float x, y, m, s, x_squared, tan_theta, theta;

	int result;

	theta = 0.0;

	result = FALSE;

	if (velocity > 0.001)
	{
		x = range;

		if (x > 0.001)
		{
			y = target->y - source->y;

			x_squared = x * x;

			m = (x_squared * G) / (2.0 * velocity * velocity);

			s = x_squared - (4.0 * m * m) - (4.0 * m * y);

			if (s >= 0.0)
			{
				tan_theta = (x - sqrt(s)) / (2.0 * m);

				theta = atan(tan_theta);

				result = TRUE;
			}
		}
	}

	*angle_of_projection = theta;

	return (result);
}

static int get_angle_of_projection(const vec3d *source, const vec3d *target, float velocity, float *angle_of_projection)
{
	if (velocity > 0.001)
	{
		float range = get_2d_range(source, target);

		return get_angle_of_projection_with_range(source, target, velocity, angle_of_projection, range);
	}
	else
	{
		return FALSE;
	}
}

typedef struct eech_aim_target
{
	vec3d position;
	vec3d point;
	vec3d motion_vector;
	int mobile;
} eech_aim_target;

typedef struct eech_aim_world
{
	float map_min[3];
	float map_max[3];
	float comms_weapon_lag;
	int source_is_gunship;
} eech_aim_world;

static int point_inside_map_area(const eech_aim_world *world, const vec3d *position)
{
	return (position->x >= world->map_min[0]) && (position->x <= world->map_max[0]) && (position->z >= world->map_min[2]) && (position->z <= world->map_max[2]);
}

/* weapon.c get_lead_and_ballistic_intercept_point_and_angle_of_projection. Returns the result; *need_draw is set when the final iteration's range
   jitter is reached without a draw (nothing else of the call is meaningful then); *fault when the table does not cover an index. The triangulated
   range of the player's gunship is not supported: the campaign refuses to ask. */
static int lead_and_ballistic(const eech_aim_table *table, int weapon, const vec3d *pitch_device_position, float parent_velocity, const eech_aim_target *target,
	const eech_aim_world *world, int has_draw, int draw_sample, vec3d *intercept_point, float *angle_of_projection, float *time_of_flight, eech_aim_echo *echo,
	int *need_draw, int *fault)
{
	int result, number_of_iterations;

	float range, target_move_distance;

	vec3d current_target_position, new_intercept_point, target_motion_vector;

	result = FALSE;

	*angle_of_projection = 0.0;

	current_target_position = target->point;
	*intercept_point = current_target_position;

	range = get_3d_range(pitch_device_position, intercept_point);

	{
		eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_RANGE);

		if (event)
		{
			event->a = 0;
			event->b = 0;
			event->f[0] = range;
		}
	}

	if (range < 0.001)
	{
		return FALSE;
	}

	// target point differs to position so test inside map area before terrain elevation check

	if (point_inside_map_area(world, intercept_point))
	{
		float target_true_velocity = 0.0;

		target_true_velocity = get_3d_vector_magnitude(&target->motion_vector);

		if (target_true_velocity > 0.1)
		{
			// moving target

			target_motion_vector = target->motion_vector;
			normalise_3d_vector_given_magnitude(&target_motion_vector, target_true_velocity);
			new_intercept_point = *intercept_point;

			// refine ballistic calulation
			if (world->source_is_gunship || target_true_velocity > 200)
			{
				number_of_iterations = 5;
			}
			else
			{
				number_of_iterations = 3;
			}

			{
				eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_PATH);

				if (event)
				{
					event->a = AIM_LEAD_PATH_MOVING;
					event->b = number_of_iterations;
					event->f[0] = target_true_velocity;
				}
			}

			{
				int ok = ballistic_pitch_deflection(table, range, pitch_device_position->y - intercept_point->y, angle_of_projection, time_of_flight, number_of_iterations > 1, FALSE,
					parent_velocity, fault);

				echo_ballistic(echo, weapon, range, pitch_device_position->y - intercept_point->y, number_of_iterations > 1, FALSE, parent_velocity, ok, *angle_of_projection,
					*time_of_flight);

				if (!ok)
				{
					return FALSE;
				}
			}

			while (number_of_iterations > 0)
			{
				target_move_distance = (*time_of_flight + 0.5 * world->comms_weapon_lag) * target_true_velocity / pow(number_of_iterations, 0.5);

				new_intercept_point.x = current_target_position.x + (target_motion_vector.x * target_move_distance);
				new_intercept_point.y = current_target_position.y + (target_motion_vector.y * target_move_distance);
				new_intercept_point.z = current_target_position.z + (target_motion_vector.z * target_move_distance);

				{
					eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_ITERATION);

					if (event)
					{
						event->a = number_of_iterations;
						event->f[0] = target_move_distance;
						event->f[1] = new_intercept_point.x;
						event->f[2] = new_intercept_point.y;
						event->f[3] = new_intercept_point.z;
					}
				}

				*intercept_point = new_intercept_point;

				if (!point_inside_map_area(world, &new_intercept_point))
				{
					break;
				}

				range = get_3d_range(pitch_device_position, intercept_point);

				{
					eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_RANGE);

					if (event)
					{
						event->a = 1;
						event->b = number_of_iterations;
						event->f[0] = range;
					}
				}

				if (number_of_iterations == 1)
				{
					double jitter;

					if (!has_draw)
					{
						*need_draw = TRUE;

						return FALSE;
					}

					// the campaign's sfrand1 (): ((double) (rand16 () - 16384)) * (1.0 / 16384.0)
					jitter = ((double) (draw_sample - 16384)) * (1.0 / 16384.0);

					range += 5 * jitter;

					{
						eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_RANGE);

						if (event)
						{
							event->a = 2;
							event->b = number_of_iterations;
							event->f[0] = range;
						}
					}
				}
				else
				{
					/* the original reports its post-jitter range after every iteration (the statement after the conditional) */
					eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_RANGE);

					if (event)
					{
						event->a = 2;
						event->b = number_of_iterations;
						event->f[0] = range;
					}
				}

				{
					int ok = ballistic_pitch_deflection(table, range, pitch_device_position->y - intercept_point->y, angle_of_projection, time_of_flight, number_of_iterations > 1,
						FALSE, parent_velocity, fault);

					echo_ballistic(echo, weapon, range, pitch_device_position->y - intercept_point->y, number_of_iterations > 1, FALSE, parent_velocity, ok, *angle_of_projection,
						*time_of_flight);

					if (!ok)
					{
						return FALSE;
					}
				}

				number_of_iterations--;
			}
		}
		else
		{
			// stationary target

			eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_PATH);

			if (event)
			{
				event->a = AIM_LEAD_PATH_STATIONARY;
				event->b = 0;
				event->f[0] = target_true_velocity;
			}

			{
				int ok = ballistic_pitch_deflection(table, range, pitch_device_position->y - intercept_point->y, angle_of_projection, time_of_flight, FALSE, FALSE, parent_velocity,
					fault);

				echo_ballistic(echo, weapon, range, pitch_device_position->y - intercept_point->y, FALSE, FALSE, parent_velocity, ok, *angle_of_projection, *time_of_flight);

				if (!ok)
				{
					return FALSE;
				}
			}
		}
	}
	else
	{
		//
		// target off map
		//

		eech_aim_event *event = echo_next(echo, AIM_EVENT_LEAD_PATH);

		if (event)
		{
			event->a = AIM_LEAD_PATH_OFF_MAP;
			event->b = 0;
			event->f[0] = 0.0;
		}

		return get_angle_of_projection(pitch_device_position, intercept_point, parent_velocity, angle_of_projection);
	}

	return TRUE;
}

/* weapon.c get_intercept_point (guided weapons): the target point carried along the target's motion by three passes of its own range ratio */
static void guided_intercept_point(const vec3d *pitch_device_position, float weapon_velocity, const eech_aim_target *target, vec3d *intercept_point, eech_aim_echo *echo)
{
	int number_of_iterations;

	float target_true_velocity, velocity_ratio, target_move_distance;

	vec3d target_position, target_motion_vector;

	*intercept_point = target->point;

	target_true_velocity = get_3d_vector_magnitude(&target->motion_vector);

	if ((weapon_velocity > 0.001) && (target_true_velocity > 0.001))
	{
		if (target_true_velocity < weapon_velocity)
		{
			velocity_ratio = target_true_velocity / weapon_velocity;

			target_position = *intercept_point;

			target_motion_vector = target->motion_vector;

			normalise_any_3d_vector(&target_motion_vector);

			number_of_iterations = 3;

			while (number_of_iterations--)
			{
				target_move_distance = get_3d_range(pitch_device_position, intercept_point) * velocity_ratio;

				intercept_point->x = target_position.x + (target_motion_vector.x * target_move_distance);
				intercept_point->y = target_position.y + (target_motion_vector.y * target_move_distance);
				intercept_point->z = target_position.z + (target_motion_vector.z * target_move_distance);
			}
		}
	}

	{
		eech_aim_event *event = echo_next(echo, AIM_EVENT_INTERCEPT);

		if (event)
		{
			event->f[0] = pitch_device_position->x;
			event->f[1] = pitch_device_position->y;
			event->f[2] = pitch_device_position->z;
			event->f[3] = weapon_velocity;
			event->f[4] = target_true_velocity;
			event->f[5] = target->point.x;
			event->f[6] = target->point.y;
			event->f[7] = target->point.z;
			event->f[8] = intercept_point->x;
			event->f[9] = intercept_point->y;
			event->f[10] = intercept_point->z;
		}
	}
}

/* weapon.c get_target_intercept_point (the HUD's prediction for the weapon the player has selected): weapon_velocity 0 for no weapon */
void eech_aim_intercept(const float *device_position, int has_weapon, float cruise_velocity, int acquire_forward, float source_velocity, const float *target_position,
	const float *target_point, const float *target_motion_vector, float *intercept_point, eech_aim_echo *echo)
{
	vec3d device, point;
	eech_aim_target target;
	float weapon_velocity;

	device.x = device_position[0];
	device.y = device_position[1];
	device.z = device_position[2];

	target.position.x = target_position[0];
	target.position.y = target_position[1];
	target.position.z = target_position[2];
	target.point.x = target_point[0];
	target.point.y = target_point[1];
	target.point.z = target_point[2];
	target.motion_vector.x = target_motion_vector[0];
	target.motion_vector.y = target_motion_vector[1];
	target.motion_vector.z = target_motion_vector[2];
	target.mobile = TRUE;

	if (has_weapon)
	{
		weapon_velocity = cruise_velocity;

		if (acquire_forward)
		{
			weapon_velocity += source_velocity;
		}
	}
	else
	{
		weapon_velocity = 0.0;
	}

	guided_intercept_point(&device, weapon_velocity, &target, &point, echo);

	intercept_point[0] = point.x;
	intercept_point[1] = point.y;
	intercept_point[2] = point.z;
}

/* the lead and ballistic solve as the campaign asks for it directly (the decoy and fuze timer at weapon launch). out: intercept point (3), angle, time of flight. */
int eech_aim_lead(const eech_aim_table *table, int weapon, const float *device_position, float parent_velocity, const float *target_point, const float *target_motion_vector,
	const float *map_min, const float *map_max, float comms_weapon_lag, int source_is_gunship, int has_draw, int draw_sample, float *out, eech_aim_echo *echo, int *need_draw,
	int *fault)
{
	vec3d device, intercept_point;
	eech_aim_target target;
	eech_aim_world world;
	float angle_of_projection, time_of_flight;
	int result;

	device.x = device_position[0];
	device.y = device_position[1];
	device.z = device_position[2];

	memset(&target, 0, sizeof(target));
	target.point.x = target_point[0];
	target.point.y = target_point[1];
	target.point.z = target_point[2];
	target.motion_vector.x = target_motion_vector[0];
	target.motion_vector.y = target_motion_vector[1];
	target.motion_vector.z = target_motion_vector[2];

	memcpy(world.map_min, map_min, sizeof(world.map_min));
	memcpy(world.map_max, map_max, sizeof(world.map_max));
	world.comms_weapon_lag = comms_weapon_lag;
	world.source_is_gunship = source_is_gunship;

	*need_draw = FALSE;
	*fault = FALSE;
	angle_of_projection = 0.0f;
	time_of_flight = 0.0f;

	result = lead_and_ballistic(table, weapon, &device, parent_velocity, &target, &world, has_draw, draw_sample, &intercept_point, &angle_of_projection, &time_of_flight, echo,
		need_draw, fault);

	out[0] = intercept_point.x;
	out[1] = intercept_point.y;
	out[2] = intercept_point.z;
	out[3] = angle_of_projection;
	out[4] = time_of_flight;

	return result;
}

/* weapon.c update_entity_weapon_system_weapon_and_target_vectors (the pitch device's part of it) and get_pitch_device_to_target_vector.
   With a tracking point (the player's EO) the intercept vector is the tracking point's direction from the device and the target vector keeps its default.
   out: weapon vector (3), target vector (3), intercept vector (3), range. Returns the valid flag. */
int eech_aim_vectors(const eech_aim_table *table, int weapon, int aiming_type, float cruise_velocity, int acquire_forward, float source_velocity, const float *device_position,
	const float *device_attitude, const float *target_position, const float *target_point, const float *target_motion_vector, int target_mobile, int has_tracking,
	const float *tracking_point, const float *map_min, const float *map_max, float comms_weapon_lag, int source_is_gunship, int has_draw, int draw_sample, float *out,
	eech_aim_echo *echo, int *need_draw, int *fault)
{
	matrix3x3 attitude;
	vec3d device, weapon_vector, target_vector, intercept_point_vector, intercept_point_position;
	eech_aim_target target;
	eech_aim_world world;
	float intercept_point_range, weapon_velocity, pitch, time_of_flight, dx, dz, heading;
	int result;
	matrix3x3 m;

	*need_draw = FALSE;
	*fault = FALSE;

	device.x = device_position[0];
	device.y = device_position[1];
	device.z = device_position[2];

	memcpy(attitude, device_attitude, sizeof(attitude));

	// *weapon_vector_ptr = vp.zv
	weapon_vector.x = attitude[2][0];
	weapon_vector.y = attitude[2][1];
	weapon_vector.z = attitude[2][2];

	// the defaults the update sets before it asks (force 180 degree error)
	target_vector.x = 0.0;
	target_vector.y = 0.0;
	target_vector.z = -1.0;

	intercept_point_vector.x = 0.0;
	intercept_point_vector.y = 0.0;
	intercept_point_vector.z = -1.0;

	intercept_point_range = 0.0;

	result = FALSE;

	if (has_tracking)
	{
		// use get vector to EO tracking point, if any
		intercept_point_vector.x = tracking_point[0] - device.x;
		intercept_point_vector.y = tracking_point[1] - device.y;
		intercept_point_vector.z = tracking_point[2] - device.z;

		result = TRUE;

		// the original passes the offset vector itself as the second point of the range
		intercept_point_range = get_3d_range(&device, &intercept_point_vector);

		normalise_any_3d_vector(&intercept_point_vector);
	}
	else
	{
		memset(&target, 0, sizeof(target));
		target.position.x = target_position[0];
		target.position.y = target_position[1];
		target.position.z = target_position[2];
		target.point.x = target_point[0];
		target.point.y = target_point[1];
		target.point.z = target_point[2];
		target.motion_vector.x = target_motion_vector[0];
		target.motion_vector.y = target_motion_vector[1];
		target.motion_vector.z = target_motion_vector[2];
		target.mobile = target_mobile;

		memcpy(world.map_min, map_min, sizeof(world.map_min));
		memcpy(world.map_max, map_max, sizeof(world.map_max));
		world.comms_weapon_lag = comms_weapon_lag;
		world.source_is_gunship = source_is_gunship;

		target_vector.x = target.position.x - device.x;
		target_vector.y = target.position.y - device.y;
		target_vector.z = target.position.z - device.z;

		normalise_any_3d_vector(&target_vector);

		switch (aiming_type)
		{
			case AIM_AIMING_TYPE_CALC_INTERCEPT_POINT: // guided weapons
			{
				if (target.mobile)
				{
					weapon_velocity = cruise_velocity;

					if (acquire_forward)
					{
						weapon_velocity += source_velocity;
					}

					guided_intercept_point(&device, weapon_velocity, &target, &intercept_point_position, echo);
				}
				else
				{
					intercept_point_position = target.point;
				}

				intercept_point_vector.x = intercept_point_position.x - device.x;
				intercept_point_vector.y = intercept_point_position.y - device.y;
				intercept_point_vector.z = intercept_point_position.z - device.z;

				intercept_point_range = get_3d_range(&intercept_point_position, &device);

				normalise_any_3d_vector(&intercept_point_vector);

				result = TRUE;

				break;
			}
			case AIM_AIMING_TYPE_CALC_ANGLE_OF_PROJECTION: // unguided weapons
			case AIM_AIMING_TYPE_CALC_LEAD_AND_BALLISTIC:
			{
				pitch = 0.0f;
				time_of_flight = 0.0f;

				result = lead_and_ballistic(table, weapon, &device, acquire_forward * source_velocity, &target, &world, has_draw, draw_sample, &intercept_point_position, &pitch,
					&time_of_flight, echo, need_draw, fault);

				if (*need_draw)
				{
					return FALSE;
				}

				if (result)
				{
					dx = intercept_point_position.x - device.x;
					dz = intercept_point_position.z - device.z;

					heading = atan2(dx, dz);

					get_3d_transformation_matrix(m, heading, pitch, 0.0);

					intercept_point_vector.x = m[2][0];
					intercept_point_vector.y = m[2][1];
					intercept_point_vector.z = m[2][2];

					intercept_point_range = get_3d_range(&intercept_point_position, &device);

					normalise_any_3d_vector(&intercept_point_vector);
				}

				// the lead solver's own outputs, for the campaign's trace of it
				out[10] = pitch;
				out[11] = time_of_flight;
				out[12] = intercept_point_position.x;
				out[13] = intercept_point_position.y;
				out[14] = intercept_point_position.z;

				break;
			}
			default:
			{
				return FALSE;
			}
		}
	}

	out[0] = weapon_vector.x;
	out[1] = weapon_vector.y;
	out[2] = weapon_vector.z;
	out[3] = target_vector.x;
	out[4] = target_vector.y;
	out[5] = target_vector.z;
	out[6] = intercept_point_vector.x;
	out[7] = intercept_point_vector.y;
	out[8] = intercept_point_vector.z;
	out[9] = intercept_point_range;

	return result;
}

/* ps_anim.c update_person_animation: the target-facing attitude of a person who has a target. Returns the heading; writes the attitude. */
float eech_aim_person_face(const float *position, const float *target_position, float *attitude)
{
	vec3d fire_heading;
	float ps_heading;
	matrix3x3 m;

	fire_heading.x = target_position[0] - position[0];
	fire_heading.y = 0.0;
	fire_heading.z = target_position[2] - position[2];

	normalise_any_3d_vector(&fire_heading);

	ps_heading = atan2(fire_heading.x, fire_heading.z);

	get_3d_transformation_heading_matrix(m, ps_heading);

	memcpy(attitude, m, sizeof(m));

	return ps_heading;
}

/*
 * The physical weapon flight of EECH's weapons (EEDCS-45, D2): the projectile simulation that produces the unguided ballistic tables and the guided
 * missiles' range runs, and the flight integrators they share with weapon flight. It is EECH's own arithmetic, moved here expression for expression
 * from
 *   - aphavoc/source/entity/mobile/weapon/wn_move.c: calculate_projectory, calculate_range, move_unguided_weapon, update_guided_weapon, and
 *     generate_ballistics_tables / generate_guided_missiles_tables (the two diagnostic text files and the order they are written in);
 *   - aphavoc/source/entity/mobile/weapon/wn_funcs.c: get_weapon_drag, get_weapon_current_weight;
 *   - aphavoc/source/misc/airspeed.c: get_precise_air_density;
 * so the host and the original reference produce the same bits. Neither a rounding mode nor a type is chosen here: the engine runs a host callback
 * under round-toward-zero and every float and double expression below is the engine's own, in its own order. The only change to the original text
 * is where it read `weapon_database[sub_type].x` (it reads the same figure from `fig->x`, a float or int of the same type) and where it reached for the
 * campaign (`ASSERT`, `frand1`, `get_system_time` and the wind lookup are the context's: a draw, the host's clock and a wind sample the host owns).
 *
 * What is not here: which weapon is launched, at what, when and how many (the fire decision, the salvo and the ammunition), the entity mirror, the
 * campaign's random generator and seed, and the terrain, object and model lookups. The weapon's physical figures are supplied once by the campaign
 * (transitional mirrors of the weapon database a host with its own model does not need); the tables this produces are the host's own and are not
 * handed back (the test fixture build asks for an echo of them, to observe what the host generated).
 *
 * The functions are pure functions of the figures: no state is kept between weapons except the open diagnostic file, which the original also held
 * open across the weapons of one generation pass.
 */
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "eech_arith.h"

#define G (9.81f)
#define deg(RAD) ((RAD) * (180.0f / PI))
#define wrap_angle(THETA) (fmod ((THETA), PI2))
#define sign(VAL) ((VAL) >= 0 ? 1 : -1)

#define RANGE_STEP 20.0
#define WRITE_STEP 250.0
#define PITCH_STEP 5
#define VELOCITY_TESTS 1 + 4
#define VELOCITY_TEST_STEP 50.0
#define NUM_SINGLE_DEGREE_VALUES 61
#define NUM_NEGATIVE_PITCH_VALUES (((90 - (NUM_SINGLE_DEGREE_VALUES - 1) / 2) / PITCH_STEP))
#define TOTAL_PITCH_INDICES (2*NUM_NEGATIVE_PITCH_VALUES + NUM_SINGLE_DEGREE_VALUES)

#define TRUE 1
#define FALSE 0

#define WEAPON_CLASS_AIR_TO_AIR (1 << 0)
#define WEAPON_CLASS_AIR_TO_SURFACE (1 << 1)
#define WEAPON_CLASS_SURFACE_TO_AIR (1 << 2)
#define WEAPON_CLASS_SURFACE_TO_SURFACE (1 << 3)

#define PARACHUTE_STATUS_OPEN3 3

typedef struct vec2d
{
	float x, y;
} vec2d;

static float get_2d_vector_magnitude(const vec2d *vector)
{
	float x, y;

	x = (vector->x * vector->x);
	y = (vector->y * vector->y);

	return (sqrt(x + y));
}

static void get_identity_matrix3x3(matrix3x3 m)
{
	m[0][0] = 1.0;
	m[0][1] = 0.0;
	m[0][2] = 0.0;

	m[1][0] = 0.0;
	m[1][1] = 1.0;
	m[1][2] = 0.0;

	m[2][0] = 0.0;
	m[2][1] = 0.0;
	m[2][2] = 1.0;
}

/* One weapon's physical figures, as the campaign mirrors them from its weapon database (the same types as the database's fields). */
typedef struct wf_figures
{
	char name[64];
	float start_weight;
	float final_weight;
	float max_range;
	float muzzle_velocity;
	float cruise_velocity;
	float g_max;
	float boost_time;
	float cruise_time;
	float inhibit_time;
	float drag_coefficient;
	float flightpath_deviation;
	float boost_power;
	float sustain_power;
	float sustain_time;
	float diameter;
	float max_altitude;
	int ignore_gravity;
	int flight_profile_or_self_destr;
	int parachute_type;
	unsigned int weapon_class;
	float cruise_time_max_error;
	float detonation_radius;
	float max_seeker_limit;
	float max_launch_angle_error;
	int survive_ground_impact;
	int in_flight_collision_check;
	int guidance_type;
	int aiming_type;
	int decoy_type;
	int warhead_type;
	int hellfire_ii;
	int smoke_grenade;
	int submunition_count;
	int submunition_velocity_factor;
} wf_figures;

/* The part of the original's `weapon` entity the integrators touch. */
typedef struct wf_weapon
{
	struct
	{
		vec3d position;
		union
		{
			matrix3x3 attitude;
			struct
			{
				vec3d xv, yv, zv;
			};
		};
		vec3d motion_vector;
		float velocity;
	} mob;
	float weapon_lifetime;
	int parachute_status;
} wf_weapon;

/* What the integrators ask of the world: a wind sample at a point, a draw from the campaign's random stream, the host's clock. A missing draw or wind is a
   failure, never a default: `failed` says so. */
typedef struct wf_context
{
	void (*wind) (const vec3d *position, vec3d *wind_velocity, void *user);
	double (*draw) (void *user);
	void *user;
	unsigned int system_time;
	float delta_time;
	int failed;
} wf_context;

typedef struct ballistics_data
{
	float drop_angle;
	float flight_time;
} ballistics_data;

typedef struct wf_table
{
	ballistics_data *rows[VELOCITY_TESTS][TOTAL_PITCH_INDICES];
	ballistics_data *block;
} wf_table;

/* What each simulated flight left behind, in order (the test fixture observes it; the model does not need it). */
typedef struct wf_flight_record
{
	int velocity_test;
	int pitch_index;
	int range_mark;
	int range_values;
	int steps;
	float end_time;
} wf_flight_record;

typedef struct wf_flights
{
	wf_flight_record *records;
	int capacity;
	int count;
	int overflow;
} wf_flights;

typedef struct wf_range_record
{
	int ran;
	int zero_pitch;
	float end_z;
	float end_y;
	float end_velocity;
	float end_lifetime;
} wf_range_record;

typedef wf_range_record wf_range;

static void wf_flights_add(wf_flights *flights, int velocity_test, int pitch_index, int range_mark, int range_values, int steps, float end_time)
{
	if (flights->count < flights->capacity)
	{
		wf_flight_record *record = &flights->records[flights->count];

		record->velocity_test = velocity_test;
		record->pitch_index = pitch_index;
		record->range_mark = range_mark;
		record->range_values = range_values;
		record->steps = steps;
		record->end_time = end_time;
	}
	else
	{
		flights->overflow = 1;
	}

	flights->count++;
}

static double wf_frand1(wf_context *ctx)
{
	if (!ctx->draw)
	{
		ctx->failed = 1;

		return 0.0;
	}

	return ctx->draw(ctx->user);
}

static float get_precise_air_density (float altitude)
{
	float
		m = 0.0289644, // physics constants
		r = 8.31447,
		l = 0.0065,
		p0 = 101325, // standard air pressure, Pa
		t0 = 288.15, // standard temperature, K
		t = t0 - l * altitude, // actual temperature, K
		p = p0 * exp(- (G * m * altitude) / (r * t0)), // air pressure, Pa
		air_density = p * m / (r * t); // kg/m^3
	
	return air_density;
}

static float get_weapon_drag(float speed, float alt, float diameter, float drag_coefficient)
{
	float
		l = 0.0065,
		t0 = 288.15, // standard temperature, K
		area = PI * diameter * diameter / 4, // projectile area, m^2
		t = t0 - l * alt, // actual temperature, K
		air_density = get_precise_air_density(alt), // kg/m^3
		mach = speed / (20.046 * sqrt(t)), // velocity, mach
		cd; // drag coefficient

	// get drag coefficient (air resistance law of 1943)
	
	if (mach <= 0.8)
		cd = 0.158;
	else if (mach <= 0.9)
		cd = 0.137 * mach * mach  + 0.0865 * mach + 0.000898;
	else if (mach <= 1.0)
		cd = 1.35 * mach - 1.025;
	else if (mach <= 1.4)
		cd = - 0.925 * mach * mach + 2.335 * mach - 1.085;
	else if (mach <= 4)
		cd = - 0.0917 * mach + 0.4993;
	else
		cd = 0.1325;

	return 0.5 * drag_coefficient * air_density * speed * speed * cd * area;
}

static float get_weapon_current_weight(float lifetime, const wf_figures *fig)
{
	if (lifetime > 0)
	{
		float boost_consumption = fig->boost_power * fig->boost_time;
		float sustain_consumption = fig->sustain_power * fig->sustain_time;
		float boost_scale = boost_consumption / (boost_consumption + sustain_consumption);
		float sustain_scale = sustain_consumption / (boost_consumption + sustain_consumption);
		float sustain_time_scale = fig->sustain_time / (fig->boost_time + fig->sustain_time);
		float boost_time_scale = fig->boost_time / (fig->boost_time + fig->sustain_time);
		float time_scale = lifetime / (fig->boost_time + fig->sustain_time);
		float delta_weight = fig->start_weight - fig->final_weight;
		float boost_fuel_weight = delta_weight * boost_scale * max(0.0f, (time_scale - sustain_time_scale) / boost_time_scale);
		float sustain_fuel_weight = sustain_time_scale ? (delta_weight * sustain_scale * min(1.0f, time_scale / sustain_time_scale)) : 0;
		
		return fig->final_weight + boost_fuel_weight + sustain_fuel_weight;
	}
	else
		return fig->final_weight;
}

static void move_unguided_weapon (wf_weapon *raw, const wf_figures *fig, wf_context *ctx, vec3d *new_position, float delta_time, int wind)
{
	float
		acceleration = 0,
		heading,
		pitch,
		h_vel,
		v_vel,
		parachute_modifier = 1,
		weapon_weight;
	vec3d
		wind_velocity;
	
	weapon_weight = get_weapon_current_weight(raw->weapon_lifetime, fig);

	// acceleration

	if (raw->weapon_lifetime - fig->sustain_time > 0) // boost
		acceleration += fig->boost_power / weapon_weight;
	else if (raw->weapon_lifetime > 0) // sustain
		acceleration += fig->sustain_power / weapon_weight;

	if (fig->parachute_type) // speed up eject seat if parachute is closed (the pilot, the co-pilot and the crate)
	{
		if (raw->parachute_status != PARACHUTE_STATUS_OPEN3)
			parachute_modifier = 0.05;
	}
			
	acceleration -= get_weapon_drag(raw->mob.velocity, raw->mob.position.y, fig->diameter * parachute_modifier, fig->drag_coefficient) / weapon_weight;

	if (acceleration > 9999 || acceleration < -9999)
		acceleration = 0;

	// update velocity

	raw->mob.velocity = bound (raw->mob.velocity + acceleration * delta_time, 0.0, 1.5 * fig->cruise_velocity);

	// wind

	if (wind)
	{
		ctx->wind (&raw->mob.position, &wind_velocity, ctx->user);
	
		wind_velocity.x = delta_time * get_weapon_drag(wind_velocity.x, raw->mob.position.y, fig->diameter * parachute_modifier, fig->drag_coefficient * 1.5) / weapon_weight;	
		wind_velocity.y = delta_time * get_weapon_drag(wind_velocity.y, raw->mob.position.y, fig->diameter * parachute_modifier, fig->drag_coefficient * 1.5) / weapon_weight;	
		wind_velocity.z = delta_time * get_weapon_drag(wind_velocity.z, raw->mob.position.y, fig->diameter * parachute_modifier, fig->drag_coefficient * 1.5) / weapon_weight;	
	}
	else
		wind_velocity.x = wind_velocity.y = wind_velocity.z = 0;
	
	// motion vector

	raw->mob.motion_vector.x = raw->mob.zv.x * raw->mob.velocity + wind_velocity.x;
	raw->mob.motion_vector.y = raw->mob.zv.y * raw->mob.velocity + wind_velocity.y;
	raw->mob.motion_vector.z = raw->mob.zv.z * raw->mob.velocity + wind_velocity.z;

	// position

	new_position->x += raw->mob.motion_vector.x * delta_time;
	new_position->y += raw->mob.motion_vector.y * delta_time;
	new_position->z += raw->mob.motion_vector.z * delta_time;

	// apply gravity (after moving weapon)

	if (!fig->ignore_gravity)
	{
		heading = get_heading_from_attitude_matrix (raw->mob.attitude);

		pitch = get_pitch_from_attitude_matrix (raw->mob.attitude);

		h_vel = cos (pitch) * raw->mob.velocity;

		v_vel = (sin (pitch) * raw->mob.velocity) - (G * delta_time);

		raw->mob.velocity = bound (sqrt ((h_vel * h_vel) + (v_vel * v_vel)), 0.0, 1.5 * fig->cruise_velocity);

		pitch = atan2 (v_vel, h_vel);

		get_3d_transformation_matrix (raw->mob.attitude, heading, pitch, 0.0);
	}
}

static void update_guided_weapon (wf_weapon *raw, const wf_figures *fig, wf_context *ctx, vec3d *new_position, float delta_time, float turn_rate, float range, int wind)
{
	float
		acceleration = 0,
		displacement_modifier,
		current_life_time,
		weapon_weight = get_weapon_current_weight(raw->weapon_lifetime, fig);
	int 
		engine_mode = 0;
	vec3d
		wind_velocity;

	if (raw->weapon_lifetime - fig->sustain_time > 0)
		engine_mode = 2; // boost
	else if (raw->weapon_lifetime > 0)
		engine_mode = 1; // sustain
	
	current_life_time = fig->boost_time + fig->sustain_time - raw->weapon_lifetime;
	displacement_modifier = 25 * delta_time * bound ((range - 500) / 1000, 0, 1) * bound(min(current_life_time, raw->weapon_lifetime) / fig->inhibit_time, 0, 1);

	//
	// acceleration
	//

	if (engine_mode == 2)
		acceleration = fig->boost_power / weapon_weight;
	else if (engine_mode == 1)
		acceleration = fig->sustain_power / weapon_weight;
	acceleration -= get_weapon_drag(raw->mob.velocity, raw->mob.position.y, fig->diameter, fig->drag_coefficient) / weapon_weight;
	acceleration -= fabs(turn_rate) * ctx->delta_time * raw->mob.velocity;
	if (!fig->ignore_gravity)
		acceleration -= sin(get_pitch_from_attitude_matrix(raw->mob.attitude)) * G;

	if (acceleration > 9999 || acceleration < -9999)
		acceleration = 0;

	// udate velocity

	raw->mob.velocity = bound (raw->mob.velocity + acceleration * delta_time, 0, 1.5 * fig->cruise_velocity);

	// wind

	if (wind)
	{
		ctx->wind (&raw->mob.position, &wind_velocity, ctx->user);

		wind_velocity.x = delta_time * get_weapon_drag(wind_velocity.x, raw->mob.position.y, fig->diameter, fig->drag_coefficient * 1.5) / weapon_weight;	
		wind_velocity.y = delta_time * get_weapon_drag(wind_velocity.y, raw->mob.position.y, fig->diameter, fig->drag_coefficient * 1.5) / weapon_weight;	
		wind_velocity.z = delta_time * get_weapon_drag(wind_velocity.z, raw->mob.position.y, fig->diameter, fig->drag_coefficient * 1.5) / weapon_weight;	
	}
	else
		wind_velocity.x = wind_velocity.y = wind_velocity.z = 0;
		
	// motion vector

	raw->mob.motion_vector.x = raw->mob.zv.x * raw->mob.velocity + wind_velocity.x;
	raw->mob.motion_vector.y = raw->mob.zv.y * raw->mob.velocity + wind_velocity.y;
	raw->mob.motion_vector.z = raw->mob.zv.z * raw->mob.velocity + wind_velocity.z;

	// position

	new_position->x += raw->mob.motion_vector.x * delta_time;
	new_position->y += raw->mob.motion_vector.y * delta_time;
	new_position->z += raw->mob.motion_vector.z * delta_time;
	
	// flightpath adjustments

	if (raw->weapon_lifetime > 0.0 && displacement_modifier)
	{
		vec3d displacement;
		float spiral = (fig->flight_profile_or_self_destr == 2),
				deviation = fig->flightpath_deviation;

		displacement.x = displacement.y = displacement.z = 0;

		if (spiral)
		{
			float timer = wrap_angle(0.75f * PI2 * raw->weapon_lifetime);

			displacement.x -= displacement_modifier * sin(timer) * (1 + 0.2 * wf_frand1(ctx));
			displacement.y += displacement_modifier * cos(timer) * (0.5 + 0.5 * wf_frand1(ctx));
		}

		if (deviation)
		{
			float
				angle_x = sin((wrap_angle(get_3d_vector_magnitude(new_position) / 222) + wrap_angle(ctx->system_time / 1000)) / 2),
				angle_y = sin((wrap_angle(get_3d_vector_magnitude(new_position) / 111) + wrap_angle(ctx->system_time / 2000)) / 2);

			displacement.x += deviation * displacement_modifier * sign(angle_x) * sin(angle_x) * sin(angle_x);
			displacement.y += deviation * displacement_modifier * (sign(angle_y) * sin(angle_y) * sin(angle_y) + 0.5) / 2;
//			debug_log("angle x %f y %f", angle_x, angle_y);
		}
	
		multiply_matrix3x3_vec3d(&displacement, raw->mob.attitude, &displacement);

		new_position->x += bound(displacement.x, - delta_time * (10 * spiral + deviation), delta_time * (10 * spiral + deviation));
		new_position->y += bound(displacement.y, - delta_time * (10 * spiral + deviation), delta_time * (10 * spiral + deviation));
		new_position->z += bound(displacement.z, - delta_time * (10 * spiral + deviation), delta_time * (10 * spiral + deviation));
	}
}

static void calculate_projectory(wf_weapon *wpn, const wf_figures *fig, wf_context *ctx, FILE* output, int velocity_test, wf_table *table, wf_flights *flights)
{
	int
		i,
		pitch_index = 0,
		num_range_values;

	float
		pitch,
		test_velocity[256],
		test_time[256],
		test_drop[256],
		test_range[256],
		test_drop_angle[256],
		test_drag[256],
		max_speed = 0;
	
	ballistics_data** data = table->rows[velocity_test];

	num_range_values = (int)(fig->max_range / RANGE_STEP) + 2;
	
	// initialize -90 and +90
	data[0] = table->block + ((size_t) velocity_test * TOTAL_PITCH_INDICES + 0) * num_range_values;
	data[TOTAL_PITCH_INDICES - 1] = table->block + ((size_t) velocity_test * TOTAL_PITCH_INDICES + (TOTAL_PITCH_INDICES - 1)) * num_range_values;

	for (i=0; i < num_range_values; i++)
	{
		data[0][i].drop_angle = 0.0;
		data[0][i].flight_time = 0.0;
		data[TOTAL_PITCH_INDICES-1][i].drop_angle = 0.0;
		data[TOTAL_PITCH_INDICES-1][i].flight_time = 0.0;
	}
	
	for (i=0; i < 256; i++)
	{
		test_velocity[i] = test_time[i] = test_drop[i] = test_drop_angle[i] = test_drag[i] = 0.0;
		test_range[i] = fig->max_range;
	}	

	// initialize all the other pitches
	pitch_index = 1;
	pitch = rad(-90.0 + PITCH_STEP);
	while (pitch_index < (TOTAL_PITCH_INDICES - 1))
	{
		float
			time = 0.0,
			pitch_ratio,
			max_range,
			delta_time = 0.02;

		int
			range_mark = 0,
			steps = 0,
			write_step = 0,
			write_range = WRITE_STEP;
		
		data[pitch_index] = table->block + ((size_t) velocity_test * TOTAL_PITCH_INDICES + pitch_index) * num_range_values;

		if (velocity_test && (fig->weapon_class & WEAPON_CLASS_SURFACE_TO_AIR || fig->weapon_class & WEAPON_CLASS_SURFACE_TO_SURFACE))
		{
			ballistics_data
					*source_ptr = table->rows[velocity_test - 1][pitch_index],
					*dest_ptr = table->rows[velocity_test][pitch_index];
		
			memcpy(dest_ptr, source_ptr, sizeof(ballistics_data) * num_range_values);
			pitch_index++;
			continue;
		}
		
		// initialize weapon for new firing
		wpn->weapon_lifetime = fig->boost_time + fig->sustain_time;
		wpn->mob.velocity = fig->muzzle_velocity + velocity_test * VELOCITY_TEST_STEP;

		wpn->mob.position.x = 0.0;
		wpn->mob.position.y = 0.0;
		wpn->mob.position.z = 0.0;

		get_identity_matrix3x3(wpn->mob.attitude);
		get_3d_transformation_matrix(wpn->mob.attitude, 0.0, pitch, 0.0);

		// initialize 0-range:
		data[pitch_index][range_mark].drop_angle = 0.0;
		data[pitch_index][range_mark].flight_time = 0.0;
		range_mark++;

		pitch_ratio = tan(pitch);

		// don't calculate out to entire range for very high or low pitch values (takes too long)
		max_range = min(fig->max_range,
			fig->max_range * cos(pitch) * 1.25f);

		while (TRUE)
		{
			float
				fake_y,
				fake_z,
				expected_height;
			vec2d magnitude[2];

			time += delta_time;
			wpn->weapon_lifetime -= delta_time;
			move_unguided_weapon(wpn, fig, ctx, &wpn->mob.position, delta_time, FALSE);
			steps++;

			if (wpn->weapon_lifetime < - fig->cruise_time ||
					wpn->mob.position.y < max(- fig->max_range, - 4000.0f) ||
					wpn->mob.velocity < 20.0 && wpn->weapon_lifetime < 0)
				break;
			
			// increase delta_time when projectile slows down a lot so as to ease amount of calculations a little

			if (wpn->mob.velocity < 75.0)
				delta_time = 0.1;
			else if (wpn->mob.velocity < 150.0)
				delta_time = 0.04;
			else
				delta_time = 0.02;

			if (pitch >= 0 && pitch <= PI / 4 && !velocity_test) // collect data for statistics, only for first velocity test
			{
				if (wpn->mob.velocity > max_speed)
					max_speed = wpn->mob.velocity;

				if (wpn->mob.position.z >= write_range - 1)
				{
					magnitude[0].x = test_range[write_step] - range_mark * RANGE_STEP;
					magnitude[0].y = test_drop[write_step];
					magnitude[1].x = wpn->mob.position.z - range_mark * RANGE_STEP;
					magnitude[1].y = wpn->mob.position.y;

					if(get_2d_vector_magnitude(&magnitude[1]) < get_2d_vector_magnitude(&magnitude[0]))
					{
						test_velocity[write_step] = wpn->mob.velocity;
						test_time[write_step] = time;
						test_drop[write_step] = wpn->mob.position.y;
						test_range[write_step] = wpn->mob.position.z;
						test_drop_angle[write_step] = pitch - atan(test_drop[write_step]/test_range[write_step]);
						test_drag[write_step] = get_weapon_drag(wpn->mob.velocity, wpn->mob.position.y, fig->diameter, fig->drag_coefficient) / get_weapon_current_weight(wpn->weapon_lifetime, fig);
					}
					
					if (wpn->mob.position.z >= 10000)
						write_range += WRITE_STEP * 4;
					else if (wpn->mob.position.z >= 5000)
						write_range +=  WRITE_STEP * 2;
					else
						write_range += WRITE_STEP;
					
					write_step++;
				}
			}

			expected_height = pitch_ratio * wpn->mob.position.z;
			fake_y = wpn->mob.position.y + cos(pitch) * (expected_height - wpn->mob.position.y);
			fake_z = wpn->mob.position.z - sin(pitch) * (expected_height - wpn->mob.position.y);
			
			if (fake_z >= ((float)(range_mark * RANGE_STEP) - 1.0)) // this is a range we want to sample
			{
				int
					stop_index = min((int)((fake_z + 1) / RANGE_STEP), num_range_values);
				float
					drop_angle;

				// adjust for expected height if there was no drop, so as to calculate real drop

				drop_angle = pitch - atan(wpn->mob.position.y / wpn->mob.position.z);

				// if the projectile is really fast we might actually have passed several sample values, so set the value for all of them
				for (; range_mark < stop_index; range_mark++)
				{
					data[pitch_index][range_mark].drop_angle = drop_angle;
					data[pitch_index][range_mark].flight_time = time;
				}

				if (range_mark == num_range_values)
					break;
			}
		}

		wf_flights_add (flights, velocity_test, pitch_index, range_mark, num_range_values, steps, time);

		// if there's any spots left just copy the last to those (it will be wrong, but better than arbitrary values)
		for (; range_mark < num_range_values; range_mark++)
		{
			data[pitch_index][range_mark] = data[pitch_index][range_mark-1];
		}

		// we have smaller steps around 0 pitch to get a little better accuracy there
		pitch_index++;
		if (pitch_index <= NUM_NEGATIVE_PITCH_VALUES)
			pitch += rad(PITCH_STEP);
		else if (pitch_index < (NUM_NEGATIVE_PITCH_VALUES + NUM_SINGLE_DEGREE_VALUES))
			pitch += rad(1.0);
		else
			pitch += rad(PITCH_STEP);
	}
	
	// write collected ballistics data
	if (output && !velocity_test)
	{
		fputs("range     velocity    time     drop  drop angle   drag force\n", output);

		i = 0;
		
		if (output)
			while (test_time[i])
			{
				fprintf(output, "%5.2f m  %6.2f m/s %5.2f s %6.1f m %2.2f degrees %4.1f\n",
					test_range[i], test_velocity[i], test_time[i], - test_drop[i], deg(test_drop_angle[i]), test_drag[i]);
				i++;
			}
		fprintf(output, "top velocity %6.1f m/s \n", max_speed);
	}
}

static void calculate_range(wf_weapon *wpn, const wf_figures *fig, wf_context *ctx, FILE* output, int zero_pitch, wf_range *result)
{
	float
		pitch = 0,
		range_step = 100,
		write_step = 100;

	if (output)
	{

		float
			time = 0.0,
			delta_time = 0.1;

		int
			last_write_range = 0;

		fputs(" range    altitude    velocity    time     drag   weight\n", output);
	
		wpn->weapon_lifetime = fig->boost_time + fig->sustain_time;
		wpn->mob.velocity = fig->muzzle_velocity;
		wpn->mob.position.x = wpn->mob.position.y = wpn->mob.position.z = 0.0;

		if (!zero_pitch)
		{
			if (fig->weapon_class & WEAPON_CLASS_AIR_TO_AIR || fig->weapon_class & WEAPON_CLASS_AIR_TO_SURFACE)
			{
				wpn->mob.position.y = fig->max_altitude;
				pitch = - atan(fig->max_altitude / fig->max_range);
			}
			else if (fig->weapon_class & WEAPON_CLASS_SURFACE_TO_AIR)
				pitch = atan(fig->max_altitude / fig->max_range);
			else
				return;
		}

		get_identity_matrix3x3(wpn->mob.attitude);
		get_3d_transformation_matrix(wpn->mob.attitude, 0.0, pitch, 0.0);

		while (wpn->mob.position.z < fig->max_range)
		{
			time += delta_time;
			wpn->weapon_lifetime -= delta_time;

			update_guided_weapon(wpn, fig, ctx, &wpn->mob.position, delta_time, FALSE, FALSE, FALSE);
			
			// increase delta_time when projectile slows down a lot so as to ease amount of calculations a little

			if (wpn->mob.velocity < 150.0)
			{
				delta_time = 0.04;
				if (wpn->mob.velocity < 75.0)
					delta_time = 0.1;

				if (wpn->mob.velocity < 20.0 && wpn->weapon_lifetime < 0)
					break;
			}
			
			if (wpn->mob.position.z > 300000 || wpn->mob.position.y > 1.1 * fig->max_altitude || wpn->mob.position.y < - fig->max_altitude)
				break;

			if (output && wpn->mob.position.z > (last_write_range + write_step - 1))
			{
				last_write_range += write_step;
				fprintf(output, "%5d m %f m  %6.1f m/s %5.1f s, %.2f, %.2f \n",
					last_write_range, wpn->mob.position.y, wpn->mob.velocity, time, get_weapon_drag(wpn->mob.velocity, wpn->mob.position.y, fig->diameter, fig->drag_coefficient), get_weapon_current_weight(wpn->weapon_lifetime, fig));
			}

			if (last_write_range >= 20000)
			{
				write_step = 5000;
				range_step = 2500;
			}
			else if (last_write_range >= 10000)
			{
				write_step = 2000;
				range_step = 1000;
			}
			else if (last_write_range >= 5000)
			{
				write_step = 1000;
				range_step = 500;
			}
			else if (last_write_range >= 500)
			{
				write_step = 500;
				range_step = 250;
			}
		}

		fprintf(output, "%5.0f m %f m  %6.1f m/s %5.1f s, %.2f, pitch %.2f \n",
			wpn->mob.position.z, wpn->mob.position.y, wpn->mob.velocity, time, get_weapon_drag(wpn->mob.velocity, wpn->mob.position.y, fig->diameter, fig->drag_coefficient), deg(pitch));

		result->ran = 1;
		result->end_z = wpn->mob.position.z;
		result->end_y = wpn->mob.position.y;
		result->end_velocity = wpn->mob.velocity;
		result->end_lifetime = wpn->weapon_lifetime;
	}
		
}


/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the generation passes (generate_ballistics_tables, generate_guided_missiles_tables) */

static FILE *ballistics_file;
static FILE *range_file;

/* generate_ballistics_tables opens the file in the working directory, writes the note and, for each weapon, its header; a file that cannot be opened is not an error: the
   original goes on without it (the tables do not depend on it) */
int eech_wn_ballistics_open(void)
{
	/* binary mode: the original wrote through the engine's own file shim, which writes the text as it is (LF) */
	ballistics_file = fopen("ballistics-data.txt", "wb");

	if (ballistics_file)
	{
		fputs("Note: this file only lists ballistics data generated from the currently used GWUT File\n"
			  "Changing the values will have no effect, change the GWUT file to see change\n"
			  "This file can be used to help getting good values for the GWUT file\n\n", ballistics_file);
	}

	return ballistics_file != NULL;
}

void eech_wn_ballistics_close(void)
{
	if (ballistics_file)
	{
		fclose(ballistics_file);

		ballistics_file = NULL;
	}
}

/* The most a table may span: the largest weapon range there is is 80 km; a limit well above it keeps the table's size an honest function of the figures. */
#define WF_MAX_TABLE_RANGE 200000.0f

/* The floats a table holds for these figures, or 0 when the figures cannot give a table (not a finite range in (0, WF_MAX_TABLE_RANGE]). */
static size_t wf_table_cells(const wf_figures *fig)
{
	if (!(fig->max_range > 0.0f) || !(fig->max_range <= WF_MAX_TABLE_RANGE))
	{
		return 0;
	}

	/* (VELOCITY_TESTS is the original's unparenthesised `1 + 4`) */
	return (size_t) (VELOCITY_TESTS) * (TOTAL_PITCH_INDICES) * (size_t) ((int) (fig->max_range / RANGE_STEP) + 2) * 2;
}

/* One unguided weapon's table: the header line, then five velocity tests. `cells` holds velocity_tests * TOTAL_PITCH_INDICES * range_values pairs and `cell_capacity`
   floats; nothing is written unless the capacity is exactly what the figures need. Returns 0, or -1 when the figures or the room are wrong or the flight records did not fit. */
int eech_wn_ballistics_weapon(const wf_figures *fig, float *cells, size_t cell_capacity, wf_flight_record *records, int record_capacity, int *flight_count)
{
	wf_weapon wpn;
	wf_context ctx;
	wf_table *table;
	wf_flights flights;
	unsigned i;

	if (!cells || (wf_table_cells(fig) == 0) || (cell_capacity != wf_table_cells(fig)) || !records || (record_capacity < 0) || !flight_count)
	{
		return -1;
	}

	memset(&wpn, 0, sizeof(wpn));
	memset(&ctx, 0, sizeof(ctx));

	table = (wf_table *) calloc(1, sizeof(wf_table));

	if (!table)
	{
		return -1;
	}

	table->block = (ballistics_data *) cells;

	flights.records = records;
	flights.capacity = record_capacity;
	flights.count = 0;
	flights.overflow = 0;

	if (ballistics_file)
	{
		fprintf(ballistics_file, "\n%s   max_range: %.0f m, muzzle velocity: %.0f m/s, cruise velocity: %.0f m/s, drag: %.2f\n",
			fig->name,
			fig->max_range,
			fig->muzzle_velocity,
			fig->cruise_velocity,
			fig->drag_coefficient);
	}

	for(i = 0; i < VELOCITY_TESTS; i++)
		calculate_projectory(&wpn, fig, &ctx, ballistics_file, i, table, &flights);

	if (ballistics_file)
	{
		fflush(ballistics_file);
	}

	free(table);

	*flight_count = flights.count;

	return (flights.overflow || ctx.failed) ? -1 : 0;
}

int eech_wn_range_open(void)
{
	range_file = fopen("guided-missiles-data.txt", "wb");

	if (range_file)
	{
		fputs("Note: this file only lists ballistics data generated from the currently used GWUT File\n"
			  "Changing the values will have no effect, change the GWUT file to see change\n"
			  "This file can be used to help getting good values for the GWUT file\n\n", range_file);
	}

	return range_file != NULL;
}

void eech_wn_range_close(void)
{
	if (range_file)
	{
		fclose(range_file);

		range_file = NULL;
	}
}

/* One guided weapon's header and its two range runs (level, then aimed). Returns 0, or -1 when a run asked for a draw or a wind sample it was not given. */
int eech_wn_range_weapon(const wf_figures *fig, wf_range_record *results)
{
	wf_weapon wpn;
	wf_context ctx;
	int i;

	memset(&wpn, 0, sizeof(wpn));
	memset(&ctx, 0, sizeof(ctx));
	memset(results, 0, 2 * sizeof(wf_range_record));

	if (range_file)
	{
		fprintf(range_file, "\n%s \n"
				"max_range: %.0f m, max_altitude %.0f m, muzzle velocity: %.0f m/s, cruise velocity: %.0f m/s, cruise_time %.1f s\n"
				"sustain_time %.1f s, sustain_power %.1f, g_max %.1f, drag: %.2f \n"
				"boost_time %.1f s, boost_power %.1f, start_weight %.1f, final_weight %.1f \n",
			fig->name,
			fig->max_range,
			fig->max_altitude,
			fig->muzzle_velocity,
			fig->cruise_velocity,
			fig->cruise_time,
			fig->sustain_time,
			fig->sustain_power,
			fig->g_max,
			fig->drag_coefficient,
			fig->boost_time,
			fig->boost_power,
			fig->start_weight,
			fig->final_weight);
	}

	for(i = 0; i < 2; i++)
	{
		results[i].zero_pitch = i;

		calculate_range(&wpn, fig, &ctx, range_file, i, &results[i]);
	}

	if (range_file)
	{
		fflush(range_file);
	}

	return ctx.failed ? -1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the weapon step (EEDCS-45): weapon_movement, expression for expression, with the campaign's world reached through lookups.
 *
 * A request carries the weapon's state and the answers to every lookup asked so far. The step runs from the beginning each time, as a pure function of the request: a
 * lookup it needs and was not given is recorded (with every other lookup it can already name) and the step stops with `need_lookup`; the campaign answers the pending lookups
 * (pure questions of its world: the terrain, the objects, a launcher's or a target's facts) and asks again with them. A random draw the step needs from the campaign's shared
 * stream is asked for the same way (`need_draw`, one at a time, in order). Answers are matched BY VALUE (the lookup's kind and arguments), never by position, so a lookup
 * the step asked for speculatively and no longer needs is simply unused. The entity-local random stream (the weapon's own seed) never leaves the campaign: the request
 * carries its first four outputs, which is all any one call of the original uses.
 *
 * The step ends with an ordered list of ACTIONS the campaign applies to its mirror of the weapon in the order the original wrote and called them: the state it left (before
 * anything that looks at the weapon), the position, a target-link change, a spawn of submunitions, the Tacview update, the ground effect. Nothing is applied until the step is
 * done, so a step that stops for a lookup or a draw has changed nothing. The step also produces an ECHO (the test fixture build replays it into the observer's hooks in the
 * order the original called them); the shipping module never asks for it.
 *
 * Not carried over: the original's debug-drawing calls (a keyboard toggle of the interactive game, off in the module) and its ASSERTs.
 */
#include <setjmp.h>
#include <string.h>

typedef struct wf_event
{
	int kind;
	int a, b, c, d;
	float f[12];
} wf_event;

#define WF_ACTIONS_MAX 32
#define WF_LOOKUPS_MAX 16
#define WF_ECHO_MAX 64
#define WF_DRAWS_MAX 8

/* lookup kinds.  A lookup is its kind, the integers a, b and the floats f[0..5]; the answer repeats them and fills f[6..11], read below as r0... */
enum
{
	WL_VIEW_WEAPON = 1,		/* r0 the camera is viewing this weapon */
	WL_LAUNCHER_EO,			/* r0..r2 the launcher's EO tracking point, r3 it is valid */
	WL_LAUNCHER_INT,		/* a: 0 the launcher is not an AI, 1 its laser is on, 2 it has line of sight to its target.  r0 */
	WL_LAUNCHER_TARGET,		/* the launcher's target.  r0 it has one, r1 it is the weapon's own target, r2..r4 its position */
	WL_TARGET_INT,			/* a: which target (0 the weapon's own, 1 the launcher's); b: 0 mobile, 1 decoy type, 2 radar jammer on, 3 infra-red jammer on, 4 airborne aircraft, 5 not an AI.  r0 */
	WL_TARGET_VEC,			/* a: which target; b: 0 position, 1 target point, 2 motion vector.  r0..r2 */
	WL_WIND,				/* f0..f2 a point.  r0..r2 the wind velocity there */
	WL_GROUND_PROBE,		/* f0..f2 a point inside the map area.  r0 it is below the ground */
	WL_GROUND_IMPACT,		/* f0..f2 the old position, f3..f5 the new.  r0 the terrain class, r1..r3 the impact point */
	WL_FACE_NORMAL,			/* f0, f2 x and z.  r0..r2 the terrain face normal there */
	WL_ELEVATION,			/* f0, f2 x and z.  r0 the terrain elevation there */
	WL_COLLISION,			/* a: which target is the weapon's now (-1 none); f0..f2 the old position, f3..f5 the new.  r0 something was hit, r1 it is not the weapon's target, r2 its type, r3..r5 where the segment meets it */
	WL_TREES,				/* f0..f2 the old position, f3..f5 the new.  r0 a tree was hit, r1..r3 where the segment meets it */
	WL_KINDS
};

/* action kinds */
enum
{
	WA_STATE = 1,			/* a missile phase, b weapon on ground, c kill code; f0 velocity, f1..f3 motion vector, f4 weapon lifetime, f5 decoy timer */
	WA_ATTITUDE,			/* f0..f8 the attitude matrix, row by row */
	WA_SPAWN,				/* a the number of submunitions, f0 their parent velocity: they leave from the weapon as it is now */
	WA_TACVIEW,				/* the weapon is written to the Tacview recording, if one is open */
	WA_POSITION,			/* f0..f2 the weapon's position */
	WA_TARGET_LINK,			/* a: 0 the weapon has no target now, 1 its target is the launcher's, 2 its target is the one the collision lookup hit */
	WA_SMOKE_EFFECT,		/* the killed-explosion effect, with the kill code at hit-land for its duration */
	WA_KINDS
};

/* echo kinds (the test fixture build's observer replays them, in order, into the hooks the original called).  The numbers of the path, guided and target branches are the
   fixture's own (mb_wpnfx.h). */
enum
{
	WE_PATH = 20, WE_SUBMUNITIONS, WE_TRACKING, WE_GUIDANCE, WE_GUIDANCE_SOURCE, WE_TGT_BRANCH, WE_TGT_END, WE_INTERCEPT, WE_GUIDED, WE_PRECISION, WE_GROUND, WE_IMPACT,
	WE_CONTACT_ANY, WE_CONTACT_TREES
};

enum
{
	WP_FUZE_TIMER, WP_EXHAUSTED, WP_ON_GROUND, WP_GUIDED, WP_UNGUIDED, WP_LOAL_CLIMB_DIVE, WP_KILLED_BY_GUIDANCE, WP_CONTACT_TARGET, WP_CONTACT_TREES, WP_OUT_OF_MAP_AREA,
	WP_ABOVE_CEILING, WP_GROUND_WATER, WP_GROUND_SURVIVED, WP_GROUND_SMOKE_GRENADE, WP_GROUND_LAND
};

enum { WG_TOO_CLOSE, WG_OVERSHOT, WG_TARGET_LOST, WG_TURNED };

enum { WT_TRACKING_POINT, WT_MOBILE_INTERCEPT, WT_JAMMER_CHAFF, WT_JAMMER_IR, WT_SMOKE_DISTRACT, WT_DECOY_INTERCEPT };

/* the terrain class of water, the kill codes and the weapon database's enumerations, as the campaign numbers them */
#define WF_TERRAIN_CLASS_WATER 1
enum
{
	WF_KILL_OK, WF_KILL_EXHAUSTED, WF_KILL_OUT_OF_BOUNDS, WF_KILL_HIT_LAND, WF_KILL_HIT_WATER, WF_KILL_HIT_TARGET, WF_KILL_OVERSHOT_TARGET, WF_KILL_SELF_DESTRUCT
};
enum { MISSILE_PHASE1, MISSILE_PHASE2, MISSILE_PHASE3, MISSILE_FINAL_PHASE };
enum
{
	WF_GUIDANCE_NONE, WF_GUIDANCE_PASSIVE_INFRA_RED, WF_GUIDANCE_SEMI_ACTIVE_RADAR, WF_GUIDANCE_ACTIVE_RADAR, WF_GUIDANCE_SEMI_ACTIVE_LASER,
	WF_GUIDANCE_SEMI_ACTIVE_LASER_BEAM_RIDING, WF_GUIDANCE_ACTIVE_LASER, WF_GUIDANCE_RADIO_COMMAND, WF_GUIDANCE_WIRE_GUIDED
};
enum { WF_DECOY_NONE, WF_DECOY_CHAFF, WF_DECOY_FLARE, WF_DECOY_SMOKE_GRENADE };
enum { WF_DECOY_WEAPON_NONE, WF_DECOY_WEAPON_CHAFF, WF_DECOY_WEAPON_FLARE, WF_DECOY_WEAPON_SMOKE_GRENADE };
#define WF_WARHEAD_CONVENTIONAL_MUNITIONS 8

#define HELLFIRE_LOBL_CLIMB_RATIO	(0.14)
#define HELLFIRE_LOBL_DIVE_RATIO	(0.31)
#define HELLFIRE_LOAL_CLIMB1_RATIO	(0.084)
#define HELLFIRE_LOAL_CLIMB2_RATIO	(0.22)
#define HELLFIRE_LOAL_DIVE_RATIO	(0.5)
#define HELLFIRE_LOAL_CLIMB1_TIME	(0.0)
#define HELLFIRE_LOAL_CLIMB2_TIME	(-10.0)
#define HELLFIRE_LOAL_XZ	(1000.0)
#define HELLFIRE_LOAL_CLIMB1_Y		(300.0)
#define HELLFIRE_LOAL_CLIMB2_Y		(40.0)
#define HELLFIRE_LOAL_DIVE_Y		(-400.0)

typedef struct wf_step_in
{
	const wf_figures *figures;
	float delta_time;
	unsigned int system_time;
	/* the weapon as the campaign holds it */
	float position[3];
	float attitude[9];
	float motion_vector[3];
	float velocity;
	float weapon_lifetime;
	float decoy_timer;
	int missile_phase;
	int loal_mode;
	int weapon_on_ground;
	int parachute_status;
	int kill_code;
	/* the first four outputs of the weapon's own stream (raw integers in [0, 32767]); the stream and its seed never leave the campaign */
	int entity_samples[4];
	int has_launcher;
	int has_target;
	int server;
	float map_min[3];
	float map_max[3];
	int echo;
	const wf_event *answers;
	int answer_count;
	const int *draws;
	int draw_count;
} wf_step_in;

typedef struct wf_step_out
{
	wf_event actions[WF_ACTIONS_MAX];
	int action_count;
	wf_event lookups[WF_LOOKUPS_MAX];
	int lookup_count;
	wf_event echo[WF_ECHO_MAX];
	int echo_count;
	int need_lookup;
	int need_draw;
	int failed;
	char reason[96];
} wf_step_out;

typedef struct wf_run
{
	const wf_step_in *in;
	wf_step_out *out;
	jmp_buf escape;
	wf_weapon raw;
	const wf_figures *fig;
	wf_context ctx;
	float decoy_timer;
	int missile_phase;
	int loal_mode;
	int weapon_on_ground;
	int kill_code;
	/* the weapon's target now: whether it has one, and whose (0 the one it started with, 1 the launcher's) */
	int has_target;
	int target_which;
	int draw_cursor;
} wf_run;

static void wf_fail(wf_run *r, const char *reason)
{
	r->out->failed = 1;
	snprintf(r->out->reason, sizeof(r->out->reason), "%s", reason);
	longjmp(r->escape, 1);
}

static void wf_echo(wf_run *r, int kind, int a, int b, int c, int d, const float *f, int fn)
{
	int i;

	if (r->in->echo && r->out->echo_count < WF_ECHO_MAX)
	{
		wf_event *e = &r->out->echo[r->out->echo_count++];

		memset(e, 0, sizeof(*e));
		e->kind = kind;
		e->a = a;
		e->b = b;
		e->c = c;
		e->d = d;
		for (i = 0; i < fn; i++)
		{
			e->f[i] = f[i];
		}
	}
}

static void wf_echo_path(wf_run *r, int path)
{
	wf_echo(r, WE_PATH, path, 0, 0, 0, NULL, 0);
}

static void wf_action(wf_run *r, int kind, int a, int b, int c, const float *f, int fn)
{
	int i;
	wf_event *e;

	if (r->out->action_count >= WF_ACTIONS_MAX)
	{
		wf_fail(r, "the step produced more actions than it can carry");
	}

	e = &r->out->actions[r->out->action_count++];

	memset(e, 0, sizeof(*e));
	e->kind = kind;
	e->a = a;
	e->b = b;
	e->c = c;
	e->d = r->out->echo_count;	/* the echo events the original's hooks saw before this action */
	for (i = 0; i < fn; i++)
	{
		e->f[i] = f[i];
	}
}

/* what the weapon is now, for the campaign's mirror: applied before anything that looks at the weapon, and at the end */
static void wf_snapshot(wf_run *r)
{
	float f[9];

	f[0] = r->raw.mob.velocity;
	f[1] = r->raw.mob.motion_vector.x;
	f[2] = r->raw.mob.motion_vector.y;
	f[3] = r->raw.mob.motion_vector.z;
	f[4] = r->raw.weapon_lifetime;
	f[5] = r->decoy_timer;

	wf_action(r, WA_STATE, r->missile_phase, r->weapon_on_ground, r->kill_code, f, 6);

	memcpy(f, &r->raw.mob.attitude[0][0], sizeof(f));

	wf_action(r, WA_ATTITUDE, 0, 0, 0, f, 9);
}

static void wf_position_action(wf_run *r, const vec3d *position)
{
	float f[3];

	f[0] = position->x;
	f[1] = position->y;
	f[2] = position->z;

	wf_action(r, WA_POSITION, 0, 0, 0, f, 3);
}

/* a lookup: kind, integer arguments a..d and up to six float arguments */
typedef struct wf_query
{
	int kind;
	int a, b, c, d;
	float f[6];
} wf_query;

static wf_query wf_q(int kind, int a, int b, int c, int d, float f0, float f1, float f2, float f3, float f4, float f5)
{
	wf_query q;

	q.kind = kind;
	q.a = a;
	q.b = b;
	q.c = c;
	q.d = d;
	q.f[0] = f0;
	q.f[1] = f1;
	q.f[2] = f2;
	q.f[3] = f3;
	q.f[4] = f4;
	q.f[5] = f5;

	return q;
}

static const wf_event *wf_find(wf_run *r, const wf_query *q)
{
	int i;

	for (i = 0; i < r->in->answer_count; i++)
	{
		const wf_event *e = &r->in->answers[i];

		if (e->kind == q->kind && e->a == q->a && e->b == q->b && e->c == q->c && e->d == q->d && memcmp(e->f, q->f, sizeof(q->f)) == 0)
		{
			return e;
		}
	}

	return NULL;
}

/* Asks for a batch of lookups: when every one is answered, returns; otherwise records each one that is not (a batch is asked for in one round) and stops the step. */
static void wf_need(wf_run *r, const wf_query *queries, int count)
{
	int i, missing = 0;

	for (i = 0; i < count; i++)
	{
		if (!wf_find(r, &queries[i]))
		{
			missing++;

			if (r->out->lookup_count >= WF_LOOKUPS_MAX)
			{
				wf_fail(r, "the step asked for more lookups than it can carry");
			}
			else
			{
				wf_event *e = &r->out->lookups[r->out->lookup_count++];

				memset(e, 0, sizeof(*e));
				e->kind = queries[i].kind;
				e->a = queries[i].a;
				e->b = queries[i].b;
				e->c = queries[i].c;
				e->d = queries[i].d;
				memcpy(e->f, queries[i].f, sizeof(queries[i].f));
			}
		}
	}

	if (missing)
	{
		r->out->need_lookup = 1;

		longjmp(r->escape, 1);
	}
}

/* One lookup, as the result slots r0.. */
static const float *wf_ask(wf_run *r, wf_query q)
{
	wf_need(r, &q, 1);

	return wf_find(r, &q)->f + 6;
}

/* the campaign's shared stream, one draw at a time: the sample is a raw integer in [0, 32767]; frand1 is its macro's own expression */
static double wf_draw(void *user)
{
	wf_run *r = (wf_run *) user;

	if (r->draw_cursor < r->in->draw_count)
	{
		return ((double) r->in->draws[r->draw_cursor++]) * (1.0 / 32767.0);
	}

	r->out->need_draw = 1;

	longjmp(r->escape, 1);
}

/* the entity-local stream: its outputs as the campaign computed them from the entity's own seed; frand1x and sfrand1x are the macros' own expressions */
static double wf_frand1x(wf_run *r, int index)
{
	return ((double) r->in->entity_samples[index]) * (1.0 / 32767.0);
}

static double wf_sfrand1x(wf_run *r, int index)
{
	return ((double) (r->in->entity_samples[index] - 16384)) * (1.0 / 16384.0);
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

static float get_3d_unit_vector_dot_product(const vec3d *a, const vec3d *b)
{
	float result;

	result = (a->x * b->x) + (a->y * b->y) + (a->z * b->z);

	result = bound(result, -1.0, 1.0);

	return (result);
}

static void get_3d_vector_cross_product(vec3d *result, const vec3d *v1, const vec3d *v2)
{
	result->x = (v1->y * v2->z) - (v2->y * v1->z);
	result->y = (v1->z * v2->x) - (v2->z * v1->x);
	result->z = (v1->x * v2->y) - (v2->x * v1->y);
}

static void multiply_matrix3x3_matrix3x3(matrix3x3 result, matrix3x3 a, matrix3x3 b)
{
	result[0][0] = ((a[0][0] * b[0][0]) + (a[0][1] * b[1][0]) + (a[0][2] * b[2][0]));
	result[0][1] = ((a[0][0] * b[0][1]) + (a[0][1] * b[1][1]) + (a[0][2] * b[2][1]));
	result[0][2] = ((a[0][0] * b[0][2]) + (a[0][1] * b[1][2]) + (a[0][2] * b[2][2]));

	result[1][0] = ((a[1][0] * b[0][0]) + (a[1][1] * b[1][0]) + (a[1][2] * b[2][0]));
	result[1][1] = ((a[1][0] * b[0][1]) + (a[1][1] * b[1][1]) + (a[1][2] * b[2][1]));
	result[1][2] = ((a[1][0] * b[0][2]) + (a[1][1] * b[1][2]) + (a[1][2] * b[2][2]));

	result[2][0] = ((a[2][0] * b[0][0]) + (a[2][1] * b[1][0]) + (a[2][2] * b[2][0]));
	result[2][1] = ((a[2][0] * b[0][1]) + (a[2][1] * b[1][1]) + (a[2][2] * b[2][1]));
	result[2][2] = ((a[2][0] * b[0][2]) + (a[2][1] * b[1][2]) + (a[2][2] * b[2][2]));
}

static void get_3d_transformation_matrix_from_face_normal_and_heading(matrix3x3 result, const vec3d *normal, float heading)
{
	if (fabs(normal->y) > 0.0001)
	{
		memcpy(&result[1][0], normal, sizeof(vec3d));

		result[2][0] = sin(heading);
		result[2][2] = cos(heading);
		result[2][1] = - ((normal->x * result[2][0]) + (normal->z * result[2][2])) / normal->y;

		normalise_any_3d_vector((vec3d *) &result[2][0]);

		get_3d_vector_cross_product((vec3d *) &result[0][0], (vec3d *) &result[1][0], (vec3d *) &result[2][0]);
	}
	else
	{
		get_identity_matrix3x3(result);
	}
}

/* the map area and ceiling, from the extents the request carries (en_world.h) */
static int wf_inside_map_area(const wf_run *r, const vec3d *p)
{
	return (p->x >= r->in->map_min[0]) && (p->x <= r->in->map_max[0]) && (p->z >= r->in->map_min[2]) && (p->z <= r->in->map_max[2]);
}

/* point_below_ground: outside the map area is below the ground; inside, the terrain decides */
static int wf_below_ground(wf_run *r, const vec3d *p)
{
	if (!wf_inside_map_area(r, p))
	{
		return 1;
	}

	return wf_ask(r, wf_q(WL_GROUND_PROBE, 0, 0, 0, 0, p->x, p->y, p->z, 0, 0, 0))[0] != 0.0f;
}

enum { TI_MOBILE, TI_DECOY_TYPE, TI_RADAR_JAMMER, TI_IR_JAMMER, TI_AIRBORNE, TI_NOT_AI };
enum { TV_POSITION, TV_POINT, TV_MOTION };
enum { LI_NOT_AI, LI_LASER_ON, LI_LOS };

static int wf_launcher_int(wf_run *r, int what)
{
	if (!r->in->has_launcher)
	{
		wf_fail(r, "the weapon's guidance asks for its launcher and it has none");
	}

	return wf_ask(r, wf_q(WL_LAUNCHER_INT, what, 0, 0, 0, 0, 0, 0, 0, 0, 0))[0] != 0.0f;
}

static int wf_target_which(const wf_run *r)
{
	return r->has_target ? r->target_which : -1;
}

static wf_query wf_target_int_query(const wf_run *r, int what)
{
	return wf_q(WL_TARGET_INT, r->target_which, what, 0, 0, 0, 0, 0, 0, 0, 0);
}

static int wf_target_int(wf_run *r, int what)
{
	return (int) wf_ask(r, wf_target_int_query(r, what))[0];
}

static wf_query wf_target_vec_query(const wf_run *r, int what)
{
	return wf_q(WL_TARGET_VEC, r->target_which, what, 0, 0, 0, 0, 0, 0, 0, 0);
}

static void wf_target_vec(wf_run *r, int what, vec3d *out)
{
	const float *a = wf_ask(r, wf_target_vec_query(r, what));

	out->x = a[0];
	out->y = a[1];
	out->z = a[2];
}

static void wf_launcher_eo(wf_run *r, vec3d *position, int *valid)
{
	const float *a;

	if (!r->in->has_launcher)
	{
		wf_fail(r, "the weapon's guidance asks for its launcher and it has none");
	}

	a = wf_ask(r, wf_q(WL_LAUNCHER_EO, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0));

	position->x = a[0];
	position->y = a[1];
	position->z = a[2];
	*valid = a[3] != 0.0f;
}

static void wf_wind(const vec3d *position, vec3d *wind_velocity, void *user)
{
	wf_run *r = (wf_run *) user;
	const float *a = wf_ask(r, wf_q(WL_WIND, 0, 0, 0, 0, position->x, position->y, position->z, 0, 0, 0));

	wind_velocity->x = a[0];
	wind_velocity->y = a[1];
	wind_velocity->z = a[2];
}

static int get_decoy_type_for_weapon(wf_run *r)
{
	switch (r->fig->guidance_type)
	{
		case WF_GUIDANCE_NONE:
		{
			return WF_DECOY_WEAPON_NONE;
		}
		case WF_GUIDANCE_SEMI_ACTIVE_RADAR:
		case WF_GUIDANCE_ACTIVE_RADAR:
		case WF_GUIDANCE_RADIO_COMMAND:
		{
			return WF_DECOY_WEAPON_CHAFF;
		}
		case WF_GUIDANCE_PASSIVE_INFRA_RED:
		{
			return WF_DECOY_WEAPON_FLARE;
		}
		case WF_GUIDANCE_SEMI_ACTIVE_LASER:
		case WF_GUIDANCE_SEMI_ACTIVE_LASER_BEAM_RIDING:
		case WF_GUIDANCE_ACTIVE_LASER:
		case WF_GUIDANCE_WIRE_GUIDED:
		{
			return WF_DECOY_WEAPON_SMOKE_GRENADE;
		}
	}

	wf_fail(r, "invalid weapon guidance type");

	return WF_DECOY_WEAPON_NONE;
}

static void get_smoke_grenade_distract_position(wf_run *r, vec3d *distract_position)
{
	vec3d grenade_position;

	wf_target_vec(r, TV_POSITION, &grenade_position);

	*distract_position = grenade_position;

	distract_position->x += wf_sfrand1x(r, 0) * 50.0;
	distract_position->z += wf_sfrand1x(r, 1) * 50.0;

	if (wf_inside_map_area(r, distract_position))
	{
		distract_position->y = wf_ask(r, wf_q(WL_ELEVATION, 0, 0, 0, 0, distract_position->x, 0, distract_position->z, 0, 0, 0))[0];
	}
	else
	{
		*distract_position = grenade_position;
	}
}

static void get_jammer_distract_position(wf_run *r, vec3d *target_position)
{
	if (wf_frand1x(r, 0) > 0.5)
	{
		target_position->x += wf_sfrand1x(r, 1) * 35.0;
		target_position->y += wf_sfrand1x(r, 2) * 35.0;
		target_position->z += wf_sfrand1x(r, 3) * 35.0;
	}
}

static void get_intercept_point(wf_run *r, vec3d *intercept_point)
{
	int number_of_iterations;

	float weapon_velocity, target_true_velocity, velocity_ratio, target_move_distance;

	vec3d *weapon_position, target_position, target_motion_vector;

	wf_target_vec(r, TV_POINT, intercept_point);

	target_position = *intercept_point;
	weapon_position = &r->raw.mob.position;

	weapon_velocity = r->raw.mob.velocity;

	wf_target_vec(r, TV_MOTION, &target_motion_vector);

	target_true_velocity = get_3d_vector_magnitude(&target_motion_vector);

	if ((weapon_velocity > 0.001) && (target_true_velocity > 0.001))
	{
		if (target_true_velocity < weapon_velocity)
		{
			velocity_ratio = target_true_velocity / weapon_velocity;

			normalise_any_3d_vector(&target_motion_vector);

			number_of_iterations = 3;

			while (number_of_iterations--)
			{
				target_move_distance = get_3d_range(weapon_position, intercept_point) * velocity_ratio;

				intercept_point->x = target_position.x + (target_motion_vector.x * target_move_distance);
				intercept_point->y = target_position.y + (target_motion_vector.y * target_move_distance);
				intercept_point->z = target_position.z + (target_motion_vector.z * target_move_distance);
			}
		}
	}

	{
		float f[5];

		f[0] = weapon_velocity;
		f[1] = target_true_velocity;
		f[2] = intercept_point->x;
		f[3] = intercept_point->y;
		f[4] = intercept_point->z;

		wf_echo(r, WE_INTERCEPT, 0, 0, 0, 0, f, 5);
	}
}

static int get_target_position(wf_run *r, vec3d *position, int can_guide_on_ground_lock)
{
	int target_position_valid = FALSE;

	int weapon_decoy_type;

	if (can_guide_on_ground_lock)
	{
		// track the locked ground point
		wf_launcher_eo(r, position, &target_position_valid);

		wf_echo(r, WE_TGT_BRANCH, WT_TRACKING_POINT, target_position_valid, 0, 0, NULL, 0);
	}

	if (!target_position_valid && r->has_target)
	{
		if (wf_target_int(r, TI_MOBILE))
		{
			weapon_decoy_type = wf_target_int(r, TI_DECOY_TYPE);

			if (weapon_decoy_type == WF_DECOY_NONE)
			{
				wf_query batch[4];

				/* the facts the intercept and the jammer tests read, in one round */
				batch[0] = wf_target_vec_query(r, TV_POINT);
				batch[1] = wf_target_vec_query(r, TV_MOTION);
				batch[2] = wf_target_int_query(r, TI_RADAR_JAMMER);
				batch[3] = wf_target_int_query(r, TI_IR_JAMMER);

				wf_need(r, batch, 4);

				get_intercept_point(r, position);

				wf_echo(r, WE_TGT_BRANCH, WT_MOBILE_INTERCEPT, 1, 0, 0, NULL, 0);

				//
				// not a decoy, test for jammers
				//

				if (wf_target_int(r, TI_RADAR_JAMMER) && (get_decoy_type_for_weapon(r) == WF_DECOY_WEAPON_CHAFF))
				{
					get_jammer_distract_position(r, position);

					wf_echo(r, WE_TGT_BRANCH, WT_JAMMER_CHAFF, 1, 0, 0, NULL, 0);
				}
				else if (wf_target_int(r, TI_IR_JAMMER) && (get_decoy_type_for_weapon(r) == WF_DECOY_WEAPON_FLARE))
				{
					get_jammer_distract_position(r, position);

					wf_echo(r, WE_TGT_BRANCH, WT_JAMMER_IR, 1, 0, 0, NULL, 0);
				}
			}
			else if (weapon_decoy_type == WF_DECOY_SMOKE_GRENADE)
			{
				get_smoke_grenade_distract_position(r, position);

				wf_echo(r, WE_TGT_BRANCH, WT_SMOKE_DISTRACT, 1, 0, 0, NULL, 0);
			}
			else
			{
				get_intercept_point(r, position);

				wf_echo(r, WE_TGT_BRANCH, WT_DECOY_INTERCEPT, 1, 0, 0, NULL, 0);
			}
		}
		else
		{
			wf_target_vec(r, TV_POINT, position);
		}

		target_position_valid = TRUE;
	}

	{
		float f[3];

		f[0] = position->x;
		f[1] = position->y;
		f[2] = position->z;

		wf_echo(r, WE_TGT_END, target_position_valid, 0, 0, 0, f, 3);
	}

	return (target_position_valid);
}

static void check_guidance_source(wf_run *r, int laser_guided)
{
	/* new_target: whether there is one, and whether it is the weapon's target as the weapon holds it (the weapon's target has not changed in this step before this) */
	int new_has, new_is_current;

	float target_range, theta;

	vec3d weapon_to_target, launcher_target_position;

	const float *answer = NULL;

	if (r->in->server)
	{
		new_has = r->has_target;
		new_is_current = 1;

		//
		// check guidance source exists for all weapons
		//

		if (r->in->has_launcher)
		{
			//
			// check guidance source has changed target for weapons launched by human pilots only
			//

			if (wf_launcher_int(r, LI_NOT_AI))
			{
				// no target if no laser designation
				if (laser_guided && !wf_launcher_int(r, LI_LASER_ON))
				{
					new_has = 0;
					new_is_current = !r->has_target;
				}
				else
				{
					answer = wf_ask(r, wf_q(WL_LAUNCHER_TARGET, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0));

					new_has = answer[0] != 0.0f;
					new_is_current = new_has ? (answer[1] != 0.0f) : !r->has_target;
				}

				//
				// check target is in weapon lock cone
				//

				if (new_has)
				{
					if (!new_is_current)
					{
						launcher_target_position.x = answer[2];
						launcher_target_position.y = answer[3];
						launcher_target_position.z = answer[4];

						weapon_to_target.x = launcher_target_position.x - r->raw.mob.position.x;
						weapon_to_target.y = launcher_target_position.y - r->raw.mob.position.y;
						weapon_to_target.z = launcher_target_position.z - r->raw.mob.position.z;

						target_range = (weapon_to_target.x * weapon_to_target.x) +
											(weapon_to_target.y * weapon_to_target.y) +
											(weapon_to_target.z * weapon_to_target.z);

						if (target_range > 1.0)
						{
							target_range = sqrt(target_range);

							normalise_3d_vector_given_magnitude(&weapon_to_target, target_range);

							theta = acos(get_3d_unit_vector_dot_product(&r->raw.mob.zv, &weapon_to_target));

							if (theta > r->fig->max_launch_angle_error)
							{
								new_has = 0;
								new_is_current = !r->has_target;
							}
						}
					}
				}
			}
		}
		else
		{
			new_has = 0;
			new_is_current = !r->has_target;
		}

		wf_echo(r, WE_GUIDANCE_SOURCE, r->in->has_launcher != 0, !new_is_current, new_has != 0, 0, NULL, 0);

		if (!new_is_current)
		{
			wf_action(r, WA_TARGET_LINK, new_has ? 1 : 0, 0, 0, NULL, 0);

			r->has_target = new_has;
			r->target_which = 1;
		}
	}
}

static void move_guided_weapon(wf_run *r, vec3d *new_position, vec3d *intercept_point, int loal)
{
	wf_weapon *raw = &r->raw;
	const wf_figures *fig = r->fig;

	float length, cos_turn_demand, turn_demand, max_turn_rate, turn_rate, inhibit_damp_factor = 1, time_elapsed, current_life_time;

	int target_lost = FALSE;

	vec3d uvec_intercept_point;

	matrix3x3 m1, m2;

	current_life_time = fig->boost_time + fig->sustain_time - raw->weapon_lifetime;

	// adjust for high trajectory

	if (fig->flight_profile_or_self_destr == 1 && r->missile_phase != MISSILE_FINAL_PHASE)
	{
		float range, dive_ratio;
		vec3d *weapon_position;

		weapon_position = &raw->mob.position;

		range = get_2d_range(weapon_position, intercept_point);

		dive_ratio = (weapon_position->y - intercept_point->y) / range;

		if (r->loal_mode && !loal)
		{
				switch (r->missile_phase)
				{
				case MISSILE_PHASE1:
					if (dive_ratio > HELLFIRE_LOAL_DIVE_RATIO || range < 1800)
					{
						r->missile_phase = MISSILE_FINAL_PHASE;
						break;
					}

					if (dive_ratio < HELLFIRE_LOAL_CLIMB1_RATIO && range > 3000)
					{
					// initial climb of 30%
						float new_aim_y = weapon_position->y + range * 0.3;
						intercept_point->y = max(new_aim_y, intercept_point->y);
						break;
					}

					r->missile_phase = MISSILE_PHASE2;
					// fall through

				case MISSILE_PHASE2:
					if (dive_ratio > HELLFIRE_LOAL_DIVE_RATIO || range < 1500)
					{
						r->missile_phase = MISSILE_FINAL_PHASE;
						break;
					}

					if (dive_ratio < HELLFIRE_LOAL_CLIMB2_RATIO && range > 2500)
					{
						// seconds phase climb at 4%
						float new_aim_y = weapon_position->y + range * 0.04;
						intercept_point->y = max(new_aim_y, intercept_point->y);

						break;
					}

					r->missile_phase = MISSILE_PHASE3;
					// fall through

				case MISSILE_PHASE3:
					if (dive_ratio > HELLFIRE_LOAL_DIVE_RATIO || range < 1000)
					{
						r->missile_phase = MISSILE_FINAL_PHASE;
						break;
					}
					else
					{
						// third phase dive at 20%
						float new_aim_y = weapon_position->y - range * 0.2;
						intercept_point->y = max(new_aim_y, intercept_point->y);

						break;
					}
				default:
					wf_fail(r, "invalid missile phase");
				}
		}
		else if (!r->loal_mode)  // lobl mode
		{
				switch (r->missile_phase)
				{
				case MISSILE_PHASE1:
					if (dive_ratio > HELLFIRE_LOBL_DIVE_RATIO || range < 2000)
					{
						r->missile_phase = MISSILE_FINAL_PHASE;
						break;
					}

					if (dive_ratio < HELLFIRE_LOBL_CLIMB_RATIO && range > 2500)
					{
						// intial phase climb at 7%
						float new_aim_y = weapon_position->y + range * 0.07;
						intercept_point->y = max(new_aim_y, intercept_point->y);

						break;
					}

					r->missile_phase = MISSILE_PHASE2;
				case MISSILE_PHASE2:
					if (dive_ratio > HELLFIRE_LOBL_DIVE_RATIO || range < 1500)
					{
						r->missile_phase = MISSILE_FINAL_PHASE;
						break;
					}
					else
					{
						// second phase dive at 3%
						float new_aim_y = weapon_position->y - range * 0.03;
						intercept_point->y = max(new_aim_y, intercept_point->y);

						break;
					}

					break;
				default:
					wf_fail(r, "invalid missile phase");
				}
		}
	}

	//
	// get unit vector to intercept point
	//

	uvec_intercept_point.x = intercept_point->x - raw->mob.position.x;
	uvec_intercept_point.y = intercept_point->y - raw->mob.position.y;
	uvec_intercept_point.z = intercept_point->z - raw->mob.position.z;

	length = get_3d_vector_magnitude(&uvec_intercept_point);

	// make higher trajectory /thealx/

	if (fig->flight_profile_or_self_destr != 1)
	{
		time_elapsed = min(2.0f, 0.5f * (max(length - 500, 0.0f) / raw->mob.velocity)); // back to normal trajectory if < 2 secs left
		uvec_intercept_point.y += 5 * time_elapsed * time_elapsed / 2; // flightpath raised 10m
	}

	//
	// guard against divide by zero (weapon must be very close to the intercept point)
	//

	if (length < 1.0)
	{
		float f[4];

		f[0] = length;
		f[1] = 0.0f;
		f[2] = 0.0f;
		f[3] = 0.0f;

		wf_echo(r, WE_GUIDED, WG_TOO_CLOSE, 0, 0, 0, f, 4);

		r->kill_code = WF_KILL_HIT_TARGET;

		return;
	}

	normalise_3d_vector_given_magnitude(&uvec_intercept_point, length);

	//
	// get turn demand
	//

	cos_turn_demand = get_3d_unit_vector_dot_product(&raw->mob.zv, &uvec_intercept_point);

	//
	// check for overshot target (but only when speed is high enough, otherwise intercept point might be
	// unusually much offset
	//

	if (raw->mob.velocity > 20.0 && cos_turn_demand < fig->max_seeker_limit)   // lost guidance
	{
		target_lost = TRUE;

		if (r->has_target)
			if (wf_target_int(r, TI_AIRBORNE))
				if (current_life_time > fig->inhibit_time && fig->detonation_radius >= length)  // weapon will detonate if armed and close enough to target
				{
					float f[4];

					f[0] = length;
					f[1] = cos_turn_demand;
					f[2] = current_life_time;
					f[3] = 0.0f;

					wf_echo(r, WE_GUIDED, WG_OVERSHOT, 0, 0, 0, f, 4);

					r->kill_code = WF_KILL_OVERSHOT_TARGET;

					return;
				}
	}

	if (target_lost && fig->hellfire_ii && raw->weapon_lifetime > 0)
		target_lost = FALSE; // some magic stuff with inertial unit, only when motor is alive

	{
		float f[4];

		f[0] = (float) target_lost;
		f[1] = length;
		f[2] = cos_turn_demand;
		f[3] = raw->mob.velocity;

		wf_echo(r, WE_GUIDED, WG_TARGET_LOST, 0, 0, 0, f, 4);
	}

	if (!target_lost)
	{

		//
		// get attitude matrix (where y-axis is perpendicular to turn demand plane)
		//

		get_3d_vector_cross_product(&raw->mob.yv, &raw->mob.zv, &uvec_intercept_point);

		normalise_any_3d_vector(&raw->mob.yv);

		get_3d_vector_cross_product(&raw->mob.xv, &raw->mob.yv, &raw->mob.zv);

		normalise_any_3d_vector(&raw->mob.xv);

		//
		// close angle (rotate about y-axis)
		//

		turn_demand = acos(cos_turn_demand);

		//
		// damp turn rate during inhibit period
		//

		if (current_life_time < fig->inhibit_time)
		{
			//
			// damp using y = x * x curve
			//

			inhibit_damp_factor = current_life_time / fig->inhibit_time;

			inhibit_damp_factor *= inhibit_damp_factor;

		}
		else if (fig->flight_profile_or_self_destr == 1 && r->missile_phase != MISSILE_FINAL_PHASE)
		{
			// arneh - limit turn rate in first phases of hellfire trajectory to get a smoother arc

			inhibit_damp_factor = 0.5;
		}

		max_turn_rate = G * fig->g_max * (raw->weapon_lifetime > 0 ? 1.25 : 1) / raw->mob.velocity * inhibit_damp_factor;
		turn_rate = bound(turn_demand, - max_turn_rate, max_turn_rate);
		get_3d_transformation_matrix(m1, turn_rate * r->in->delta_time, 0.0, 0.0);
		multiply_matrix3x3_matrix3x3(m2, m1, raw->mob.attitude);

		//
		// set roll to zero to remove jitter on weapon view
		//

		get_3d_transformation_matrix(raw->mob.attitude, get_heading_from_attitude_matrix(m2), get_pitch_from_attitude_matrix(m2), 0.0);

		{
			float f[4];

			f[0] = turn_demand;
			f[1] = max_turn_rate;
			f[2] = turn_rate;
			f[3] = inhibit_damp_factor;

			wf_echo(r, WE_GUIDED, WG_TURNED, 0, 0, 0, f, 4);
		}
	}
	else
		turn_rate = 0;

	update_guided_weapon(raw, fig, &r->ctx, new_position, r->in->delta_time, turn_rate, length, TRUE);
}

static void wf_weapon_movement(wf_run *r)
{
	wf_weapon *raw = &r->raw;
	const wf_figures *fig = r->fig;

	int i, intercept_point_valid, high_precision_collision, hit_ground, tracking_point = FALSE;

	float dx, dy, dz, cruise_time_error, heading;

	vec3d old_position, new_position, intercept_point, face_normal, test_point;

	////////////////////////////////////////
	//
	// ensure velocity >= 0.0
	//
	////////////////////////////////////////

	raw->mob.velocity = bound(raw->mob.velocity, 0.0, 1.5 * fig->cruise_velocity);

	////////////////////////////////////////
	//
	// ensure valid positions
	//
	////////////////////////////////////////

	old_position = raw->mob.position;

	new_position = raw->mob.position;

	memset(&intercept_point, 0, sizeof(intercept_point));
	memset(&face_normal, 0, sizeof(face_normal));
	memset(&test_point, 0, sizeof(test_point));

	////////////////////////////////////////
	//
	// update lifetime
	//
	////////////////////////////////////////

	raw->weapon_lifetime -= r->in->delta_time;

	cruise_time_error = 0.5 * wf_frand1x(r, 0) * fig->cruise_time_max_error;

	if (r->decoy_timer && !fig->decoy_type && raw->weapon_lifetime <= - r->decoy_timer) // time for fireworks!
	{
		if (fig->warhead_type == WF_WARHEAD_CONVENTIONAL_MUNITIONS) // create submunitions if it's needed
		{
			if (r->in->server)
			{
				int count = fig->submunition_count;
				float f[1];

				wf_echo(r, WE_SUBMUNITIONS, count, 0, 0, 0, NULL, 0);

				/* the submunitions leave from the weapon as it is: its state first, then the spawn */
				wf_snapshot(r);

				f[0] = raw->mob.velocity * fig->submunition_velocity_factor;

				wf_action(r, WA_SPAWN, count, 0, 0, f, 1);
			}

			raw->weapon_lifetime = - fig->cruise_time - fig->cruise_time_max_error;
		}

		if (fig->flight_profile_or_self_destr == 3)
			r->kill_code = WF_KILL_SELF_DESTRUCT;

		r->decoy_timer = 0;

		wf_echo_path(r, WP_FUZE_TIMER);

		return;
	}

	if (raw->weapon_lifetime <= - fig->cruise_time - cruise_time_error) // this is the end of the journey..
	{
		if (fig->flight_profile_or_self_destr == 3)
			r->kill_code = WF_KILL_SELF_DESTRUCT;
		else
			r->kill_code = WF_KILL_EXHAUSTED;

		wf_echo_path(r, WP_EXHAUSTED);

		return;
	}

	////////////////////////////////////////
	//
	// do not move weapon on ground
	//
	////////////////////////////////////////

	if (r->weapon_on_ground)
	{
		raw->mob.velocity = 0.0;

		raw->mob.motion_vector.x = 0.0;
		raw->mob.motion_vector.y = 0.0;
		raw->mob.motion_vector.z = 0.0;

		wf_echo_path(r, WP_ON_GROUND);

		return;
	}

	////////////////////////////////////////
	//
	// guidance
	//
	////////////////////////////////////////

	/* the facts this step reads first, in one round: the launcher's tracking point, and the target's kind and point */
	{
		wf_query batch[3];
		int n = 0;

		if (r->in->has_launcher)
		{
			batch[n++] = wf_q(WL_LAUNCHER_EO, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
		}

		if (fig->guidance_type != WF_GUIDANCE_NONE && r->has_target)
		{
			batch[n++] = wf_target_int_query(r, TI_MOBILE);
			batch[n++] = wf_target_vec_query(r, TV_POINT);
		}

		if (n)
		{
			wf_need(r, batch, n);
		}
	}

	// check tracking point

	if (r->in->has_launcher)
	{
		vec3d tracking_point_position;

		wf_launcher_eo(r, &tracking_point_position, &tracking_point);
	}

	wf_echo(r, WE_TRACKING, tracking_point, 0, 0, 0, NULL, 0);

	switch (fig->guidance_type)
	{
		////////////////////////////////////////
		case WF_GUIDANCE_NONE:
		////////////////////////////////////////
		{
			intercept_point_valid = FALSE;

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_PASSIVE_INFRA_RED:
		////////////////////////////////////////
		{
			intercept_point_valid = get_target_position(r, &intercept_point, FALSE);

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_SEMI_ACTIVE_RADAR:
		////////////////////////////////////////
		{
			check_guidance_source(r, FALSE);

			intercept_point_valid = get_target_position(r, &intercept_point, FALSE);

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_ACTIVE_RADAR:
		////////////////////////////////////////
		{
			//
			// active radar lock on after launch (switch from NULL target to valid target)
			//

			if (r->in->server)
			{
				if (!r->has_target)  // has no target
				{
					if (fig->flight_profile_or_self_destr != 1)
					{
						check_guidance_source(r, FALSE);
					}
					else
					{
						//HELLFIRE//

						//
						// Hellfire flight profile only look for a new target after climb
						//

						if (raw->weapon_lifetime < HELLFIRE_LOAL_CLIMB1_TIME)
						{
							check_guidance_source(r, FALSE);
						}

						//HELLFIRE//
					}
				}
			}

			intercept_point_valid = get_target_position(r, &intercept_point, FALSE);

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_SEMI_ACTIVE_LASER:
		////////////////////////////////////////
		{
			int do_guidance_source_check;

			do_guidance_source_check = TRUE;

			//HELLFIRE// LOAL for weapons launched by human pilots only

			if (r->in->server)
			{
				if (!r->has_target)
				{
					if (fig->flight_profile_or_self_destr == 1)
					{
						if (r->in->has_launcher)
						{
							if (wf_launcher_int(r, LI_NOT_AI))
							{
								//
								// Hellfire flight profile only look for a new target after climb
								//

								do_guidance_source_check = FALSE;

								if (raw->weapon_lifetime < HELLFIRE_LOAL_CLIMB1_TIME)
									if (wf_launcher_int(r, LI_LASER_ON))
										if (wf_launcher_int(r, LI_LOS))
											do_guidance_source_check = TRUE;
							}
						}
					}
				}
			}

			//HELLFIRE//

			if (do_guidance_source_check)
			{
				check_guidance_source(r, TRUE);
			}

			if (wf_launcher_int(r, LI_NOT_AI)
				&& (!wf_launcher_int(r, LI_LASER_ON) || (!tracking_point && !wf_launcher_int(r, LI_LOS))))
			{
				intercept_point_valid = FALSE;
			}
			else
				intercept_point_valid = get_target_position(r, &intercept_point, TRUE);

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_SEMI_ACTIVE_LASER_BEAM_RIDING:
		////////////////////////////////////////
		{
			check_guidance_source(r, TRUE);

			if (wf_launcher_int(r, LI_NOT_AI)
				&& (!wf_launcher_int(r, LI_LASER_ON) || (!tracking_point && !wf_launcher_int(r, LI_LOS))))
			{
				intercept_point_valid = FALSE;
			}
			else
				intercept_point_valid = get_target_position(r, &intercept_point, TRUE);

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_ACTIVE_LASER:
		////////////////////////////////////////
		{
			intercept_point_valid = get_target_position(r, &intercept_point, FALSE);

			break;
		}
		////////////////////////////////////////
		case WF_GUIDANCE_RADIO_COMMAND:
		case WF_GUIDANCE_WIRE_GUIDED:
		////////////////////////////////////////
		{
			check_guidance_source(r, FALSE);

			if (wf_launcher_int(r, LI_NOT_AI)
				&& !tracking_point && !wf_launcher_int(r, LI_LOS))
			{
				intercept_point_valid = FALSE;
			}
			else
				intercept_point_valid = get_target_position(r, &intercept_point, TRUE);

			break;
		}
		default:
		{
			wf_fail(r, "invalid weapon guidance type");

			intercept_point_valid = FALSE;
		}
	}

	////////////////////////////////////////
	//
	// movement
	//
	////////////////////////////////////////

	wf_echo(r, WE_GUIDANCE, fig->guidance_type, intercept_point_valid, 0, 0, NULL, 0);

	if (intercept_point_valid && (raw->weapon_lifetime > 0 || raw->mob.velocity > 200))
	{
		move_guided_weapon(r, &new_position, &intercept_point, FALSE);

		wf_echo_path(r, WP_GUIDED);
	}
	else
	{
		// unless missile is a LOAL hellfire in phase 1 or 2, move as an unguided weapon when no target
		if (!r->loal_mode || r->missile_phase > MISSILE_PHASE2 || (raw->weapon_lifetime <= 0 && raw->mob.velocity <= 200))
		{
			move_unguided_weapon(raw, fig, &r->ctx, &new_position, r->in->delta_time, fig->aiming_type != 2);

			wf_echo_path(r, WP_UNGUIDED);
		}
		else
		{
			//HELLFIRE//

			//
			// Hellfire flight profile
			//

			intercept_point = raw->mob.position;

			//
			// climb or dive
			//

			if (raw->weapon_lifetime > HELLFIRE_LOAL_CLIMB1_TIME)
			{
				intercept_point.x += HELLFIRE_LOAL_XZ * raw->mob.zv.x;
				intercept_point.y += HELLFIRE_LOAL_CLIMB1_Y;
				intercept_point.z += HELLFIRE_LOAL_XZ * raw->mob.zv.z;
			}
			else if (raw->weapon_lifetime > HELLFIRE_LOAL_CLIMB2_TIME)
			{
				intercept_point.x += HELLFIRE_LOAL_XZ * raw->mob.zv.x;
				intercept_point.y += HELLFIRE_LOAL_CLIMB2_Y;
				intercept_point.z += HELLFIRE_LOAL_XZ * raw->mob.zv.z;
			}
			else
			{
				intercept_point.x += HELLFIRE_LOAL_XZ * raw->mob.zv.x;
				intercept_point.y += HELLFIRE_LOAL_DIVE_Y;
				intercept_point.z += HELLFIRE_LOAL_XZ * raw->mob.zv.z;
			}

			move_guided_weapon(r, &new_position, &intercept_point, TRUE);

			wf_echo_path(r, WP_LOAL_CLIMB_DIVE);

			//HELLFIRE//
		}
	}

	// TODO check rotation: the Tacview update looks at the weapon as the movement left it
	wf_snapshot(r);

	wf_action(r, WA_TACVIEW, 0, 0, 0, NULL, 0);

	if (r->kill_code != WF_KILL_OK)
	{
		wf_echo_path(r, WP_KILLED_BY_GUIDANCE);

		return;
	}

	////////////////////////////////////////
	//
	// collision detection precision
	//
	////////////////////////////////////////

	high_precision_collision = FALSE;

	if (r->has_target)
	{
		if (wf_target_int(r, TI_NOT_AI))
		{
			high_precision_collision = TRUE;
		}
		else
		{
			vec3d target_position;

			wf_target_vec(r, TV_POSITION, &target_position);

			if (get_3d_range(&new_position, &target_position) < 50)
			{
				high_precision_collision = TRUE;
			}
		}
	}

	if (r->in->has_launcher)
	{
		if (wf_launcher_int(r, LI_NOT_AI))
		{
			high_precision_collision = TRUE;
		}
	}

	if (wf_ask(r, wf_q(WL_VIEW_WEAPON, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0))[0] != 0.0f)
	{
		high_precision_collision = TRUE;
	}

	wf_echo(r, WE_PRECISION, high_precision_collision, 0, 0, 0, NULL, 0);

	////////////////////////////////////////
	//
	// target collision detection
	//
	////////////////////////////////////////

	if (r->in->server)
	{
		if (fig->in_flight_collision_check)
		{
			if (high_precision_collision || (!r->has_target))
			{
				const float *hit = wf_ask(r, wf_q(WL_COLLISION, wf_target_which(r), 0, 0, 0, old_position.x, old_position.y, old_position.z, new_position.x, new_position.y, new_position.z));

				{
					float f[6];

					if (hit[0] != 0.0f)
					{
						new_position.x = hit[3];
						new_position.y = hit[4];
						new_position.z = hit[5];
					}

					f[0] = old_position.x;
					f[1] = old_position.y;
					f[2] = old_position.z;
					f[3] = new_position.x;
					f[4] = new_position.y;
					f[5] = new_position.z;

					wf_echo(r, WE_CONTACT_ANY, hit[0] != 0.0f, (int) hit[2], 0, 0, f, 6);
				}

				if (hit[0] != 0.0f)
				{
					if (hit[1] != 0.0f)
					{
						//
						// switch target parent to new_target (on server only)
						//

						wf_action(r, WA_TARGET_LINK, 2, 0, 0, NULL, 0);
					}

					wf_echo_path(r, WP_CONTACT_TARGET);

					r->kill_code = WF_KILL_HIT_TARGET;
				}
				else if (high_precision_collision)
				{
					const float *tree = wf_ask(r, wf_q(WL_TREES, 0, 0, 0, 0, old_position.x, old_position.y, old_position.z, new_position.x, new_position.y, new_position.z));
					float f[6];

					if (tree[0] != 0.0f)
					{
						new_position.x = tree[1];
						new_position.y = tree[2];
						new_position.z = tree[3];
					}

					f[0] = old_position.x;
					f[1] = old_position.y;
					f[2] = old_position.z;
					f[3] = new_position.x;
					f[4] = new_position.y;
					f[5] = new_position.z;

					wf_echo(r, WE_CONTACT_TREES, tree[0] != 0.0f, 0, 0, 0, f, 6);

					if (tree[0] != 0.0f)
					{
						wf_echo_path(r, WP_CONTACT_TREES);

						r->kill_code = WF_KILL_SELF_DESTRUCT;
					}
				}
			}
		}
	}

	////////////////////////////////////////
	//
	// terrain collision detection
	//
	////////////////////////////////////////

	//
	// UP TO THIS POINT THE WEAPON HAS NOT BEEN MOVED SO IT IS OK TO RETURN WITHOUT BOUNDS CHECKING THE POSITION
	//

	//
	// from now on only overwrite kill_code if kill_code == WEAPON_KILL_CODE_OK
	//

	//
	// inside map area
	//

	if (!wf_inside_map_area(r, &new_position))
	{
		bound_position_to_map_volume(&new_position, r->in->map_min, r->in->map_max);

		wf_echo_path(r, WP_OUT_OF_MAP_AREA);

		wf_position_action(r, &new_position);

		if (r->kill_code == WF_KILL_OK)
		{
			r->kill_code = WF_KILL_OUT_OF_BOUNDS;
		}

		return;
	}

	//
	// below map ceiling
	//

	if (!(new_position.y <= r->in->map_max[1]))
	{
		bound_position_to_map_volume(&new_position, r->in->map_min, r->in->map_max);

		wf_echo_path(r, WP_ABOVE_CEILING);

		wf_position_action(r, &new_position);

		if (r->kill_code == WF_KILL_OK)
		{
			r->kill_code = WF_KILL_OUT_OF_BOUNDS;
		}

		return;
	}

	//
	// set weapon position (set again if new_position changes)
	//

	wf_position_action(r, &new_position);

	//
	// terrain collision
	//

	hit_ground = FALSE;

	/* the terrain under the new position and, for a precise weapon, under the nine points between: one round (the nine are only read as far as the first that is below ground) */
	{
		wf_query batch[10];
		int n = 0;
		vec3d point;

		batch[n++] = wf_q(WL_GROUND_PROBE, 0, 0, 0, 0, new_position.x, new_position.y, new_position.z, 0, 0, 0);

		if (high_precision_collision)
		{
			dx = (new_position.x - old_position.x) * 0.1;
			dy = (new_position.y - old_position.y) * 0.1;
			dz = (new_position.z - old_position.z) * 0.1;

			point = old_position;

			for (i = 0; i < 9; i++)
			{
				point.x += dx;
				point.y += dy;
				point.z += dz;

				if (wf_inside_map_area(r, &point))
				{
					batch[n++] = wf_q(WL_GROUND_PROBE, 0, 0, 0, 0, point.x, point.y, point.z, 0, 0, 0);
				}
			}
		}

		wf_need(r, batch, n);
	}

	if (wf_below_ground(r, &new_position))
	{
		hit_ground = TRUE;
	}
	else
	{
		if (high_precision_collision)
		{
			dx = (new_position.x - old_position.x) * 0.1;
			dy = (new_position.y - old_position.y) * 0.1;
			dz = (new_position.z - old_position.z) * 0.1;

			test_point = old_position;

			//
			// new_position has already been tested so there is no need to test again (also the interpolated new_position
			// would be inaccurate and may be tested as above ground when it is below)
			//

			for (i = 0; i < 9; i++)
			{
				test_point.x += dx;
				test_point.y += dy;
				test_point.z += dz;

				if (wf_inside_map_area(r, &test_point))
				{
					if (wf_below_ground(r, &test_point))
					{
						hit_ground = TRUE;

						new_position = test_point;

						break;
					}
				}
			}
		}
	}

	wf_echo(r, WE_GROUND, hit_ground, high_precision_collision, 0, 0, NULL, 0);

	if (hit_ground)
	{
		const float *impact = wf_ask(r, wf_q(WL_GROUND_IMPACT, 0, 0, 0, 0, old_position.x, old_position.y, old_position.z, new_position.x, new_position.y, new_position.z));
		int terrain_class = (int) impact[0];

		new_position.x = impact[1];
		new_position.y = impact[2];
		new_position.z = impact[3];

		wf_echo(r, WE_IMPACT, terrain_class, fig->survive_ground_impact, 0, 0, NULL, 0);

		wf_position_action(r, &new_position);

		if (terrain_class == WF_TERRAIN_CLASS_WATER)
		{
			wf_echo_path(r, WP_GROUND_WATER);

			if (r->kill_code == WF_KILL_OK)
			{
				r->kill_code = WF_KILL_HIT_WATER;
			}
		}
		else
		{
			if (fig->survive_ground_impact)
			{
				wf_echo_path(r, WP_GROUND_SURVIVED);

				//
				// align weapon with terrain
				//

				heading = get_heading_from_attitude_matrix(raw->mob.attitude);

				{
					const float *normal = wf_ask(r, wf_q(WL_FACE_NORMAL, 0, 0, 0, 0, new_position.x, 0, new_position.z, 0, 0, 0));

					face_normal.x = normal[0];
					face_normal.y = normal[1];
					face_normal.z = normal[2];
				}

				get_3d_transformation_matrix_from_face_normal_and_heading(raw->mob.attitude, &face_normal, heading);

				//
				// smoke grenade effect
				//

				if (fig->smoke_grenade)
				{
					if (r->in->server)
					{
						wf_echo_path(r, WP_GROUND_SMOKE_GRENADE);

						/* the effect looks at the weapon (aligned with the terrain) and at its kill code, which is hit-land for the call */
						wf_snapshot(r);

						wf_action(r, WA_SMOKE_EFFECT, 0, 0, 0, NULL, 0);

						r->kill_code = WF_KILL_OK;
					}
				}

				r->weapon_on_ground = TRUE;
			}
			else
			{
				if (r->kill_code == WF_KILL_OK)
				{
					wf_echo_path(r, WP_GROUND_LAND);

					r->kill_code = WF_KILL_HIT_LAND;
				}
			}
		}
	}
}

/* The step: the weapon in, the actions (or the lookups and draws still needed) out.  Returns nonzero on a failure the reason explains. */
int eech_wn_weapon_step(const wf_step_in *in, wf_step_out *out)
{
	wf_run run;

	memset(out, 0, sizeof(*out));
	memset(&run, 0, sizeof(run));

	run.in = in;
	run.out = out;
	run.fig = in->figures;

	if (!in->figures)
	{
		out->failed = 1;
		snprintf(out->reason, sizeof(out->reason), "%s", "the host holds no figures for this weapon");

		return 1;
	}

	run.raw.mob.position.x = in->position[0];
	run.raw.mob.position.y = in->position[1];
	run.raw.mob.position.z = in->position[2];
	memcpy(&run.raw.mob.attitude[0][0], in->attitude, sizeof(in->attitude));
	run.raw.mob.motion_vector.x = in->motion_vector[0];
	run.raw.mob.motion_vector.y = in->motion_vector[1];
	run.raw.mob.motion_vector.z = in->motion_vector[2];
	run.raw.mob.velocity = in->velocity;
	run.raw.weapon_lifetime = in->weapon_lifetime;
	run.raw.parachute_status = in->parachute_status;
	run.decoy_timer = in->decoy_timer;
	run.missile_phase = in->missile_phase;
	run.loal_mode = in->loal_mode;
	run.weapon_on_ground = in->weapon_on_ground;
	run.kill_code = in->kill_code;
	run.has_target = in->has_target;
	run.target_which = 0;
	run.ctx.wind = wf_wind;
	run.ctx.draw = wf_draw;
	run.ctx.user = &run;
	run.ctx.system_time = in->system_time;
	run.ctx.delta_time = in->delta_time;

	if (setjmp(run.escape) == 0)
	{
		wf_weapon_movement(&run);

		wf_snapshot(&run);
	}

	if (out->need_lookup || out->need_draw || out->failed)
	{
		/* a step that has not finished has changed nothing */
		out->action_count = 0;
		out->echo_count = 0;
	}

	return out->failed;
}

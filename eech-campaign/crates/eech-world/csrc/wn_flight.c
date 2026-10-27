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

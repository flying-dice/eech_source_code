/*
 * The host's physical step for an alive fixed-wing aircraft.
 *
 * The arithmetic is EECH's own, kept expression for expression so that the
 * host and the original reference produce the same bits: fw_move.c
 * (basic_fixed_wing_movement and the functions it calls), with the shared
 * helpers in eech_arith.h.  The campaign decides everything that is guidance:
 * whether the aircraft moves at all, which sub-mode applies (taxi, flight,
 * landing, or the absolute turn-around/aiming flight), the waypoint, the guide's
 * speed, the navigator's acceleration vector (collision avoidance, pursuit,
 * waypoint following) and the cruise/turn parameters.  This file performs the
 * physical step those decisions ask for: velocity integration with its limits,
 * position, attitude, the motion vector, the map bound and the terrain clamp.
 * The engine's step delta (`delta_seconds`) and the frame delta (`frame_delta`,
 * a reciprocal used for the motion vector) are different values and stay
 * separate.
 *
 * `motion_vector` is both an input (the previous step's, which the flight
 * attitude and the navigator follow) and the output.  The legacy absolute
 * flight's ground branch is unreachable (it is only entered airborne) and is
 * not carried over.
 */
#include "eech_arith.h"

#define PHASE_TAXI 0
#define PHASE_FLIGHT 1
#define PHASE_LAND 2
#define PHASE_ABSOLUTE 3

#define GROUND_ACCELERATION 3.0f
#define AIR_ACCELERATION 30.0f
#define PITCH_MIN -30.0f
#define PITCH_MAX 30.0f
#define AIRCRAFT_MAX_FLARE_ANGLE rad(15.0f)
#define AIRCRAFT_MAX_LANDING_FLARE_ANGLE rad(4.0f)

static void bound_vector_scale(vec3d *vector, float min_scale, float max_scale)
{
	float distance;

	distance = get_3d_vector_magnitude(vector);

	if (distance > max_scale)
	{
		normalise_3d_vector_given_magnitude(vector, distance);

		vector->x *= max_scale;
		vector->y *= max_scale;
		vector->z *= max_scale;
	}
	else if (distance < min_scale)
	{
		normalise_3d_vector_given_magnitude(vector, distance);

		vector->x *= min_scale;
		vector->y *= min_scale;
		vector->z *= min_scale;
	}
}

void eech_fw_step(
	int phase,
	float *position,
	float *velocity_io,
	float *attitude,
	float delta_seconds,
	float frame_delta,
	float wp_vel,
	float cruise_velocity,
	float g_max,
	float fuselage_angle,
	float cg_to_ground,
	float max_roll,
	float turn_speed_multi,
	const float *waypoint,
	const float *acceleration,
	float *motion_vector,
	int navigating,
	float elevation,
	const float *map_min,
	const float *map_max,
	int tacview_logging,
	int *rotated)
{
	vec3d current_pos, new_pos;
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	float velocity = *velocity_io;
	float old_roll = 0.0, old_pitch = 0.0, old_heading = 0.0;
	float terrain_elevation = elevation;

	current_pos.x = position[0];
	current_pos.y = position[1];
	current_pos.z = position[2];

	*rotated = 0;

	if (tacview_logging)
	{
		old_roll = get_roll_from_attitude_matrix(*matrix);
		old_pitch = get_pitch_from_attitude_matrix(*matrix);
		old_heading = get_heading_from_attitude_matrix(*matrix);
	}

	if (phase == PHASE_TAXI)
	{
		vec3d wp_pos, wp_vec, dist;
		float desired_velocity, current_heading, desired_heading, delta_heading, new_heading, current_pitch, new_pitch;

		wp_pos.x = waypoint[0];
		wp_pos.y = waypoint[1];
		wp_pos.z = waypoint[2];

		wp_vec.x = wp_pos.x - current_pos.x;
		wp_vec.y = wp_pos.y - (current_pos.y - cg_to_ground);
		wp_vec.z = wp_pos.z - current_pos.z;

		desired_velocity = wp_vel;

		velocity += min((desired_velocity - velocity), GROUND_ACCELERATION) * delta_seconds;

		current_heading = get_heading_from_attitude_matrix(*matrix);

		desired_heading = atan2(wp_vec.x, wp_vec.z);

		delta_heading = desired_heading - current_heading;

		if (delta_heading <= -PI)
		{
			delta_heading += PI2;
		}

		if (delta_heading >= PI)
		{
			delta_heading -= PI2;
		}

		delta_heading = bound(delta_heading, rad(-30.0), rad(30.0));

		new_heading = current_heading + (delta_heading * delta_seconds);

		current_pitch = get_pitch_from_attitude_matrix(*matrix);

		if (current_pitch > fuselage_angle)
		{
			new_pitch = current_pitch + ((fuselage_angle - current_pitch) * delta_seconds);
		}
		else
		{
			new_pitch = fuselage_angle;
		}

		get_3d_transformation_matrix(*matrix, new_heading, new_pitch, 0.0);

		dist.x = min(velocity * (*matrix)[2][0] * delta_seconds, fabs(wp_vec.x));
		dist.y = 0.0;
		dist.z = min(velocity * (*matrix)[2][2] * delta_seconds, fabs(wp_vec.z));

		new_pos.x = current_pos.x + dist.x;
		new_pos.y = current_pos.y + dist.y;
		new_pos.z = current_pos.z + dist.z;

		bound_position_to_map_volume(&new_pos, map_min, map_max);

		new_pos.y = terrain_elevation + cg_to_ground;

		motion_vector[0] = velocity * (*matrix)[2][0];
		motion_vector[1] = 0.0;
		motion_vector[2] = velocity * (*matrix)[2][2];
	}
	else if (phase == PHASE_FLIGHT)
	{
		vec3d acc, vel, norm_mv;
		float min_speed, max_speed, acc_avail, d;
		float pitch, calc_old_heading, heading, change_in_heading, roll, factor;

		acc.x = acceleration[0];
		acc.y = acceleration[1];
		acc.z = acceleration[2];

		vel.x = motion_vector[0];
		vel.y = motion_vector[1];
		vel.z = motion_vector[2];

		/* fixed_wing_set_velocity */
		min_speed = max_speed = cruise_velocity;

		min_speed *= 0.7;

		max_speed *= 2.0;

		acc_avail = 1.0;

		d = get_3d_vector_magnitude(&acc);

		if (d > g_max / 10)
		{
			normalise_3d_vector_given_magnitude(&acc, d / (g_max / 10));
		}

		vel.x += acc.x * acc_avail * delta_seconds;
		vel.y += acc.y * acc_avail * delta_seconds;
		vel.z += acc.z * acc_avail * delta_seconds;

		bound_vector_scale(&vel, min_speed, max_speed);

		velocity = get_3d_vector_magnitude(&vel);

		new_pos.x = current_pos.x + vel.x * delta_seconds;
		new_pos.y = current_pos.y + vel.y * delta_seconds;
		new_pos.z = current_pos.z + vel.z * delta_seconds;

		/* fixed_wing_calc_attitude: follows the previous step's motion vector */
		calc_old_heading = get_heading_from_attitude_matrix(*matrix);

		roll = get_roll_from_attitude_matrix(*matrix);

		norm_mv.x = motion_vector[0];
		norm_mv.y = motion_vector[1];
		norm_mv.z = motion_vector[2];

		normalise_any_3d_vector(&norm_mv);

		heading = atan2(norm_mv.x, norm_mv.z);

		pitch = asin(norm_mv.y);

		change_in_heading = heading - calc_old_heading;

		if (change_in_heading > PI)
		{
			change_in_heading -= PI2;
		}
		else if (change_in_heading < (-PI))
		{
			change_in_heading += PI2;
		}

		factor = max_roll * 0.01;

		if (fabs(change_in_heading) >= factor)
		{
			roll += change_in_heading;
		}
		else
		{
			factor = (factor - fabs(change_in_heading)) / factor;

			roll -= (roll * factor * delta_seconds);

			roll += (change_in_heading * (1.0 - factor));
		}

		if (max_roll >= PI)
		{
			if (roll > PI)
			{
				roll -= PI2;
			}
			else if (roll < (-PI))
			{
				roll += PI2;
			}
		}
		else
		{
			roll = bound(roll, -max_roll, max_roll);
		}

		get_3d_transformation_matrix(*matrix, heading, pitch, roll);

		/* the motion vector is calculated BEFORE the bound, otherwise it may be zeroed */
		motion_vector[0] = (new_pos.x - current_pos.x) * frame_delta;
		motion_vector[1] = (new_pos.y - current_pos.y) * frame_delta;
		motion_vector[2] = (new_pos.z - current_pos.z) * frame_delta;
	}
	else if (phase == PHASE_LAND)
	{
		vec3d wp_pos, wp_vec;
		float factor, range, delta_vel, heading, pitch, roll, desired_heading, desired_pitch, delta_heading, delta_pitch, delta_roll;

		wp_pos.x = waypoint[0];
		wp_pos.y = waypoint[1];
		wp_pos.z = waypoint[2];

		desired_pitch = 0.0;

		wp_vec.x = wp_pos.x - current_pos.x;
		wp_vec.y = wp_pos.y - current_pos.y;
		wp_vec.z = wp_pos.z - current_pos.z;

		range = get_3d_vector_magnitude(&wp_vec);

		normalise_any_3d_vector(&wp_vec);

		heading = get_heading_from_attitude_matrix(*matrix);

		pitch = get_pitch_from_attitude_matrix(*matrix);

		roll = get_roll_from_attitude_matrix(*matrix);

		desired_heading = atan2(wp_vec.x, wp_vec.z);

		if (range < 100.0)
		{
			desired_pitch = AIRCRAFT_MAX_LANDING_FLARE_ANGLE;
		}
		else if (range < 1000.0)
		{
			factor = (1000.0 - range) * 0.001;

			desired_pitch = AIRCRAFT_MAX_FLARE_ANGLE * factor;
		}
		else
		{
			desired_pitch = asin(wp_vec.y);
		}

		delta_heading = (desired_heading - heading) * delta_seconds;

		delta_pitch = (desired_pitch - pitch) * delta_seconds;

		delta_roll = -roll * delta_seconds;

		heading += delta_heading;

		pitch += delta_pitch;

		roll += delta_roll;

		get_3d_transformation_matrix(*matrix, heading, pitch, roll);

		delta_vel = (wp_vel - velocity);

		velocity += delta_vel;

		wp_vec.x *= velocity;
		wp_vec.y *= velocity;
		wp_vec.z *= velocity;

		new_pos.x = current_pos.x + wp_vec.x * delta_seconds;
		new_pos.y = current_pos.y + wp_vec.y * delta_seconds;
		new_pos.z = current_pos.z + wp_vec.z * delta_seconds;

		motion_vector[0] = (new_pos.x - current_pos.x) * frame_delta;
		motion_vector[1] = (new_pos.y - current_pos.y) * frame_delta;
		motion_vector[2] = (new_pos.z - current_pos.z) * frame_delta;
	}
	else
	{
		/* basic_fixed_wing_movement_absolute, airborne */
		vec3d wp_pos, wp_vec, normalised_wp_vec, dist;
		float required_heading, required_pitch, required_roll, delta_heading, delta_pitch, heading, pitch, roll, max_turn_rate;

		wp_pos.x = waypoint[0];
		wp_pos.y = waypoint[1];
		wp_pos.z = waypoint[2];

		wp_vec.x = wp_pos.x - current_pos.x;
		wp_vec.y = wp_pos.y - (current_pos.y - cg_to_ground);
		wp_vec.z = wp_pos.z - current_pos.z;

		velocity += min(((wp_vel / turn_speed_multi) - velocity), AIR_ACCELERATION) * delta_seconds;

		normalised_wp_vec = wp_vec;

		normalise_any_3d_vector(&normalised_wp_vec);

		/* get_heading_and_pitch_from_3d_unit_vector */
		required_heading = atan2(normalised_wp_vec.x, normalised_wp_vec.z);
		required_pitch = asin(normalised_wp_vec.y);

		heading = get_heading_from_attitude_matrix(*matrix);

		pitch = get_pitch_from_attitude_matrix(*matrix);

		roll = get_roll_from_attitude_matrix(*matrix);

		if (velocity)
		{
			max_turn_rate = g_max / velocity;
		}
		else
		{
			max_turn_rate = g_max;
		}

		delta_heading = (required_heading - heading);

		if (delta_heading <= -PI)
		{
			delta_heading += PI2;
		}

		if (delta_heading >= PI)
		{
			delta_heading -= PI2;
		}

		delta_heading = bound(delta_heading, -max_turn_rate, max_turn_rate);

		/* the position follows the attitude's z axis from BEFORE this step's turn */
		dist.x = velocity * (*matrix)[2][0] * delta_seconds;
		dist.y = velocity * (*matrix)[2][1] * delta_seconds;
		dist.z = velocity * (*matrix)[2][2] * delta_seconds;

		new_pos.x = current_pos.x + dist.x;
		new_pos.y = current_pos.y + dist.y;
		new_pos.z = current_pos.z + dist.z;

		heading += delta_heading * delta_seconds;

		delta_pitch = (required_pitch - pitch);

		delta_pitch = bound(delta_pitch, PITCH_MIN, PITCH_MAX);

		pitch += delta_pitch * delta_seconds;

		required_roll = (delta_heading / max_turn_rate) * rad(70);

		roll += (required_roll - roll) * delta_seconds;

		get_3d_transformation_matrix(*matrix, heading, pitch, roll);

		motion_vector[0] = (new_pos.x - current_pos.x) * frame_delta;
		motion_vector[1] = (new_pos.y - current_pos.y) * frame_delta;
		motion_vector[2] = (new_pos.z - current_pos.z) * frame_delta;
	}

	bound_position_to_map_volume(&new_pos, map_min, map_max);

	if (navigating)
	{
		if (new_pos.y < (terrain_elevation + 100.0))
		{
			new_pos.y = terrain_elevation + 100.0;
		}
	}
	else
	{
		if (new_pos.y < terrain_elevation)
		{
			new_pos.y = terrain_elevation;
		}
	}

	/* the absolute flight never reported a rotation */
	if (tacview_logging && phase != PHASE_ABSOLUTE)
	{
		if (fabs(get_roll_from_attitude_matrix(*matrix) - old_roll) > rad(0.001)
			|| fabs(get_pitch_from_attitude_matrix(*matrix) - old_pitch) > rad(0.001)
			|| fabs(get_heading_from_attitude_matrix(*matrix) - old_heading) > rad(0.001))
		{
			*rotated = 1;
		}
	}

	position[0] = new_pos.x;
	position[1] = new_pos.y;
	position[2] = new_pos.z;
	*velocity_io = velocity;
}

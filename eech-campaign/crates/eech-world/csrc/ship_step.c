/*
 * The host's physical step for an alive ship.
 *
 * The arithmetic is EECH's own, kept expression for expression so that the
 * host and the original reference produce the same bits: sh_move.c
 * (ship_vehicle_movement), with the shared helpers in eech_arith.h.  The
 * campaign decides whether the ship moves at all (moving, primary guide) and
 * supplies the waypoint, the guide's speed, the acceleration limit and the
 * class's g limit; this file performs the step.  The engine's step delta
 * (`delta_seconds`) and the frame delta (`frame_delta`, a reciprocal used for
 * the motion vector) are different values and stay separate.
 */
#include "eech_arith.h"

void eech_ship_step(
	float *position,
	float *velocity_io,
	float *attitude,
	float delta_seconds,
	float frame_delta,
	float guide_velocity,
	float max_acceleration,
	float g_max,
	const float *waypoint,
	const float *map_min,
	const float *map_max,
	float *motion_vector)
{
	vec3d wp_vec, new_pos;
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	float velocity = *velocity_io;
	float roll, pitch, heading, sqr_range, turn_rate, required_heading, delta_heading, new_velocity;

	wp_vec.x = waypoint[0] - position[0];
	wp_vec.y = 0;
	wp_vec.z = waypoint[2] - position[2];

	sqr_range = ((wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z));

	/* sh_move.c writes z twice and leaves x as it was; kept as is */
	if (fabs(sqr_range) < 1 * CENTIMETRE)
	{
		wp_vec.z = 0;
		wp_vec.y = 0;
		wp_vec.z = 1;
	}

	normalise_any_3d_vector(&wp_vec);

	heading = get_heading_from_attitude_matrix(*matrix);

	required_heading = atan2(wp_vec.x, wp_vec.z);

	{
		float angle, range, v;

		range = sqrt(sqr_range);

		v = sqrt(0.5 * range * g_max);

		angle = (((*matrix)[2][0] * wp_vec.x) + ((*matrix)[2][2] * wp_vec.z));

		if (angle < 0.707)
		{
			new_velocity = bound(v, 0.0, guide_velocity);
		}
		else
		{
			new_velocity = guide_velocity;
		}
	}

	turn_rate = 0.0;

	if (velocity != 0.0)
	{
		turn_rate = fabs(g_max / velocity);
	}

	delta_heading = required_heading - heading;

	if (delta_heading <= -PI)
	{
		delta_heading += PI2;
	}

	if (delta_heading >= PI)
	{
		delta_heading -= PI2;
	}

	delta_heading = bound(delta_heading, -turn_rate, turn_rate);

	heading += delta_heading * delta_seconds;

	pitch = 0.0;

	roll = 0.0;

	get_3d_transformation_matrix(*matrix, heading, rad(pitch), rad(roll));

	{
		float maximum_acceleration, required_acceleration;

		required_acceleration = (new_velocity - velocity);

		maximum_acceleration = max_acceleration;

		velocity += min(required_acceleration, maximum_acceleration) * delta_seconds;
	}

	/* the position follows the new attitude's z axis */
	new_pos.x = position[0] + (velocity * (*matrix)[2][0] * delta_seconds);
	new_pos.y = 0.0;
	new_pos.z = position[2] + (velocity * (*matrix)[2][2] * delta_seconds);

	bound_position_to_map_volume(&new_pos, map_min, map_max);

	motion_vector[0] = (new_pos.x - position[0]) * frame_delta;
	motion_vector[1] = 0.0;
	motion_vector[2] = (new_pos.z - position[2]) * frame_delta;

	new_pos.y = 0.0;

	position[0] = new_pos.x;
	position[1] = new_pos.y;
	position[2] = new_pos.z;
	*velocity_io = velocity;
}

/*
 * The physical half of an alive AI helicopter's step (helicopter_movement, helicop/hc_move.c), in two phases around
 * the campaign's guidance.  Every expression is kept from the engine so the host and the original reference produce the
 * same bits.  The DLL supplies guidance inputs (the world acceleration and the climb demand, the desired heading, pitch
 * and roll, the turn-rate limit, the terrain elevation under the aircraft); the heading wrap, the turn-rate bound, the
 * attitude blend, the position integration and the floor clamp are the host's physical integration.
 *
 * Not here, and still campaign-side coupling: rotor RPM and spin-up, terrain look-ahead, the desired pitch/roll/heading
 * guidance, formation and avoidance, pilot control, and the death and crash movement.
 */
#include "eech_arith.h"

/*
 * Phase one: the motion vector, in world coordinates, then the speed it implies.
 * `acceleration` is the world acceleration's x and z (the campaign's decision), `wp_vec_y` the climb demand.
 */
void eech_hc_begin(float *motion_vector, float acceleration_x, float acceleration_z, float wp_vec_y, float delta_seconds, float *speed, int *moved)
{
	vec3d *mv = (vec3d *)motion_vector;
	float s;

	mv->x += acceleration_x * delta_seconds;
	mv->y = wp_vec_y;
	mv->z += acceleration_z * delta_seconds;

	s = get_3d_vector_magnitude(mv);

	*speed = s;
	*moved = (fabs(s) > 0.1) ? 1 : 0;
}

/*
 * Phase two: the attitude, then the position integrated with the motion vector, the map bound and the floor clamp.
 * `desired_heading` is the guidance heading, wrapped here to the shortest turn from the attitude's heading and bound to
 * `max_turn_rate`, the aircraft's physical turn limit (the engine's order: wrap, then bound, then integrate).  The
 * attitude is the one the campaign's guidance read, so the heading is the same one the DLL passed `desired_heading`
 * against.  The floor clamp may change the motion vector's y after the speed was computed; the speed is not recomputed.
 */
void eech_hc_end(float *position, float *motion_vector, float *attitude, float desired_heading, float max_turn_rate, float desired_pitch, float desired_roll, float delta_seconds, float elevation, float cg_to_ground, int terrain_follow, const float *map_min, const float *map_max, int *rotated)
{
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	vec3d *pos = (vec3d *)position;
	vec3d *mv = (vec3d *)motion_vector;
	float current_heading, current_pitch, current_roll;
	float delta_heading;
	float new_heading, new_pitch, new_roll;
	float min_height;

	current_heading = get_heading_from_attitude_matrix(*matrix);
	current_pitch = get_pitch_from_attitude_matrix(*matrix);
	current_roll = get_roll_from_attitude_matrix(*matrix);

	delta_heading = (desired_heading - current_heading);

	if (delta_heading < -PI)
	{
		delta_heading += PI2;
	}
	else if (delta_heading > PI)
	{
		delta_heading -= PI2;
	}

	delta_heading = bound(delta_heading, -max_turn_rate, max_turn_rate);

	new_heading = current_heading + (delta_heading * delta_seconds);

	new_pitch = current_pitch + (desired_pitch - current_pitch) * 0.5 * delta_seconds;
	new_roll = current_roll + (desired_roll - current_roll) * 0.5 * delta_seconds;

	*rotated = (fabs(new_pitch - current_pitch) > rad(0.1) || fabs(new_heading - current_heading) > rad(0.1) || fabs(new_roll - current_roll) > rad(0.1)) ? 1 : 0;

	get_3d_transformation_matrix(*matrix, new_heading, new_pitch, new_roll);

	pos->x += (mv->x * delta_seconds);
	pos->y += (mv->y * delta_seconds);
	pos->z += (mv->z * delta_seconds);

	bound_position_to_map_volume(pos, map_min, map_max);

	min_height = elevation + cg_to_ground;

	if (terrain_follow)
	{
		min_height += 2.0;
	}

	/* bound to floor.  Allow for the pitch of the aircraft (the new attitude) ... assume 5m to nose.  Stop motion vector. */
	if (pos->y <= min_height)
	{
		float pitch;

		pitch = get_pitch_from_attitude_matrix(*matrix);

		min_height += 5.0 * sin(fabs(pitch));

		mv->y = (mv->y > 0.0f) ? mv->y : 0.0f;

		pos->y = (pos->y > min_height) ? pos->y : min_height;
	}
}

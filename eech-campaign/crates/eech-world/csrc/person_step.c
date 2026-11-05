/*
 * The host's physical step for an alive infantryman, in the two phases the
 * terrain seam needs.
 *
 * The arithmetic is EECH's own, kept expression for expression so that the
 * host and the original reference produce the same bits: ps_move.c
 * (person_movement), with the shared helpers in eech_arith.h.  The campaign
 * picks the animation (and so the speed `ps_vel`, and whether the man walks on
 * the ground) and keeps the stand-animation random draw; this file does the
 * movement.
 *
 * `eech_person_advance` is the horizontal phase: it gives the new position (its
 * height is only meaningful for a man who does not follow the ground), the
 * increment and the heading.  `eech_person_settle` is the final phase: the
 * height (the terrain elevation sampled by the campaign at the advanced
 * position, for a man who follows the ground), the attitude, the speed and the
 * motion vector.
 */
#include "eech_arith.h"

void eech_person_advance(
	const float *position,
	float ps_vel,
	float delta_seconds,
	const float *waypoint,
	float *new_position,
	float *increment,
	float *heading_out)
{
	vec3d new_pos, normalised_wp_vec, wp_vec;
	float distance, movement_distance, heading, x_inc, y_inc, z_inc;

	wp_vec.x = waypoint[0] - position[0];
	wp_vec.y = waypoint[1] - position[1];
	wp_vec.z = waypoint[2] - position[2];

	normalised_wp_vec = wp_vec;

	distance = normalise_any_3d_vector(&normalised_wp_vec);

	movement_distance = min(ps_vel * delta_seconds, distance);

	x_inc = normalised_wp_vec.x * movement_distance;
	y_inc = normalised_wp_vec.y * movement_distance;
	z_inc = normalised_wp_vec.z * movement_distance;

	new_pos.x = position[0] + x_inc;
	new_pos.y = position[1] + y_inc;
	new_pos.z = position[2] + z_inc;

	heading = atan2(normalised_wp_vec.x, normalised_wp_vec.z);

	new_position[0] = new_pos.x;
	new_position[1] = new_pos.y;
	new_position[2] = new_pos.z;
	increment[0] = x_inc;
	increment[1] = y_inc;
	increment[2] = z_inc;
	*heading_out = heading;
}

void eech_person_settle(
	float *position,
	float *velocity_io,
	float *attitude,
	const float *new_position,
	const float *increment,
	float heading,
	float ps_vel,
	float delta_seconds,
	int terrain_follow,
	float elevation,
	float *motion_vector)
{
	vec3d new_pos;
	matrix3x3 *matrix = (matrix3x3 *)attitude;

	new_pos.x = new_position[0];
	new_pos.y = new_position[1];
	new_pos.z = new_position[2];

	if (terrain_follow)
	{
		new_pos.y = elevation + 0.01;
	}

	get_3d_transformation_heading_matrix(*matrix, heading);

	position[0] = new_pos.x;
	position[1] = new_pos.y;
	position[2] = new_pos.z;

	*velocity_io = ps_vel;

	motion_vector[0] = increment[0] / delta_seconds;
	motion_vector[1] = increment[1] / delta_seconds;
	motion_vector[2] = increment[2] / delta_seconds;
}

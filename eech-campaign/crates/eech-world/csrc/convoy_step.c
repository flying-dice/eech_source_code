/*
 * The host's physical step for an alive routed vehicle (ground convoy member).
 *
 * The arithmetic is EECH's own, kept expression for expression so that the
 * host and the original reference produce the same bits: rv_move.c
 * (routed_vehicle_movement), matrix.c (get_3d_transformation_heading_pitch_matrix,
 * get_3d_transformation_matrix_from_face_normal_and_heading,
 * get_identity_matrix3x3) and vector.c (normalise_any_3d_vector,
 * get_3d_vector_cross_product).  The campaign decides whether to move; this
 * file only performs the step it is asked for.  Build flags match the engine's
 * numeric flags (build.rs).
 */
#include "eech_arith.h"

static void get_3d_vector_cross_product(vec3d *result, const vec3d *v1, const vec3d *v2)
{
	result->x = (v1->y * v2->z) - (v2->y * v1->z);
	result->y = (v1->z * v2->x) - (v2->z * v1->x);
	result->z = (v1->x * v2->y) - (v2->x * v1->y);
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

static void get_3d_transformation_heading_pitch_matrix(matrix3x3 m, float heading, float pitch)
{
	float sinh, cosh, sinp, cosp;

	sinh = sin(heading);
	cosh = cos(heading);

	sinp = sin(pitch);
	cosp = cos(pitch);

	m[0][0] = cosh;
	m[0][1] = 0;
	m[0][2] = -sinh;
	m[1][0] = -(sinh * sinp);
	m[1][1] = cosp;
	m[1][2] = -(cosh * sinp);
	m[2][0] = cosp * sinh;
	m[2][1] = sinp;
	m[2][2] = cosp * cosh;
}

static void get_3d_transformation_matrix_from_face_normal_and_heading(matrix3x3 result, const vec3d *normal, float heading)
{
	if (fabs(normal->y) > 0.0001)
	{
		memcpy(&result[1][0], normal, sizeof(vec3d));

		result[2][0] = sin(heading);
		result[2][2] = cos(heading);
		result[2][1] = -((normal->x * result[2][0]) + (normal->z * result[2][2])) / normal->y;

		normalise_any_3d_vector((vec3d *)&result[2][0]);

		get_3d_vector_cross_product((vec3d *)&result[0][0], (vec3d *)&result[1][0], (vec3d *)&result[2][0]);
	}
	else
	{
		get_identity_matrix3x3(result);
	}
}

/*
 * One MOVE step.  `position`, `velocity` and `attitude` are the host's record
 * and are updated in place; `rotated` is set only when `tacview_logging`.
 * `terrain_present` selects the terrain-following attitude and height.
 */
void eech_convoy_step(
	float *position,
	float *velocity_io,
	float *attitude,
	float delta_seconds,
	float desired_velocity,
	float max_acceleration,
	const float *waypoint,
	int tacview_logging,
	int terrain_present,
	float elevation,
	int forest,
	const float *face_normal_in,
	const float *map_min,
	const float *map_max,
	int *rotated)
{
	vec3d wp_vec, new_pos, face_normal;
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	float velocity = *velocity_io;
	float distance, pitch, heading;

	*rotated = 0;

	wp_vec.x = waypoint[0] - position[0];
	wp_vec.y = waypoint[1] - position[1];
	wp_vec.z = waypoint[2] - position[2];

	new_pos.x = position[0];
	new_pos.y = position[1];
	new_pos.z = position[2];

	/* not on top of waypoint (sqr check within 0.1m) */
	if (((wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z)) > 1.0)
	{
		float old_heading = 0, old_roll = 0, old_pitch = 0, movement_distance, delta_velocity, max_accel;

		max_accel = max_acceleration;

		delta_velocity = (desired_velocity - velocity);

		delta_velocity = bound(delta_velocity, -(10.0 * max_accel), max_accel);

		velocity += delta_velocity * delta_seconds;

		heading = atan2(wp_vec.x, wp_vec.z);

		distance = normalise_any_3d_vector(&wp_vec);

		if (tacview_logging)
		{
			old_roll = get_roll_from_attitude_matrix(*matrix);
			old_pitch = get_pitch_from_attitude_matrix(*matrix);
			old_heading = get_heading_from_attitude_matrix(*matrix);
		}

		if (terrain_present)
		{
			new_pos.y = elevation;

			if (forest)
			{
				new_pos.y -= 20.0;
			}

			face_normal.x = face_normal_in[0];
			face_normal.y = face_normal_in[1];
			face_normal.z = face_normal_in[2];

			get_3d_transformation_matrix_from_face_normal_and_heading(*matrix, &face_normal, heading);
		}
		else
		{
			pitch = asin(wp_vec.y);

			get_3d_transformation_heading_pitch_matrix(*matrix, heading, pitch);
		}

		if (tacview_logging)
		{
			if (fabs(get_roll_from_attitude_matrix(*matrix) - old_roll) > rad(0.001) ||
				fabs(get_pitch_from_attitude_matrix(*matrix) - old_pitch) > rad(0.001) ||
				fabs(heading - old_heading) > rad(0.001))
			{
				*rotated = 1;
			}
		}

		movement_distance = min(distance, velocity * delta_seconds);

		new_pos.x += wp_vec.x * movement_distance;
		new_pos.y += wp_vec.y * movement_distance;
		new_pos.z += wp_vec.z * movement_distance;
	}

	bound_position_to_map_volume(&new_pos, map_min, map_max);

	position[0] = new_pos.x;
	position[1] = new_pos.y;
	position[2] = new_pos.z;
	*velocity_io = velocity;
}

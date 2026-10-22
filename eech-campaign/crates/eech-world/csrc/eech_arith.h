/*
 * EECH's own arithmetic shared by the host's physical steps (convoy_step.c,
 * ship_step.c, person_step.c, fw_step.c): the macros and vector, matrix and map-bound
 * helpers, each kept expression for expression from the engine (maths/matrix.c,
 * vector.c, en_world.c) so that the host and the original reference produce the
 * same bits.
 */
#ifndef EECH_ARITH_H
#define EECH_ARITH_H

#include <math.h>
#include <string.h>

#define PI (3.14159265359f)
#define PI2 (PI*2)
#define CENTIMETRE (0.01f)
#define rad(DEG) ((DEG) * (PI / 180.0f))
#define bound(VALUE, LOWER, UPPER) (((VALUE) < (LOWER)) ? (LOWER) : (((VALUE) > (UPPER)) ? (UPPER) : (VALUE)))
#define min(a, b) (((a) < (b)) ? (a) : (b))

#define get_heading_from_attitude_matrix(ATTITUDE) (atan2((ATTITUDE)[2][0], (ATTITUDE)[2][2]))
#define get_pitch_from_attitude_matrix(ATTITUDE) (asin((ATTITUDE)[2][1]))
#define get_roll_from_attitude_matrix(ATTITUDE) (atan2(-(ATTITUDE)[0][1], (ATTITUDE)[1][1]))

typedef struct vec3d
{
	float x, y, z;
} vec3d;

typedef float matrix3x3[3][3];

static float normalise_any_3d_vector(vec3d *vector)
{
	float x, y, z, length;

	x = ((vector->x) * (vector->x));
	y = ((vector->y) * (vector->y));
	z = ((vector->z) * (vector->z));

	length = sqrt(x + y + z);

	if (length > 0)
	{
		float one_over_length;

		one_over_length = 1.0 / length;

		vector->x = (vector->x * one_over_length);
		vector->y = (vector->y * one_over_length);
		vector->z = (vector->z * one_over_length);
	}
	else
	{
		vector->x = 0;
		vector->y = 0;
		vector->z = 0;
	}

	return (length);
}

static float get_3d_vector_magnitude(const vec3d *vector)
{
	float x, y, z;

	x = (vector->x * vector->x);
	y = (vector->y * vector->y);
	z = (vector->z * vector->z);

	return (sqrt(x + y + z));
}

static void normalise_3d_vector_given_magnitude(vec3d *vector, float length)
{
	float one_over_length;

	one_over_length = 1.0 / length;

	vector->x = (vector->x * one_over_length);
	vector->y = (vector->y * one_over_length);
	vector->z = (vector->z * one_over_length);
}

/* clamp the position to the map volume, per axis, as bound_position_to_map_volume does */
static void bound_position_to_map_volume(vec3d *position, const float *map_min, const float *map_max)
{
	if (position->x < map_min[0])
	{
		position->x = map_min[0];
	}
	else if (position->x > map_max[0])
	{
		position->x = map_max[0];
	}

	if (position->y < map_min[1])
	{
		position->y = map_min[1];
	}
	else if (position->y > map_max[1])
	{
		position->y = map_max[1];
	}

	if (position->z < map_min[2])
	{
		position->z = map_min[2];
	}
	else if (position->z > map_max[2])
	{
		position->z = map_max[2];
	}
}

static void get_3d_transformation_matrix(matrix3x3 m, float heading, float pitch, float roll)
{
	float sinh, cosh, sinp, cosp, sinr, cosr, sinrsinh, sinrcosh, cosrsinh, cosrcosh;

	sinh = sin(heading);
	cosh = cos(heading);

	sinp = sin(pitch);
	cosp = cos(pitch);

	sinr = sin(roll);
	cosr = cos(roll);

	sinrsinh = sinr * sinh;

	sinrcosh = sinr * cosh;

	cosrsinh = cosr * sinh;

	cosrcosh = cosr * cosh;

	m[0][0] = cosrcosh + (sinrsinh * sinp);

	m[0][1] = -sinr * cosp;

	m[0][2] = (sinrcosh * sinp) - cosrsinh;

	m[1][0] = sinrcosh - (cosrsinh * sinp);

	m[1][1] = cosr * cosp;

	m[1][2] = -sinrsinh - (cosrcosh * sinp);

	m[2][0] = cosp * sinh;

	m[2][1] = sinp;

	m[2][2] = cosp * cosh;
}

static void get_3d_transformation_heading_matrix(matrix3x3 m, float heading)
{
	float sinh, cosh;

	sinh = sin(heading);
	cosh = cos(heading);

	m[0][0] = cosh;
	m[0][1] = 0.0;
	m[0][2] = -sinh;
	m[1][0] = 0.0;
	m[1][1] = 1.0;
	m[1][2] = 0.0;
	m[2][0] = sinh;
	m[2][1] = 0.0;
	m[2][2] = cosh;
}

#endif

/*
 * EECH's own arithmetic shared by the host's physical steps (convoy_step.c,
 * ship_step.c, person_step.c, fw_step.c): the macros and vector, matrix and map-bound
 * helpers, each kept expression for expression from the engine (maths/matrix.c,
 * vector.c, en_world.c) so that the host and the original reference produce the
 * same bits.
 */
#ifndef EECH_ARITH_H
#define EECH_ARITH_H

#include <float.h>
#include <math.h>
#include <string.h>

#define PI (3.14159265359f)
#define PI2 (PI*2)
#define CENTIMETRE (0.01f)
#define rad(DEG) ((DEG) * (PI / 180.0f))
#define bound(VALUE, LOWER, UPPER) (((VALUE) < (LOWER)) ? (LOWER) : (((VALUE) > (UPPER)) ? (UPPER) : (VALUE)))
#define min(a, b) (((a) < (b)) ? (a) : (b))
#define max(a, b) (((a) > (b)) ? (a) : (b))

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

/* maths/vector.c: both vectors are normalised first, so only the angle between them counts */
static float get_3d_vector_dot_product(const vec3d *a, const vec3d *b)
{
	vec3d p, q;
	float result;

	p.x = a->x;
	p.y = a->y;
	p.z = a->z;
	q.x = b->x;
	q.y = b->y;
	q.z = b->z;

	normalise_any_3d_vector(&p);
	normalise_any_3d_vector(&q);

	result = (p.x * q.x) + (p.y * q.y) + (p.z * q.z);

	result = bound(result, -1.0, 1.0);

	return (result);
}

/* maths/range.c */
static float get_sqr_2d_range(const vec3d *v1, const vec3d *v2)
{
	float dx, dz, range;

	dx = v1->x - v2->x;
	dz = v1->z - v2->z;

	range = (dx * dx) + (dz * dz);

	return (range);
}

static float get_sqr_3d_range(const vec3d *v1, const vec3d *v2)
{
	float dx, dy, dz, range;

	dx = v1->x - v2->x;
	dy = v1->y - v2->y;
	dz = v1->z - v2->z;

	range = (dx * dx) + (dy * dy) + (dz * dz);

	return (range);
}

/* maths/vector.c: the perpendicular foot of point c on segment ab (point_d), FLT_MAX when there is none */
static float get_point_to_point_distance(vec3d *uv_ab, const vec3d *point_a, const vec3d *point_b)
{
	uv_ab->x = point_b->x - point_a->x;
	uv_ab->y = point_b->y - point_a->y;
	uv_ab->z = point_b->z - point_a->z;

	return sqrt((uv_ab->x * uv_ab->x) + (uv_ab->y * uv_ab->y) + (uv_ab->z * uv_ab->z));
}

static float get_3d_perp_dist_of_point_from_line(const vec3d *point_a, const vec3d *point_b, const vec3d *point_c, vec3d *point_d)
{
	float mag_ab, mag_ac, mag_ad, theta, cos_theta, dist;
	vec3d uv_ab, uv_ac;

	mag_ab = get_point_to_point_distance(&uv_ab, point_a, point_b);

	if (mag_ab == 0.0)
	{
		if (point_d)
		{
			*point_d = *point_a;
		}

		return (FLT_MAX);
	}

	normalise_3d_vector_given_magnitude(&uv_ab, mag_ab);

	uv_ac.x = point_c->x - point_a->x;
	uv_ac.y = point_c->y - point_a->y;
	uv_ac.z = point_c->z - point_a->z;

	mag_ac = sqrt((uv_ac.x * uv_ac.x) + (uv_ac.y * uv_ac.y) + (uv_ac.z * uv_ac.z));

	if (mag_ac == 0.0)
	{
		if (point_d)
		{
			*point_d = *point_a;
		}

		return (FLT_MAX);
	}

	normalise_3d_vector_given_magnitude(&uv_ac, mag_ac);

	cos_theta = (uv_ab.x * uv_ac.x) + (uv_ab.y * uv_ac.y) + (uv_ab.z * uv_ac.z);

	if (cos_theta < 0.0)
	{
		if (point_d)
		{
			*point_d = *point_a;
		}

		return (FLT_MAX);
	}

	mag_ad = mag_ac * cos_theta;

	if (mag_ad > mag_ab)
	{
		if (point_d)
		{
			*point_d = *point_b;
		}

		return (FLT_MAX);
	}

	if (point_d)
	{
		point_d->x = point_a->x + (uv_ab.x * mag_ad);
		point_d->y = point_a->y + (uv_ab.y * mag_ad);
		point_d->z = point_a->z + (uv_ab.z * mag_ad);
	}

	theta = acos(cos_theta);

	dist = mag_ac * sin(theta);

	return (dist);
}

/* en_world.c bound_position_to_map_area: x and z only */
static void bound_position_to_map_area(vec3d *position, const float *map_min, const float *map_max)
{
	if (position->x < map_min[0])
	{
		position->x = map_min[0];
	}
	else if (position->x > map_max[0])
	{
		position->x = map_max[0];
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

/* maths/matrix.c */
static void multiply_matrix3x3_vec3d(vec3d *result, const matrix3x3 m, const vec3d *v)
{
	vec3d temp;

	temp.x = v->x * m[0][0];
	temp.x += v->y * m[1][0];
	temp.x += v->z * m[2][0];

	temp.y = v->x * m[0][1];
	temp.y += v->y * m[1][1];
	temp.y += v->z * m[2][1];

	temp.z = v->x * m[0][2];
	temp.z += v->y * m[1][2];
	temp.z += v->z * m[2][2];

	result->x = temp.x;
	result->y = temp.y;
	result->z = temp.z;
}

static void multiply_transpose_matrix3x3_vec3d(vec3d *result, const matrix3x3 m, const vec3d *v)
{
	vec3d temp;

	temp.x = v->x * m[0][0];
	temp.x += v->y * m[0][1];
	temp.x += v->z * m[0][2];

	temp.y = v->x * m[1][0];
	temp.y += v->y * m[1][1];
	temp.y += v->z * m[1][2];

	temp.z = v->x * m[2][0];
	temp.z += v->y * m[2][1];
	temp.z += v->z * m[2][2];

	result->x = temp.x;
	result->y = temp.y;
	result->z = temp.z;
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

static void get_matrix3x3_from_unit_vec3d(matrix3x3 result, const vec3d *v)
{
	float heading, pitch;

	heading = atan2(v->x, v->z);

	pitch = asin(v->y);

	get_3d_transformation_matrix(result, heading, pitch, 0.0);
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

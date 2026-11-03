/*
 * The physical test of whether a line segment hits ONE collision candidate (EEDCS-49, D3): the narrow phase of EECH's own
 * get_line_of_sight_collision_entity (aphavoc/source/entity/en_misc/en_maths/en_cllsn.c: accurate_stationary_bounding_box_collision_test and
 * accurate_moving_bounding_box_collision_test) with the line-triangle and line-quad intersections they call (modules/maths/vector.c). The two
 * intersections below are the SAME TEXT as the ones in tree_step.c (a test compares the two copies), and the rest is the original expression for expression.
 *
 * What the host owns here: the transform of the model's collision triangles (or, for a scene with none, of its bounding box's corners) into the world under
 * the candidate's pose, the choice of which sweep positions of a moving candidate are tested (its speed against its body length), the order of the faces and
 * the intersection arithmetic. The model's own data (the triangles, one depth at a time, and the bounds) is the host's ModelStore, read through the callback.
 *
 * What the campaign keeps and sends: WHICH candidate and in which order (the sector scan, the exclusions, the broad-phase radius test), and the candidate's
 * live pose (position, attitude, a moving candidate's sweep ends and speed), derived by the campaign from its own entity state in the original's order.
 *
 * Exactly kept: the intersection helpers write the intercept point and normal BEFORE the hit decision, so a miss leaves partial writes. The point and normal the
 * caller passes in are inputs, and the final words are returned whether or not there was a hit. Float, double and narrowing steps are the original's:
 * `(a + b) * 0.5` and `body_length * 0.25` promote through double as they do there. The caller (src/entity_host.rs) runs this under round-toward-zero with the
 * caller's complete floating-point state saved and restored, as the tree kernel does; this file does not touch the environment and is NOT compiled with
 * -frounding-math, so its constants fold at the default rounding as the original's did.
 *
 * POSE (floats): [0..2] segment old, [3..5] segment new, [6..8] the intercept point the caller holds, [9..11] the face normal the caller holds, then
 *   stationary: [12..14] position, [15..23] attitude (row-major 3x3)
 *   moving:     [12..14] old position, [15..17] new position, [18..26] attitude, [27] the candidate's speed (the magnitude of its motion vector)
 * The state is none: nothing is kept between calls.
 */
#include <math.h>
#include <stddef.h>

#include "eech_arith.h"

#ifndef TRUE
#define TRUE 1
#define FALSE 0
#endif

#define TEST_OLD_POSITION (1)
#define TEST_MID_POSITION (2)
#define TEST_NEW_POSITION (4)

/* ------------------------------------------------------------------------------------------------------------------------------------ */
/* maths/vector.c: the intersections, without the ASSERTs or debug logging; the float, double and narrowing steps are the original's. */
/* ------------------------------------------------------------------------------------------------------------------------------------ */

static int get_3d_line_triangle_intersection(const vec3d *p1, const vec3d *p2, const vec3d *pa, const vec3d *pb, const vec3d *pc, vec3d *pi, vec3d *ni)
{
	float length, d, denom, mu, angle_aib, angle_aic, angle_bic, acos_angle_aib, acos_angle_aic, acos_angle_bic, total;

	vec3d vec_ab, vec_ac, vec_12, vec_ia, vec_ib, vec_ic;

	vec_ab.x = pb->x - pa->x;
	vec_ab.y = pb->y - pa->y;
	vec_ab.z = pb->z - pa->z;

	vec_ac.x = pc->x - pa->x;
	vec_ac.y = pc->y - pa->y;
	vec_ac.z = pc->z - pa->z;

	ni->x = (vec_ab.y * vec_ac.z) - (vec_ab.z * vec_ac.y);
	ni->y = (vec_ab.z * vec_ac.x) - (vec_ab.x * vec_ac.z);
	ni->z = (vec_ab.x * vec_ac.y) - (vec_ab.y * vec_ac.x);

	length = sqrt((ni->x * ni->x) + (ni->y * ni->y) + (ni->z * ni->z));

	if (length == 0.0)
	{
		return (FALSE);
	}

	normalise_3d_vector_given_magnitude(ni, length);

	d = -(ni->x * pa->x) - (ni->y * pa->y) - (ni->z * pa->z);

	vec_12.x = p2->x - p1->x;
	vec_12.y = p2->y - p1->y;
	vec_12.z = p2->z - p1->z;

	denom = (ni->x * vec_12.x) + (ni->y * vec_12.y) + (ni->z * vec_12.z);

	if (fabs(denom) == 0.0)
	{
		return (FALSE);
	}

	mu = -(d + (ni->x * p1->x) + (ni->y * p1->y) + (ni->z * p1->z)) / denom;

	if ((mu < 0.0) || (mu > 1.0))
	{
		return (FALSE);
	}

	pi->x = p1->x + mu * (vec_12.x);
	pi->y = p1->y + mu * (vec_12.y);
	pi->z = p1->z + mu * (vec_12.z);

	vec_ia.x = pa->x - pi->x;
	vec_ia.y = pa->y - pi->y;
	vec_ia.z = pa->z - pi->z;

	length = sqrt((vec_ia.x * vec_ia.x) + (vec_ia.y * vec_ia.y) + (vec_ia.z * vec_ia.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ia, length);

	vec_ib.x = pb->x - pi->x;
	vec_ib.y = pb->y - pi->y;
	vec_ib.z = pb->z - pi->z;

	length = sqrt((vec_ib.x * vec_ib.x) + (vec_ib.y * vec_ib.y) + (vec_ib.z * vec_ib.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ib, length);

	vec_ic.x = pc->x - pi->x;
	vec_ic.y = pc->y - pi->y;
	vec_ic.z = pc->z - pi->z;

	length = sqrt((vec_ic.x * vec_ic.x) + (vec_ic.y * vec_ic.y) + (vec_ic.z * vec_ic.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ic, length);

	angle_aib = (vec_ia.x * vec_ib.x) + (vec_ia.y * vec_ib.y) + (vec_ia.z * vec_ib.z);
	angle_bic = (vec_ib.x * vec_ic.x) + (vec_ib.y * vec_ic.y) + (vec_ib.z * vec_ic.z);
	angle_aic = (vec_ic.x * vec_ia.x) + (vec_ic.y * vec_ia.y) + (vec_ic.z * vec_ia.z);

	acos_angle_aib = acos(angle_aib);
	acos_angle_bic = acos(angle_bic);
	acos_angle_aic = acos(angle_aic);

	total = acos_angle_aib + acos_angle_bic + acos_angle_aic;

	if ((total < (-PI2 + 0.01)) || (total > (PI2 - 0.01)))
	{
		return (TRUE);
	}

	return (FALSE);
}

static int get_3d_line_quad_intersection(const vec3d *p1, const vec3d *p2, const vec3d *pa, const vec3d *pb, const vec3d *pc, const vec3d *pd, vec3d *pi, vec3d *ni)
{
	float length, d, denom, mu, angle_aib, angle_aic, angle_aid, angle_bic, angle_cid, acos_angle_aib, acos_angle_aic, acos_angle_aid, acos_angle_bic,
		acos_angle_cid, total;

	vec3d vec_ab, vec_ac, vec_12, vec_ia, vec_ib, vec_ic, vec_id;

	vec_ab.x = pb->x - pa->x;
	vec_ab.y = pb->y - pa->y;
	vec_ab.z = pb->z - pa->z;

	vec_ac.x = pc->x - pa->x;
	vec_ac.y = pc->y - pa->y;
	vec_ac.z = pc->z - pa->z;

	ni->x = (vec_ab.y * vec_ac.z) - (vec_ab.z * vec_ac.y);
	ni->y = (vec_ab.z * vec_ac.x) - (vec_ab.x * vec_ac.z);
	ni->z = (vec_ab.x * vec_ac.y) - (vec_ab.y * vec_ac.x);

	length = sqrt((ni->x * ni->x) + (ni->y * ni->y) + (ni->z * ni->z));

	if (length == 0.0)
	{
		return (FALSE);
	}

	normalise_3d_vector_given_magnitude(ni, length);

	d = -(ni->x * pa->x) - (ni->y * pa->y) - (ni->z * pa->z);

	vec_12.x = p2->x - p1->x;
	vec_12.y = p2->y - p1->y;
	vec_12.z = p2->z - p1->z;

	denom = (ni->x * vec_12.x) + (ni->y * vec_12.y) + (ni->z * vec_12.z);

	if (fabs(denom) == 0.0)
	{
		return (FALSE);
	}

	mu = -(d + (ni->x * p1->x) + (ni->y * p1->y) + (ni->z * p1->z)) / denom;

	if ((mu < 0.0) || (mu > 1.0))
	{
		return (FALSE);
	}

	pi->x = p1->x + mu * (vec_12.x);
	pi->y = p1->y + mu * (vec_12.y);
	pi->z = p1->z + mu * (vec_12.z);

	vec_ia.x = pa->x - pi->x;
	vec_ia.y = pa->y - pi->y;
	vec_ia.z = pa->z - pi->z;

	length = sqrt((vec_ia.x * vec_ia.x) + (vec_ia.y * vec_ia.y) + (vec_ia.z * vec_ia.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ia, length);

	vec_ib.x = pb->x - pi->x;
	vec_ib.y = pb->y - pi->y;
	vec_ib.z = pb->z - pi->z;

	length = sqrt((vec_ib.x * vec_ib.x) + (vec_ib.y * vec_ib.y) + (vec_ib.z * vec_ib.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ib, length);

	vec_ic.x = pc->x - pi->x;
	vec_ic.y = pc->y - pi->y;
	vec_ic.z = pc->z - pi->z;

	length = sqrt((vec_ic.x * vec_ic.x) + (vec_ic.y * vec_ic.y) + (vec_ic.z * vec_ic.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ic, length);

	angle_aib = (vec_ia.x * vec_ib.x) + (vec_ia.y * vec_ib.y) + (vec_ia.z * vec_ib.z);
	angle_bic = (vec_ib.x * vec_ic.x) + (vec_ib.y * vec_ic.y) + (vec_ib.z * vec_ic.z);
	angle_aic = (vec_ic.x * vec_ia.x) + (vec_ic.y * vec_ia.y) + (vec_ic.z * vec_ia.z);

	acos_angle_aib = acos(angle_aib);
	acos_angle_bic = acos(angle_bic);
	acos_angle_aic = acos(angle_aic);

	total = acos_angle_aib + acos_angle_bic + acos_angle_aic;

	if ((total < (-PI2 + 0.01)) || (total > (PI2 - 0.01)))
	{
		return (TRUE);
	}

	vec_id.x = pd->x - pi->x;
	vec_id.y = pd->y - pi->y;
	vec_id.z = pd->z - pi->z;

	length = sqrt((vec_id.x * vec_id.x) + (vec_id.y * vec_id.y) + (vec_id.z * vec_id.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_id, length);

	angle_cid = (vec_ic.x * vec_id.x) + (vec_ic.y * vec_id.y) + (vec_ic.z * vec_id.z);
	angle_aid = (vec_id.x * vec_ia.x) + (vec_id.y * vec_ia.y) + (vec_id.z * vec_ia.z);

	acos_angle_cid = acos(angle_cid);
	acos_angle_aid = acos(angle_aid);

	total = acos_angle_aic + acos_angle_cid + acos_angle_aid;

	if ((total < (-PI2 + 0.01)) || (total > (PI2 - 0.01)))
	{
		return (TRUE);
	}

	return (FALSE);
}


/* en_cllsn.c: the two face wrappers, minus the debug drawing */
#define get_3d_line_triangle_face_intersection(OLD, NEW, P1, P2, P3, INTERCEPT, NORMAL) get_3d_line_triangle_intersection(OLD, NEW, P1, P2, P3, INTERCEPT, NORMAL)
#define get_3d_line_quad_face_intersection(OLD, NEW, P1, P2, P3, P4, INTERCEPT, NORMAL) get_3d_line_quad_intersection(OLD, NEW, P1, P2, P3, P4, INTERCEPT, NORMAL)

/* One step of the model's collision-triangle cursor: 1 and nine floats for a triangle, 0 for the end or the absence of one, negative for a refusal. */
typedef int (*eech_entity_triangle_fn)(void *context, int depth, float *points);

/* The eight corners of the bounds (xmin, xmax, ymin, ymax, zmin, zmax) in the original's order, rotated by the attitude. */
static void box_corners(vec3d *p, const float *bounds, const matrix3x3 attitude)
{
	p[0].x = bounds[0];
	p[0].y = bounds[2];
	p[0].z = bounds[4];
	multiply_matrix3x3_vec3d(&p[0], attitude, &p[0]);

	p[1].x = bounds[0];
	p[1].y = bounds[3];
	p[1].z = bounds[4];
	multiply_matrix3x3_vec3d(&p[1], attitude, &p[1]);

	p[2].x = bounds[1];
	p[2].y = bounds[3];
	p[2].z = bounds[4];
	multiply_matrix3x3_vec3d(&p[2], attitude, &p[2]);

	p[3].x = bounds[1];
	p[3].y = bounds[2];
	p[3].z = bounds[4];
	multiply_matrix3x3_vec3d(&p[3], attitude, &p[3]);

	p[4].x = bounds[0];
	p[4].y = bounds[2];
	p[4].z = bounds[5];
	multiply_matrix3x3_vec3d(&p[4], attitude, &p[4]);

	p[5].x = bounds[0];
	p[5].y = bounds[3];
	p[5].z = bounds[5];
	multiply_matrix3x3_vec3d(&p[5], attitude, &p[5]);

	p[6].x = bounds[1];
	p[6].y = bounds[3];
	p[6].z = bounds[5];
	multiply_matrix3x3_vec3d(&p[6], attitude, &p[6]);

	p[7].x = bounds[1];
	p[7].y = bounds[2];
	p[7].z = bounds[5];
	multiply_matrix3x3_vec3d(&p[7], attitude, &p[7]);
}

/* The six faces of a box in the original's order; the first face the segment crosses ends the test. */
static int box_faces(const vec3d *old_position, const vec3d *new_position, const vec3d *q, vec3d *intercept, vec3d *normal)
{
	if (get_3d_line_quad_face_intersection(old_position, new_position, &q[0], &q[1], &q[2], &q[3], intercept, normal))
	{
		return TRUE;
	}
	if (get_3d_line_quad_face_intersection(old_position, new_position, &q[3], &q[2], &q[6], &q[7], intercept, normal))
	{
		return TRUE;
	}
	if (get_3d_line_quad_face_intersection(old_position, new_position, &q[7], &q[6], &q[5], &q[4], intercept, normal))
	{
		return TRUE;
	}
	if (get_3d_line_quad_face_intersection(old_position, new_position, &q[4], &q[5], &q[1], &q[0], intercept, normal))
	{
		return TRUE;
	}
	if (get_3d_line_quad_face_intersection(old_position, new_position, &q[1], &q[5], &q[6], &q[2], intercept, normal))
	{
		return TRUE;
	}
	if (get_3d_line_quad_face_intersection(old_position, new_position, &q[0], &q[4], &q[7], &q[3], intercept, normal))
	{
		return TRUE;
	}

	return FALSE;
}

/* p + position for each of `count` points */
static void translate(vec3d *q, const vec3d *p, int count, const vec3d *position)
{
	int i;

	for (i = 0; i < count; i++)
	{
		q[i].x = p[i].x + position->x;
		q[i].y = p[i].y + position->y;
		q[i].z = p[i].z + position->z;
	}
}

/* accurate_stationary_bounding_box_collision_test: the model at one pose. 1 hit, 0 miss, negative: the triangle cursor refused. */
static int stationary(const float *pose, const float *bounds, eech_entity_triangle_fn triangles, void *context, vec3d *intercept, vec3d *normal)
{
	vec3d p[8], position, old_position, new_position;
	matrix3x3 attitude;
	int depth, found;

	old_position.x = pose[0];
	old_position.y = pose[1];
	old_position.z = pose[2];
	new_position.x = pose[3];
	new_position.y = pose[4];
	new_position.z = pose[5];
	position.x = pose[12];
	position.y = pose[13];
	position.z = pose[14];
	memcpy(attitude, &pose[15], 9 * sizeof(float));

	depth = 0;
	found = triangles(context, depth++, (float *) p);
	if (found < 0)
	{
		return found;
	}

	if (found)
	{
		do
		{
			multiply_matrix3x3_vec3d(&p[0], attitude, &p[0]);
			multiply_matrix3x3_vec3d(&p[1], attitude, &p[1]);
			multiply_matrix3x3_vec3d(&p[2], attitude, &p[2]);

			p[0].x += position.x;
			p[0].y += position.y;
			p[0].z += position.z;

			p[1].x += position.x;
			p[1].y += position.y;
			p[1].z += position.z;

			p[2].x += position.x;
			p[2].y += position.y;
			p[2].z += position.z;

			if (get_3d_line_triangle_face_intersection(&old_position, &new_position, &p[0], &p[1], &p[2], intercept, normal))
			{
				return TRUE;
			}

			found = triangles(context, depth++, (float *) p);
			if (found < 0)
			{
				return found;
			}
		}
		while (found);
	}
	else
	{
		int corner;

		box_corners(p, bounds, (const float (*)[3]) attitude);

		for (corner = 0; corner < 8; corner++)
		{
			p[corner].x += position.x;
			p[corner].y += position.y;
			p[corner].z += position.z;
		}

		if (box_faces(&old_position, &new_position, p, intercept, normal))
		{
			return TRUE;
		}
	}

	return FALSE;
}

/* accurate_moving_bounding_box_collision_test: the model at the sweep's old, new and middle positions, as many of them as the candidate's speed asks for. */
static int moving(const float *pose, const float *bounds, eech_entity_triangle_fn triangles, void *context, vec3d *intercept, vec3d *normal)
{
	vec3d p[8], q[8], old_position, new_position, target_old_position, target_new_position, target_mid_position;
	matrix3x3 attitude;
	int depth, test_flags, found;
	float true_velocity, body_length;

	old_position.x = pose[0];
	old_position.y = pose[1];
	old_position.z = pose[2];
	new_position.x = pose[3];
	new_position.y = pose[4];
	new_position.z = pose[5];
	target_old_position.x = pose[12];
	target_old_position.y = pose[13];
	target_old_position.z = pose[14];
	target_new_position.x = pose[15];
	target_new_position.y = pose[16];
	target_new_position.z = pose[17];
	memcpy(attitude, &pose[18], 9 * sizeof(float));
	true_velocity = pose[27];

	/* determine which tests are required using target velocity and body length */
	body_length = bounds[5] - bounds[4];

	if (true_velocity > body_length)
	{
		test_flags = TEST_OLD_POSITION + TEST_MID_POSITION + TEST_NEW_POSITION;
	}
	else if (true_velocity > (body_length * 0.25))
	{
		test_flags = TEST_OLD_POSITION + TEST_NEW_POSITION;
	}
	else
	{
		test_flags = TEST_MID_POSITION;
	}

	depth = 0;
	found = triangles(context, depth++, (float *) p);
	if (found < 0)
	{
		return found;
	}

	if (found)
	{
		do
		{
			multiply_matrix3x3_vec3d(&p[0], attitude, &p[0]);
			multiply_matrix3x3_vec3d(&p[1], attitude, &p[1]);
			multiply_matrix3x3_vec3d(&p[2], attitude, &p[2]);

			if (test_flags & TEST_OLD_POSITION)
			{
				translate(q, p, 3, &target_old_position);

				if (get_3d_line_triangle_face_intersection(&old_position, &new_position, &q[0], &q[1], &q[2], intercept, normal))
				{
					return TRUE;
				}
			}

			if (test_flags & TEST_NEW_POSITION)
			{
				translate(q, p, 3, &target_new_position);

				if (get_3d_line_triangle_face_intersection(&old_position, &new_position, &q[0], &q[1], &q[2], intercept, normal))
				{
					return TRUE;
				}
			}

			if (test_flags & TEST_MID_POSITION)
			{
				target_mid_position.x = (target_old_position.x + target_new_position.x) * 0.5;
				target_mid_position.y = (target_old_position.y + target_new_position.y) * 0.5;
				target_mid_position.z = (target_old_position.z + target_new_position.z) * 0.5;

				translate(q, p, 3, &target_mid_position);

				if (get_3d_line_triangle_face_intersection(&old_position, &new_position, &q[0], &q[1], &q[2], intercept, normal))
				{
					return TRUE;
				}
			}

			found = triangles(context, depth++, (float *) p);
			if (found < 0)
			{
				return found;
			}
		}
		while (found);
	}
	else
	{
		box_corners(p, bounds, (const float (*)[3]) attitude);

		if (test_flags & TEST_OLD_POSITION)
		{
			translate(q, p, 8, &target_old_position);

			if (box_faces(&old_position, &new_position, q, intercept, normal))
			{
				return TRUE;
			}
		}

		if (test_flags & TEST_NEW_POSITION)
		{
			translate(q, p, 8, &target_new_position);

			if (box_faces(&old_position, &new_position, q, intercept, normal))
			{
				return TRUE;
			}
		}

		if (test_flags & TEST_MID_POSITION)
		{
			target_mid_position.x = (target_old_position.x + target_new_position.x) * 0.5;
			target_mid_position.y = (target_old_position.y + target_new_position.y) * 0.5;
			target_mid_position.z = (target_old_position.z + target_new_position.z) * 0.5;

			translate(q, p, 8, &target_mid_position);

			if (box_faces(&old_position, &new_position, q, intercept, normal))
			{
				return TRUE;
			}
		}
	}

	return FALSE;
}

/* The entry: `pose` as described at the top of the file, `bounds` the scene's bounding box without lines. The final intercept point and normal (six floats) are written to
   `result` whether or not there was a hit. Returns 1 for a hit, 0 for a miss, negative when the triangle cursor refused. */
int eech_entity_collision(const float *pose, int is_moving, const float *bounds, eech_entity_triangle_fn triangles, void *context, float *result)
{
	vec3d intercept, normal;
	int hit;

	intercept.x = pose[6];
	intercept.y = pose[7];
	intercept.z = pose[8];
	normal.x = pose[9];
	normal.y = pose[10];
	normal.z = pose[11];

	if (is_moving)
	{
		hit = moving(pose, bounds, triangles, context, &intercept, &normal);
	}
	else
	{
		hit = stationary(pose, bounds, triangles, context, &intercept, &normal);
	}

	result[0] = intercept.x;
	result[1] = intercept.y;
	result[2] = intercept.z;
	result[3] = normal.x;
	result[4] = normal.y;
	result[5] = normal.z;

	return hit;
}

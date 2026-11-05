/*
 * ENTITY_CANDIDATE: what EECH's own get_line_of_sight_collision_entity (aphavoc/source/entity/en_misc/en_maths/en_cllsn.c) decides about ONE candidate before the narrow phase
 * (EEDCS-49, D3, third slice). The narrow phase itself is entity_step.c.
 *
 * The original's per-candidate decisions, expression for expression, over the FACTS the campaign reads from the entity it is visiting: which branch (the mobile flag first, then,
 * outside a sector whose tallest structure is below the segment, the fixed flag), whether the candidate is moving (`|motion|^2 > 1`), the sweep ends a moving candidate has (from
 * the `UPDATED` answer and the frame time), the broad-phase box of radius + 10 m (the radius is the host's own model data), the city's height-then-xz box, and the live pose of a
 * stationary fixed candidate (its heading, or the terrain-aligned placement from the face normal the campaign asks the terrain for once the broad phase has passed). The campaign keeps
 * WHICH candidates are visited and in what order (the sector lists), the two services that have effects of their own (the destructive `UPDATED` getter and the terrain face-normal
 * question, each performed exactly when this function names it, never earlier) and the consequences of a hit.
 *
 * It is a pure function of its facts: there is no state between calls and the environment is not touched (the caller, src/entity_host.rs, runs it under round-toward-zero with the
 * caller's complete floating-point state saved and restored, as the narrow phase and the tree kernel are; this file is NOT compiled with -frounding-math). A candidate that needs
 * something answers NEED_*, the campaign performs the service or reads the facts and asks again with them filled in, and every earlier fact is identical, so the decision is the one the
 * original made in a single pass.
 *
 * THE FACTS ARE STAGED, in the order the original needed them. The campaign first sends only what the original read for every candidate: the segment, the mobile flag, the sector's tallest
 * structure height and, for a mobile candidate, its shape, position and motion vector (a building listed under a passed city sends its shape and position). A candidate that is not mobile in
 * a sector below the segment is skipped HERE, before the original read its fixed flag or any shape. Otherwise the host asks for the fixed facts (NEED_FIXED: the fixed flag, whether it is
 * a city or a city building and the shape, position, city box the branch reads) and, once the broad phase has passed, for the placement facts (NEED_PLACEMENT: the attitude of a mobile
 * candidate, the heading and align flag of a fixed one). The radius of the candidate's scene is the host's own model data and is asked for (NEED_RADIUS, which never leaves the host) only by
 * a branch that has reached its broad phase: a skipped candidate needs no model.
 *
 * FACTS (floats; every flag is exactly 0 or 1):
 *   [0..2] segment old end, [3..5] segment new end,
 *   [6] collision-test-mobile flag, [7] collision-test-fixed flag (meaningful once [42] says it was read), [8] the entity is a city, [9] the entity is a city building,
 *   [10] the candidate is a building listed under a city that has passed its box (not listed in the sector), [11] the city has a first building,
 *   [12] the sector's tallest structure height, [13..15] position, [16..18] motion vector, [19] frame time,
 *   [20] the UPDATED answer (0 not asked, 1 false, 2 true), [21] heading, [22] align-with-terrain flag, [23] face normal given, [24..26] terrain face normal,
 *   [27..35] attitude (row-major; meaningful for a mobile candidate), [36..38] city bounding box min, [39..41] max,
 *   [42] the fixed facts were read, [43] the placement facts were read.
 * RESULT: a CANDIDATE_* state and, for TEST, the candidate's pose for the narrow phase: 12 floats (position, attitude) or 16 (old position, new position, attitude, speed).
 */
#include <math.h>
#include <stddef.h>

#include "eech_arith.h"

#define CANDIDATE_SKIP (1)
#define CANDIDATE_NEED_UPDATED (2)
#define CANDIDATE_NEED_NORMAL (3)
#define CANDIDATE_BUILDINGS (4)
#define CANDIDATE_TEST (5)
#define CANDIDATE_NEED_FIXED (6)
#define CANDIDATE_NEED_PLACEMENT (7)

/* the branch has reached its broad phase and the radius of the candidate's scene is not known yet: asked of the host's own model data by src/entity_host.rs, never sent to the campaign */
#define CANDIDATE_NEED_RADIUS (8)

/* a candidate that is neither mobile nor fixed reached the pose derivation: the original's debug_fatal ("Invalid target") */
#define CANDIDATE_INVALID_TARGET (-1)

/* the box test of every branch: the six comparisons, in the original's order and form */
static int box_meets_segment(const vec3d *target_min_position, const vec3d *target_max_position, const vec3d *source_min_position, const vec3d *source_max_position)
{
	return (
		(!(target_max_position->x < source_min_position->x)) &&
		(!(target_min_position->x > source_max_position->x)) &&
		(!(target_max_position->y < source_min_position->y)) &&
		(!(target_min_position->y > source_max_position->y)) &&
		(!(target_max_position->z < source_min_position->z)) &&
		(!(target_min_position->z > source_max_position->z)));
}

/* maths/vector.c and maths/matrix.c: the same text as convoy_step.c (a test compares the copies) */
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

/* accurate_stationary_bounding_box_collision_test, the part before the narrow phase: the candidate's position and attitude */
static int stationary_candidate(const float *facts, int mobile, int fixed, float *candidate, int *candidate_count)
{
	vec3d position, terrain_face_normal;
	matrix3x3 attitude;
	float heading;

	/* the live placement is read once the broad phase has passed: the attitude of a mobile candidate, the heading and align flag of a fixed one */
	if (facts[43] == 0.0f)
	{
		return CANDIDATE_NEED_PLACEMENT;
	}

	position.x = facts[13];
	position.y = facts[14];
	position.z = facts[15];

	if (mobile)
	{
		memcpy(attitude, &facts[27], 9 * sizeof(float));
	}
	else if (fixed)
	{
		heading = facts[21];

		if (facts[22] != 0.0f)
		{
			if (facts[23] == 0.0f)
			{
				return CANDIDATE_NEED_NORMAL;
			}

			terrain_face_normal.x = facts[24];
			terrain_face_normal.y = facts[25];
			terrain_face_normal.z = facts[26];

			get_3d_transformation_matrix_from_face_normal_and_heading(attitude, &terrain_face_normal, heading);
		}
		else
		{
			get_3d_transformation_matrix(attitude, heading, 0.0, 0.0);
		}
	}
	else
	{
		return CANDIDATE_INVALID_TARGET;
	}

	candidate[0] = position.x;
	candidate[1] = position.y;
	candidate[2] = position.z;
	memcpy(&candidate[3], attitude, 9 * sizeof(float));
	*candidate_count = 12;

	return CANDIDATE_TEST;
}

/* The entry: `facts` as described at the top of the file; `has_radius` says whether `radius`, the candidate scene's radius (the host's model data), is known: a branch that needs it
   while it is not answers CANDIDATE_NEED_RADIUS and is asked again with it; `candidate` has room for 16 floats. Returns a CANDIDATE_* state or CANDIDATE_INVALID_TARGET. Nothing of the
   caller's is read other than `facts`. */
int eech_entity_candidate(const float *facts, int has_radius, float radius, float *candidate, int *candidate_count)
{
	vec3d source_old_position, source_new_position, source_min_position, source_max_position, target_position, target_motion_vector, target_old_position, target_new_position,
		target_min_position, target_max_position;
	float sqr_velocity;
	int mobile, fixed;

	*candidate_count = 0;

	source_old_position.x = facts[0];
	source_old_position.y = facts[1];
	source_old_position.z = facts[2];
	source_new_position.x = facts[3];
	source_new_position.y = facts[4];
	source_new_position.z = facts[5];

	mobile = (facts[6] != 0.0f);
	fixed = (facts[7] != 0.0f);

	target_position.x = facts[13];
	target_position.y = facts[14];
	target_position.z = facts[15];

	if (source_old_position.x <= source_new_position.x)
	{
		source_min_position.x = source_old_position.x;
		source_max_position.x = source_new_position.x;
	}
	else
	{
		source_min_position.x = source_new_position.x;
		source_max_position.x = source_old_position.x;
	}

	if (source_old_position.y <= source_new_position.y)
	{
		source_min_position.y = source_old_position.y;
		source_max_position.y = source_new_position.y;
	}
	else
	{
		source_min_position.y = source_new_position.y;
		source_max_position.y = source_old_position.y;
	}

	if (source_old_position.z <= source_new_position.z)
	{
		source_min_position.z = source_old_position.z;
		source_max_position.z = source_new_position.z;
	}
	else
	{
		source_min_position.z = source_new_position.z;
		source_max_position.z = source_old_position.z;
	}

	if (facts[10] != 0.0f)
	{
		/* a building listed under a city whose box the segment meets: stationary, and the sector's tallest-structure skip was already applied to its city */
		if (!has_radius)
		{
			return CANDIDATE_NEED_RADIUS;
		}

		target_min_position.x = target_position.x - radius - 10;
		target_min_position.y = target_position.y - radius - 10;
		target_min_position.z = target_position.z - radius - 10;

		target_max_position.x = target_position.x + radius + 10;
		target_max_position.y = target_position.y + radius + 10;
		target_max_position.z = target_position.z + radius + 10;

		if (box_meets_segment(&target_min_position, &target_max_position, &source_min_position, &source_max_position))
		{
			return stationary_candidate(facts, mobile, fixed, candidate, candidate_count);
		}

		return CANDIDATE_SKIP;
	}

	if (mobile)
	{
		/* MOBILE TARGET */
		target_motion_vector.x = facts[16];
		target_motion_vector.y = facts[17];
		target_motion_vector.z = facts[18];

		sqr_velocity = (target_motion_vector.x * target_motion_vector.x + target_motion_vector.y * target_motion_vector.y + target_motion_vector.z * target_motion_vector.z);

		if (sqr_velocity > 1.0)
		{
			/* moving mobile */
			if (facts[20] == 0.0f)
			{
				return CANDIDATE_NEED_UPDATED;
			}

			if (facts[20] == 2.0f)
			{
				target_new_position = target_position;

				target_old_position.x = target_new_position.x - target_motion_vector.x * facts[19];
				target_old_position.y = target_new_position.y - target_motion_vector.y * facts[19];
				target_old_position.z = target_new_position.z - target_motion_vector.z * facts[19];
			}
			else
			{
				target_old_position = target_position;

				target_new_position.x = target_old_position.x + target_motion_vector.x * facts[19];
				target_new_position.y = target_old_position.y + target_motion_vector.y * facts[19];
				target_new_position.z = target_old_position.z + target_motion_vector.z * facts[19];
			}

			if (target_old_position.x <= target_new_position.x)
			{
				target_min_position.x = target_old_position.x;
				target_max_position.x = target_new_position.x;
			}
			else
			{
				target_min_position.x = target_new_position.x;
				target_max_position.x = target_old_position.x;
			}

			if (target_old_position.y <= target_new_position.y)
			{
				target_min_position.y = target_old_position.y;
				target_max_position.y = target_new_position.y;
			}
			else
			{
				target_min_position.y = target_new_position.y;
				target_max_position.y = target_old_position.y;
			}

			if (target_old_position.z <= target_new_position.z)
			{
				target_min_position.z = target_old_position.z;
				target_max_position.z = target_new_position.z;
			}
			else
			{
				target_min_position.z = target_new_position.z;
				target_max_position.z = target_old_position.z;
			}

			if (!has_radius)
			{
				return CANDIDATE_NEED_RADIUS;
			}

			target_min_position.x -= radius + 10;
			target_min_position.y -= radius + 10;
			target_min_position.z -= radius + 10;

			target_max_position.x += radius + 10;
			target_max_position.y += radius + 10;
			target_max_position.z += radius + 10;

			if (box_meets_segment(&target_min_position, &target_max_position, &source_min_position, &source_max_position))
			{
				/* accurate_moving_bounding_box_collision_test: the sweep ends, the attitude and the speed */
				if (facts[43] == 0.0f)
				{
					return CANDIDATE_NEED_PLACEMENT;
				}

				candidate[0] = target_old_position.x;
				candidate[1] = target_old_position.y;
				candidate[2] = target_old_position.z;

				candidate[3] = target_new_position.x;
				candidate[4] = target_new_position.y;
				candidate[5] = target_new_position.z;

				memcpy(&candidate[6], &facts[27], 9 * sizeof(float));

				candidate[15] = get_3d_vector_magnitude(&target_motion_vector);
				*candidate_count = 16;

				return CANDIDATE_TEST;
			}

			return CANDIDATE_SKIP;
		}

		/* stationary mobile */
		if (!has_radius)
		{
			return CANDIDATE_NEED_RADIUS;
		}

		target_min_position.x = target_position.x - radius - 10;
		target_min_position.y = target_position.y - radius - 10;
		target_min_position.z = target_position.z - radius - 10;

		target_max_position.x = target_position.x + radius + 10;
		target_max_position.y = target_position.y + radius + 10;
		target_max_position.z = target_position.z + radius + 10;

		if (box_meets_segment(&target_min_position, &target_max_position, &source_min_position, &source_max_position))
		{
			return stationary_candidate(facts, mobile, fixed, candidate, candidate_count);
		}

		return CANDIDATE_SKIP;
	}

	/* everything that is not mobile is skipped in a sector whose tallest structure is below the segment */
	if (source_min_position.y > facts[12])
	{
		return CANDIDATE_SKIP;
	}

	/* the fixed flag is read only now: the original did not read it in a sector it skipped */
	if (facts[42] == 0.0f)
	{
		return CANDIDATE_NEED_FIXED;
	}

	if (!fixed)
	{
		return CANDIDATE_SKIP;
	}

	/* FIXED TARGET */
	if (facts[8] != 0.0f)
	{
		/* a city: its box is tested, height first, and only then its buildings one by one (in the campaign's list order) */
		if (facts[11] == 0.0f)
		{
			return CANDIDATE_SKIP;
		}

		target_min_position.x = facts[36];
		target_min_position.y = facts[37];
		target_min_position.z = facts[38];
		target_max_position.x = facts[39];
		target_max_position.y = facts[40];
		target_max_position.z = facts[41];

		target_min_position.y += target_position.y;
		target_max_position.y += target_position.y;

		if ((!(target_max_position.y < source_min_position.y)) && (!(target_min_position.y > source_max_position.y)))
		{
			target_min_position.x += target_position.x;
			target_min_position.z += target_position.z;

			target_max_position.x += target_position.x;
			target_max_position.z += target_position.z;

			if ((!(target_max_position.x < source_min_position.x)) && (!(target_min_position.x > source_max_position.x)) && (!(target_max_position.z < source_min_position.z)) &&
				(!(target_min_position.z > source_max_position.z)))
			{
				return CANDIDATE_BUILDINGS;
			}
		}

		return CANDIDATE_SKIP;
	}

	if (facts[9] != 0.0f)
	{
		/* a building listed directly in a sector is skipped */
		return CANDIDATE_SKIP;
	}

	if (!has_radius)
	{
		return CANDIDATE_NEED_RADIUS;
	}

	target_min_position.x = target_position.x - radius - 10;
	target_min_position.y = target_position.y - radius - 10;
	target_min_position.z = target_position.z - radius - 10;

	target_max_position.x = target_position.x + radius + 10;
	target_max_position.y = target_position.y + radius + 10;
	target_max_position.z = target_position.z + radius + 10;

	if (box_meets_segment(&target_min_position, &target_max_position, &source_min_position, &source_max_position))
	{
		return stationary_candidate(facts, mobile, fixed, candidate, candidate_count);
	}

	return CANDIDATE_SKIP;
}

"""Narrowing at the original's float operations in csrc/entity_candidate_step.c (ENV-I686-ENTITY-CANDIDATE-NARROW-ULP-1). Byte-exact replacements, each asserted to match once."""
from pathlib import Path

p = Path("R:/d3-evidence/collision-characterization-20261029/eech-campaign/crates/eech-world/csrc/entity_candidate_step.c")
d = p.read_bytes()
assert b"\r\n" not in d
T = b"\t"


def rep(old, new, count=1):
    global d
    assert d.count(old) == count, (d.count(old), old)
    d = d.replace(old, new)


# ---- the macro and the two local helpers ------------------------------------------------------------------------------------------------
rep(
    b'#include "eech_arith.h"\n\n#define CANDIDATE_SKIP (1)',
    b'''#include "eech_arith.h"

/*
 * NARROWING AT THE ORIGINAL'S FLOAT OPERATIONS (ENV-I686-ENTITY-CANDIDATE-NARROW-ULP-1). The original's arithmetic is single-precision: on the Windows x64 target (SSE, __FLT_EVAL_METHOD__ 0) every
 * float operation, and every assignment to a float, rounds to float, and the next operation computes from the rounded value. On i686 (x87, __FLT_EVAL_METHOD__ 2, and `-std=gnu99` implies
 * -fexcess-precision=fast) the compiler keeps whole expressions and, once a static helper is inlined, the variables between statements in extended precision. MEASURED (docs/d3-entity-collision-provider.md):
 * the same original text compiled for i686 is not even one answer, it changes with the compilation context (the original text and this file at -O1 differ on 1657 of 2000 placements), while with
 * SSE float math on i686 it equals the x64 words on all 2000 under round-toward-zero.
 *
 * NARROW (EXPRESSION) is that rounding made explicit: the expression, unchanged, is stored into an addressable volatile float and read back. It is used at each float operation of the placement, the
 * magnitude, the speed test, the sweep ends and the broad-phase boxes; where the target already evaluates in float (x86-64, SSE) it is the expression itself, so those builds are untouched. No flag,
 * global or tolerance is involved, and the shared helpers of eech_arith.h (used by the other steps) are not changed: the two this file needs narrowed have local copies below.
 */
#if defined(__FLT_EVAL_METHOD__) && __FLT_EVAL_METHOD__ != 0
static float candidate_narrow(float value)
{
	volatile float stored = value;

	return stored;
}

#define NARROW(EXPRESSION) candidate_narrow(EXPRESSION)
#else
#define NARROW(EXPRESSION) (EXPRESSION)
#endif

#define CANDIDATE_SKIP (1)''',
)

# ---- cross product, face-normal placement, local normalise and magnitude --------------------------------------------------------------------
rep(
    b'''	result->x = (v1->y * v2->z) - (v2->y * v1->z);
	result->y = (v1->z * v2->x) - (v2->z * v1->x);
	result->z = (v1->x * v2->y) - (v2->x * v1->y);
}
''',
    b'''	result->x = NARROW(NARROW(v1->y * v2->z) - NARROW(v2->y * v1->z));
	result->y = NARROW(NARROW(v1->z * v2->x) - NARROW(v2->z * v1->x));
	result->z = NARROW(NARROW(v1->x * v2->y) - NARROW(v2->x * v1->y));
}

/* maths/vector.c normalise_any_3d_vector, the same statements with each float operation narrowed (eech_arith.h's copy is shared and is not changed) */
static float candidate_normalise_any_3d_vector(vec3d *vector)
{
	float x, y, z, length;

	x = NARROW(vector->x * vector->x);
	y = NARROW(vector->y * vector->y);
	z = NARROW(vector->z * vector->z);

	length = NARROW(sqrt(NARROW(NARROW(x + y) + z)));

	if (length > 0)
	{
		float one_over_length;

		one_over_length = NARROW(1.0 / length);

		vector->x = NARROW(vector->x * one_over_length);
		vector->y = NARROW(vector->y * one_over_length);
		vector->z = NARROW(vector->z * one_over_length);
	}
	else
	{
		vector->x = 0;
		vector->y = 0;
		vector->z = 0;
	}

	return (length);
}

/* maths/vector.c get_3d_vector_magnitude, the same statements with each float operation narrowed */
static float candidate_get_3d_vector_magnitude(const vec3d *vector)
{
	float x, y, z;

	x = NARROW(vector->x * vector->x);
	y = NARROW(vector->y * vector->y);
	z = NARROW(vector->z * vector->z);

	return (NARROW(sqrt(NARROW(NARROW(x + y) + z))));
}
''',
)
rep(
    b'''		result[2][0] = sin(heading);
		result[2][2] = cos(heading);
		result[2][1] = -((normal->x * result[2][0]) + (normal->z * result[2][2])) / normal->y;

		normalise_any_3d_vector((vec3d *)&result[2][0]);
''',
    b'''		result[2][0] = NARROW(sin(heading));
		result[2][2] = NARROW(cos(heading));
		result[2][1] = NARROW(-NARROW(NARROW(normal->x * result[2][0]) + NARROW(normal->z * result[2][2])) / normal->y);

		candidate_normalise_any_3d_vector((vec3d *)&result[2][0]);
''',
)

# ---- the broad-phase boxes ------------------------------------------------------------------------------------------------------------------
for axis in (b"x", b"y", b"z"):
    rep(
        b"target_min_position." + axis + b" = target_position." + axis + b" - radius - 10;\n",
        b"target_min_position." + axis + b" = NARROW(NARROW(target_position." + axis + b" - radius) - 10);\n",
        3,
    )
    rep(
        b"target_max_position." + axis + b" = target_position." + axis + b" + radius + 10;\n",
        b"target_max_position." + axis + b" = NARROW(NARROW(target_position." + axis + b" + radius) + 10);\n",
        3,
    )
    rep(
        b"target_min_position." + axis + b" -= radius + 10;\n",
        b"target_min_position." + axis + b" = NARROW(target_min_position." + axis + b" - NARROW(radius + 10));\n",
    )
    rep(
        b"target_max_position." + axis + b" += radius + 10;\n",
        b"target_max_position." + axis + b" = NARROW(target_max_position." + axis + b" + NARROW(radius + 10));\n",
    )
    rep(
        b"target_old_position." + axis + b" = target_new_position." + axis + b" - target_motion_vector." + axis + b" * facts[19];\n",
        b"target_old_position." + axis + b" = NARROW(target_new_position." + axis + b" - NARROW(target_motion_vector." + axis + b" * facts[19]));\n",
    )
    rep(
        b"target_new_position." + axis + b" = target_old_position." + axis + b" + target_motion_vector." + axis + b" * facts[19];\n",
        b"target_new_position." + axis + b" = NARROW(target_old_position." + axis + b" + NARROW(target_motion_vector." + axis + b" * facts[19]));\n",
    )

# ---- the speed test and the magnitude ------------------------------------------------------------------------------------------------------
rep(
    b"sqr_velocity = (target_motion_vector.x * target_motion_vector.x + target_motion_vector.y * target_motion_vector.y + target_motion_vector.z * target_motion_vector.z);",
    b"sqr_velocity = NARROW(NARROW(NARROW(target_motion_vector.x * target_motion_vector.x) + NARROW(target_motion_vector.y * target_motion_vector.y)) +\n\t\t\tNARROW(target_motion_vector.z * target_motion_vector.z));",
)
rep(b"candidate[15] = get_3d_vector_magnitude(&target_motion_vector);", b"candidate[15] = candidate_get_3d_vector_magnitude(&target_motion_vector);")

p.write_bytes(d)
print("patched", len(d))

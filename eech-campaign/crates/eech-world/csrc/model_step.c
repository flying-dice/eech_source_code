/*
 * The host's private numeric kernel of the collision-model service (EEDCS-50, D3). Everything here is EECH's own arithmetic, kept expression for expression from
 * the original, so that the host and the original produce the same bits:
 *
 *   eech_model_triangle       the float part of 3dobjdb.c get_object_3d_collision_object_geometry_triangle: the object's bounding box scales the three
 *                             short points of one triangle (max (fabs (min), fabs (max)) / 32767.0, in float, then (float) short * scale)
 *   eech_model_scene_bounds   3dsubobj.c initialise_scene_bounding_sub_objects, recurse_initialise_sub_scene_bounding_box and rotate_sub_scene_boundaries:
 *                             a scene's bounding_box, bounding_box2 and radius from its main object and its sub-object tree (keyframe 0 of each)
 *
 * What is NOT here: any file, any table, any walk of faces and surfaces (the decoder's integer stepping is the host's Rust, over bounds-checked arrays), any
 * pointer from the legacy layout. The caller hands this file plain numbers and receives plain numbers.
 *
 * The floating-point environment: the original runs under round-toward-zero (the engine entry saves the caller's environment and sets it; the 64-bit
 * build keeps the x87 control word and MXCSR in step). The host is called from Lua outside the engine, in whatever state its caller left. Each public entry
 * therefore (1) reads and REQUIRES the x87 and SSE exception masks to be 63 and 63 before any arithmetic (it never masks or unmasks an exception), (2) saves
 * the caller's COMPLETE state (the x87 environment with fegetenv and the whole of MXCSR, control and status bits, with stmxcsr, because fesetenv alone does
 * not give the SSE word back on the Windows toolchain), (3) sets round-toward-zero for both units and reads it back, (4) runs the original's arithmetic in
 * a body that is never inlined into the entry, and (5) restores the caller's state (fesetenv, ldmxcsr, and a read-back that must equal what was saved) on
 * EVERY exit: an answer, a refusal, a bad argument. Nothing of the guard is held between calls. A restore that does not read back as saved is an environment
 * failure and the answer is withheld. The file is deliberately NOT compiled with -frounding-math: the original's compile-time constant folding happened at the
 * default rounding. Whether this reproduces the original is established by the comparison with the retained original rows, not by this comment.
 *
 * STATUS: PREPARATION (host-private component; the campaign DLL's queries still run the original code).
 */
#include <fenv.h>
#include <stddef.h>
#include <stdint.h>

#include "eech_arith.h"

#define MODEL_OK 0
#define MODEL_MASKS 1
#define MODEL_ENVIRONMENT 2
#define MODEL_BAD_ARGUMENT 3

#define MODEL_MAX_DEPTH 64

typedef struct eech_model_node
{
	int32_t object;      /* index into the bounds table (0 is the null object) */
	int32_t collision;   /* collision_contribution */
	int32_t first_child; /* the children are consecutive elements of the same array */
	int32_t child_count;
	float keyframe[9];   /* keyframe 0: x y z heading pitch roll scale_x scale_y scale_z */
} eech_model_node;

size_t eech_model_node_size(void)
{
	return sizeof(eech_model_node);
}

/* ------------------------------------------------------------------------------------------------------------------------------------ */
/* The environment boundary of the public entries. */
/* ------------------------------------------------------------------------------------------------------------------------------------ */

typedef struct eech_model_caller_environment
{
	fenv_t x87;
	uint32_t mxcsr;
} eech_model_caller_environment;

#define MXCSR_ROUNDING_MASK 0x6000u
#define MXCSR_ROUND_TOWARD_ZERO 0x6000u

#if defined(__x86_64__) || defined(__i386__)
static uint32_t read_mxcsr(void)
{
	uint32_t mxcsr = 0;

	__asm__ __volatile__("stmxcsr %0" : "=m"(mxcsr));

	return (mxcsr);
}

static void write_mxcsr(uint32_t mxcsr)
{
	__asm__ __volatile__("ldmxcsr %0" : : "m"(mxcsr));
}
#else
static uint32_t read_mxcsr(void)
{
	return (0);
}

static void write_mxcsr(uint32_t mxcsr)
{
	(void) mxcsr;
}
#endif

/*
 * The floating-point state of this thread, read and never written: the low six bits of the x87 control word and bits 7 to 12 of MXCSR are the exception masks
 * (63 when every exception is masked); the SSE rounding control is bits 13 and 14 of MXCSR (0 nearest, 1 down, 2 up, 3 toward zero). The raw x87 control word and
 * the raw MXCSR (control AND status bits) are reported too, for exact comparison. Returns 0, or -1 where the registers cannot be read.
 */
int eech_model_fp_state(int *x87_masks, int *sse_masks, int *sse_rounding, uint32_t *x87_control, uint32_t *mxcsr_bits)
{
#if defined(__x86_64__) || defined(__i386__)
	uint16_t control_word = 0;
	uint32_t mxcsr = read_mxcsr();

	__asm__ __volatile__("fnstcw %0" : "=m"(control_word));
	*x87_masks = (int) (control_word & 0x3fu);
	*sse_masks = (int) ((mxcsr >> 7) & 0x3fu);
	*sse_rounding = (int) ((mxcsr >> 13) & 0x3u);
	*x87_control = (uint32_t) control_word;
	*mxcsr_bits = mxcsr;
	return 0;
#else
	*x87_masks = 0;
	*sse_masks = 0;
	*sse_rounding = 0;
	*x87_control = 0;
	*mxcsr_bits = 0;
	return -1;
#endif
}

static int save_caller_environment(eech_model_caller_environment *saved)
{
#if defined(__x86_64__) || defined(__i386__)
	if (fegetenv(&saved->x87) != 0)
	{
		return (-1);
	}

	saved->mxcsr = read_mxcsr();

	return (0);
#else
	(void) saved;
	return (-1);
#endif
}

static int restore_caller_environment(const eech_model_caller_environment *saved)
{
#if defined(__x86_64__) || defined(__i386__)
	int failed = 0;

	if (fesetenv(&saved->x87) != 0)
	{
		failed = 1;
	}

	write_mxcsr(saved->mxcsr);

	if (read_mxcsr() != saved->mxcsr)
	{
		failed = 1;
	}

	return (failed ? -1 : 0);
#else
	(void) saved;
	return (-1);
#endif
}

static int set_round_toward_zero(void)
{
	if (fesetround(FE_TOWARDZERO) != 0)
	{
		return (-1);
	}

	write_mxcsr((read_mxcsr() & ~MXCSR_ROUNDING_MASK) | MXCSR_ROUND_TOWARD_ZERO);

	if (fegetround() != FE_TOWARDZERO || (read_mxcsr() & MXCSR_ROUNDING_MASK) != MXCSR_ROUND_TOWARD_ZERO)
	{
		return (-1);
	}

	return (0);
}

/* Requires the original's exception masks, saves the caller's complete state and sets round-toward-zero. On a non-OK result the caller's state is as the caller left it. */
static int enter_environment(eech_model_caller_environment *caller)
{
	int x87_masks = 0, sse_masks = 0, sse_rounding = 0;
	uint32_t x87_control = 0, mxcsr = 0;

	if (eech_model_fp_state(&x87_masks, &sse_masks, &sse_rounding, &x87_control, &mxcsr) != 0 || x87_masks != 63 || sse_masks != 63)
	{
		return (MODEL_MASKS);
	}

	if (save_caller_environment(caller) != 0)
	{
		return (MODEL_ENVIRONMENT);
	}

	if (set_round_toward_zero() != 0)
	{
		restore_caller_environment(caller);

		return (MODEL_ENVIRONMENT);
	}

	return (MODEL_OK);
}

/* Restores the caller's complete state: `result` when that works, MODEL_ENVIRONMENT (and the answer withheld by the caller) when it does not. */
static int leave_environment(const eech_model_caller_environment *caller, int result)
{
	if (restore_caller_environment(caller) != 0)
	{
		return (MODEL_ENVIRONMENT);
	}

	return (result);
}

/* ------------------------------------------------------------------------------------------------------------------------------------ */
/* get_object_3d_collision_object_geometry_triangle: the float part */
/* ------------------------------------------------------------------------------------------------------------------------------------ */

/*
 * 3dobjdb.c, expression for expression:
 *
 *	xmax = max ( fabs ( obj->bounding_box.xmin ), fabs ( obj->bounding_box.xmax ) );   (float xmax; fabs is the double function)
 *	xmax /= 32767.0;                                                                   (a double division, converted back to float)
 *	return_points[n].x = ( float ) obj->points[ ... ].x * xmax;                        (float * float)
 *
 * `points` is nine shorts: the three points' x, y, z in the order the original writes them. `bounds` is the OBJECT's bounding box (xmin, xmax, ymin, ymax,
 * zmin, zmax), not the scene's.
 */
static void __attribute__((noinline)) triangle_body(const int16_t *points, const float *bounds, float *out)
{
	float xmax, ymax, zmax;
	int n;

	xmax = max(fabs(bounds[0]), fabs(bounds[1]));
	ymax = max(fabs(bounds[2]), fabs(bounds[3]));
	zmax = max(fabs(bounds[4]), fabs(bounds[5]));

	xmax /= 32767.0;
	ymax /= 32767.0;
	zmax /= 32767.0;

	for (n = 0; n < 3; n++)
	{
		out[n * 3 + 0] = (float) points[n * 3 + 0] * xmax;
		out[n * 3 + 1] = (float) points[n * 3 + 1] * ymax;
		out[n * 3 + 2] = (float) points[n * 3 + 2] * zmax;
	}
}

int eech_model_triangle(const int16_t *points, const float *bounds, float *out)
{
	eech_model_caller_environment caller;
	int result;

	if (!points || !bounds || !out)
	{
		return (MODEL_BAD_ARGUMENT);
	}

	result = enter_environment(&caller);

	if (result != MODEL_OK)
	{
		return (result);
	}

	triangle_body(points, bounds, out);

	return (leave_environment(&caller, MODEL_OK));
}

/* ------------------------------------------------------------------------------------------------------------------------------------ */
/* 3dsubobj.c: a scene's bounds and radius */
/* ------------------------------------------------------------------------------------------------------------------------------------ */

typedef struct vec3
{
	float x, y, z;
} vec3;

typedef struct bounds
{
	float xmin, xmax, ymin, ymax, zmin, zmax;
} bounds;

typedef struct bounds_context
{
	const float *object_bounds; /* 12 floats per object: bounding_box then bounding_box2 */
	int32_t object_count;
	const eech_model_node *nodes;
	int32_t node_count;
	vec3 scene_position;
	matrix3x3 scene_rotation;
	bounds scene_bounds, scene_bounds2, rotated_bounds, rotated_bounds2;
	int contributes_to_collision;
	int error;
} bounds_context;

static void rotate_sub_scene_boundaries(bounds_context *context, bounds *results, const bounds *source)
{
	vec3 cube_points[8];
	int count;

	if ((source->xmin == 0.0) && (source->xmax == 0.0) && (source->ymin == 0.0) && (source->ymax == 0.0) && (source->zmin == 0.0) && (source->zmax == 0.0))
	{
		results->xmin = 0.0;
		results->xmax = 0.0;
		results->ymin = 0.0;
		results->ymax = 0.0;
		results->zmin = 0.0;
		results->zmax = 0.0;
	}
	else
	{
		cube_points[0].x = source->xmin; cube_points[0].y = source->ymin; cube_points[0].z = source->zmin;
		cube_points[1].x = source->xmin; cube_points[1].y = source->ymin; cube_points[1].z = source->zmax;
		cube_points[2].x = source->xmin; cube_points[2].y = source->ymax; cube_points[2].z = source->zmin;
		cube_points[3].x = source->xmin; cube_points[3].y = source->ymax; cube_points[3].z = source->zmax;
		cube_points[4].x = source->xmax; cube_points[4].y = source->ymin; cube_points[4].z = source->zmin;
		cube_points[5].x = source->xmax; cube_points[5].y = source->ymin; cube_points[5].z = source->zmax;
		cube_points[6].x = source->xmax; cube_points[6].y = source->ymax; cube_points[6].z = source->zmin;
		cube_points[7].x = source->xmax; cube_points[7].y = source->ymax; cube_points[7].z = source->zmax;

		for (count = 0; count < 8; count++)
		{
			float x, y, z;

			x = cube_points[count].x * context->scene_rotation[0][0] + cube_points[count].y * context->scene_rotation[1][0] + cube_points[count].z * context->scene_rotation[2][0];
			y = cube_points[count].x * context->scene_rotation[0][1] + cube_points[count].y * context->scene_rotation[1][1] + cube_points[count].z * context->scene_rotation[2][1];
			z = cube_points[count].x * context->scene_rotation[0][2] + cube_points[count].y * context->scene_rotation[1][2] + cube_points[count].z * context->scene_rotation[2][2];

			cube_points[count].x = x;
			cube_points[count].y = y;
			cube_points[count].z = z;
		}

		results->xmin = cube_points[0].x;
		results->xmax = cube_points[0].x;
		results->ymin = cube_points[0].y;
		results->ymax = cube_points[0].y;
		results->zmin = cube_points[0].z;
		results->zmax = cube_points[0].z;

		for (count = 1; count < 8; count++)
		{
			results->xmin = min(results->xmin, cube_points[count].x);
			results->ymin = min(results->ymin, cube_points[count].y);
			results->zmin = min(results->zmin, cube_points[count].z);

			results->xmax = max(results->xmax, cube_points[count].x);
			results->ymax = max(results->ymax, cube_points[count].y);
			results->zmax = max(results->zmax, cube_points[count].z);
		}
	}
}

static const bounds *object_box(const bounds_context *context, int32_t object, int second)
{
	return ((const bounds *) (context->object_bounds + (size_t) object * 12 + (second ? 6 : 0)));
}

static void recurse_initialise_sub_scene_bounding_box(bounds_context *context, int32_t node_index, int depth)
{
	const eech_model_node *sub_object;
	vec3 old_scene_position, relative_position;
	matrix3x3 matrix, new_rotation, old_scene_rotation;
	int old_contributes_to_collision, count;

	if (context->error)
	{
		return;
	}

	if (depth >= MODEL_MAX_DEPTH || node_index < 0 || node_index >= context->node_count)
	{
		context->error = 1;

		return;
	}

	sub_object = &context->nodes[node_index];

	if (sub_object->object < 0 || sub_object->object >= context->object_count || sub_object->child_count < 0 ||
		(sub_object->child_count > 0 && (sub_object->first_child < 0 || sub_object->first_child > context->node_count - sub_object->child_count)))
	{
		context->error = 1;

		return;
	}

	/* Save state variables */
	old_scene_position = context->scene_position;
	old_contributes_to_collision = context->contributes_to_collision;
	memcpy(old_scene_rotation, context->scene_rotation, sizeof(matrix3x3));

	/* Set the collision flag */
	if (!sub_object->collision)
	{
		context->contributes_to_collision = 0;
	}

	/* Figure out new position */
	relative_position.x = (sub_object->keyframe[0] * context->scene_rotation[0][0]) + (sub_object->keyframe[1] * context->scene_rotation[1][0]) + (sub_object->keyframe[2] * context->scene_rotation[2][0]);
	relative_position.y = (sub_object->keyframe[0] * context->scene_rotation[0][1]) + (sub_object->keyframe[1] * context->scene_rotation[1][1]) + (sub_object->keyframe[2] * context->scene_rotation[2][1]);
	relative_position.z = (sub_object->keyframe[0] * context->scene_rotation[0][2]) + (sub_object->keyframe[1] * context->scene_rotation[1][2]) + (sub_object->keyframe[2] * context->scene_rotation[2][2]);

	context->scene_position.x += relative_position.x;
	context->scene_position.y += relative_position.y;
	context->scene_position.z += relative_position.z;

	get_3d_transformation_matrix(matrix, sub_object->keyframe[3], -sub_object->keyframe[4], -sub_object->keyframe[5]);

	new_rotation[0][0] = ((matrix[0][0] * context->scene_rotation[0][0]) + (matrix[0][1] * context->scene_rotation[1][0]) + (matrix[0][2] * context->scene_rotation[2][0]));
	new_rotation[0][1] = ((matrix[0][0] * context->scene_rotation[0][1]) + (matrix[0][1] * context->scene_rotation[1][1]) + (matrix[0][2] * context->scene_rotation[2][1]));
	new_rotation[0][2] = ((matrix[0][0] * context->scene_rotation[0][2]) + (matrix[0][1] * context->scene_rotation[1][2]) + (matrix[0][2] * context->scene_rotation[2][2]));

	new_rotation[1][0] = ((matrix[1][0] * context->scene_rotation[0][0]) + (matrix[1][1] * context->scene_rotation[1][0]) + (matrix[1][2] * context->scene_rotation[2][0]));
	new_rotation[1][1] = ((matrix[1][0] * context->scene_rotation[0][1]) + (matrix[1][1] * context->scene_rotation[1][1]) + (matrix[1][2] * context->scene_rotation[2][1]));
	new_rotation[1][2] = ((matrix[1][0] * context->scene_rotation[0][2]) + (matrix[1][1] * context->scene_rotation[1][2]) + (matrix[1][2] * context->scene_rotation[2][2]));

	new_rotation[2][0] = ((matrix[2][0] * context->scene_rotation[0][0]) + (matrix[2][1] * context->scene_rotation[1][0]) + (matrix[2][2] * context->scene_rotation[2][0]));
	new_rotation[2][1] = ((matrix[2][0] * context->scene_rotation[0][1]) + (matrix[2][1] * context->scene_rotation[1][1]) + (matrix[2][2] * context->scene_rotation[2][1]));
	new_rotation[2][2] = ((matrix[2][0] * context->scene_rotation[0][2]) + (matrix[2][1] * context->scene_rotation[1][2]) + (matrix[2][2] * context->scene_rotation[2][2]));

	memcpy(context->scene_rotation, new_rotation, sizeof(matrix3x3));

	context->scene_rotation[0][0] *= sub_object->keyframe[6];
	context->scene_rotation[1][0] *= sub_object->keyframe[7];
	context->scene_rotation[2][0] *= sub_object->keyframe[8];

	context->scene_rotation[0][1] *= sub_object->keyframe[6];
	context->scene_rotation[1][1] *= sub_object->keyframe[7];
	context->scene_rotation[2][1] *= sub_object->keyframe[8];

	context->scene_rotation[0][2] *= sub_object->keyframe[6];
	context->scene_rotation[1][2] *= sub_object->keyframe[7];
	context->scene_rotation[2][2] *= sub_object->keyframe[8];

	/* Compute the bounding boxes */
	rotate_sub_scene_boundaries(context, &context->rotated_bounds, object_box(context, sub_object->object, 0));
	rotate_sub_scene_boundaries(context, &context->rotated_bounds2, object_box(context, sub_object->object, 1));

	context->rotated_bounds.xmin += context->scene_position.x;
	context->rotated_bounds.ymin += context->scene_position.y;
	context->rotated_bounds.zmin += context->scene_position.z;

	context->rotated_bounds.xmax += context->scene_position.x;
	context->rotated_bounds.ymax += context->scene_position.y;
	context->rotated_bounds.zmax += context->scene_position.z;

	context->scene_bounds.xmin = min(context->scene_bounds.xmin, context->rotated_bounds.xmin);
	context->scene_bounds.ymin = min(context->scene_bounds.ymin, context->rotated_bounds.ymin);
	context->scene_bounds.zmin = min(context->scene_bounds.zmin, context->rotated_bounds.zmin);

	context->scene_bounds.xmax = max(context->scene_bounds.xmax, context->rotated_bounds.xmax);
	context->scene_bounds.ymax = max(context->scene_bounds.ymax, context->rotated_bounds.ymax);
	context->scene_bounds.zmax = max(context->scene_bounds.zmax, context->rotated_bounds.zmax);

	if (((context->rotated_bounds2.xmax - context->rotated_bounds2.xmin) != 0) || ((context->rotated_bounds2.ymax - context->rotated_bounds2.ymin) != 0) ||
		((context->rotated_bounds2.zmax - context->rotated_bounds2.zmin) != 0))
	{
		context->rotated_bounds2.xmin += context->scene_position.x;
		context->rotated_bounds2.ymin += context->scene_position.y;
		context->rotated_bounds2.zmin += context->scene_position.z;

		context->rotated_bounds2.xmax += context->scene_position.x;
		context->rotated_bounds2.ymax += context->scene_position.y;
		context->rotated_bounds2.zmax += context->scene_position.z;

		if (context->contributes_to_collision)
		{
			context->scene_bounds2.xmin = min(context->scene_bounds2.xmin, context->rotated_bounds2.xmin);
			context->scene_bounds2.ymin = min(context->scene_bounds2.ymin, context->rotated_bounds2.ymin);
			context->scene_bounds2.zmin = min(context->scene_bounds2.zmin, context->rotated_bounds2.zmin);

			context->scene_bounds2.xmax = max(context->scene_bounds2.xmax, context->rotated_bounds2.xmax);
			context->scene_bounds2.ymax = max(context->scene_bounds2.ymax, context->rotated_bounds2.ymax);
			context->scene_bounds2.zmax = max(context->scene_bounds2.zmax, context->rotated_bounds2.zmax);
		}
	}

	/* Calculate sub objects */
	for (count = 0; count < sub_object->child_count; count++)
	{
		recurse_initialise_sub_scene_bounding_box(context, sub_object->first_child + count, depth + 1);
	}

	/* Restore the original position & rotation */
	context->scene_position = old_scene_position;
	memcpy(context->scene_rotation, old_scene_rotation, sizeof(matrix3x3));
	context->contributes_to_collision = old_contributes_to_collision;
}

static void __attribute__((noinline)) scene_bounds_body(bounds_context *context, int32_t main_object, int32_t top_first, int32_t top_count, float *out_bounds, float *out_radius)
{
	int count;

	memset(&context->scene_bounds, 0, sizeof(bounds));
	memset(&context->scene_bounds2, 0, sizeof(bounds));

	context->scene_position.x = 0;
	context->scene_position.y = 0;
	context->scene_position.z = 0;

	get_3d_transformation_matrix(context->scene_rotation, 0, 0, 0);

	if (main_object != 0)
	{
		rotate_sub_scene_boundaries(context, &context->rotated_bounds, object_box(context, main_object, 0));
		rotate_sub_scene_boundaries(context, &context->rotated_bounds2, object_box(context, main_object, 1));

		context->scene_bounds.xmin = min(context->scene_bounds.xmin, context->rotated_bounds.xmin);
		context->scene_bounds.ymin = min(context->scene_bounds.ymin, context->rotated_bounds.ymin);
		context->scene_bounds.zmin = min(context->scene_bounds.zmin, context->rotated_bounds.zmin);

		context->scene_bounds.xmax = max(context->scene_bounds.xmax, context->rotated_bounds.xmax);
		context->scene_bounds.ymax = max(context->scene_bounds.ymax, context->rotated_bounds.ymax);
		context->scene_bounds.zmax = max(context->scene_bounds.zmax, context->rotated_bounds.zmax);

		context->scene_bounds2.xmin = min(context->scene_bounds2.xmin, context->rotated_bounds2.xmin);
		context->scene_bounds2.ymin = min(context->scene_bounds2.ymin, context->rotated_bounds2.ymin);
		context->scene_bounds2.zmin = min(context->scene_bounds2.zmin, context->rotated_bounds2.zmin);

		context->scene_bounds2.xmax = max(context->scene_bounds2.xmax, context->rotated_bounds2.xmax);
		context->scene_bounds2.ymax = max(context->scene_bounds2.ymax, context->rotated_bounds2.ymax);
		context->scene_bounds2.zmax = max(context->scene_bounds2.zmax, context->rotated_bounds2.zmax);
	}

	context->contributes_to_collision = 1;

	for (count = 0; count < top_count; count++)
	{
		recurse_initialise_sub_scene_bounding_box(context, top_first + count, 0);
	}

	if (context->error)
	{
		return;
	}

	memcpy(out_bounds, &context->scene_bounds, sizeof(bounds));
	memcpy(out_bounds + 6, &context->scene_bounds2, sizeof(bounds));

	/* Calculate the radius based on the bounding_box values */
	{
		double dxmin, dxmax, dymin, dymax, dzmin, dzmax, radius;

		dxmax = ((double) context->scene_bounds.xmax * (double) context->scene_bounds.xmax);
		dxmin = ((double) context->scene_bounds.xmin * (double) context->scene_bounds.xmin);

		dymax = ((double) context->scene_bounds.ymax * (double) context->scene_bounds.ymax);
		dymin = ((double) context->scene_bounds.ymin * (double) context->scene_bounds.ymin);

		dzmax = ((double) context->scene_bounds.zmax * (double) context->scene_bounds.zmax);
		dzmin = ((double) context->scene_bounds.zmin * (double) context->scene_bounds.zmin);

		radius = 0;

		radius = max(radius, (dxmax + dymax + dzmax));
		radius = max(radius, (dxmax + dymax + dzmin));
		radius = max(radius, (dxmax + dymin + dzmax));
		radius = max(radius, (dxmax + dymin + dzmin));
		radius = max(radius, (dxmin + dymax + dzmax));
		radius = max(radius, (dxmin + dymax + dzmin));
		radius = max(radius, (dxmin + dymin + dzmax));
		radius = max(radius, (dxmin + dymin + dzmin));

		radius = sqrt(radius);

		*out_radius = radius;
	}
}

/*
 * `object_bounds` is 12 floats per object (bounding_box then bounding_box2), `object_count` objects, index 0 the null object. `nodes` is the scene's sub-object
 * array, the children of a node consecutive; the scene's own sub-objects are `top_count` nodes from `top_first`. `out_bounds` receives 12 floats (the scene's
 * bounding_box, bounding_box2) and `out_radius` the radius. MODEL_BAD_ARGUMENT for any index outside what was given or a tree deeper than MODEL_MAX_DEPTH.
 */
int eech_model_scene_bounds(int32_t main_object, const float *object_bounds, int32_t object_count, const eech_model_node *nodes, int32_t node_count, int32_t top_first,
	int32_t top_count, float *out_bounds, float *out_radius)
{
	eech_model_caller_environment caller;
	bounds_context context;
	int result;

	if (!object_bounds || !out_bounds || !out_radius || object_count < 1 || node_count < 0 || (node_count > 0 && !nodes) || main_object < 0 || main_object >= object_count ||
		top_count < 0 || (top_count > 0 && (top_first < 0 || top_first > node_count - top_count)))
	{
		return (MODEL_BAD_ARGUMENT);
	}

	memset(&context, 0, sizeof(context));
	context.object_bounds = object_bounds;
	context.object_count = object_count;
	context.nodes = nodes;
	context.node_count = node_count;

	result = enter_environment(&caller);

	if (result != MODEL_OK)
	{
		return (result);
	}

	scene_bounds_body(&context, main_object, top_first, top_count, out_bounds, out_radius);

	return (leave_environment(&caller, context.error ? MODEL_BAD_ARGUMENT : MODEL_OK));
}

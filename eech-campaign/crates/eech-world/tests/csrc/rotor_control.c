/*
 * rotor_control.c -- the TEST-ONLY numeric CONTROL of the original rotor sound routine (EEDCS-51, D3; ENV-HOST-ROTOR-CI-1). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
 *
 * WHAT IT IS. The arithmetic statements of the original update_local_helicopter_rotor_sounds (rotor_control_body.h: token-identical after three declared changes), compiled by the SAME compiler with the SAME flags for
 * the SAME target as the host's env_ref.c kernel (build.rs gives both the same settings), in a SURROGATE of the engine's compilation context (collaborators in the engine's shape with stub bodies in another translation
 * unit, reduced entity layouts, see rotor_control_body.h and rotor_control.h). Words in, words out: no floating-point value crosses the boundary. It characterises what a compiler does with those expressions on a
 * target, to be set beside the measured Windows x64 oracle and beside the kernel. It is NOT the engine's own code generation, not a second runtime or a second physical world, nothing in the shipping host or module
 * references it, and it computes no answer for any campaign.
 *
 * TWO VARIANTS, never to be confused. UNMODIFIED (U): STAGE and STAGE_EXTRA are empty. STAGED (S): STAGE stores a variable into an addressable volatile float and reloads it, so the arithmetic that follows continues
 * from the single-precision value, and STAGE_EXTRA makes one extra call; S is diagnostic changed code, a different program, and never an oracle or a proof about the original.
 *
 * Nothing here says which answer is right: the measured oracle is the Windows x64 original of e54423b5c / 06de242ae, and what this control computes on another target is a measurement of that target only.
 */

#include <float.h>
#include <math.h>
#include <string.h>

#include "rotor_control.h"

#define STAGE(TAG, VARIABLE)
#define STAGE_EXTRA(TAG, EXPRESSION)

#define CONTROL_NAME control_original_unmodified
#include "rotor_control_body.h"
#undef CONTROL_NAME
#undef STAGE
#undef STAGE_EXTRA

/* S: the variable is written to an addressable volatile float and read back (so the arithmetic after it continues from the narrowed value) and the stored value is recorded */
#define STAGE(TAG, VARIABLE) { volatile float control_stored = (VARIABLE); (VARIABLE) = control_stored; control_stage (TAG, control_stored); }
#define STAGE_EXTRA(TAG, EXPRESSION) control_stage (TAG, EXPRESSION);

#define CONTROL_NAME control_original_staged
#include "rotor_control_body.h"
#undef CONTROL_NAME
#undef STAGE
#undef STAGE_EXTRA

static dynamics_type control_dynamics;
static helicopter control_helicopter;

/* in_w = { rpm, acceleration x, y, z, left engine rpm, right engine rpm, APU rpm } as words; out_w = { amplification[6], pitch[6] } as words (looping, slap, turbine 1, turbine 2, APU, winding), the words 0xdededede
   where the original wrote nothing; stage_w = the CONTROL_STAGES staged values (STAGED variant only; 0xdededede otherwise) */
void eech_rotor_control_run (int staged, int player, int have_flight_model, const unsigned int *in_w, unsigned int *out_w, unsigned int *stage_w)
{
	entity en;

	control_reset ();

	memset (&en, 0, sizeof (en));
	en.type = ENTITY_TYPE_HELICOPTER;
	en.data = &control_helicopter;
	memcpy (&control_helicopter.main_rotor_rpm, &in_w[0], sizeof (float));
	memcpy (&control_dynamics.model_acceleration_vector.x, &in_w[1], sizeof (float));
	memcpy (&control_dynamics.model_acceleration_vector.y, &in_w[2], sizeof (float));
	memcpy (&control_dynamics.model_acceleration_vector.z, &in_w[3], sizeof (float));
	memcpy (&control_dynamics.left_engine_n1_rpm.value, &in_w[4], sizeof (float));
	memcpy (&control_dynamics.right_engine_n1_rpm.value, &in_w[5], sizeof (float));
	memcpy (&control_dynamics.apu_rpm.value, &in_w[6], sizeof (float));
	en.player = player;
	current_flight_dynamics = have_flight_model ? &control_dynamics : 0;

	if (staged)
	{
		control_original_staged (&en);
	}
	else
	{
		control_original_unmodified (&en);
	}

	current_flight_dynamics = 0;

	control_written_words (out_w);
	control_stage_words (stage_w);
}

#define CONTROL_STRINGIFY_(X) #X
#define CONTROL_STRINGIFY(X) CONTROL_STRINGIFY_(X)

/* what the compiler says about itself and the arithmetic it compiles this file for; the version and flags are also recorded by build.rs from the compiler it ran */
const char *eech_rotor_control_facts (void)
{
	return "__VERSION__=" __VERSION__
#ifdef __GNUC__
		" __GNUC__=" CONTROL_STRINGIFY (__GNUC__) "." CONTROL_STRINGIFY (__GNUC_MINOR__) "." CONTROL_STRINGIFY (__GNUC_PATCHLEVEL__)
#endif
#ifdef __OPTIMIZE__
		" __OPTIMIZE__=1"
#else
		" __OPTIMIZE__=0"
#endif
#ifdef __OPTIMIZE_SIZE__
		" __OPTIMIZE_SIZE__=1"
#endif
#ifdef __FAST_MATH__
		" __FAST_MATH__=1"
#else
		" __FAST_MATH__=0"
#endif
#ifdef __FLT_EVAL_METHOD__
		" __FLT_EVAL_METHOD__=" CONTROL_STRINGIFY (__FLT_EVAL_METHOD__)
#endif
#ifdef __FLT_EVAL_METHOD_TS_18661_3__
		" __FLT_EVAL_METHOD_TS_18661_3__=" CONTROL_STRINGIFY (__FLT_EVAL_METHOD_TS_18661_3__)
#endif
#ifdef FLT_EVAL_METHOD
		" FLT_EVAL_METHOD=" CONTROL_STRINGIFY (FLT_EVAL_METHOD)
#endif
#ifdef __FP_FAST_FMA
		" __FP_FAST_FMA=1"
#else
		" __FP_FAST_FMA=0"
#endif
#ifdef __FLT_ROUNDS
		" __FLT_ROUNDS=defined"
#endif
#ifdef __SSE_MATH__
		" __SSE_MATH__=1"
#else
		" __SSE_MATH__=0"
#endif
#ifdef __SSE2_MATH__
		" __SSE2_MATH__=1"
#else
		" __SSE2_MATH__=0"
#endif
#ifdef __i386__
		" __i386__=1"
#endif
#ifdef __x86_64__
		" __x86_64__=1"
#endif
#ifdef _WIN32
		" _WIN32=1"
#endif
#ifdef __MINGW32__
		" __MINGW32__=1"
#endif
#ifdef __MINGW64_VERSION_MAJOR
		" __MINGW64_VERSION_MAJOR=" CONTROL_STRINGIFY (__MINGW64_VERSION_MAJOR)
#endif
#ifdef __SIZEOF_LONG_DOUBLE__
		" sizeof(long double)=" CONTROL_STRINGIFY (__SIZEOF_LONG_DOUBLE__)
#endif
		;
}

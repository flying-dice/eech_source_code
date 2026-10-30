/*
 * TEST SUPPORT (EEDCS-50, D3; feature fpu-test-support). Never part of the shipping module.
 *
 * The engine runs every host callback under round-toward-zero (the entry sets it and hands the caller's complete environment back). A unit test of the Lua provider's number
 * parsing wants the same mode around the code under test, and wants to know that the code left it as it found it. These two entries do exactly that on the calling thread:
 *
 *   eech_test_enter_toward_zero   saves this thread's complete floating-point environment (the x87 environment and, on x86, the whole of MXCSR) and sets round-toward-zero
 *   eech_test_leave_toward_zero   reports whether the rounding mode was STILL toward-zero (1: the code under test did not change it; 0: it did) and hands the saved
 *                                 environment back; -1 where it cannot be read
 *
 * They call no engine code and touch nothing else.
 */

#ifdef EECH_FPU_TEST_SUPPORT

#include <fenv.h>
#include <stdint.h>

static _Thread_local fenv_t saved_x87;
static _Thread_local uint32_t saved_mxcsr;
static _Thread_local int saved_valid;

#if defined(__x86_64__) || defined(__i386__)
static uint32_t read_mxcsr (void)
{
	uint32_t value = 0;
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (value));
	return value;
}

static void write_mxcsr (uint32_t value)
{
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (value));
}
#else
static uint32_t read_mxcsr (void)
{
	return 0;
}

static void write_mxcsr (uint32_t value)
{
	(void) value;
}
#endif

int eech_test_enter_toward_zero (void)
{
	if (fegetenv (&saved_x87) != 0)
	{
		saved_valid = 0;
		return -1;
	}
	saved_mxcsr = read_mxcsr ();
	saved_valid = 1;
	return (fesetround (FE_TOWARDZERO) == 0) ? 0 : -1;
}

int eech_test_leave_toward_zero (void)
{
	int kept;

	if (!saved_valid)
	{
		return -1;
	}
	kept = (fegetround () == FE_TOWARDZERO) ? 1 : 0;
	fesetenv (&saved_x87);
	write_mxcsr (saved_mxcsr);
	saved_valid = 0;
	return kept;
}

#endif

/*
 * Harness prelude: the parts of EECH's global environment (project.h,
 * modules/system/assert.h) that the extracted functions expect to exist.
 *
 * ASSERT / debug_assert keep EECH's debug-build meaning (the check is
 * performed); a failed check is reported to the harness instead of aborting.
 */

#ifndef EECH_SHIM_PRELUDE_H
#define EECH_SHIM_PRELUDE_H

#include <float.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>

/* the TypeScript port models C float arithmetic as IEEE single precision
   evaluated at declared type; the reference must do the same */
#if !defined (FLT_EVAL_METHOD) || FLT_EVAL_METHOD != 0
#error "the C reference harness requires FLT_EVAL_METHOD == 0"
#endif

#define TRUE 1
#define FALSE 0

#define DEBUG_MODULE 0
#define LANDING_DEBUG 0

void harness_assert (const char *expression);

#define ASSERT(E) if (!(E)) (harness_assert (#E))
#define debug_assert(E) if (!(E)) (harness_assert (#E))

#endif

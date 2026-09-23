/*
 * Harness environment.
 *
 * What the platform, the engine modules (modules/system, modules/maths) and
 * the network layer supply to the original EECH translation units. It holds
 * no campaign behaviour. The generated build/c-reference/project.h includes
 * this first, then verbatim fragments and whole original headers (see
 * c-reference/extract.mjs).
 *
 * Every definition here is either:
 *   - platform: the C library, and the Windows SDK's min/max macros;
 *   - engine: types that included headers only name in prototypes
 *     (matrix3x3, viewpoint), and the debug output functions;
 *   - debug-build validation that is not ported.
 */

#ifndef EECH_HARNESS_ENV_H
#define EECH_HARNESS_ENV_H

#include <float.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* the TypeScript port models C float arithmetic as IEEE single precision
   evaluated at declared type; the reference must do the same */
#if !defined (FLT_EVAL_METHOD) || FLT_EVAL_METHOD != 0
#error "the C reference harness requires FLT_EVAL_METHOD == 0"
#endif

#define TRUE 1
#define FALSE 0

/* Windows SDK windef.h: EECH relies on these platform macros (e.g. gp_updt.c
   max (raw->sleep, 0.0f)). Note max (NaN, 0.0f) is 0.0f, unlike fmax. */
#define max(a,b) (((a) > (b)) ? (a) : (b))
#define min(a,b) (((a) < (b)) ? (a) : (b))

/* modules/system/assert.h with EECH's debug-build meaning: the check is
   performed and a failure is reported to the harness. */
void harness_assert (const char *expression);

#define ASSERT(E) if (!(E)) (harness_assert (#E))
#define debug_assert(E) if (!(E)) (harness_assert (#E))

/* modules/system/debug.h */
extern void debug_fatal (const char *string, ...);
extern void debug_log (const char *string, ...);

/* modules/system/memblock.h: the engine allocator (macros onto
   malloc_fast_memory / malloc_heap_memory); the platform allocator here */
extern void *malloc_fast_mem (int size);
extern void *malloc_heap_mem (int size);
extern void free_mem (void *ptr);

/* engine types that included headers only name in prototypes */
typedef float matrix3x3[3][3];
typedef struct VIEWPOINT viewpoint;

/* en_debug/en_valid.h: debug-build checks of the comms dispatch context,
   excluded from the port (docs/port-manifest.md) */
#define validate_client_server_local_fn()
#define validate_client_server_remote_fn()

#endif

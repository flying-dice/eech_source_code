/*
 * Native kernel environment: what the platform, the engine modules
 * (modules/system, modules/maths) and the network layer supply to the
 * original EECH translation units. It holds no campaign behaviour. The
 * generated project.h includes it first.
 *
 * Provenance: eech-core-ts/c-reference/eech_harness_env.h (81ed32e). The
 * differences: ASSERT and debug_fatal end the current kernel call through the
 * native abort path (eech_kernel_internal.h) instead of the harness's, and the
 * stack attribute marshalling of patch P1 (docs/patches.md) is declared here.
 */

#ifndef EECH_NATIVE_ENV_H
#define EECH_NATIVE_ENV_H

#include <float.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* C float arithmetic at declared type: the C reference and the TSTL port
   model exactly this (docs/fpu.md) */
#if !defined (FLT_EVAL_METHOD) || (FLT_EVAL_METHOD != 0)
#error "the EECH native kernel requires FLT_EVAL_METHOD == 0 (SSE float arithmetic)"
#endif

#define TRUE 1
#define FALSE 0

/* Windows SDK windef.h: EECH relies on these platform macros. Note max (NaN,
   0.0f) is 0.0f, unlike fmax. */
#ifndef max
#define max(a,b) (((a) > (b)) ? (a) : (b))
#endif
#ifndef min
#define min(a,b) (((a) < (b)) ? (a) : (b))
#endif

/* modules/system/assert.h with EECH's debug-build meaning: the check is
   performed, and a failure ends the kernel call with an assertion outcome */
void eech_native_assert (const char *expression, const char *file, int line);

#define ASSERT(E) if (!(E)) (eech_native_assert (#E, __FILE__, __LINE__))
#define debug_assert(E) if (!(E)) (eech_native_assert (#E, __FILE__, __LINE__))

/* modules/system/debug.h */
extern void debug_fatal (const char *string, ...);
extern void debug_log (const char *string, ...);

/* modules/system/memblock.h: the engine allocator; the kernel's arena here */
extern void *malloc_fast_mem (int size);
extern void *malloc_heap_mem (int size);
extern void free_mem (void *ptr);

/* engine types that included headers only name in prototypes */
typedef float matrix3x3[3][3];
typedef struct VIEWPOINT viewpoint;

/* en_debug/en_valid.h: debug-build checks of the comms dispatch context */
#define validate_client_server_local_fn()
#define validate_client_server_remote_fn()

/*
 * Patch P1 (64-bit blocker B1, docs/patches.md): en_creat.c materialises its
 * variadic attribute list with the marshaller instead of reinterpreting the
 * va_list as the i386 argument stack.
 */
#define EECH_STACK_ATTRIBUTES_SIZE 1024

#ifdef EECH_ORIGINAL_STACK_ATTRIBUTES
#define EECH_STACK_ATTRIBUTES_STORAGE(NAME) char NAME[1]
#define EECH_STACK_ATTRIBUTES(NAME, PARGS) ((void) (NAME), (char *) (PARGS))
#else
#define EECH_STACK_ATTRIBUTES_STORAGE(NAME) char NAME[EECH_STACK_ATTRIBUTES_SIZE]
#define EECH_STACK_ATTRIBUTES(NAME, PARGS) eech_marshal_stack_attributes ((NAME), sizeof (NAME), (PARGS))
#endif

extern char *eech_marshal_stack_attributes (char *buffer, size_t size, va_list pargs);

#endif

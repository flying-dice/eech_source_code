/*
 * EECH headless build: the platform layer's own interface (csrc/), used by the
 * replacement backends and by the engine entry points (eech_engine.c).
 */
#ifndef EECH_HEADLESS_H
#define EECH_HEADLESS_H

#include <setjmp.h>
#include <stddef.h>
#include <stdint.h>

/* Windows path -> native path, '/' separated, resolved case-insensitively */
void eech_native_path (const char *in, char *out, size_t size);

/* simulated milliseconds: timeGetTime and every EECH clock read this; the host advances it */
uint32_t eech_headless_time_ms (void);
void eech_headless_advance_time_ms (uint32_t ms);

/*
 * debug_fatal unwinds to the innermost engine entry point instead of ending
 * the process. EECH state after a fatal error is not trusted again: the
 * engine is poisoned and every later entry fails.
 */
extern jmp_buf *eech_fatal_target;
extern char eech_fatal_message[1024];
extern int eech_engine_poisoned;

/* engine log sink (stderr unless the host installs one) */
typedef void (*eech_log_sink) (int level, const char *message, void *user);
void eech_set_log_sink (eech_log_sink sink, void *user);
void eech_log (int level, const char *format, ...) __attribute__ ((format (printf, 2, 3)));

#endif

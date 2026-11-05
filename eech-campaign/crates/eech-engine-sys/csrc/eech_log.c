/*
 * EECH headless build: simulated time, the fatal-error unwind target and the
 * log sink shared by the platform layer and the engine entry points.
 */

#include <stdarg.h>
#include <stdio.h>

#include "eech_headless.h"

static uint32_t simulated_ms;

uint32_t eech_headless_time_ms (void)
{
	return simulated_ms;
}

void eech_headless_advance_time_ms (uint32_t ms)
{
	simulated_ms += ms;
}

void eech_headless_set_time_ms (uint32_t ms)
{
	simulated_ms = ms;
}

jmp_buf *eech_fatal_target;
char eech_fatal_message[1024];
int eech_engine_poisoned;

static eech_log_sink log_sink;
static void *log_user;

void eech_set_log_sink (eech_log_sink sink, void *user)
{
	log_sink = sink;
	log_user = user;
}

void eech_log (int level, const char *format, ...)
{
	char message[2048];
	va_list args;
	va_start (args, format);
	vsnprintf (message, sizeof (message), format, args);
	va_end (args);
	if (log_sink)
	{
		log_sink (level, message, log_user);
	}
	else
	{
		fprintf (stderr, "eech: %s\n", message);
	}
}

/*
 * A NaN position that reached a lookup which cannot take one (patch N1): the
 * callers, as offsets in this module, so they map to source (addr2line); the
 * first reports in full, then every 1000th.
 */
#ifdef __MINGW32__
extern char __ImageBase;
#define MODULE_BASE ((uintptr_t) &__ImageBase)
#else
#define MODULE_BASE ((uintptr_t) 0)
#endif

void eech_report_nan_position (const char *what, void *caller0, void *caller1, void *caller2)
{
	static unsigned reports;
	reports++;
	if (reports <= 10 || reports % 1000 == 0)
	{
		eech_log (1, "NaN position in %s (report %u): callers +0x%lx +0x%lx +0x%lx", what, reports,
			(unsigned long) ((uintptr_t) caller0 - MODULE_BASE), (unsigned long) ((uintptr_t) caller1 - MODULE_BASE),
			(unsigned long) ((uintptr_t) caller2 - MODULE_BASE));
	}
}

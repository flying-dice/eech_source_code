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

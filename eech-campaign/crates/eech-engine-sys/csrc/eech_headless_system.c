/*
 * EECH headless build: replaces modules/system/startup.c (WinMain, the window
 * and its message pump) and modules/system/debug.c (the debug windows).
 * There is no window or message loop: the host drives the engine through
 * eech_engine.c. The application globals keep their Win32 declarations.
 */

#include <stdarg.h>

#include "system.h"

#include "eech_headless.h"

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* startup.c */

HWND application_window;
HINSTANCE application_instance;
CRITICAL_SECTION application_critical_section;
BOOL application_active = TRUE, bFullScreen, bExiting;
char application_current_directory[1024];
char application_debug_fatal_string[1024];
RECT application_window_position;
DWORD system_thread_id;

int application_video_width = 640, application_video_height = 480, application_video_windowed = TRUE;

#define MAX_EXIT_FUNCTIONS 64

static void (*exit_functions[MAX_EXIT_FUNCTIONS]) (void);
static int number_of_exit_functions;

void register_exit_function (void (*fn) (void))
{
	if (number_of_exit_functions < MAX_EXIT_FUNCTIONS)
	{
		exit_functions[number_of_exit_functions++] = fn;
	}
}

/* runs the registered exit functions, most recent first (startup.c order) */
void eech_run_exit_functions (void)
{
	while (number_of_exit_functions > 0)
	{
		exit_functions[--number_of_exit_functions] ();
	}
}

BOOL register_user_message_function (int parm, long (*fn) (void *))
{
	(void) parm;
	(void) fn;
	return TRUE;
}

BOOL register_system_message_function (int parm, long (*fn) (HWND, UINT, WPARAM, LPARAM))
{
	(void) parm;
	(void) fn;
	return TRUE;
}

BOOL register_pre_activate_message_function (void ((*fn) (int)))
{
	(void) fn;
	return TRUE;
}

BOOL register_post_activate_message_function (void ((*fn) (int)))
{
	(void) fn;
	return TRUE;
}

BOOL initialise_windows (HINSTANCE instance, int show)
{
	(void) instance;
	(void) show;
	return TRUE;
}

void deinitialise_windows (void)
{
}

void process_all_waiting_window_messages (void)
{
}

void end_application (void)
{
	bExiting = TRUE;
	eech_run_exit_functions ();
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* debug.c */

int debug_fatal_warning_tone;

void debug_set_window_creation (enum debug_window_creation create)
{
	(void) create;
}

void debug_set_windows_update (enum debug_update_state update)
{
	(void) update;
}

void initialise_debug_system (int logging)
{
	(void) logging;
}

void breakout (struct EVENT *ev)
{
	(void) ev;
	debug_fatal ("user break");
}

void debug_fatal (const char *string, ...)
{
	va_list args;
	va_start (args, string);
	vsnprintf (eech_fatal_message, sizeof (eech_fatal_message), string, args);
	va_end (args);
	snprintf (application_debug_fatal_string, sizeof (application_debug_fatal_string), "%s", eech_fatal_message);
	eech_log (0, "debug_fatal: %s", eech_fatal_message);
	eech_engine_poisoned = 1;
	if (eech_fatal_target)
	{
		longjmp (*eech_fatal_target, 1);
	}
	/* no entry point to unwind to: the engine cannot continue */
	abort ();
}

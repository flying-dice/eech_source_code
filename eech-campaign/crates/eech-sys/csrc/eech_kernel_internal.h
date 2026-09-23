/*
 * Internals shared by the native host layer (csrc/*.c). Included after
 * project.h. Not part of the FFI surface.
 */

#ifndef EECH_KERNEL_INTERNAL_H
#define EECH_KERNEL_INTERNAL_H

#include <setjmp.h>

#include "eech_kernel.h"

#if defined (__GNUC__)
#define EECH_NORETURN __attribute__ ((noreturn))
#define EECH_PRINTF(F, A) __attribute__ ((format (printf, F, A)))
#else
#define EECH_NORETURN __declspec (noreturn)
#define EECH_PRINTF(F, A)
#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// entries and aborts (eech_kernel.c)
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/* the host of the running entry (NULL outside an entry) */
extern const eech_host
	*eech_current_host;

/*
 * The abort point of the innermost operation. An entry installs one; the
 * legacy replay installs one per operation, as the C reference harness does.
 */
extern jmp_buf
	*eech_abort_point;

extern int
	eech_abort_status;

/* ends the innermost operation with `status`: message is the fixed text the C
   reference prints, detail the formatted diagnostic */
EECH_NORETURN void eech_abort (int status, const char *message, const char *detail_format, ...) EECH_PRINTF (3, 4);

/* runs `body (argument)` as an entry */
typedef void (*eech_entry_body) (void *argument);

int eech_enter (const eech_host *host, eech_entry_body body, void *argument);

int eech_in_entry (void);

/* the kernel is open (eech_k_open) */
extern int
	eech_kernel_open;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// entity identity (eech_kernel.c)
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

eech_ref eech_ref_of (entity *en);

/* the live entity a reference names, or NULL (never an abort) */
entity *eech_entity_of (eech_ref ref);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// host calls (eech_env.c); a failed callback aborts the entry with EECH_STATUS_HOST_ERROR
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void eech_host_mobile_position (entity *en, vec3d *position);

void eech_host_object_bounds (int object, struct OBJECT_3D_BOUNDS *bounds);

void eech_host_event (const eech_event *event);

void eech_host_declare (int what, eech_ref ref, int object, const float *values);

/* legacy replay output (a no-op outside a replay) */
void eech_out (const char *format, ...) EECH_PRINTF (1, 2);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// kernel state (eech_tables.c, eech_env.c)
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void eech_initialise_tables (void);

void eech_reset_environment (void);

void eech_arena_free_all (void);

void *eech_arena_alloc (size_t size);

/* host session state (eech_env.c) */
extern int
	eech_single_player;

extern entity
	*eech_session,
	*eech_update_root;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// legacy replay (eech_legacy.c)
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

extern int
	eech_legacy_active;

/* labels of the replay (the harness's label_of / task_label_of) */
const char *eech_legacy_label_of (entity *en);

const char *eech_legacy_task_label_of (entity *en);

/* the replay's "observe-supply-tasks" and "observe-tasks" switches */
extern int
	eech_legacy_observe_supply_tasks;

void eech_legacy_print_attributes (const char *buffer);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// names (generated eech_names.c)
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

typedef struct eech_name_entry
{
	const char
		*name;

	int
		value;
} eech_name_entry;

typedef struct eech_name_table
{
	const char
		*table;

	const eech_name_entry
		*entries;
} eech_name_table;

extern const eech_name_table
	eech_name_tables[];

#endif

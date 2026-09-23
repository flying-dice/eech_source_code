/*
 * The private FFI surface of the native EECH kernel.
 *
 * This header is the complete contract between the legacy C and Rust
 * (eech-sys/src/ffi.rs mirrors it). It exposes no EECH type: entities cross
 * as (index, generation) references, and everything else as plain ints,
 * floats and strings. Nothing outside eech-sys sees it.
 *
 * Calls:
 *   - every eech_k_* call that runs EECH code is an "entry": it installs the
 *     host, the campaign floating-point environment (round toward zero) and an
 *     abort point, and restores the caller's environment before returning;
 *   - an entry never unwinds: ASSERT, debug_fatal, an unported dependency, a
 *     reached boundary or a failed host callback end it with a status, and the
 *     message is read with eech_k_last_message / eech_k_last_detail;
 *   - entries are not reentrant (EECH_STATUS_REENTRANT) and the kernel is one
 *     per process (docs/global-state.md);
 *   - host callbacks run under the host's floating-point environment. A
 *     callback returns 0 for success; any other value ends the entry with
 *     EECH_STATUS_HOST_ERROR. The host must not call eech_k_* from a callback.
 */

#ifndef EECH_KERNEL_H
#define EECH_KERNEL_H

#ifdef __cplusplus
extern "C" {
#endif

enum
{
	EECH_STATUS_OK = 0,
	EECH_STATUS_ASSERT = 1,			/* an EECH ASSERT failed: message is the expression */
	EECH_STATUS_FATAL = 2,			/* EECH called debug_fatal: message is the format, detail the text */
	EECH_STATUS_UNPORTED = 3,		/* EECH reached a dependency the kernel does not provide */
	EECH_STATUS_BOUNDARY = 4,		/* EECH reached the adopted slice's boundary (message names it) */
	EECH_STATUS_HOST_ERROR = 5,		/* a host callback failed */
	EECH_STATUS_INVALID = 6,		/* the caller's request is invalid (unknown entity, bad scenario line, ...) */
	EECH_STATUS_REENTRANT = 7,		/* an entry was called while another was running */
	EECH_STATUS_NOT_OPEN = 8,		/* no campaign is open */
	EECH_STATUS_FPU_DRIFT = 9		/* the campaign floating-point environment changed under EECH */
};

typedef struct eech_ref
{
	int
		index;						/* -1: no entity (NULL) */

	unsigned int
		generation;
} eech_ref;

enum
{
	EECH_EVENT_TRANSMIT_FLOAT = 1,			/* refs[0] entity; ints[0] float type; floats[0] value */
	EECH_EVENT_TRANSMIT_CREATE = 2,			/* ints[0] entity type; ints[1] index */
	EECH_EVENT_TRANSMIT_DESTROY = 3,		/* refs[0] entity */
	EECH_EVENT_TRANSMIT_TASK_POINTERS = 4,	/* refs[0] task */
	EECH_EVENT_TRANSMIT_SWITCH_PARENT = 5,	/* refs[0] entity; refs[1] parent; ints[0] list type */
	EECH_EVENT_TRANSMIT_DESTROY_FAMILY = 6,	/* refs[0] entity (ENTITY_COMMS_DESTROY_LOCAL_FAMILY) */
	EECH_EVENT_MISSION_CREATED = 10,		/* refs[0] task (campaign screen CAMPAIGN_SCREEN_MISSION_CREATED) */
	EECH_EVENT_FORCE_LOW_ON_SUPPLIES = 11,	/* refs[0] force; refs[1] sender; ints[0] cargo sub type; ints[1] the force's side */
	EECH_EVENT_SUPPLY_TASK_REQUESTED = 12	/* refs[0..3] requester, supplier, cargo, start keysite; ints[0] movement; floats[0] priority */
};

typedef struct eech_event
{
	int
		kind;

	eech_ref
		refs[4];

	int
		ints[4];

	float
		floats[4];
} eech_event;

enum
{
	EECH_DECLARE_MOBILE_POSITION = 1,		/* ref, values[0..2] x y z */
	EECH_DECLARE_OBJECT_BOUNDS = 2			/* object, values[0..5] xmin xmax ymin ymax zmin zmax */
};

typedef struct eech_host
{
	void
		*context;

	/* the physical position of a mobile (aircraft) the campaign reads */
	int (*mobile_position) (void *context, eech_ref mobile, float *xyz);

	/* the 3D object database: bounds of one object (object_3d_index_numbers) */
	int (*object_bounds) (void *context, int object, float *bounds);

	/* something the campaign did that the outside world observes */
	int (*event) (void *context, const eech_event *event);

	/* legacy scenario replay only: a line of C reference harness output */
	int (*output) (void *context, const char *text);

	/* legacy scenario replay only: the scenario declares physical state */
	int (*declare) (void *context, int what, eech_ref ref, int object, const float *values);
} eech_host;

/* the message and formatted detail of the last non-OK status (never NULL) */
const char *eech_k_last_message (void);
const char *eech_k_last_detail (void);

/*
 * Lifecycle. open resets every piece of kernel state (docs/global-state.md),
 * initialises the dispatch tables and the entity heap, and creates the
 * session and update entities. close frees everything.
 */
int eech_k_open (int heap_size, int entity_update_frame_rate);
void eech_k_close (void);
int eech_k_is_open (void);

/* host configuration of the session (comms model, game type and status) */
int eech_k_configure (int server, int single_player, int game_type, int game_status);

/* one host frame: the frame delta (set_delta_time), then `count` update passes */
int eech_k_step (const eech_host *host, float delta, int locked, int count);

/* entity references */
int eech_k_ref_valid (eech_ref ref);
int eech_k_entity_type (eech_ref ref);
int eech_k_entity_sub_type (eech_ref ref);
int eech_k_entity_side (eech_ref ref);

/*
 * Semantic restore (a campaign as a saved game holds it). Each returns the new
 * entity through `out`.
 */
int eech_k_set_world_map (const eech_host *host, int x_sectors, int z_sectors, int sector_side_length);
int eech_k_restore_force (int side, eech_ref *out);
int eech_k_restore_keysite (int side, int sub_type, int in_use, float x, float y, float z, float ammo, float fuel, int usable_state, int landing_types, int on_update_list, eech_ref *out);
int eech_k_restore_group (int sub_type, int side, float ammo, float fuel, eech_ref keysite, int independent, eech_ref *out);
int eech_k_restore_member (eech_ref group, int entity_type, int aircraft_sub_type, eech_ref *out);
int eech_k_register_group (eech_ref group);

/* read-only views (snapshot) */
int eech_k_next_entity (eech_ref after, eech_ref *out);		/* used list order; after.index -1 starts */
int eech_k_keysite_view (eech_ref keysite, float *ammo, float *fuel, int *ammo_crates, int *fuel_crates, int *unassigned_tasks);
int eech_k_group_view (eech_ref group, float *ammo, float *fuel, float *sleep, int *member_count);
int eech_k_task_view (eech_ref task, int *sub_type, int *state, float *priority, float *expire, eech_ref *objective, eech_ref *keysite, int *route_length);
int eech_k_cargo_view (eech_ref cargo, int *sub_type, eech_ref *keysite);
int eech_k_force_view (eech_ref force, int *side, int *supply_tasks_created);

/* name <-> ordinal tables of EECH enumerations (generated in eech_names.c) */
int eech_k_enum_lookup (const char *table, const char *name);
const char *eech_k_enum_name (const char *table, int value);

/*
 * Legacy scenario replay: runs one eech-core-ts C reference scenario (the
 * text c-reference/harness.c reads on stdin) and writes the harness's output
 * through host->output. The kernel must be closed; the replay opens and
 * closes it. Returns OK when the scenario ended as the harness ends (with
 * a result line or the f32 lines).
 */
int eech_k_legacy_replay (const eech_host *host, const char *scenario);

/* legacy replay in a dedicated process: a fault inside the NULL page is the
   harness's "result null-dereference" outcome (Linux only; never installed by
   the library itself) */
int eech_k_legacy_install_fault_handler (void);

/* a probe of 64-bit blocker B1 (tests only) */
int eech_k_probe_va_list_reinterpretation (int *read_back, int count, ...);
int eech_k_probe_marshalled (int *read_back, int count, ...);

#ifdef __cplusplus
}
#endif

#endif

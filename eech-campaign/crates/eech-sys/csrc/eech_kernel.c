/*
 * The native kernel's entries, lifecycle, entity identity, semantic restore
 * and read-only views (eech_kernel.h).
 */

#include <fenv.h>

#include "project.h"

#include "eech_kernel_internal.h"

/* FENV_ACCESS: GCC does not implement the pragma; this file is compiled with -frounding-math (build/main.rs) */

/* the campaign's rounding mode: EECH's, toward zero. INVESTIGATION ONLY:
   EECH_FPU_ROUNDING=nearest at build time (docs/fpu.md) */
#ifndef EECH_CAMPAIGN_ROUNDING
#define EECH_CAMPAIGN_ROUNDING FE_TOWARDZERO
#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// entries and aborts
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

const eech_host
	*eech_current_host = NULL;

jmp_buf
	*eech_abort_point = NULL;

int
	eech_abort_status = EECH_STATUS_OK,
	eech_kernel_open = FALSE;

static int
	entry_depth = 0;

static char
	last_message[512] = "",
	last_detail[1024] = "";

/* the caller's floating-point environment, restored around host callbacks */
static fenv_t
	host_fenv;

const char *eech_k_last_message (void) { return last_message; }

const char *eech_k_last_detail (void) { return last_detail; }

int eech_in_entry (void) { return entry_depth > 0; }

/* entities the last abort names (e.g. the boundary's group and task) */
eech_ref
	eech_last_refs[4];

int
	eech_last_ref_count = 0;

int eech_k_last_refs (eech_ref *out)
{
	int
		i;

	for (i = 0; i < eech_last_ref_count; i++)
	{
		out[i] = eech_last_refs[i];
	}

	return eech_last_ref_count;
}

static void set_last (const char *message, const char *detail)
{
	snprintf (last_message, sizeof (last_message), "%s", message);
	snprintf (last_detail, sizeof (last_detail), "%s", detail);
}

void eech_abort (int status, const char *message, const char *detail_format, ...)
{
	char
		detail[1024];

	va_list
		pargs;

	va_start (pargs, detail_format);

	vsnprintf (detail, sizeof (detail), detail_format, pargs);

	va_end (pargs);

	set_last (message, detail);

	eech_abort_status = status;

	if (!eech_abort_point)
	{
		/* an abort outside any entry is a defect of the host layer itself */
		fprintf (stderr, "eech kernel: abort outside an entry: %s (%s)\n", message, detail);

		abort ();
	}

	longjmp (*eech_abort_point, 1);
}

void eech_native_assert (const char *expression, const char *file, int line)
{
	eech_abort (EECH_STATUS_ASSERT, expression, "ASSERT (%s) failed at %s:%d", expression, file, line);
}

/*
 * The campaign runs rounding toward zero (EECH startup.c ::
 * set_fpu_rounding_mode_zero; docs/fpu.md). The host's environment is saved
 * and restored around every entry, and reinstated for every host callback.
 */
int eech_enter (const eech_host *host, eech_entry_body body, void *argument)
{
	jmp_buf
		point;

	volatile int
		status;

	if (entry_depth > 0)
	{
		set_last ("reentrant call", "an eech_k_* entry was called while another was running (from a host callback?)");

		return EECH_STATUS_REENTRANT;
	}

	fegetenv (&host_fenv);

	entry_depth = 1;

	eech_current_host = host;

	eech_abort_point = &point;

	eech_abort_status = EECH_STATUS_OK;

	set_last ("", "");

	eech_last_ref_count = 0;

	fesetround (EECH_CAMPAIGN_ROUNDING);

	if (setjmp (point) == 0)
	{
		body (argument);

		status = (fegetround () == EECH_CAMPAIGN_ROUNDING) ? EECH_STATUS_OK : EECH_STATUS_FPU_DRIFT;

		if (status == EECH_STATUS_FPU_DRIFT)
		{
			set_last ("floating-point environment drifted", "the rounding mode changed during a kernel call");
		}
	}
	else
	{
		status = eech_abort_status;
	}

	fesetenv (&host_fenv);

	eech_abort_point = NULL;

	eech_current_host = NULL;

	entry_depth = 0;

	return status;
}

/* a host callback, under the host's floating-point environment */
int eech_call_host (int (*callback) (void), const char *what)
{
	int
		result;

	fenv_t
		campaign_fenv;

	fegetenv (&campaign_fenv);

	fesetenv (&host_fenv);

	result = callback ();

	fesetenv (&campaign_fenv);

	if (result != 0)
	{
		eech_abort (EECH_STATUS_HOST_ERROR, what, "host callback %s failed (%d)", what, result);
	}

	return result;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// entity identity: a reference is (heap index, generation). The generation
// counts allocations of the index, so a reference to a destroyed entity, or
// to a later entity at the same index, is detected.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static unsigned int
	*generations = NULL;

/* en_heap.c's get_free_entity is compiled as eech_original_get_free_entity
   (spec.rs UNIT_DEFINES); every allocation of the original code passes here */
entity *get_free_entity (int index)
{
	entity
		*en;

	en = eech_original_get_free_entity (index);

	if (en && generations)
	{
		generations[en - entities]++;
	}

	return en;
}

eech_ref eech_ref_of (entity *en)
{
	eech_ref
		ref;

	if (!en || !generations)
	{
		ref.index = -1;
		ref.generation = 0;
	}
	else
	{
		ref.index = (int) (en - entities);
		ref.generation = generations[ref.index];
	}

	return ref;
}

entity *eech_entity_of (eech_ref ref)
{
	entity
		*en;

	if (!eech_kernel_open || !generations || (ref.index < 0) || (ref.index >= number_of_entities))
	{
		return NULL;
	}

	en = &entities[ref.index];

	if ((en->type == ENTITY_TYPE_UNKNOWN) || (generations[ref.index] != ref.generation))
	{
		return NULL;
	}

	return en;
}

int eech_k_ref_valid (eech_ref ref) { return eech_entity_of (ref) != NULL; }

int eech_k_entity_type (eech_ref ref)
{
	entity *en = eech_entity_of (ref);

	return en ? (int) en->type : -1;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// lifecycle
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

extern entity
	**eech_shim_lists;

extern int
	eech_shim_list_entities;

extern void eech_legacy_reset (void);

static update
	*update_data;

typedef struct
{
	int
		heap_size,
		frame_rate,
		set_session;
} open_arguments;

static void open_heap_body (open_arguments *a);

static entity *new_entity (entity_types type, void *data)
{
	entity
		*en;

	en = get_free_entity (ENTITY_INDEX_DONT_CARE);

	if (!en)
	{
		eech_abort (EECH_STATUS_INVALID, "entity heap exhausted", "the campaign exhausted its entity heap of %d", number_of_entities);
	}

	set_local_entity_type (en, type);
	set_local_entity_data (en, data);

	return en;
}

static void *new_raw (size_t size)
{
	void *raw = eech_arena_alloc (size);

	memset (raw, 0, size);

	return raw;
}

/*
 * The dispatch tables and what EECH builds with them at start-up
 * (highlevl.c: initialise_group_task_array). Runs inside an entry: the
 * original initialisers ASSERT. EECH's own deinitialiser undoes it.
 */
static int
	tables_initialised = FALSE;

void eech_kernel_initialise_tables (void)
{
	eech_initialise_tables ();

	tables_initialised = TRUE;
}

static void deinitialise_body (void *argument)
{
	/* suitable.c: the group-to-task suitability array (static there) */
	deinitialise_group_task_array ();
}

static void open_body (void *argument)
{
	open_arguments
		*a = (open_arguments *) argument;

	eech_kernel_initialise_tables ();

	open_heap_body (a);
}

static void open_heap_body (open_arguments *a)
{
	initialise_entity_heap (a->heap_size);

	generations = (unsigned int *) new_raw (sizeof (unsigned int) * (size_t) a->heap_size);

	eech_shim_list_entities = a->heap_size;

	eech_shim_lists = (entity **) new_raw (sizeof (entity *) * (size_t) a->heap_size * NUM_LIST_TYPES * 4);

	command_line_entity_update_frame_rate = a->frame_rate;

	/* the session and update entities come first, as in the C reference */
	eech_session = new_entity (ENTITY_TYPE_SESSION, NULL);

	update_data = (update *) new_raw (sizeof (update));

	eech_update_root = new_entity (ENTITY_TYPE_UPDATE, update_data);

	/* up_update.c :: set_update_entity (it only assigns and logs) */
	update_entity = eech_update_root;

	/* the legacy replay leaves session_entity to its "session" line */
	if (a->set_session)
	{
		session_entity = eech_session;
	}
}

/*
 * Initialised original globals the kernel must return to their load-time
 * values for every campaign (docs/global-state.md): captured on first use,
 * restored on every reset. No original initialiser is restated here.
 */
static int
	load_time_captured = FALSE,
	load_time_keysite_icon_timer_flag;

static float
	load_time_completed_task_expire_time;

static void restore_load_time_state (void)
{
	if (!load_time_captured)
	{
		load_time_keysite_icon_timer_flag = keysite_icon_timer_flag;
		load_time_completed_task_expire_time = get_completed_task_expire_time ();
		load_time_captured = TRUE;
	}

	/* ks_int.c: the campaign map's keysite icon blink flag */
	keysite_icon_timer_flag = load_time_keysite_icon_timer_flag;

	/* task.c: static, with EECH's own setter */
	set_completed_task_expire_time (load_time_completed_task_expire_time);
}

/*
 * A digest of the original databases the kernel compiles in. They are
 * writable data in C; the kernel must never change them (a later campaign
 * would inherit the change). Tests compare the digest before and after runs.
 */
unsigned int eech_k_database_digest (void)
{
	unsigned int
		hash = 2166136261u;

	#define DIGEST(OBJECT) \
	{ \
		const unsigned char *p = (const unsigned char *) &(OBJECT); \
		size_t i; \
		for (i = 0; i < sizeof (OBJECT); i++) { hash = (hash ^ p[i]) * 16777619u; } \
	}

	DIGEST (aircraft_database);
	DIGEST (group_database);
	DIGEST (keysite_database);
	DIGEST (task_database);
	DIGEST (waypoint_database);
	DIGEST (vec3d_type_database);
	DIGEST (entity_attribute_database);

	#undef DIGEST

	return hash;
}

static void reset_all (void)
{
	if (tables_initialised)
	{
		tables_initialised = FALSE;

		if (eech_enter (NULL, deinitialise_body, NULL) != EECH_STATUS_OK)
		{
			fprintf (stderr, "eech kernel: deinitialisation failed: %s\n", eech_k_last_detail ());

			abort ();
		}
	}

	eech_arena_free_all ();

	/* en_heap.c */
	number_of_entities = 0;
	entities = NULL;
	first_free_entity = NULL;
	first_used_entity = NULL;
	start_of_local_entity_heap = 0;
	first_free_local_entity = NULL;
	first_used_local_entity = NULL;

	generations = NULL;
	eech_shim_lists = NULL;
	eech_shim_list_entities = 0;
	update_data = NULL;

	eech_reset_environment ();

	restore_load_time_state ();

	eech_legacy_reset ();

	eech_kernel_open = FALSE;
}

int eech_k_open (int heap_size, int entity_update_frame_rate)
{
	open_arguments
		a;

	int
		status;

	if (eech_in_entry ())
	{
		set_last ("reentrant call", "eech_k_open was called during a kernel call");

		return EECH_STATUS_REENTRANT;
	}

	if ((heap_size < 2) || (heap_size >= MAX_NUM_ENTITIES))
	{
		set_last ("invalid heap size", "the entity heap size must be in [2, MAX_NUM_ENTITIES)");

		return EECH_STATUS_INVALID;
	}

	reset_all ();

	a.heap_size = heap_size;
	a.frame_rate = entity_update_frame_rate;
	a.set_session = TRUE;

	eech_kernel_open = TRUE;

	status = eech_enter (NULL, open_body, &a);

	if (status != EECH_STATUS_OK)
	{
		reset_all ();
	}

	return status;
}

/* the legacy replay (eech_legacy.c): tables first, the heap on its first scenario line */
void eech_kernel_begin_replay (void)
{
	reset_all ();

	eech_kernel_open = TRUE;
}

void eech_kernel_open_heap (int heap_size)
{
	open_arguments
		a;

	a.heap_size = heap_size;
	a.frame_rate = command_line_entity_update_frame_rate;
	a.set_session = FALSE;

	open_heap_body (&a);
}

void eech_kernel_end_replay (void)
{
	reset_all ();
}

void eech_k_close (void)
{
	if (!eech_in_entry ())
	{
		reset_all ();
	}
}

int eech_k_is_open (void) { return eech_kernel_open; }

int eech_k_configure (int server, int single_player, int type, int status)
{
	if (!eech_kernel_open)
	{
		return EECH_STATUS_NOT_OPEN;
	}

	system_comms_model = server ? COMMS_MODEL_SERVER : COMMS_MODEL_CLIENT;
	eech_single_player = single_player;
	game_type = (game_types) type;
	game_status = (game_status_types) status;

	return EECH_STATUS_OK;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// step: one host frame (flight.c's loop)
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

typedef struct
{
	float
		delta;

	int
		locked,
		count;
} step_arguments;

static void step_body (void *argument)
{
	step_arguments
		*a = (step_arguments *) argument;

	int
		c;

	/* time.c :: set_delta_time measures the frame; the host supplies it */
	system_delta_time = a->delta;
	system_one_over_delta_time = 1.0 / system_delta_time;
	locked_frame_rate = a->locked;

	/* one update per time acceleration step */
	for (c = 0; c < a->count; c++)
	{
		update_client_server_entities ();
	}
}

int eech_k_step (const eech_host *host, float delta, int locked, int count)
{
	step_arguments
		a;

	if (!eech_kernel_open)
	{
		set_last ("no campaign is open", "");

		return EECH_STATUS_NOT_OPEN;
	}

	a.delta = delta;
	a.locked = locked;
	a.count = count;

	return eech_enter (host, step_body, &a);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// semantic restore: list membership and raw state as a restored campaign
// holds them (the C reference scenario builder's method)
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/*
 * The pointer updates of insert_local_entity_into_parents_child_list through
 * the dispatch tables, without its LINK_CHILD / LINK_PARENT notifications.
 */
void eech_link_entity_raw (entity *en, list_types type, entity *parent, entity *pred)
{
	entity
		*succ;

	succ = pred ? get_local_entity_child_succ (pred, type) : get_local_entity_first_child (parent, type);

	set_local_entity_child_succ (en, type, succ);
	set_local_entity_child_pred (en, type, pred);
	set_local_entity_parent (en, type, parent);

	if (succ)
	{
		set_local_entity_child_pred (succ, type, en);
	}

	if (pred)
	{
		set_local_entity_child_succ (pred, type, en);
	}
	else
	{
		set_local_entity_first_child (parent, type, en);
	}
}

entity *eech_last_child (entity *parent, list_types type)
{
	entity
		*en,
		*last = NULL;

	for (en = get_local_entity_first_child (parent, type); en; en = get_local_entity_child_succ (en, type))
	{
		last = en;
	}

	return last;
}

entity *eech_new_entity (entity_types type, void *data) { return new_entity (type, data); }

void *eech_new_raw (size_t size) { return new_raw (size); }

/* restore calls run as entries: the original accessors may ASSERT */
typedef struct
{
	int
		ints[8];

	float
		floats[8];

	eech_ref
		refs[2],
		*out;
} restore_arguments;

static void restore_force_body (void *argument)
{
	restore_arguments
		*a = (restore_arguments *) argument;

	force
		*raw;

	entity
		*en;

	raw = (force *) new_raw (sizeof (force));

	raw->side = (entity_sides) a->ints[0];

	en = new_entity (ENTITY_TYPE_FORCE, raw);

	eech_link_entity_raw (en, LIST_TYPE_FORCE, eech_session, eech_last_child (eech_session, LIST_TYPE_FORCE));

	*a->out = eech_ref_of (en);
}

int eech_k_restore_force (int side, eech_ref *out)
{
	restore_arguments
		a;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	if ((side <= ENTITY_SIDE_NEUTRAL) || (side >= NUM_ENTITY_SIDES))
	{
		set_last ("invalid side", "a force's side must be BLUE_FORCE or RED_FORCE");

		return EECH_STATUS_INVALID;
	}

	if (get_local_force_entity ((entity_sides) side))
	{
		set_last ("duplicate force", "the side already has a force");

		return EECH_STATUS_INVALID;
	}

	a.ints[0] = side;
	a.out = out;

	return eech_enter (NULL, restore_force_body, &a);
}

static void restore_keysite_body (void *argument)
{
	restore_arguments
		*a = (restore_arguments *) argument;

	keysite
		*raw;

	entity
		*en,
		*force_en;

	raw = (keysite *) new_raw (sizeof (keysite));

	raw->side = (entity_sides) a->ints[0];
	raw->sub_type = a->ints[1];
	raw->in_use = a->ints[2];
	raw->keysite_usable_state = a->ints[3];
	raw->landing_types = a->ints[4];
	raw->alive = TRUE;
	raw->position.x = a->floats[0];
	raw->position.y = a->floats[1];
	raw->position.z = a->floats[2];
	raw->supplies.ammo_supply_level = a->floats[3];
	raw->supplies.fuel_supply_level = a->floats[4];

	en = new_entity (ENTITY_TYPE_KEYSITE, raw);

	force_en = get_local_force_entity (raw->side);

	if (!force_en)
	{
		eech_abort (EECH_STATUS_INVALID, "keysite without force", "a keysite's side has no force");
	}

	eech_link_entity_raw (en, LIST_TYPE_KEYSITE_FORCE, force_en, eech_last_child (force_en, LIST_TYPE_KEYSITE_FORCE));

	if (a->ints[5])
	{
		/* ks_creat.c: a keysite joins the update list at the head */
		eech_link_entity_raw (en, LIST_TYPE_UPDATE, eech_update_root, eech_last_child (eech_update_root, LIST_TYPE_UPDATE));
	}

	*a->out = eech_ref_of (en);
}

int eech_k_restore_keysite (int side, int sub_type, int in_use, float x, float y, float z, float ammo, float fuel, int usable_state, int landing_types, int on_update_list, eech_ref *out)
{
	restore_arguments
		a;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	if ((sub_type < 0) || (sub_type >= NUM_ENTITY_SUB_TYPE_KEYSITES) || (side <= ENTITY_SIDE_NEUTRAL) || (side >= NUM_ENTITY_SIDES))
	{
		set_last ("invalid keysite", "unknown keysite type or side");

		return EECH_STATUS_INVALID;
	}

	a.ints[0] = side;
	a.ints[1] = sub_type;
	a.ints[2] = in_use;
	a.ints[3] = usable_state;
	a.ints[4] = landing_types;
	a.ints[5] = on_update_list;
	a.floats[0] = x;
	a.floats[1] = y;
	a.floats[2] = z;
	a.floats[3] = ammo;
	a.floats[4] = fuel;
	a.out = out;

	return eech_enter (NULL, restore_keysite_body, &a);
}

static void restore_group_body (void *argument)
{
	restore_arguments
		*a = (restore_arguments *) argument;

	group
		*raw;

	entity
		*en,
		*keysite_en;

	raw = (group *) new_raw (sizeof (group));

	raw->sub_type = a->ints[0];
	raw->side = (entity_sides) a->ints[1];
	raw->alive = TRUE;
	raw->supplies.ammo_supply_level = a->floats[0];
	raw->supplies.fuel_supply_level = a->floats[1];

	en = new_entity (ENTITY_TYPE_GROUP, raw);

	if (a->ints[2])
	{
		entity *force_en = get_local_force_entity (raw->side);

		if (!force_en)
		{
			eech_abort (EECH_STATUS_INVALID, "group without force", "an independent group's side has no force");
		}

		eech_link_entity_raw (en, LIST_TYPE_INDEPENDENT_GROUP, force_en, eech_last_child (force_en, LIST_TYPE_INDEPENDENT_GROUP));
	}
	else
	{
		keysite_en = eech_entity_of (a->refs[0]);

		if (keysite_en)
		{
			eech_link_entity_raw (en, LIST_TYPE_KEYSITE_GROUP, keysite_en, eech_last_child (keysite_en, LIST_TYPE_KEYSITE_GROUP));
		}
	}

	/* gp_updt.c: a group's timers run on the update list */
	eech_link_entity_raw (en, LIST_TYPE_UPDATE, eech_update_root, eech_last_child (eech_update_root, LIST_TYPE_UPDATE));

	*a->out = eech_ref_of (en);
}

int eech_k_restore_group (int sub_type, int side, float ammo, float fuel, eech_ref keysite_ref, int independent, eech_ref *out)
{
	restore_arguments
		a;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	if ((sub_type < 0) || (sub_type >= NUM_ENTITY_SUB_TYPE_GROUPS) || (side <= ENTITY_SIDE_NEUTRAL) || (side >= NUM_ENTITY_SIDES))
	{
		set_last ("invalid group", "unknown group type or side");

		return EECH_STATUS_INVALID;
	}

	if (!independent && (keysite_ref.index >= 0) && (eech_k_entity_type (keysite_ref) != ENTITY_TYPE_KEYSITE))
	{
		set_last ("invalid group base", "a group's base is not a live keysite");

		return EECH_STATUS_INVALID;
	}

	a.ints[0] = sub_type;
	a.ints[1] = side;
	a.ints[2] = independent;
	a.floats[0] = ammo;
	a.floats[1] = fuel;
	a.refs[0] = keysite_ref;
	a.out = out;

	return eech_enter (NULL, restore_group_body, &a);
}

static void restore_member_body (void *argument)
{
	restore_arguments
		*a = (restore_arguments *) argument;

	aircraft
		*raw;

	entity
		*en,
		*group_en;

	group_en = eech_entity_of (a->refs[0]);

	raw = (aircraft *) new_raw (sizeof (aircraft));

	raw->mob.sub_type = a->ints[1];

	en = new_entity ((entity_types) a->ints[0], raw);

	eech_link_entity_raw (en, LIST_TYPE_MEMBER, group_en, eech_last_child (group_en, LIST_TYPE_MEMBER));

	/* gp_pack.c restores member_count from the saved game; it is kept equal to the members */
	((group *) get_local_entity_data (group_en))->member_count++;

	*a->out = eech_ref_of (en);
}

int eech_k_restore_member (eech_ref group_ref, int entity_type, int aircraft_sub_type, eech_ref *out)
{
	restore_arguments
		a;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	if (eech_k_entity_type (group_ref) != ENTITY_TYPE_GROUP)
	{
		set_last ("invalid member group", "a member's group is not a live group");

		return EECH_STATUS_INVALID;
	}

	if (((entity_type != ENTITY_TYPE_HELICOPTER) && (entity_type != ENTITY_TYPE_FIXED_WING)) || (aircraft_sub_type < 0) || (aircraft_sub_type >= NUM_ENTITY_SUB_TYPE_AIRCRAFT))
	{
		set_last ("invalid member", "members are aircraft (helicopter or fixed wing) of a known type");

		return EECH_STATUS_INVALID;
	}

	a.ints[0] = entity_type;
	a.ints[1] = aircraft_sub_type;
	a.refs[0] = group_ref;
	a.out = out;

	return eech_enter (NULL, restore_member_body, &a);
}

static void register_group_body (void *argument)
{
	restore_arguments
		*a = (restore_arguments *) argument;

	entity
		*en,
		*force_en;

	en = eech_entity_of (a->refs[0]);

	force_en = get_local_force_entity ((entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE));

	eech_link_entity_raw (en, LIST_TYPE_AIR_REGISTRY, force_en, eech_last_child (force_en, LIST_TYPE_AIR_REGISTRY));
}

int eech_k_register_group (eech_ref group_ref)
{
	restore_arguments
		a;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	if (eech_k_entity_type (group_ref) != ENTITY_TYPE_GROUP)
	{
		set_last ("invalid group", "not a live group");

		return EECH_STATUS_INVALID;
	}

	a.refs[0] = group_ref;

	return eech_enter (NULL, register_group_body, &a);
}

typedef struct
{
	int
		x,
		z,
		side;
} map_arguments;

static void set_world_map_body (void *argument)
{
	map_arguments
		*a = (map_arguments *) argument;

	/* campaign script parser (parsgen.c) sets the map; en_creat.c ::
	   create_local_only_entities creates the sectors under SERVER/TX */
	set_entity_world_map_size (a->x, a->z, a->side);

	create_local_sector_entities ();
}

int eech_k_set_world_map (const eech_host *host, int x_sectors, int z_sectors, int sector_side_length)
{
	map_arguments
		a;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	a.x = x_sectors;
	a.z = z_sectors;
	a.side = sector_side_length;

	return eech_enter (host, set_world_map_body, &a);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// read-only views: raw state, read without running EECH code
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int eech_k_next_entity (eech_ref after, eech_ref *out)
{
	entity
		*en;

	if (!eech_kernel_open) return EECH_STATUS_NOT_OPEN;

	if (after.index < 0)
	{
		en = first_used_entity;
	}
	else
	{
		en = eech_entity_of (after);

		if (!en)
		{
			set_last ("invalid entity", "stale or unknown entity reference");

			return EECH_STATUS_INVALID;
		}

		en = en->succ;
	}

	*out = eech_ref_of (en);

	return EECH_STATUS_OK;
}

int eech_k_entity_sub_type (eech_ref ref)
{
	entity *en = eech_entity_of (ref);

	if (!en) return -1;

	switch (en->type)
	{
		case ENTITY_TYPE_KEYSITE: return ((keysite *) en->data)->sub_type;
		case ENTITY_TYPE_GROUP: return ((group *) en->data)->sub_type;
		case ENTITY_TYPE_TASK: return ((task *) en->data)->sub_type;
		case ENTITY_TYPE_CARGO: return ((cargo *) en->data)->mob.sub_type;
		case ENTITY_TYPE_HELICOPTER:
		case ENTITY_TYPE_FIXED_WING: return ((aircraft *) en->data)->mob.sub_type;
		default: return -1;
	}
}

int eech_k_entity_side (eech_ref ref)
{
	entity *en = eech_entity_of (ref);

	if (!en) return -1;

	switch (en->type)
	{
		case ENTITY_TYPE_KEYSITE: return ((keysite *) en->data)->side;
		case ENTITY_TYPE_GROUP: return ((group *) en->data)->side;
		case ENTITY_TYPE_TASK: return ((task *) en->data)->side;
		case ENTITY_TYPE_FORCE: return ((force *) en->data)->side;
		case ENTITY_TYPE_CARGO: return ((cargo *) en->data)->mob.side;
		case ENTITY_TYPE_HELICOPTER:
		case ENTITY_TYPE_FIXED_WING: return ((aircraft *) en->data)->mob.side;
		default: return -1;
	}
}

static int count_list (entity *parent, list_types type, int sub_type)
{
	entity
		*en;

	int
		count = 0;

	for (en = ((keysite *) parent->data)->cargo_root.first_child; en; en = ((cargo *) en->data)->cargo_link.child_succ)
	{
		if ((sub_type < 0) || (((cargo *) en->data)->mob.sub_type == sub_type))
		{
			count++;
		}
	}

	return count;
}

int eech_k_keysite_view (eech_ref ref, float *ammo, float *fuel, int *ammo_crates, int *fuel_crates, int *unassigned_tasks)
{
	entity
		*en,
		*t;

	keysite
		*raw;

	en = eech_entity_of (ref);

	if (!en || (en->type != ENTITY_TYPE_KEYSITE)) return EECH_STATUS_INVALID;

	raw = (keysite *) en->data;

	*ammo = raw->supplies.ammo_supply_level;
	*fuel = raw->supplies.fuel_supply_level;
	*ammo_crates = count_list (en, LIST_TYPE_CARGO, ENTITY_SUB_TYPE_CARGO_AMMO);
	*fuel_crates = count_list (en, LIST_TYPE_CARGO, ENTITY_SUB_TYPE_CARGO_FUEL);

	*unassigned_tasks = 0;

	for (t = raw->unassigned_task_root.first_child; t; t = ((task *) t->data)->task_link.child_succ)
	{
		(*unassigned_tasks)++;
	}

	return EECH_STATUS_OK;
}

int eech_k_group_view (eech_ref ref, float *ammo, float *fuel, float *sleep, int *member_count)
{
	entity
		*en;

	group
		*raw;

	en = eech_entity_of (ref);

	if (!en || (en->type != ENTITY_TYPE_GROUP)) return EECH_STATUS_INVALID;

	raw = (group *) en->data;

	*ammo = raw->supplies.ammo_supply_level;
	*fuel = raw->supplies.fuel_supply_level;
	*sleep = raw->sleep;
	*member_count = raw->member_count;

	return EECH_STATUS_OK;
}

int eech_k_task_view (eech_ref ref, int *sub_type, int *state, float *priority, float *expire, eech_ref *objective, eech_ref *keysite_out, int *route_length)
{
	entity
		*en;

	task
		*raw;

	en = eech_entity_of (ref);

	if (!en || (en->type != ENTITY_TYPE_TASK)) return EECH_STATUS_INVALID;

	raw = (task *) en->data;

	*sub_type = raw->sub_type;
	*state = raw->task_state;
	*priority = raw->task_priority;
	*expire = raw->expire_timer;
	*objective = eech_ref_of (raw->task_dependent_link.parent);
	*keysite_out = eech_ref_of (raw->task_link.parent);
	*route_length = (int) raw->route_length;

	return EECH_STATUS_OK;
}

int eech_k_cargo_view (eech_ref ref, int *sub_type, eech_ref *keysite_out)
{
	entity
		*en;

	cargo
		*raw;

	en = eech_entity_of (ref);

	if (!en || (en->type != ENTITY_TYPE_CARGO)) return EECH_STATUS_INVALID;

	raw = (cargo *) en->data;

	*sub_type = raw->mob.sub_type;
	*keysite_out = eech_ref_of (raw->cargo_link.parent);

	return EECH_STATUS_OK;
}

int eech_k_force_view (eech_ref ref, int *side, int *supply_tasks_created)
{
	entity
		*en;

	force
		*raw;

	en = eech_entity_of (ref);

	if (!en || (en->type != ENTITY_TYPE_FORCE)) return EECH_STATUS_INVALID;

	raw = (force *) en->data;

	*side = raw->side;
	*supply_tasks_created = raw->task_generation[ENTITY_SUB_TYPE_TASK_SUPPLY].created;

	return EECH_STATUS_OK;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// names
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static const eech_name_entry *find_table (const char *table)
{
	int
		i;

	for (i = 0; eech_name_tables[i].table; i++)
	{
		if (strcmp (eech_name_tables[i].table, table) == 0)
		{
			return eech_name_tables[i].entries;
		}
	}

	return NULL;
}

int eech_k_enum_lookup (const char *table, const char *name)
{
	const eech_name_entry
		*e = find_table (table);

	for (; e && e->name; e++)
	{
		if (strcmp (e->name, name) == 0)
		{
			return e->value;
		}
	}

	return -1;
}

const char *eech_k_enum_name (const char *table, int value)
{
	const eech_name_entry
		*e = find_table (table);

	for (; e && e->name; e++)
	{
		if (e->value == value)
		{
			return e->name;
		}
	}

	return NULL;
}

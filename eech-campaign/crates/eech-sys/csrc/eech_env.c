/*
 * The native kernel's environment: what the engine modules, the network
 * layer and the game shell supply to the original campaign code.
 *
 * Provenance: eech-core-ts/c-reference/harness.c (81ed32e), "environment",
 * "fail-loud stubs", keysite.c and fc_msgs.c sections, transport. Every
 * environmental input the C reference took from its scenario now comes from
 * the host (eech_host): the physical position of mobiles, the 3D object
 * database; every environmental output (the transport, the campaign screen)
 * goes to the host as an event.
 */

#include "project.h"

#include "eech_kernel_internal.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// host session state
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

entity
	*session_entity,
	*gunship_entity = NULL,		/* helicop.h: the player's gunship. The campaign core has none. */
	*eech_session,
	*eech_update_root;

comms_model_types
	system_comms_model = COMMS_MODEL_SERVER;

/* the server simulation transmits (entity creation uses stack attributes) */
comms_data_flow_types
	system_comms_data_flow = COMMS_DATA_FLOW_TX;

int
	command_line_entity_update_frame_rate = 2,
	command_line_downwash = FALSE,
	eech_single_player = FALSE;

FILE
	*tacview_log_file = NULL;		/* tacview logging is excluded: never logging */

/* global.c: the host's game status */
game_status_types
	game_status;

/* gametype.c :: game_type, as the front end assigns it */
game_types
	game_type;

/* read only by keysite.c functions the kernel never calls */
int
	random_number_seed,
	command_line_capture_aircraft,
	speech_sector_coordinates [6];

const char
	*entity_side_names [NUM_ENTITY_SIDES],
	*entity_side_short_names [NUM_ENTITY_SIDES];

object_3d_information
	*object_3d_information_database = NULL;

/* patch P2 (docs/patches.md): ks_updt.c's function-local static, reset per campaign */
float
	eech_ks_updt_task_timer = 0.0;

void eech_reset_environment (void)
{
	session_entity = NULL;
	gunship_entity = NULL;
	eech_session = NULL;
	eech_update_root = NULL;
	update_entity = NULL;
	update_succ = NULL;
	system_comms_model = COMMS_MODEL_SERVER;
	system_comms_data_flow = COMMS_DATA_FLOW_TX;
	command_line_entity_update_frame_rate = 2;
	command_line_downwash = FALSE;
	eech_single_player = FALSE;
	tacview_log_file = NULL;
	game_status = (game_status_types) 0;
	game_type = (game_types) 0;
	random_number_seed = 0;
	command_line_capture_aircraft = 0;
	memset (speech_sector_coordinates, 0, sizeof (speech_sector_coordinates));
	eech_ks_updt_task_timer = 0.0;

	/* modules/system/time.c initial values */
	system_delta_time = 0.1f;
	system_one_over_delta_time = 10.0f;
	locked_frame_rate = FALSE;

	/* en_world.c, sector.c */
	memset (&world_map, 0, sizeof (world_map));
	entity_sector_map = NULL;

	eech_reset_extracted_statics ();
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// memory: the engine allocator (modules/system/memblock.c) is the kernel's
// arena. Everything EECH allocates belongs to the open campaign and is freed
// with it, whatever state an aborted call left behind.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

typedef struct arena_block
{
	struct arena_block
		*succ,
		*pred;

	double
		align;
} arena_block;

static arena_block
	*arena_first = NULL;

void *eech_arena_alloc (size_t size)
{
	arena_block
		*block;

	block = (arena_block *) malloc (sizeof (arena_block) + size);

	if (!block)
	{
		eech_abort (EECH_STATUS_FATAL, "out of memory", "the kernel arena could not allocate %lu bytes", (unsigned long) size);
	}

	block->pred = NULL;
	block->succ = arena_first;

	if (arena_first)
	{
		arena_first->pred = block;
	}

	arena_first = block;

	return block + 1;
}

void free_mem (void *ptr)
{
	arena_block
		*block;

	if (!ptr)
	{
		return;
	}

	block = ((arena_block *) ptr) - 1;

	if (block->pred)
	{
		block->pred->succ = block->succ;
	}
	else
	{
		arena_first = block->succ;
	}

	if (block->succ)
	{
		block->succ->pred = block->pred;
	}

	free (block);
}

void eech_arena_free_all (void)
{
	arena_block
		*block,
		*succ;

	for (block = arena_first; block; block = succ)
	{
		succ = block->succ;

		free (block);
	}

	arena_first = NULL;
}

void *malloc_fast_mem (int size)
{
	return eech_arena_alloc ((size_t) size);
}

void *malloc_heap_mem (int size)
{
	return eech_arena_alloc ((size_t) size);
}

/*
 * modules/system/fpu.c :: convert_float_to_int is x87 fistp, which rounds with
 * the FPU control word. EECH sets round-toward-zero at start-up, so the
 * conversion truncates: the C (int) cast (eech-core-ts
 * docs/slices/entity-lifecycle-cargo.md). Portable on every target.
 */
void convert_float_to_int (float value, int *ptr)
{
	*ptr = (int) value;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// outcomes of the original code: debug output and debug_fatal
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void debug_fatal (const char *string, ...)
{
	char
		detail[512];

	va_list
		pargs;

	va_start (pargs, string);

	vsnprintf (detail, sizeof (detail), string, pargs);

	va_end (pargs);

	eech_abort (EECH_STATUS_FATAL, string, "%s", detail);
}

void debug_log (const char *string, ...)
{
}

void debug_colour_log (enum DEBUG_COLOURS colour, const char *string, ...)
{
}

void debug_log_entity_message (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// host calls
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

extern int eech_call_host (int (*callback) (void), const char *what);

static eech_ref
	callback_ref;

static int
	callback_object;

static float
	*callback_values;

static const eech_event
	*callback_event;

static int
	callback_what;

static int call_mobile_position (void) { return eech_current_host->mobile_position (eech_current_host->context, callback_ref, callback_values); }
static int call_object_bounds (void) { return eech_current_host->object_bounds (eech_current_host->context, callback_object, callback_values); }
static int call_event (void) { return eech_current_host->event (eech_current_host->context, callback_event); }
static int call_declare (void) { return eech_current_host->declare (eech_current_host->context, callback_what, callback_ref, callback_object, callback_values); }

void eech_host_mobile_position (entity *en, vec3d *position)
{
	float
		xyz[3];

	if (!eech_current_host || !eech_current_host->mobile_position)
	{
		eech_abort (EECH_STATUS_UNPORTED, "mobile position", "the host provides no mobile positions (entity %d)", get_local_entity_index (en));
	}

	callback_ref = eech_ref_of (en);
	callback_values = xyz;

	eech_call_host (call_mobile_position, "World::position");

	position->x = xyz[0];
	position->y = xyz[1];
	position->z = xyz[2];
}

void eech_host_object_bounds (int object, struct OBJECT_3D_BOUNDS *bounds)
{
	float
		b[6];

	if (!eech_current_host || !eech_current_host->object_bounds)
	{
		eech_abort (EECH_STATUS_UNPORTED, "object bounds", "the host provides no 3D object database (object %d)", object);
	}

	callback_object = object;
	callback_values = b;

	eech_call_host (call_object_bounds, "World::object_bounds");

	bounds->xmin = b[0];
	bounds->xmax = b[1];
	bounds->ymin = b[2];
	bounds->ymax = b[3];
	bounds->zmin = b[4];
	bounds->zmax = b[5];
}

void eech_host_event (const eech_event *event)
{
	if (!eech_current_host || !eech_current_host->event)
	{
		return;
	}

	callback_event = event;

	eech_call_host (call_event, "event sink");
}

void eech_host_declare (int what, eech_ref ref, int object, const float *values)
{
	if (!eech_current_host || !eech_current_host->declare)
	{
		eech_abort (EECH_STATUS_INVALID, "declare", "the host accepts no scenario declarations");
	}

	callback_what = what;
	callback_ref = ref;
	callback_object = object;
	callback_values = (float *) values;

	eech_call_host (call_declare, "scenario world");
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// 3dobjvis.c :: get_object_3d_bounding_box: the 3D object database belongs to
// the game's object files, i.e. the host. The pointer returned stays valid
// until the next lookup of the same object in this entry.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define OBJECT_BOUNDS_CACHE 32

static struct
{
	int
		object;

	struct OBJECT_3D_BOUNDS
		bounds;
}
	object_bounds_cache[OBJECT_BOUNDS_CACHE];

static int
	object_bounds_next;

struct OBJECT_3D_BOUNDS *get_object_3d_bounding_box (object_3d_index_numbers object)
{
	int
		slot;

	slot = object_bounds_next;

	object_bounds_next = (object_bounds_next + 1) % OBJECT_BOUNDS_CACHE;

	object_bounds_cache[slot].object = (int) object;

	eech_host_object_bounds ((int) object, &object_bounds_cache[slot].bounds);

	return &object_bounds_cache[slot].bounds;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// transport: en_comms.c :: transmit_entity_comms_message. The semantic
// contract is "an authoritative value changed"; each message becomes a host
// event (the replay also prints the C reference harness line).
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/* the only packing the kernel runs is ENTITY_COMMS_SET_TASK_POINTERS's route
   nodes (the original pack_vec3d, which checks and bounds each node in place);
   its bits go nowhere */
static int
	packing_task_pointers = FALSE;

void transmit_entity_comms_message (entity_comms_messages message, entity *en, ...)
{
	va_list
		pargs;

	eech_event
		event;

	/* en_comms.c: "trap single player or comms messages disabled" */
	if (eech_single_player)
	{
		return;
	}

	memset (&event, 0, sizeof (event));

	va_start (pargs, en);

	if (message == ENTITY_COMMS_FLOAT_VALUE)
	{
		float_types type = va_arg (pargs, float_types);

		float value = va_arg (pargs, double);

		event.kind = EECH_EVENT_TRANSMIT_FLOAT;
		event.refs[0] = eech_ref_of (en);
		event.ints[0] = (int) type;
		event.floats[0] = value;

		eech_host_event (&event);

		{
			unsigned int bits;
			memcpy (&bits, &value, sizeof (bits));
			eech_out ("transmit %s %d %08x\n", eech_legacy_label_of (en), (int) type, bits);
		}
	}
	else if (message == ENTITY_COMMS_CREATE)
	{
		/* (entity_comms_messages message, entity *en, entity_types type, int index, char *pargs) */
		entity_types type = va_arg (pargs, entity_types);

		int index = va_arg (pargs, int);

		const char *buffer = va_arg (pargs, const char *);

		event.kind = EECH_EVENT_TRANSMIT_CREATE;
		event.ints[0] = (int) type;
		event.ints[1] = index;

		eech_host_event (&event);

		if (eech_legacy_active)
		{
			eech_out ("transmit-create %d %d", (int) type, index);

			eech_legacy_print_attributes (buffer);

			eech_out ("\n");
		}
	}
	else if (message == ENTITY_COMMS_DESTROY)
	{
		event.kind = EECH_EVENT_TRANSMIT_DESTROY;
		event.refs[0] = eech_ref_of (en);

		eech_host_event (&event);

		eech_out ("transmit-destroy %s\n", eech_legacy_label_of (en));
	}
	else if (message == ENTITY_COMMS_DESTROY_LOCAL_FAMILY)
	{
		event.kind = EECH_EVENT_TRANSMIT_DESTROY_FAMILY;
		event.refs[0] = eech_ref_of (en);

		eech_host_event (&event);

		eech_out ("transmit-destroy-family %s\n", eech_legacy_label_of (en));
	}
	else if (message == ENTITY_COMMS_SET_TASK_POINTERS)
	{
		task *raw = (task *) get_local_entity_data (en);
		unsigned int loop;

		/* pack every node first: a failed ASSERT ends the operation before anything is reported */
		packing_task_pointers = TRUE;

		for (loop = 0; loop < raw->route_length; loop ++)
		{
			pack_vec3d (en, VEC3D_TYPE_POSITION, &raw->route_nodes [loop]);
		}

		packing_task_pointers = FALSE;

		event.kind = EECH_EVENT_TRANSMIT_TASK_POINTERS;
		event.refs[0] = eech_ref_of (en);

		eech_host_event (&event);

		if (eech_legacy_active)
		{
			unsigned int bits[3];

			eech_out ("transmit-task-pointers %s nodes", eech_legacy_task_label_of (en));

			for (loop = 0; loop < raw->route_length; loop ++)
			{
				memcpy (&bits[0], &raw->route_nodes [loop].x, 4);
				memcpy (&bits[1], &raw->route_nodes [loop].y, 4);
				memcpy (&bits[2], &raw->route_nodes [loop].z, 4);
				eech_out (" %08x %08x %08x", bits[0], bits[1], bits[2]);
			}

			eech_out (" formations");
			for (loop = 0; loop < raw->route_length; loop ++) eech_out (" %d", (int) raw->route_formation_types [loop]);

			eech_out (" waypoints");
			for (loop = 0; loop < raw->route_length; loop ++) eech_out (" %d", (int) raw->route_waypoint_types [loop]);

			eech_out (" dependents");
			for (loop = 0; loop < raw->route_length; loop ++) eech_out (" %s", eech_legacy_label_of (raw->route_dependents [loop]));

			eech_out (" return %s\n", eech_legacy_label_of (raw->return_keysite));
		}
	}
	else if (message == ENTITY_COMMS_SWITCH_PARENT)
	{
		/* (entity_comms_messages message, entity *en, list_types type, entity *parent) */
		list_types type = va_arg (pargs, list_types);

		entity *parent = va_arg (pargs, entity *);

		event.kind = EECH_EVENT_TRANSMIT_SWITCH_PARENT;
		event.refs[0] = eech_ref_of (en);
		event.refs[1] = eech_ref_of (parent);
		event.ints[0] = (int) type;

		eech_host_event (&event);

		eech_out ("transmit-switch-parent %s %d %s\n", eech_legacy_task_label_of (en), (int) type, eech_legacy_label_of (parent));
	}
	else
	{
		va_end (pargs);

		eech_abort (EECH_STATUS_UNPORTED, "entity comms message", "transmit_entity_comms_message: %s is not part of the kernel", eech_k_enum_name ("comms_message", (int) message) ? eech_k_enum_name ("comms_message", (int) message) : "?");
	}

	va_end (pargs);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// observation points and the adopted boundary
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/*
 * fc_msgs.c's call of taskgen.c :: create_supply_task (renamed at the call
 * site by a controlled definition, spec.rs UNIT_DEFINES): reported, then the
 * original runs.
 */
extern entity *create_supply_task (entity *requester, entity *supplier, entity *cargo, movement_types movement_type, float priority, entity *start_keysite, entity *end_keysite);

entity *eech_observe_create_supply_task (entity *requester, entity *supplier, entity *cargo, movement_types movement_type, float priority, entity *start_keysite, entity *end_keysite)
{
	eech_event
		event;

	memset (&event, 0, sizeof (event));
	event.kind = EECH_EVENT_SUPPLY_TASK_REQUESTED;
	event.refs[0] = eech_ref_of (requester);
	event.refs[1] = eech_ref_of (supplier);
	event.refs[2] = eech_ref_of (cargo);
	event.refs[3] = eech_ref_of (start_keysite);
	event.ints[0] = (int) movement_type;
	event.floats[0] = priority;
	eech_host_event (&event);

	if (eech_legacy_observe_supply_tasks)
	{
		unsigned int bits;

		memcpy (&bits, &priority, sizeof (bits));

		eech_out
		(
			"create-supply-task %s %s %s %d %08x %s %s\n",
			eech_legacy_label_of (requester), eech_legacy_label_of (supplier), eech_legacy_label_of (cargo),
			(int) movement_type, bits, eech_legacy_label_of (start_keysite), eech_legacy_label_of (end_keysite)
		);
	}

	return create_supply_task (requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite);
}

/*
 * assign.c :: assign_primary_task_to_group is the adopted slice's boundary
 * (eech-core-ts slice 6a): the assignment transaction (route, guide, the task
 * becoming ASSIGNED, the members) is not in the kernel. Reaching it ends the
 * call with EECH_STATUS_BOUNDARY naming the group and the task.
 */
static char
	boundary_message[128];

int assign_primary_task_to_group (entity *group_en, entity *task_en)
{
	if (eech_legacy_active)
	{
		/* the C reference's boundary line */
		snprintf
		(
			boundary_message, sizeof (boundary_message), "assign_primary_task_to_group %s %s",
			eech_legacy_label_of (group_en), eech_legacy_task_label_of (task_en)
		);
	}
	else
	{
		snprintf (boundary_message, sizeof (boundary_message), "assign_primary_task_to_group");
	}

	eech_last_refs[0] = eech_ref_of (group_en);
	eech_last_refs[1] = eech_ref_of (task_en);
	eech_last_ref_count = 2;

	eech_abort (EECH_STATUS_BOUNDARY, boundary_message, "assign_primary_task_to_group (group %d, task %d)", get_local_entity_index (group_en), get_local_entity_index (task_en));

	return FALSE;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// fail-loud stubs: referenced by compiled original files, never reached by
// the adopted paths. Reaching one ends the call with EECH_STATUS_UNPORTED.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define NOT_REACHED(WHAT) eech_abort (EECH_STATUS_UNPORTED, WHAT, "%s is not part of the native kernel (reached unexpectedly)", WHAT)

void pack_signed_data (int unpacked_data, int number_of_bits_to_pack) { if (!packing_task_pointers) NOT_REACHED ("pack_signed_data"); }
void pack_unsigned_data (unsigned int unpacked_data, int number_of_bits_to_pack) { if (!packing_task_pointers) NOT_REACHED ("pack_unsigned_data"); }
int unpack_signed_data (int number_of_bits_to_unpack) { NOT_REACHED ("unpack_signed_data"); return 0; }
unsigned int unpack_unsigned_data (int number_of_bits_to_unpack) { NOT_REACHED ("unpack_unsigned_data"); return 0; }
void pack_attitude_angles (entity *en, float heading, float pitch, float roll) { NOT_REACHED ("pack_attitude_angles"); }
void unpack_attitude_angles (entity *en, float *heading, float *pitch, float *roll) { NOT_REACHED ("unpack_attitude_angles"); }
void pack_char_type (char_types type) { NOT_REACHED ("pack_char_type"); }
char_types unpack_char_type (void) { NOT_REACHED ("unpack_char_type"); return 0; }
void pack_char_value (entity *en, char_types type, char value) { NOT_REACHED ("pack_char_value"); }
char unpack_char_value (entity *en, char_types type) { NOT_REACHED ("unpack_char_value"); return 0; }
void pack_float_type (float_types type) { NOT_REACHED ("pack_float_type"); }
float_types unpack_float_type (void) { NOT_REACHED ("unpack_float_type"); return 0; }
void pack_float_value (entity *en, float_types type, float value) { NOT_REACHED ("pack_float_value"); }
float unpack_float_value (entity *en, float_types type) { NOT_REACHED ("unpack_float_value"); return 0.0f; }
void pack_int_type (int_types type) { NOT_REACHED ("pack_int_type"); }
int_types unpack_int_type (void) { NOT_REACHED ("unpack_int_type"); return 0; }
void pack_int_value (entity *en, int_types type, int value) { NOT_REACHED ("pack_int_value"); }
int unpack_int_value (entity *en, int_types type) { NOT_REACHED ("unpack_int_value"); return 0; }
void pack_list_type (list_types type) { NOT_REACHED ("pack_list_type"); }
list_types unpack_list_type (void) { NOT_REACHED ("unpack_list_type"); return 0; }
void pack_string_type (string_types type) { NOT_REACHED ("pack_string_type"); }
string_types unpack_string_type (void) { NOT_REACHED ("unpack_string_type"); return 0; }
void pack_string (entity *en, string_types type, const char *s) { NOT_REACHED ("pack_string"); }
void unpack_string (entity *en, string_types type, char *s) { NOT_REACHED ("unpack_string"); }
void pack_vec3d_type (vec3d_types type) { NOT_REACHED ("pack_vec3d_type"); }
vec3d_types unpack_vec3d_type (void) { NOT_REACHED ("unpack_vec3d_type"); return 0; }
void unpack_vec3d (entity *en, vec3d_types type, vec3d *v) { NOT_REACHED ("unpack_vec3d"); }

void set_comms_model (comms_model_types model) { NOT_REACHED ("set_comms_model"); }
void set_comms_data_flow (comms_data_flow_types data_flow) { NOT_REACHED ("set_comms_data_flow"); }
void enable_entity_comms_messages (void) { NOT_REACHED ("enable_entity_comms_messages"); }
void disable_entity_comms_messages (void) { NOT_REACHED ("disable_entity_comms_messages"); }

void create_local_pylon_entities (pack_modes pack_mode) { NOT_REACHED ("create_local_pylon_entities"); }
void destroy_local_pylon_entities (void) { NOT_REACHED ("destroy_local_pylon_entities"); }
void create_local_bridge_entities (pack_modes pack_mode) { NOT_REACHED ("create_local_bridge_entities"); }
void create_local_update_entity (void) { NOT_REACHED ("create_local_update_entity"); }
void destroy_local_update_entity (void) { NOT_REACHED ("destroy_local_update_entity"); }
void create_local_camera_entity (void) { NOT_REACHED ("create_local_camera_entity"); }
void destroy_local_camera_entity (void) { NOT_REACHED ("destroy_local_camera_entity"); }
void destroy_local_sector_entities (void) { NOT_REACHED ("destroy_local_sector_entities"); }
void destroy_local_sound_effects (entity *en) { NOT_REACHED ("destroy_local_sound_effects"); }
void set_gunship_entity (entity *en) { NOT_REACHED ("set_gunship_entity"); }

void set_sector_fog_of_war_value (entity *en, entity *sector_en) { NOT_REACHED ("set_sector_fog_of_war_value"); }
void update_imap_surface_to_air_defence_level (entity *en, entity *sector, int in_use) { NOT_REACHED ("update_imap_surface_to_air_defence_level"); }
void update_imap_surface_to_surface_defence_level (entity *en, entity *sector, int in_use) { NOT_REACHED ("update_imap_surface_to_surface_defence_level"); }
int get_valid_current_game_session (void) { NOT_REACHED ("get_valid_current_game_session"); return FALSE; }
session_list_types get_current_game_session_type (void) { NOT_REACHED ("get_current_game_session_type"); return SESSION_LIST_TYPE_INVALID; }

entity *get_local_group_member_landing_entity_from_keysite (entity *en) { NOT_REACHED ("get_local_group_member_landing_entity_from_keysite"); return NULL; }

int tacview_reset_frame (void) { NOT_REACHED ("tacview_reset_frame"); return 0; }
void write_tacview_frame_header (void) { NOT_REACHED ("write_tacview_frame_header"); }

void add_group_type_to_force_info (entity *en, entity_sub_types group_type) { NOT_REACHED ("add_group_type_to_force_info"); }
void remove_group_type_from_force_info (entity *en, entity_sub_types group_type) { NOT_REACHED ("remove_group_type_from_force_info"); }
int set_local_division_name (entity *en, char *s) { NOT_REACHED ("set_local_division_name"); return 0; }

void add_default_entity_to_regen_queue (entity_sides side, entity_sub_types group_type) { NOT_REACHED ("add_default_entity_to_regen_queue"); }
int increment_regen_queue_size (entity_sides side, entity_types type, int shift) { NOT_REACHED ("increment_regen_queue_size"); return 0; }
entity *get_local_group_member_landing_entity_from_task (entity *en) { NOT_REACHED ("get_local_group_member_landing_entity_from_task"); return NULL; }
void update_imap_sector_side (entity *en, int in_use) { NOT_REACHED ("update_imap_sector_side"); }
void update_imap_importance_level (entity *en, int in_use) { NOT_REACHED ("update_imap_importance_level"); }
void update_keysite_distance_to_friendly_base (entity *en, entity_sides side) { NOT_REACHED ("update_keysite_distance_to_friendly_base"); }
void restore_local_fixed_entity (entity *en) { NOT_REACHED ("restore_local_fixed_entity"); }
void group_kill_all_members (entity *en) { NOT_REACHED ("group_kill_all_members"); }
entity *get_local_landing_entity_route (entity *landing_en, entity_sub_types type) { NOT_REACHED ("get_local_landing_entity_route"); return NULL; }
entity *get_local_entity_landing_entity (entity *en, entity_sub_types landing_type) { NOT_REACHED ("get_local_entity_landing_entity"); return NULL; }
int create_group_emergency_transfer_task (entity *en) { NOT_REACHED ("create_group_emergency_transfer_task"); return 0; }
void update_imap_distance_to_friendly_base (entity_sides side) { NOT_REACHED ("update_imap_distance_to_friendly_base"); }
void send_text_message (entity *sender, entity *target, message_text_types type, const char *text) { NOT_REACHED ("send_text_message"); }
int play_client_server_speech (entity *parent, entity *sender, entity_sides side, entity_sub_types sub_type, sound_locality_types locality, float delay, float priority, float expire_time, speech_originator_types originator, speech_category_types category, float category_silence_timer, ...) { NOT_REACHED ("play_client_server_speech"); return 0; }
int *get_speech_sector_coordinates (vec3d *pos) { NOT_REACHED ("get_speech_sector_coordinates"); return NULL; }
int get_object_3d_troop_landing_position_and_heading (int object_index, vec3d *position, float *heading) { NOT_REACHED ("get_object_3d_troop_landing_position_and_heading"); return 0; }
entity *get_local_landing_entity_task (entity *landing_en, entity_sub_types type) { NOT_REACHED ("get_local_landing_entity_task"); return NULL; }
int get_local_entity_suitable_for_player (entity *en, entity *pilot) { NOT_REACHED ("get_local_entity_suitable_for_player"); return 0; }
int get_local_entity_list_size (entity *parent, list_types type) { NOT_REACHED ("get_local_entity_list_size"); return 0; }
void get_digital_clock_values (float time_of_day, float *hours, float *minutes, float *seconds) { NOT_REACHED ("get_digital_clock_values"); }
float get_3d_terrain_point_data (float x, float z, terrain_3d_point_data *point_data) { NOT_REACHED ("get_3d_terrain_point_data"); return 0.0f; }
void free_group_callsign (entity *en) { NOT_REACHED ("free_group_callsign"); }
int file_exist (const char *filename) { NOT_REACHED ("file_exist"); return 0; }
entity *create_cap_task (entity_sides side, entity *this_keysite, entity *originator, int critical, float priority, float duration, entity *start_keysite, entity *end_keysite) { NOT_REACHED ("create_cap_task"); return NULL; }
int assign_group_callsign (entity *en) { NOT_REACHED ("assign_group_callsign"); return 0; }

float get_sector_fog_of_war_value (entity *en, entity_sides side) { NOT_REACHED ("get_sector_fog_of_war_value"); return 0.0f; }
float get_3d_terrain_point_data_elevation (terrain_3d_point_data *point_data) { NOT_REACHED ("get_3d_terrain_point_data_elevation"); return 0.0f; }
float get_local_sector_entity_enemy_surface_to_surface_defence_level (entity *sector_en, entity_sides side) { NOT_REACHED ("get_local_sector_entity_enemy_surface_to_surface_defence_level"); return 0.0f; }
void play_mobile_under_attack_speech (entity *en, entity *aggressor) { NOT_REACHED ("play_mobile_under_attack_speech"); }
void play_client_server_radio_message_response (entity *en, int speech_index, float priority, float expire_time) { NOT_REACHED ("play_client_server_radio_message_response"); }
float get_sqr_2d_range (const vec3d *v1, const vec3d *v2) { NOT_REACHED ("get_sqr_2d_range"); return 0.0f; }
int engage_targets_in_group (entity *group, entity *target_group, int expire) { NOT_REACHED ("engage_targets_in_group"); return 0; }
void create_task_completed_reactionary_tasks (entity *task) { NOT_REACHED ("create_task_completed_reactionary_tasks"); }
void create_task_assigned_reactionary_tasks (entity *task) { NOT_REACHED ("create_task_assigned_reactionary_tasks"); }
int check_group_task_type (entity *group, entity_sub_types task_type) { NOT_REACHED ("check_group_task_type"); return 0; }
void campaign_completed (entity_sides side, campaign_completed_types complete) { NOT_REACHED ("campaign_completed"); }

void update_local_entity_waypoint_list_tags (entity *parent) { NOT_REACHED ("update_local_entity_waypoint_list_tags"); }
int get_formation_database_count (void) { NOT_REACHED ("get_formation_database_count"); return 0; }

/* NATIVE: ks_updt.c's repair check (a keysite below its maximum strength, once a minute) */
entity *create_repair_task (entity_sides side, vec3d *pos, entity *objective, float priority, entity *start_keysite, entity *end_keysite) { NOT_REACHED ("create_repair_task"); return NULL; }

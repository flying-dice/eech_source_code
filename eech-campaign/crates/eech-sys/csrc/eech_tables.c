/*
 * The entity system's dispatch tables and their initialisation.
 *
 * Provenance: eech-core-ts/c-reference/harness.c (81ed32e), "dispatch
 * tables", "fail-loud table defaults", "hand-written rows" and
 * initialise_tables. The tables are the originals' (en_int.c, en_float.c,
 * en_vec3d.c, en_ptr.c, en_list.c, en_updt.c, en_msgs.c); the original
 * overload_*_functions () fill them. Differences from the C reference:
 *
 *   - an unported row ends the kernel call with EECH_STATUS_UNPORTED naming
 *     the row (the harness exits the process);
 *   - the aircraft position rows ask the host (World::position) instead of
 *     reading scenario-written raw state;
 *   - NATIVE: the keysite and task update functions (ks_updt.c, ts_updt.c)
 *     are installed, for step ();
 *   - the FORCE_LOW_ON_SUPPLIES trace and the campaign screen report events.
 */

#include "project.h"

#include "eech_kernel_internal.h"

void (*fn_set_local_entity_raw_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES]) (entity *en, int_types type, int value);
void (*fn_set_local_entity_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES]) (entity *en, int_types type, int value);
void (*fn_set_client_server_entity_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, int_types type, int value);
int (*fn_get_local_entity_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES]) (entity *en, int_types type);

void (*fn_set_local_entity_raw_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES]) (entity *en, float_types type, float value);
void (*fn_set_local_entity_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES]) (entity *en, float_types type, float value);
void (*fn_set_client_server_entity_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, float_types type, float value);
float (*fn_get_local_entity_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES]) (entity *en, float_types type);

void (*fn_set_local_entity_raw_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type, char value);
void (*fn_set_local_entity_raw_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type, const char *s);
void (*fn_set_local_entity_raw_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float heading, float pitch, float roll);

void (*fn_set_local_entity_raw_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type, vec3d *v);
void (*fn_set_local_entity_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type, vec3d *v);
void (*fn_set_client_server_entity_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, vec3d_types type, vec3d *v);
void (*fn_get_local_entity_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type, vec3d *v);
vec3d *(*fn_get_local_entity_vec3d_ptr[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type);

void (*fn_set_local_entity_ptr_value[NUM_ENTITY_TYPES][NUM_PTR_TYPES]) (entity *en, ptr_types type, void *ptr);
void *(*fn_get_local_entity_ptr_value[NUM_ENTITY_TYPES][NUM_PTR_TYPES]) (entity *en, ptr_types type);

void (*fn_set_local_entity_first_child[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type, entity *first_child);
entity *(*fn_get_local_entity_first_child[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type);
void (*fn_set_local_entity_parent[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type, entity *parent);
entity *(*fn_get_local_entity_parent[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type);
void (*fn_set_local_entity_child_succ[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type, entity *child_succ);
entity *(*fn_get_local_entity_child_succ[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type);
void (*fn_set_local_entity_child_pred[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type, entity *child_pred);
entity *(*fn_get_local_entity_child_pred[NUM_ENTITY_TYPES][NUM_LIST_TYPES]) (entity *en, list_types type);

void (*fn_update_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);

int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

const char *(*fn_get_local_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type);

/* name databases: only read on debug_fatal paths, which end the call */
entity_type_data entity_type_database[NUM_ENTITY_TYPES];
list_type_data list_type_database[NUM_LIST_TYPES];
int_type_data int_type_database[NUM_INT_TYPES];
float_type_data float_type_database[NUM_FLOAT_TYPES];
static const char *native_entity_type_names[NUM_ENTITY_TYPES];
const char **entity_type_names = native_entity_type_names;
ptr_type_data ptr_type_database[NUM_PTR_TYPES];

const char
	*overload_invalid_int_type_message = "invalid int type",
	*debug_fatal_invalid_int_type_message = "invalid int type",
	*overload_invalid_float_type_message = "invalid float type",
	*debug_fatal_invalid_float_type_message = "invalid float type",
	*overload_invalid_list_type_message = "invalid list type",
	*debug_fatal_invalid_list_type_message = "invalid list type",
	*overload_invalid_vec3d_type_message = "invalid vec3d type",
	*debug_fatal_invalid_vec3d_type_message = "invalid vec3d type",
	*overload_invalid_ptr_type_message = "invalid ptr type",
	*debug_fatal_invalid_ptr_type_message = "invalid ptr type";

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// fail-loud table defaults: every row the kernel does not provide
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/* names of ordinals for diagnostics (generated eech_names.c) */
static const char *name (const char *table, int value)
{
	const char *n = eech_k_enum_name (table, value);

	return n ? n : "?";
}

#define UNSUPPLIED(TABLE, KIND, ...) \
	eech_abort (EECH_STATUS_UNPORTED, TABLE " not supplied", TABLE " [%s] [%s]", name ("entity_type", en->type), name (KIND, __VA_ARGS__))

static int unsupplied_get_int (entity *en, int_types type) { UNSUPPLIED ("fn_get_local_entity_int_value", "int_type", type); return 0; }
static void unsupplied_set_int (entity *en, int_types type, int value) { UNSUPPLIED ("fn_set_client_server_entity_int_value", "int_type", type); }
static float unsupplied_get_float (entity *en, float_types type) { UNSUPPLIED ("fn_get_local_entity_float_value", "float_type", type); return 0.0f; }
static void unsupplied_set_float (entity *en, float_types type, float value) { UNSUPPLIED ("fn_set_client_server_entity_float_value", "float_type", type); }
static vec3d *unsupplied_get_vec3d_ptr (entity *en, vec3d_types type) { UNSUPPLIED ("fn_get_local_entity_vec3d_ptr", "vec3d_type", type); return NULL; }
static void *unsupplied_get_ptr (entity *en, ptr_types type) { UNSUPPLIED ("fn_get_local_entity_ptr_value", "ptr_type", type); return NULL; }
static entity *unsupplied_get_list (entity *en, list_types type) { UNSUPPLIED ("list accessor", "list_type", type); return NULL; }
static const char *unsupplied_get_string (entity *en, string_types type) { UNSUPPLIED ("fn_get_local_entity_string", "string_type", type); return NULL; }
static void unsupplied_set_list (entity *en, list_types type, entity *other) { UNSUPPLIED ("list setter", "list_type", type); }
static void unsupplied_update (entity *en) { eech_abort (EECH_STATUS_UNPORTED, "fn_update_client_server_entity not supplied", "fn_update_client_server_entity [%s]", name ("entity_type", en->type)); }
static void unsupplied_set_vec3d (entity *en, vec3d_types type, vec3d *v) { UNSUPPLIED ("fn_set_local_entity_raw_vec3d", "vec3d_type", type); }
static void unsupplied_set_char (entity *en, char_types type, char value) { UNSUPPLIED ("fn_set_local_entity_raw_char_value", "char_type", type); }
static void unsupplied_set_string (entity *en, string_types type, const char *s) { UNSUPPLIED ("fn_set_local_entity_raw_string", "string_type", type); }
static void unsupplied_set_attitude_angles (entity *en, float heading, float pitch, float roll) { eech_abort (EECH_STATUS_UNPORTED, "fn_set_local_entity_raw_attitude_angles not supplied", "fn_set_local_entity_raw_attitude_angles [%s]", name ("entity_type", en->type)); }
static int unsupplied_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs) { entity *en = receiver; UNSUPPLIED ("message_responses", "message", message); return FALSE; }

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// hand-written rows for entity types whose files are not compiled yet
// (eech-core-ts docs/architecture.md, "Shrinking the C reference shim")
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/* list storage for shim entity types (session, guide, aircraft members):
   allocated per campaign with the heap (eech_kernel.c) */
entity
	**eech_shim_lists;

int
	eech_shim_list_entities;

#define SHIM_SLOT(EN, TYPE, FIELD) eech_shim_lists[(((EN) - entities) * NUM_LIST_TYPES + (TYPE)) * 4 + (FIELD)]

static entity *shim_get_first_child (entity *en, list_types type) { return SHIM_SLOT (en, type, 0); }
static void shim_set_first_child (entity *en, list_types type, entity *v) { SHIM_SLOT (en, type, 0) = v; }
static entity *shim_get_parent (entity *en, list_types type) { return SHIM_SLOT (en, type, 1); }
static void shim_set_parent (entity *en, list_types type, entity *v) { SHIM_SLOT (en, type, 1) = v; }
static entity *shim_get_child_succ (entity *en, list_types type) { return SHIM_SLOT (en, type, 2); }
static void shim_set_child_succ (entity *en, list_types type, entity *v) { SHIM_SLOT (en, type, 2) = v; }
static entity *shim_get_child_pred (entity *en, list_types type) { return SHIM_SLOT (en, type, 3); }
static void shim_set_child_pred (entity *en, list_types type, entity *v) { SHIM_SLOT (en, type, 3) = v; }

static void shim_root (entity_types entity_type, list_types list)
{
	fn_get_local_entity_first_child[entity_type][list] = shim_get_first_child;
	fn_set_local_entity_first_child[entity_type][list] = shim_set_first_child;
}

static void shim_link (entity_types entity_type, list_types list)
{
	fn_get_local_entity_parent[entity_type][list] = shim_get_parent;
	fn_set_local_entity_parent[entity_type][list] = shim_set_parent;
	fn_get_local_entity_child_succ[entity_type][list] = shim_get_child_succ;
	fn_set_local_entity_child_succ[entity_type][list] = shim_set_child_succ;
	fn_get_local_entity_child_pred[entity_type][list] = shim_get_child_pred;
	fn_set_local_entity_child_pred[entity_type][list] = shim_set_child_pred;
}

/*
 * ac_vec3d.c :: VEC3D_TYPE_POSITION of an aircraft is physical state: the
 * flight model writes mob.position. The host's World supplies it on every
 * read; the raw struct holds the value last read, as EECH's did the value
 * last simulated, so the returned pointer stays valid.
 */
static vec3d *host_mobile_position (entity *en, vec3d_types type)
{
	aircraft
		*raw;

	raw = (aircraft *) get_local_entity_data (en);

	eech_host_mobile_position (en, &raw->mob.position);

	return &raw->mob.position;
}

/* the original fc_msgs.c :: response_to_force_low_on_supplies (static there) */
static int
	(*original_force_low_on_supplies) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

/*
 * Every delivery is reported (the C reference prints its trace line here),
 * then the original response runs.
 */
static int trace_force_low_on_supplies (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	va_list
		args;

	int
		sub_type;

	eech_event
		event;

	va_copy (args, pargs);

	sub_type = va_arg (args, int);

	va_end (args);

	memset (&event, 0, sizeof (event));
	event.kind = EECH_EVENT_FORCE_LOW_ON_SUPPLIES;
	event.refs[0] = eech_ref_of (receiver);
	event.refs[1] = eech_ref_of (sender);
	event.ints[0] = sub_type;
	event.ints[1] = ((force *) get_local_entity_data (receiver))->side;
	eech_host_event (&event);

	eech_out ("message %s %s %d %d\n", eech_legacy_label_of (receiver), eech_legacy_label_of (sender), (int) message, sub_type);

	return original_force_low_on_supplies (message, receiver, sender, pargs);
}

/*
 * The campaign screen (ui_menu/ingame/campaign). notify_campaign_screen
 * (ca_msgs.c, extracted) keeps its guard; its response table is the UI. The
 * mission list's MISSION_CREATED entry reports the semantic event; every
 * other entry is the UI's default (FALSE).
 */
int (*campaign_screen_message_responses[NUM_CAMPAIGN_SCREEN_MESSAGE_TARGETS][NUM_CAMPAIGN_SCREEN_MESSAGES]) (campaign_screen_messages message, entity *sender);

static int campaign_screen_default (campaign_screen_messages message, entity *sender) { return FALSE; }

static int report_mission_created (campaign_screen_messages message, entity *sender)
{
	eech_event
		event;

	memset (&event, 0, sizeof (event));
	event.kind = EECH_EVENT_MISSION_CREATED;
	event.refs[0] = eech_ref_of (sender);
	eech_host_event (&event);

	eech_out ("campaign mission-created %s\n", eech_legacy_task_label_of (sender));

	return TRUE;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// initialisation (C: initialise_entity_functions, reduced to the adopted entity types)
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void eech_initialise_tables (void)
{
	int
		i,
		j,
		k;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_STRING_TYPES; j++)
		{
			fn_get_local_entity_string[i][j] = unsupplied_get_string;
		}

		for (j = 0; j < NUM_INT_TYPES; j++)
		{
			/* en_int.c defaults: sets do nothing; gets return the type's default */
			fn_set_local_entity_raw_int_value[i][j] = harness_default_set_entity_int_value;
			fn_set_local_entity_int_value[i][j] = harness_default_set_entity_int_value;
			for (k = 0; k < NUM_COMMS_MODEL_TYPES; k++) fn_set_client_server_entity_int_value[i][j][k] = unsupplied_set_int;
			fn_get_local_entity_int_value[i][j] = harness_default_get_entity_int_value;
		}

		for (j = 0; j < NUM_FLOAT_TYPES; j++)
		{
			/* en_float.c defaults: sets do nothing; gets are not supplied */
			fn_set_local_entity_raw_float_value[i][j] = harness_default_set_entity_float_value;
			fn_set_local_entity_float_value[i][j] = harness_default_set_entity_float_value;
			for (k = 0; k < NUM_COMMS_MODEL_TYPES; k++) fn_set_client_server_entity_float_value[i][j][k] = unsupplied_set_float;
			fn_get_local_entity_float_value[i][j] = unsupplied_get_float;
		}

		for (j = 0; j < NUM_VEC3D_TYPES; j++)
		{
			fn_get_local_entity_vec3d_ptr[i][j] = unsupplied_get_vec3d_ptr;
			fn_set_local_entity_raw_vec3d[i][j] = unsupplied_set_vec3d;
		}

		for (j = 0; j < NUM_CHAR_TYPES; j++) fn_set_local_entity_raw_char_value[i][j] = unsupplied_set_char;

		for (j = 0; j < NUM_STRING_TYPES; j++) fn_set_local_entity_raw_string[i][j] = unsupplied_set_string;

		fn_set_local_entity_raw_attitude_angles[i] = unsupplied_set_attitude_angles;

		for (j = 0; j < NUM_PTR_TYPES; j++) fn_get_local_entity_ptr_value[i][j] = unsupplied_get_ptr;

		for (j = 0; j < NUM_LIST_TYPES; j++)
		{
			fn_get_local_entity_first_child[i][j] = unsupplied_get_list;
			fn_set_local_entity_first_child[i][j] = unsupplied_set_list;
			fn_get_local_entity_parent[i][j] = unsupplied_get_list;
			fn_set_local_entity_parent[i][j] = unsupplied_set_list;
			fn_get_local_entity_child_succ[i][j] = unsupplied_get_list;
			fn_set_local_entity_child_succ[i][j] = unsupplied_set_list;
			fn_get_local_entity_child_pred[i][j] = unsupplied_get_list;
			fn_set_local_entity_child_pred[i][j] = unsupplied_set_list;
		}

		for (k = 0; k < NUM_COMMS_MODEL_TYPES; k++) fn_update_client_server_entity[i][k] = unsupplied_update;

		for (j = 0; j < NUM_ENTITY_MESSAGES; j++) message_responses[i][j] = unsupplied_message_response;
	}

	/* original overloads (compiled translation units) */
	overload_group_list_functions ();
	overload_group_int_value_functions ();
	overload_group_float_value_functions ();
	overload_group_vec3d_functions ();
	overload_group_ptr_value_functions ();
	overload_group_update_functions ();
	harness_overload_group_link_parent_responses ();

	overload_update_list_functions ();
	overload_update_message_responses ();
	harness_default_update_link_responses ();

	initialise_entity_create_default_functions ();
	initialise_entity_destroy_default_functions ();

	overload_keysite_int_value_functions ();
	overload_keysite_float_value_functions ();
	overload_keysite_vec3d_functions ();
	overload_keysite_list_functions ();
	harness_overload_keysite_link_child_responses ();

	/* NATIVE: ks_updt.c (keysite update: assignment timer, supply usage and cargo, repair check) */
	overload_keysite_update_functions ();

	overload_force_int_value_functions ();
	overload_force_list_functions ();

	/* fc_msgs.c: only FORCE_LOW_ON_SUPPLIES is adopted; every other force row
	   keeps its fail-loud default */
	overload_force_message_responses ();

	original_force_low_on_supplies = message_responses[ENTITY_TYPE_FORCE][ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES];

	for (j = 0; j < NUM_ENTITY_MESSAGES; j++) message_responses[ENTITY_TYPE_FORCE][j] = unsupplied_message_response;

	message_responses[ENTITY_TYPE_FORCE][ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES] = trace_force_low_on_supplies;

	overload_task_int_value_functions ();
	overload_task_float_value_functions ();
	overload_task_list_functions ();

	/* NATIVE: ts_updt.c (task expiry and stop timers) */
	overload_task_update_functions ();

	overload_pilot_list_functions ();

	overload_task_create_functions ();
	overload_task_ptr_value_functions ();
	harness_overload_task_link_parent_response ();
	harness_overload_group_link_child_response ();
	initialise_group_task_array ();

	for (i = 0; i < NUM_CAMPAIGN_SCREEN_MESSAGE_TARGETS; i++)
	{
		for (j = 0; j < NUM_CAMPAIGN_SCREEN_MESSAGES; j++)
		{
			campaign_screen_message_responses[i][j] = campaign_screen_default;
		}
	}

	campaign_screen_message_responses[CAMPAIGN_SCREEN_TARGET_MISSION_LIST][CAMPAIGN_SCREEN_MISSION_CREATED] = report_mission_created;

	overload_waypoint_int_value_functions ();
	overload_waypoint_list_functions ();
	fn_get_local_entity_float_value[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_TASK_USER_DATA] = harness_default_get_entity_float_value;

	/* cg_funcs.c :: overload_cargo_functions, reduced to the compiled files */
	overload_mobile_int_value_functions (ENTITY_TYPE_CARGO);
	overload_mobile_list_functions (ENTITY_TYPE_CARGO);
	overload_mobile_vec3d_functions (ENTITY_TYPE_CARGO);
	overload_cargo_create_functions ();
	overload_cargo_destroy_functions ();
	overload_cargo_int_value_functions ();
	overload_cargo_list_functions ();
	harness_overload_aircraft_link_parent_responses (ENTITY_TYPE_CARGO);

	overload_sector_create_functions ();
	overload_sector_int_value_functions ();
	overload_sector_list_functions ();
	overload_sector_message_responses ();

	/* hand-written rows */
	shim_root (ENTITY_TYPE_SESSION, LIST_TYPE_FORCE);

	shim_link (ENTITY_TYPE_GUIDE, LIST_TYPE_GUIDE_STACK);

	shim_link (ENTITY_TYPE_HELICOPTER, LIST_TYPE_MEMBER);
	fn_get_local_entity_vec3d_ptr[ENTITY_TYPE_HELICOPTER][VEC3D_TYPE_POSITION] = host_mobile_position;
	overload_aircraft_float_value_functions (ENTITY_TYPE_HELICOPTER);
	fn_get_local_entity_float_value[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_SLEEP] = harness_default_get_entity_float_value;

	shim_link (ENTITY_TYPE_FIXED_WING, LIST_TYPE_MEMBER);
	fn_get_local_entity_vec3d_ptr[ENTITY_TYPE_FIXED_WING][VEC3D_TYPE_POSITION] = host_mobile_position;
	overload_aircraft_float_value_functions (ENTITY_TYPE_FIXED_WING);
	fn_get_local_entity_float_value[ENTITY_TYPE_FIXED_WING][FLOAT_TYPE_SLEEP] = harness_default_get_entity_float_value;
}

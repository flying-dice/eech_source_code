/*
 * EECH C reference harness.
 *
 * Reads one scenario on stdin, builds the entity state, runs the requested
 * operation through the ORIGINAL EECH code and prints the observable outcome.
 * Original code reaches the harness three ways:
 *
 *   - whole translation units compiled unchanged (gp_int.c, gp_float.c,
 *     gp_list.c, gp_vec3d.c, gp_ptr.c, gp_updt.c, gp_dbase.c, up_list.c,
 *     up_msgs.c; see REAL_TRANSLATION_UNITS in extract.mjs);
 *   - verbatim function extracts (build/c-reference/eech_extracted.c);
 *   - whole original headers (build/c-reference/project.h).
 *
 * The harness supplies only:
 *   - the dispatch tables and their fail-loud defaults, filled by the
 *     ORIGINAL overload_*_functions () where those files are compiled;
 *   - hand-written rows for entity types whose files are not compiled yet
 *     (session, force, keysite, guide, helicopter); every such row is listed
 *     in docs/architecture.md, "Shrinking the C reference shim";
 *   - the environment: frame delta time (set_delta_time), comms model,
 *     transport (transmit_entity_comms_message), mobile positions, and debug
 *     output.
 *
 * Input: see test/scenarios/campaign-scenario.ts :: serialiseScenario and
 * test/scenarios/update-timeline.ts :: serialiseTimeline.
 *
 * Output lines (floats as IEEE 754 single precision bit patterns, %08x):
 *
 *   transmit <entity> <float_type> <bits>        ENTITY_COMMS_FLOAT_VALUE sent by a server setter
 *   message <receiver> <sender> <message> <arg>  delivery to FORCE/LOW_ON_SUPPLIES response
 *   closest <entity|NULL> <bits|->               result of op closest
 *   range <bits get_2d_range> <bits get_approx_2d_range>
 *   step <n> <delta bits> <update list labels, comma separated, or ->
 *   timer <sleep bits> <assist_timer bits>       one line per timeline group after each step
 *   result ok | result assert <expression> | result null-dereference
 *
 * A NULL dereference (a fault inside the NULL page) is an EECH outcome and is
 * reported. Any other fatal signal is a harness or original-code defect: the
 * process dies by that signal and the caller treats the run as failed.
 *   final group <ammo bits> <fuel bits>
 *   final keysite <ammo bits> <fuel bits>        one line per keysite, scenario order
 */

#include <setjmp.h>
#include <signal.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>

#include "project.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// dispatch tables (C: en_int.c, en_float.c, en_vec3d.c, en_ptr.c, en_list.c, en_updt.c, en_msgs.c)
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void (*fn_set_local_entity_raw_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES]) (entity *en, int_types type, int value);
void (*fn_set_local_entity_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES]) (entity *en, int_types type, int value);
void (*fn_set_client_server_entity_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, int_types type, int value);
int (*fn_get_local_entity_int_value[NUM_ENTITY_TYPES][NUM_INT_TYPES]) (entity *en, int_types type);

void (*fn_set_local_entity_raw_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES]) (entity *en, float_types type, float value);
void (*fn_set_local_entity_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES]) (entity *en, float_types type, float value);
void (*fn_set_client_server_entity_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, float_types type, float value);
float (*fn_get_local_entity_float_value[NUM_ENTITY_TYPES][NUM_FLOAT_TYPES]) (entity *en, float_types type);

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

/* name databases: only read on debug_fatal paths, which abort the harness */
entity_type_data entity_type_database[NUM_ENTITY_TYPES];
list_type_data list_type_database[NUM_LIST_TYPES];
int_type_data int_type_database[NUM_INT_TYPES];
float_type_data float_type_database[NUM_FLOAT_TYPES];
vec3d_type_data vec3d_type_database[NUM_VEC3D_TYPES];
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
// environment
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define MAX_HARNESS_ENTITIES 256

static entity
	entity_storage[MAX_HARNESS_ENTITIES];

/* en_heap.h: get_local_entity_index (EN) is ((EN) - entities) */
entity
	*entities = entity_storage;

static int
	num_entities;

entity
	*session_entity;

comms_model_types
	system_comms_model = COMMS_MODEL_SERVER;

comms_data_flow_types
	system_comms_data_flow = COMMS_DATA_FLOW_RX;

int
	command_line_entity_update_frame_rate = 2;

FILE
	*tacview_log_file = NULL;		/* tacview logging is excluded: never logging */

static jmp_buf
	abort_operation;

static char
	labels[MAX_HARNESS_ENTITIES][32];

static void harness_fail (const char *what)
{
	fprintf (stderr, "harness: %s\n", what);

	exit (2);
}

/* ASSERT runs in normal control flow (never in a signal handler) */
void harness_assert (const char *expression)
{
	printf ("result assert %s\n", expression);

	longjmp (abort_operation, 1);
}

/* raw data of entity types whose files are not compiled yet (hand-written rows below) */

typedef struct
{
	entity_sub_types
		sub_type;

	entity_sides
		side;

	int
		in_use;

	vec3d
		position;

	supply_type
		supplies;
} shim_keysite;

typedef struct
{
	entity_sides
		side;
} shim_force;

typedef struct
{
	vec3d
		position;		/* physical state: supplied by the scenario */
} shim_mobile;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// final state output and NULL dereference outcome
//
// Everything reachable from the SIGSEGV handler is async-signal-safe: write (),
// reads of harness globals, sigaction () and _exit (). stdout is unbuffered
// (setvbuf in main), so nothing printed before a fault is lost and nothing
// needs flushing.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define NULL_PAGE_SIZE 4096

static group
	*final_group_raw;

static shim_keysite
	*final_keysites;

static int
	final_keysite_count;

static void write_all (const char *text, size_t length)
{
	while (length > 0)
	{
		ssize_t written = write (STDOUT_FILENO, text, length);

		if (written <= 0)
		{
			_exit (3);
		}

		text += written;
		length -= (size_t) written;
	}
}

static void write_text (const char *text)
{
	write_all (text, strlen (text));
}

static void write_float_bits (float value)
{
	static const char
		digits[] = "0123456789abcdef";

	char
		hex[8];

	unsigned int
		bits;

	int
		i;

	memcpy (&bits, &value, sizeof (bits));

	for (i = 7; i >= 0; i--)
	{
		hex[i] = digits[bits & 0xf];
		bits >>= 4;
	}

	write_all (hex, sizeof (hex));
}

static void write_final_pair (const char *prefix, float a, float b)
{
	write_text (prefix);
	write_float_bits (a);
	write_text (" ");
	write_float_bits (b);
	write_text ("\n");
}

static void emit_final_state (void)
{
	int
		i;

	if (final_group_raw)
	{
		write_final_pair ("final group ", final_group_raw->supplies.ammo_supply_level, final_group_raw->supplies.fuel_supply_level);
	}

	for (i = 0; i < final_keysite_count; i++)
	{
		write_final_pair ("final keysite ", final_keysites[i].supplies.ammo_supply_level, final_keysites[i].supplies.fuel_supply_level);
	}
}

/*
 * EECH dereferences NULL entity pointers without a check in release builds
 * (e.g. get_local_entity_type (EN) is ((EN)->type)). The original code runs
 * unguarded here too. A fault inside the NULL page is that outcome: it is
 * reported, the final state is written and the process ends. It is never
 * resumed. Any other fault restores the default action and returns, so the
 * faulting instruction re-executes and the process dies by SIGSEGV.
 */
static void segmentation_fault (int signal_number, siginfo_t *info, void *context)
{
	struct sigaction
		default_action;

	if ((uintptr_t) info->si_addr < NULL_PAGE_SIZE)
	{
		write_text ("result null-dereference\n");

		emit_final_state ();

		_exit (0);
	}

	memset (&default_action, 0, sizeof (default_action));

	default_action.sa_handler = SIG_DFL;

	sigemptyset (&default_action.sa_mask);

	sigaction (SIGSEGV, &default_action, NULL);
}

static void install_segmentation_fault_handler (void)
{
	struct sigaction
		action;

	memset (&action, 0, sizeof (action));

	action.sa_sigaction = segmentation_fault;

	action.sa_flags = SA_SIGINFO | SA_RESETHAND;

	sigemptyset (&action.sa_mask);

	sigaction (SIGSEGV, &action, NULL);
}

void debug_fatal (const char *string, ...)
{
	fprintf (stderr, "harness: debug_fatal: %s\n", string);

	exit (2);
}

void debug_log (const char *string, ...)
{
}

void debug_log_entity_message (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
}

int tacview_reset_frame (void)
{
	harness_fail ("tacview is excluded");

	return 0;
}

void write_tacview_frame_header (void)
{
	harness_fail ("tacview is excluded");
}

void add_group_type_to_force_info (entity *en, entity_sub_types group_type)
{
	harness_fail ("add_group_type_to_force_info (force.c) is not compiled");
}

void remove_group_type_from_force_info (entity *en, entity_sub_types group_type)
{
	harness_fail ("remove_group_type_from_force_info (force.c) is not compiled");
}

int set_local_division_name (entity *en, char *s)
{
	harness_fail ("set_local_division_name (division.c) is not compiled");

	return 0;
}

static const char *label_of (entity *en)
{
	return en ? labels[en - entities] : "NULL";
}

static unsigned int float_bits (float value)
{
	unsigned int
		bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

/* transport: en_comms.c transmit_entity_comms_message; only float values occur */
void transmit_entity_comms_message (entity_comms_messages message, entity *en, ...)
{
	va_list
		pargs;

	float_types
		type;

	float
		value;

	if (message != ENTITY_COMMS_FLOAT_VALUE)
	{
		harness_fail ("unexpected entity comms message");
	}

	va_start (pargs, en);

	type = va_arg (pargs, float_types);

	value = va_arg (pargs, double);

	va_end (pargs);

	printf ("transmit %s %d %08x\n", label_of (en), (int) type, float_bits (value));
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// fail-loud table defaults
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int unsupplied_get_int (entity *en, int_types type) { harness_fail ("int value not supplied"); return 0; }
static void unsupplied_set_int (entity *en, int_types type, int value) { harness_fail ("int set not supplied"); }
static float unsupplied_get_float (entity *en, float_types type) { harness_fail ("float value not supplied"); return 0.0f; }
static void unsupplied_set_float (entity *en, float_types type, float value) { harness_fail ("float set not supplied"); }
static vec3d *unsupplied_get_vec3d_ptr (entity *en, vec3d_types type) { harness_fail ("vec3d not supplied"); return NULL; }
static void *unsupplied_get_ptr (entity *en, ptr_types type) { harness_fail ("ptr value not supplied"); return NULL; }
static entity *unsupplied_get_list (entity *en, list_types type) { harness_fail ("list not supplied"); return NULL; }
static void unsupplied_set_list (entity *en, list_types type, entity *other) { harness_fail ("list set not supplied"); }
static void unsupplied_update (entity *en) { harness_fail ("update function not supplied"); }
static int unsupplied_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs) { harness_fail ("message response not supplied"); return FALSE; }

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// hand-written rows for entity types whose files are not compiled yet
// (docs/architecture.md, "Shrinking the C reference shim")
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


/* list storage for shim entity types */
static entity
	*shim_first_child[MAX_HARNESS_ENTITIES][NUM_LIST_TYPES],
	*shim_parent[MAX_HARNESS_ENTITIES][NUM_LIST_TYPES],
	*shim_child_succ[MAX_HARNESS_ENTITIES][NUM_LIST_TYPES],
	*shim_child_pred[MAX_HARNESS_ENTITIES][NUM_LIST_TYPES];

static entity *shim_get_first_child (entity *en, list_types type) { return shim_first_child[en - entities][type]; }
static void shim_set_first_child (entity *en, list_types type, entity *v) { shim_first_child[en - entities][type] = v; }
static entity *shim_get_parent (entity *en, list_types type) { return shim_parent[en - entities][type]; }
static void shim_set_parent (entity *en, list_types type, entity *v) { shim_parent[en - entities][type] = v; }
static entity *shim_get_child_succ (entity *en, list_types type) { return shim_child_succ[en - entities][type]; }
static void shim_set_child_succ (entity *en, list_types type, entity *v) { shim_child_succ[en - entities][type] = v; }
static entity *shim_get_child_pred (entity *en, list_types type) { return shim_child_pred[en - entities][type]; }
static void shim_set_child_pred (entity *en, list_types type, entity *v) { shim_child_pred[en - entities][type] = v; }

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

/* ks_int.c, ks_float.c, ks_vec3d.c */
static int shim_keysite_int (entity *en, int_types type)
{
	shim_keysite *raw = (shim_keysite *) get_local_entity_data (en);

	if (type == INT_TYPE_ENTITY_SUB_TYPE) return raw->sub_type;
	if (type == INT_TYPE_IN_USE) return raw->in_use;

	harness_fail ("keysite int value not supplied");

	return 0;
}

static float shim_keysite_float (entity *en, float_types type)
{
	shim_keysite *raw = (shim_keysite *) get_local_entity_data (en);

	return (type == FLOAT_TYPE_AMMO_SUPPLY_LEVEL) ? raw->supplies.ammo_supply_level : raw->supplies.fuel_supply_level;
}

static void shim_keysite_set_server_float (entity *en, float_types type, float value)
{
	shim_keysite *raw = (shim_keysite *) get_local_entity_data (en);

	if (type == FLOAT_TYPE_AMMO_SUPPLY_LEVEL) raw->supplies.ammo_supply_level = value;
	else raw->supplies.fuel_supply_level = value;

	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}

static vec3d *shim_keysite_position (entity *en, vec3d_types type)
{
	return &((shim_keysite *) get_local_entity_data (en))->position;
}

/* fc_int.c */
static int shim_force_int (entity *en, int_types type)
{
	return ((shim_force *) get_local_entity_data (en))->side;
}

/* ac_vec3d.c: physical position from the scenario (environment) */
static vec3d *shim_mobile_position (entity *en, vec3d_types type)
{
	return &((shim_mobile *) get_local_entity_data (en))->position;
}

static int record_force_low_on_supplies (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	int
		sub_type = va_arg (pargs, int);

	printf ("message %s %s %d %d\n", label_of (receiver), label_of (sender), (int) message, sub_type);

	return FALSE;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// initialisation (C: initialise_entity_functions, reduced to the adopted entity types)
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static void initialise_tables (void)
{
	int
		i,
		j,
		k;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_INT_TYPES; j++)
		{
			/* en_int.c defaults: sets do nothing; gets are not supplied */
			fn_set_local_entity_raw_int_value[i][j] = harness_default_set_entity_int_value;
			fn_set_local_entity_int_value[i][j] = harness_default_set_entity_int_value;
			for (k = 0; k < NUM_COMMS_MODEL_TYPES; k++) fn_set_client_server_entity_int_value[i][j][k] = unsupplied_set_int;
			fn_get_local_entity_int_value[i][j] = unsupplied_get_int;
		}

		for (j = 0; j < NUM_FLOAT_TYPES; j++)
		{
			/* en_float.c defaults: sets do nothing; gets are not supplied */
			fn_set_local_entity_raw_float_value[i][j] = harness_default_set_entity_float_value;
			fn_set_local_entity_float_value[i][j] = harness_default_set_entity_float_value;
			for (k = 0; k < NUM_COMMS_MODEL_TYPES; k++) fn_set_client_server_entity_float_value[i][j][k] = unsupplied_set_float;
			fn_get_local_entity_float_value[i][j] = unsupplied_get_float;
		}

		for (j = 0; j < NUM_VEC3D_TYPES; j++) fn_get_local_entity_vec3d_ptr[i][j] = unsupplied_get_vec3d_ptr;

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

	/* hand-written rows */
	shim_root (ENTITY_TYPE_SESSION, LIST_TYPE_FORCE);

	shim_link (ENTITY_TYPE_FORCE, LIST_TYPE_FORCE);
	shim_root (ENTITY_TYPE_FORCE, LIST_TYPE_KEYSITE_FORCE);
	shim_root (ENTITY_TYPE_FORCE, LIST_TYPE_INDEPENDENT_GROUP);
	fn_get_local_entity_int_value[ENTITY_TYPE_FORCE][INT_TYPE_SIDE] = shim_force_int;

	shim_link (ENTITY_TYPE_KEYSITE, LIST_TYPE_KEYSITE_FORCE);
	shim_root (ENTITY_TYPE_KEYSITE, LIST_TYPE_KEYSITE_GROUP);
	shim_root (ENTITY_TYPE_KEYSITE, LIST_TYPE_BUILDING_GROUP);
	fn_get_local_entity_int_value[ENTITY_TYPE_KEYSITE][INT_TYPE_ENTITY_SUB_TYPE] = shim_keysite_int;
	fn_get_local_entity_int_value[ENTITY_TYPE_KEYSITE][INT_TYPE_IN_USE] = shim_keysite_int;
	fn_get_local_entity_float_value[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL] = shim_keysite_float;
	fn_get_local_entity_float_value[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL] = shim_keysite_float;
	fn_set_client_server_entity_float_value[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL][COMMS_MODEL_SERVER] = shim_keysite_set_server_float;
	fn_set_client_server_entity_float_value[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_SERVER] = shim_keysite_set_server_float;
	fn_get_local_entity_vec3d_ptr[ENTITY_TYPE_KEYSITE][VEC3D_TYPE_POSITION] = shim_keysite_position;

	shim_link (ENTITY_TYPE_GUIDE, LIST_TYPE_GUIDE_STACK);

	shim_link (ENTITY_TYPE_HELICOPTER, LIST_TYPE_MEMBER);
	fn_get_local_entity_vec3d_ptr[ENTITY_TYPE_HELICOPTER][VEC3D_TYPE_POSITION] = shim_mobile_position;

	message_responses[ENTITY_TYPE_FORCE][ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES] = record_force_low_on_supplies;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// scenario construction
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static entity *new_entity (entity_types type, void *data, const char *label)
{
	entity
		*en;

	if (num_entities >= MAX_HARNESS_ENTITIES)
	{
		harness_fail ("too many entities");
	}

	en = &entities[num_entities];

	memset (en, 0, sizeof (*en));

	set_local_entity_type (en, type);
	set_local_entity_data (en, data);

	snprintf (labels[num_entities], sizeof (labels[num_entities]), "%s", label);

	num_entities++;

	return en;
}

/*
 * List membership as a restored campaign holds it: the pointer updates of
 * insert_local_entity_into_parents_child_list through the dispatch tables,
 * without its LINK_CHILD / LINK_PARENT notifications (the TS scenario
 * builder's insertLocalEntityIntoParentsChildListRaw does the same).
 */
static void link_entity_raw (entity *en, list_types type, entity *parent, entity *pred)
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

static entity *last_child (entity *parent, list_types type)
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

static char *next_token (char **cursor)
{
	char *token = strtok (*cursor, " \t\r\n");

	*cursor = NULL;

	if (!token) harness_fail ("truncated scenario line");

	return token;
}

static int next_int (char **cursor) { return atoi (next_token (cursor)); }

static double next_double (char **cursor) { return strtod (next_token (cursor), NULL); }

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static group
	harness_groups[MAX_HARNESS_ENTITIES];

static int
	num_groups;

static entity
	*groups[MAX_HARNESS_ENTITIES];

static void print_timeline_state (int step, entity *update_root)
{
	entity
		*en;

	int
		i;

	printf ("step %d %08x ", step, float_bits (get_delta_time ()));

	en = get_local_entity_first_child (update_root, LIST_TYPE_UPDATE);

	if (!en)
	{
		printf ("-");
	}

	while (en)
	{
		printf ("%s%s", label_of (en), get_local_entity_child_succ (en, LIST_TYPE_UPDATE) ? "," : "");

		en = get_local_entity_child_succ (en, LIST_TYPE_UPDATE);
	}

	printf ("\n");

	for (i = 0; i < num_groups; i++)
	{
		printf ("timer %08x %08x\n", float_bits (harness_groups[i].sleep), float_bits (harness_groups[i].assist_timer));
	}
}

int main (void)
{
	static shim_force force_data[MAX_HARNESS_ENTITIES];
	static shim_keysite keysite_data[MAX_HARNESS_ENTITIES];
	static shim_mobile leader_data;
	static update update_data;

	entity
		*session,
		*update_root,
		*forces[MAX_HARNESS_ENTITIES],
		*keysites[MAX_HARNESS_ENTITIES],
		*keysite_tail[NUM_ENTITY_SIDES] = { NULL },
		*group_en = NULL;

	int
		num_forces = 0,
		num_keysites = 0,
		timeline = FALSE,
		step = 0,
		i;

	char
		line[512],
		*cursor,
		*word;

	/* unbuffered: output written before a fault is never lost, and the fault
	   handler never needs to flush */
	setvbuf (stdout, NULL, _IONBF, 0);

	initialise_tables ();

	install_segmentation_fault_handler ();

	final_keysites = keysite_data;

	session = new_entity (ENTITY_TYPE_SESSION, NULL, "session");

	update_root = new_entity (ENTITY_TYPE_UPDATE, &update_data, "update");

	/* up_update.c :: set_update_entity (not compiled whole; it only assigns and logs) */
	update_entity = update_root;

	while (fgets (line, sizeof (line), stdin))
	{
		cursor = line;

		word = next_token (&cursor);

		if (strcmp (word, "session") == 0)
		{
			session_entity = next_int (&cursor) ? session : NULL;
		}
		else if (strcmp (word, "force") == 0)
		{
			char label[32];

			force_data[num_forces].side = (entity_sides) next_int (&cursor);

			snprintf (label, sizeof (label), "force%d", num_forces);

			forces[num_forces] = new_entity (ENTITY_TYPE_FORCE, &force_data[num_forces], label);

			link_entity_raw (forces[num_forces], LIST_TYPE_FORCE, session, num_forces ? forces[num_forces - 1] : NULL);

			num_forces++;
		}
		else if (strcmp (word, "keysite") == 0)
		{
			char label[32];

			shim_keysite *raw = &keysite_data[num_keysites];

			raw->side = (entity_sides) next_int (&cursor);
			raw->sub_type = next_int (&cursor);
			raw->in_use = next_int (&cursor);
			raw->position.x = next_double (&cursor);
			raw->position.y = 0.0;
			raw->position.z = next_double (&cursor);
			raw->supplies.ammo_supply_level = next_double (&cursor);
			raw->supplies.fuel_supply_level = next_double (&cursor);

			snprintf (label, sizeof (label), "keysite%d", num_keysites);

			keysites[num_keysites] = new_entity (ENTITY_TYPE_KEYSITE, raw, label);

			for (i = 0; i < num_forces; i++)
			{
				if (force_data[i].side == raw->side)
				{
					link_entity_raw (keysites[num_keysites], LIST_TYPE_KEYSITE_FORCE, forces[i], keysite_tail[raw->side]);

					keysite_tail[raw->side] = keysites[num_keysites];

					break;
				}
			}

			num_keysites++;

			final_keysite_count = num_keysites;
		}
		else if (strcmp (word, "group") == 0)
		{
			/* slice 1 group: raw fields as a restored campaign holds them */
			int parent_kind, parent_index, busy, has_leader;

			group *raw = &harness_groups[num_groups];

			memset (raw, 0, sizeof (*raw));

			raw->sub_type = next_int (&cursor);
			raw->side = (entity_sides) next_int (&cursor);
			raw->supplies.ammo_supply_level = next_double (&cursor);
			raw->supplies.fuel_supply_level = next_double (&cursor);

			parent_kind = next_int (&cursor);
			parent_index = next_int (&cursor);
			busy = next_int (&cursor);
			has_leader = next_int (&cursor);

			leader_data.position.x = next_double (&cursor);
			leader_data.position.y = 0.0;
			leader_data.position.z = next_double (&cursor);

			group_en = new_entity (ENTITY_TYPE_GROUP, raw, "group");

			final_group_raw = raw;

			num_groups++;

			if (parent_kind == 1)
			{
				link_entity_raw (group_en, LIST_TYPE_KEYSITE_GROUP, keysites[parent_index], NULL);
			}
			else if (parent_kind == 2)
			{
				for (i = 0; i < num_forces; i++)
				{
					if (force_data[i].side == raw->side)
					{
						link_entity_raw (group_en, LIST_TYPE_INDEPENDENT_GROUP, forces[i], NULL);

						break;
					}
				}
			}

			if (busy)
			{
				link_entity_raw (new_entity (ENTITY_TYPE_GUIDE, NULL, "guide"), LIST_TYPE_GUIDE_STACK, group_en, NULL);
			}

			if (has_leader)
			{
				link_entity_raw (new_entity (ENTITY_TYPE_HELICOPTER, &leader_data, "leader"), LIST_TYPE_MEMBER, group_en, NULL);
			}
		}
		else if (strcmp (word, "timeline") == 0)
		{
			timeline = TRUE;

			command_line_entity_update_frame_rate = next_int (&cursor);
		}
		else if (strcmp (word, "tgroup") == 0)
		{
			/* timeline group: raw fields and update list membership as a restored campaign holds them */
			char label[32];

			group *raw = &harness_groups[num_groups];

			memset (raw, 0, sizeof (*raw));

			raw->sub_type = next_int (&cursor);
			raw->side = (entity_sides) next_int (&cursor);
			raw->sleep = next_double (&cursor);
			raw->assist_timer = next_double (&cursor);

			snprintf (label, sizeof (label), "group%d", num_groups);

			groups[num_groups] = new_entity (ENTITY_TYPE_GROUP, raw, label);

			if (next_int (&cursor))
			{
				link_entity_raw (groups[num_groups], LIST_TYPE_UPDATE, update_root, last_child (update_root, LIST_TYPE_UPDATE));
			}

			num_groups++;
		}
		else if ((strcmp (word, "set") == 0) || (strcmp (word, "frame") == 0))
		{
			int is_set = (strcmp (word, "set") == 0);
			int group_index = 0, float_type = 0, locked = 0, count = 0, c;
			double value, delta = 0.0;

			if (is_set)
			{
				group_index = next_int (&cursor);
				float_type = next_int (&cursor);
				value = next_double (&cursor);
			}
			else
			{
				delta = next_double (&cursor);
				locked = next_int (&cursor);
				count = next_int (&cursor);
			}

			if (setjmp (abort_operation) != 0)
			{
				return 0;
			}

			if (is_set)
			{
				set_client_server_entity_float_value (groups[group_index], (float_types) float_type, value);
			}
			else
			{
				/* environment: set_delta_time (time.c) measures the frame; the scenario supplies it */
				system_delta_time = delta;
				system_one_over_delta_time = 1.0 / system_delta_time;
				locked_frame_rate = locked;

				/* host flight loop (flight.c): one update per time acceleration step */
				for (c = 0; c < count; c++)
				{
					update_client_server_entities ();
				}
			}

			print_timeline_state (step, update_root);

			step++;
		}
		else if (strcmp (word, "end") == 0)
		{
			if (timeline)
			{
				printf ("result ok\n");

				return 0;
			}
		}
		else if (strcmp (word, "op") == 0)
		{
			word = next_token (&cursor);

			if (setjmp (abort_operation) == 0)
			{
				if (strcmp (word, "assess") == 0)
				{
					assess_group_supplies (group_en);
				}
				else if (strcmp (word, "closest") == 0)
				{
					entity_sub_types type = next_int (&cursor);
					entity_sides side = (entity_sides) next_int (&cursor);
					int has_pos = next_int (&cursor);
					vec3d pos;
					float min_range;
					float actual_range = -1.0f;
					int want_range;
					int outside_of_range;
					int exclude_index;
					entity *closest;

					pos.x = next_double (&cursor);
					pos.y = 0.0;
					pos.z = next_double (&cursor);
					min_range = next_double (&cursor);
					want_range = next_int (&cursor);
					outside_of_range = next_int (&cursor);
					exclude_index = next_int (&cursor);

					closest = get_closest_keysite (type, side, has_pos ? &pos : NULL, min_range, want_range ? &actual_range : NULL, outside_of_range, exclude_index >= 0 ? keysites[exclude_index] : NULL);

					if (want_range)
					{
						printf ("closest %s %08x\n", label_of (closest), float_bits (actual_range));
					}
					else
					{
						printf ("closest %s -\n", label_of (closest));
					}
				}
				else if (strcmp (word, "range") == 0)
				{
					vec3d v1, v2;

					v1.x = next_double (&cursor);
					v1.y = 0.0;
					v1.z = next_double (&cursor);
					v2.x = next_double (&cursor);
					v2.y = 0.0;
					v2.z = next_double (&cursor);

					printf ("range %08x %08x\n", float_bits (get_2d_range (&v1, &v2)), float_bits (get_approx_2d_range (&v1, &v2)));
				}
				else if (strcmp (word, "fault-null") == 0)
				{
					/* harness self-test: a read inside the NULL page */
					volatile int *volatile null_pointer = NULL;

					printf ("read %d\n", null_pointer[1]);
				}
				else if (strcmp (word, "fault-unmapped") == 0)
				{
					/* harness self-test: a fault outside the NULL page must kill the process */
					volatile int *guard = mmap (NULL, 65536, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

					if ((void *) guard == MAP_FAILED)
					{
						harness_fail ("mmap failed");
					}

					printf ("read %d\n", guard[4096]);
				}
				else
				{
					harness_fail ("unknown op");
				}

				printf ("result ok\n");
			}

			emit_final_state ();

			return 0;
		}
		else
		{
			harness_fail ("unknown scenario line");
		}
	}

	harness_fail ("scenario has no op or end line");

	return 1;
}

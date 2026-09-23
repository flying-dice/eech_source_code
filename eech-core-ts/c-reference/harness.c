/*
 * EECH C reference harness.
 *
 * Reads one scenario on stdin, builds the entity state, runs the requested
 * operation through the ORIGINAL EECH code and prints the observable outcome.
 * Original code reaches the harness three ways:
 *
 *   - whole translation units compiled unchanged (group, update, keysite and
 *     force accessors, the entity heap, attributes, creation and destruction,
 *     mobile, cargo and sector files; see REAL_TRANSLATION_UNITS in
 *     extract.mjs);
 *   - verbatim function extracts (build/c-reference/eech_extracted*.c);
 *   - whole original headers (build/c-reference/project.h).
 *
 * The harness supplies only:
 *   - the dispatch tables and their fail-loud defaults, filled by the
 *     ORIGINAL overload_*_functions () where those files are compiled;
 *   - hand-written rows for entity types whose files are not compiled yet
 *     (session, guide, helicopter); every such row is listed in
 *     docs/architecture.md, "Shrinking the C reference shim";
 *   - the environment: frame delta time (set_delta_time), comms model,
 *     transport (transmit_entity_comms_message), mobile positions, memory,
 *     float-to-int conversion, and debug output;
 *   - fail-loud stubs for functions the compiled files reference but the
 *     adopted paths never reach.
 *
 * Input: see test/scenarios/campaign-scenario.ts :: serialiseScenario and
 * test/scenarios/update-timeline.ts :: serialiseTimeline.
 *
 * Output lines (floats as IEEE 754 single precision bit patterns, %08x):
 *
 *   transmit <entity> <float_type> <bits>        ENTITY_COMMS_FLOAT_VALUE sent by a server setter
 *   transmit-create <type> <index> <attributes>  ENTITY_COMMS_CREATE (attributes as in the scenario line)
 *   transmit-destroy <entity>                    ENTITY_COMMS_DESTROY
 *   created <label> <index|NULL>                 result of a create line
 *   allocated <label> <index>                    result of an allocate line
 *   message <receiver> <sender> <message> <arg>  delivery to FORCE/LOW_ON_SUPPLIES response
 *   closest <entity|NULL> <bits|->               result of op closest
 *   range <bits get_2d_range> <bits get_approx_2d_range>
 *   step <n> <delta bits> <update list labels, comma separated, or ->
 *   timer <sleep bits> <assist_timer bits>       one line per timeline group after each step
 *   result ok | result assert <expression> | result fatal <format> | result null-dereference
 *
 * A NULL dereference (a fault inside the NULL page) is an EECH outcome and is
 * reported. Any other fatal signal is a harness or original-code defect: the
 * process dies by that signal and the caller treats the run as failed.
 *   final group <ammo bits> <fuel bits>
 *   final keysite <ammo bits> <fuel bits>        one line per keysite, scenario order
 *
 * Lifecycle scenarios (heap / map / create / destroy lines) end with the
 * entity graph, see print_lifecycle_state.
 */

#include <setjmp.h>
#include <signal.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>

#include "project.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Floating-point environment (docs/fidelity/fpu-semantics.md).
//
// EECH runs its campaign thread with the FPU rounding toward zero
// (startup.c, start_application :: set_fpu_rounding_mode_zero, re-asserted
// after every library initialisation). The canonical oracle evaluates C float
// arithmetic at declared type (SSE, FLT_EVAL_METHOD 0) with SSE rounding
// toward zero, and sets the x87 rounding control to chop as well, so libm's x87
// paths (sqrt) and fistp agree. The x87 precision control is left at the
// platform default: whether EECH's intermediates were evaluated beyond declared
// type is unresolved (issue #7) and is not modelled.
//
// The environment is installed before any scenario line runs and checked
// before every line and at exit; a drift aborts with status 4. The `fpu`
// command reports it (test/c-reference/fpu-environment.cref.test.ts).
//
// Scenario input is parsed and narrowed under round to nearest (INPUT_FP_*):
// it is the scenario's value, not a result of EECH arithmetic.
//
// INVESTIGATION ONLY (issue #7): a build with HARNESS_FPU_VARIANT may install
// another x87 control word (HARNESS_X87_CW) or SSE rounding (HARNESS_MXCSR_RC),
// and with HARNESS_FISTP converts floats to ints with fistp.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifndef HARNESS_X87_CW
#define HARNESS_X87_CW 0x0f7f		/* round toward zero; platform default precision (64), exceptions masked */
#endif

#ifndef HARNESS_MXCSR_RC
#define HARNESS_MXCSR_RC 0x6000		/* round toward zero */
#endif

static unsigned short read_x87_control_word (void)
{
	unsigned short
		cw;

	__asm__ __volatile__ ("fnstcw %0" : "=m" (cw));

	return cw;
}

static void write_x87_control_word (unsigned short cw)
{
	__asm__ __volatile__ ("fclex; fldcw %0" : : "m" (cw));
}

static unsigned int read_mxcsr (void)
{
	unsigned int
		mxcsr;

	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));

	return mxcsr;
}

static void write_mxcsr (unsigned int mxcsr)
{
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (mxcsr));
}

#define X87_RC_MASK 0x0c00

#define MXCSR_RC_MASK 0x6000

/* MXCSR bits 0-5 are sticky exception flags, not control */
#define MXCSR_FLAGS 0x003f

static unsigned short
	installed_x87_cw;

static unsigned int
	installed_mxcsr;

static void install_fpu_environment (void)
{
	write_x87_control_word (HARNESS_X87_CW);

	write_mxcsr ((read_mxcsr () & ~MXCSR_RC_MASK) | HARNESS_MXCSR_RC);

	installed_x87_cw = read_x87_control_word ();

	installed_mxcsr = read_mxcsr () & ~MXCSR_FLAGS;
}

static void check_fpu_environment (void)
{
	if ((read_x87_control_word () != installed_x87_cw) || ((read_mxcsr () & ~MXCSR_FLAGS) != installed_mxcsr))
	{
		fprintf (stderr, "floating-point environment drifted: cw %04x mxcsr %08x\n", read_x87_control_word (), read_mxcsr ());

		_exit (4);
	}
}

// input: round to nearest in both units
#define INPUT_FP_BEGIN	{ unsigned short input_cw = read_x87_control_word (); unsigned int input_mxcsr = read_mxcsr (); \
	write_x87_control_word (input_cw & ~X87_RC_MASK); write_mxcsr (input_mxcsr & ~MXCSR_RC_MASK);
#define INPUT_FP_END	write_x87_control_word (input_cw); write_mxcsr (input_mxcsr); }

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

/* name databases: only read on debug_fatal paths, which end the operation */
entity_type_data entity_type_database[NUM_ENTITY_TYPES];
list_type_data list_type_database[NUM_LIST_TYPES];
int_type_data int_type_database[NUM_INT_TYPES];
float_type_data float_type_database[NUM_FLOAT_TYPES];
vec3d_type_data vec3d_type_database[NUM_VEC3D_TYPES];
static const char *harness_entity_type_names[NUM_ENTITY_TYPES];
const char **entity_type_names = harness_entity_type_names;
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

/* the largest entity heap a scenario may request (en_heap.c owns the heap) */
#define MAX_HARNESS_ENTITIES 1024

entity
	*session_entity;

/* helicop.h: the player's gunship. The campaign core has none. */
entity
	*gunship_entity = NULL;

comms_model_types
	system_comms_model = COMMS_MODEL_SERVER;

/* the server simulation transmits (entity creation uses stack attributes) */
comms_data_flow_types
	system_comms_data_flow = COMMS_DATA_FLOW_TX;

int
	command_line_entity_update_frame_rate = 2,
	command_line_downwash = FALSE;

FILE
	*tacview_log_file = NULL;		/* tacview logging is excluded: never logging */

static jmp_buf
	abort_operation;

static char
	labels[MAX_HARNESS_ENTITIES][32];

/* TRUE while original code runs under a setjmp (abort_operation) */
static int
	in_operation = FALSE;

static void harness_fail (const char *what)
{
	fprintf (stderr, "harness: %s\n", what);

	exit (2);
}

/* ASSERT runs in normal control flow (never in a signal handler) */
void harness_assert (const char *expression)
{
	if (!in_operation)
	{
		fprintf (stderr, "harness: ASSERT outside an operation: %s\n", expression);

		exit (2);
	}

	printf ("result assert %s\n", expression);

	longjmp (abort_operation, 1);
}

/* raw data of entity types whose files are not compiled yet (hand-written rows below) */

typedef struct
{
	vec3d
		position;		/* physical state: supplied by the scenario */
} shim_mobile;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// final state output and NULL dereference outcome
//
// The SIGSEGV handler's call graph contains no C library call except write ()
// and _exit (): fixed strings have compile-time lengths, and float bits are
// read through a union and hex-encoded by hand. The same code writes the final
// state on the normal path, so both paths produce identical text. stdout is
// unbuffered (setvbuf in main), so nothing printed before a fault is lost.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define NULL_PAGE_SIZE 4096

/* a string literal and its length, without the terminator */
#define LITERAL(TEXT) (TEXT), (sizeof (TEXT) - 1)

static group
	*final_group_raw;

static keysite
	*final_keysites[MAX_HARNESS_ENTITIES];

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

static void write_float_bits (float value)
{
	union
	{
		float
			f;

		unsigned int
			u;
	} bits;

	char
		hex[8];

	int
		i;

	bits.f = value;

	for (i = 7; i >= 0; i--)
	{
		hex[i] = "0123456789abcdef"[bits.u & 0xf];
		bits.u >>= 4;
	}

	write_all (hex, sizeof (hex));
}

static void write_final_pair (const char *prefix, size_t prefix_length, float a, float b)
{
	write_all (prefix, prefix_length);
	write_float_bits (a);
	write_all (LITERAL (" "));
	write_float_bits (b);
	write_all (LITERAL ("\n"));
}

static void emit_final_state (void)
{
	int
		i;

	if (final_group_raw)
	{
		write_final_pair (LITERAL ("final group "), final_group_raw->supplies.ammo_supply_level, final_group_raw->supplies.fuel_supply_level);
	}

	for (i = 0; i < final_keysite_count; i++)
	{
		write_final_pair (LITERAL ("final keysite "), final_keysites[i]->supplies.ammo_supply_level, final_keysites[i]->supplies.fuel_supply_level);
	}
}

/*
 * EECH dereferences NULL entity pointers without a check in release builds
 * (e.g. get_local_entity_type (EN) is ((EN)->type)). The original code runs
 * unguarded here too. A fault inside the NULL page is that outcome: it is
 * reported, the final state is written and the process ends. It is never
 * resumed.
 *
 * SA_RESETHAND has already restored the default action on entry, so for any
 * other fault the handler simply returns: the faulting instruction re-executes
 * and the process dies by SIGSEGV, which the test driver reports as a failure.
 */
static void segmentation_fault (int signal_number, siginfo_t *info, void *context)
{
	if ((uintptr_t) info->si_addr < NULL_PAGE_SIZE)
	{
		write_all (LITERAL ("result null-dereference\n"));

		emit_final_state ();

		_exit (0);
	}
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

/*
 * debug_fatal is an EECH outcome (the game stops). It is reported with its
 * format string, and the operation ends like a failed ASSERT. Outside an
 * operation it is a scenario error.
 */
void debug_fatal (const char *string, ...)
{
	if (!in_operation)
	{
		fprintf (stderr, "harness: debug_fatal outside an operation: %s\n", string);

		exit (2);
	}

	printf ("result fatal %s\n", string);

	longjmp (abort_operation, 1);
}

void debug_colour_log (enum DEBUG_COLOURS colour, const char *string, ...)
{
}

/* modules/system/memblock.h: the engine allocator; the platform one here */
void *malloc_fast_mem (int size)
{
	void *ptr = malloc ((size_t) size);

	if (!ptr) harness_fail ("out of memory");

	return ptr;
}

void *malloc_heap_mem (int size)
{
	return malloc_fast_mem (size);
}

void free_mem (void *ptr)
{
	free (ptr);
}

/*
 * modules/system/fpu.c :: convert_float_to_int is x87 fistp, which rounds
 * with the FPU control word. EECH sets round-toward-zero at start-up
 * (startup.c, set_fpu_rounding_mode_zero), so the conversion truncates: the C
 * (int) cast. See docs/slices/entity-lifecycle-cargo.md.
 */
void convert_float_to_int (float value, int *ptr)
{
#ifdef HARNESS_FISTP
	/* modules/system/fpu.h (__WATCOMC__ / MSVC) :: asm_convert_float_to_int,
	   fistp dword ptr. (The __GNUC__ branch there writes "fistp (%1)", which
	   GNU as assembles as the 16-bit fistps; not modelled.) */
	__asm__ __volatile__ ("fistpl (%1)" : : "t" (value), "r" (ptr) : "memory", "st");
#else
	*ptr = (int) value;
#endif
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// fail-loud stubs: referenced by compiled original files, never reached by the
// adopted paths (docs/architecture.md, "Shrinking the C reference shim")
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define NOT_REACHED(WHAT) harness_fail (WHAT " is not part of the port (reached unexpectedly)")

/* en_pack.c / modules/multi: save games and network messages */
void pack_signed_data (int unpacked_data, int number_of_bits_to_pack) { NOT_REACHED ("pack_signed_data"); }
void pack_unsigned_data (unsigned int unpacked_data, int number_of_bits_to_pack) { NOT_REACHED ("pack_unsigned_data"); }
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
void pack_vec3d (entity *en, vec3d_types type, vec3d *v) { NOT_REACHED ("pack_vec3d"); }
void unpack_vec3d (entity *en, vec3d_types type, vec3d *v) { NOT_REACHED ("unpack_vec3d"); }

/* comms.c / en_comms.c: only create_local_only_entities switches these */
void set_comms_model (comms_model_types model) { NOT_REACHED ("set_comms_model"); }
void set_comms_data_flow (comms_data_flow_types data_flow) { NOT_REACHED ("set_comms_data_flow"); }
void enable_entity_comms_messages (void) { NOT_REACHED ("enable_entity_comms_messages"); }
void disable_entity_comms_messages (void) { NOT_REACHED ("disable_entity_comms_messages"); }

/* local-only entity types other than sectors (en_creat.c / en_dstry.c) */
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

/* sector responses for fixed entities, aircraft and vehicles (sc_msgs.c, sector.c) */
void set_sector_fog_of_war_value (entity *en, entity *sector_en) { NOT_REACHED ("set_sector_fog_of_war_value"); }
void update_imap_surface_to_air_defence_level (entity *en, entity *sector, int in_use) { NOT_REACHED ("update_imap_surface_to_air_defence_level"); }
void update_imap_surface_to_surface_defence_level (entity *en, entity *sector, int in_use) { NOT_REACHED ("update_imap_surface_to_surface_defence_level"); }
int get_valid_current_game_session (void) { NOT_REACHED ("get_valid_current_game_session"); return FALSE; }
session_list_types get_current_game_session_type (void) { NOT_REACHED ("get_current_game_session_type"); return SESSION_LIST_TYPE_INVALID; }

/* ks_int.c: landing sites are not ported */
entity *get_local_group_member_landing_entity_from_keysite (entity *en) { NOT_REACHED ("get_local_group_member_landing_entity_from_keysite"); return NULL; }

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

/*
 * Prints an attribute buffer (en_attrs.c layout, read with get_list_item as
 * pack_entity_attributes does) in the scenario's create-line syntax, with
 * doubles narrowed to the floats the receiver stores.
 */
static void print_attributes (const char *buffer)
{
	entity_attributes
		attr;

	while (TRUE)
	{
		attr = get_list_item (buffer, entity_attributes);

		switch (attr)
		{
			case entity_attr_end:
			{
				printf (" end");

				return;
			}
			case entity_attr_int_value:
			{
				int type = get_list_item (buffer, int_types);
				int value = get_list_item (buffer, int);

				printf (" int %d %d", type, value);

				break;
			}
			case entity_attr_float_value:
			{
				int type = get_list_item (buffer, float_types);
				float value = get_list_item (buffer, double);

				printf (" float %d %08x", type, float_bits (value));

				break;
			}
			case entity_attr_parent:
			case entity_attr_child_pred:
			{
				int type = get_list_item (buffer, list_types);
				entity *other = get_list_item (buffer, entity *);

				printf (" %s %d %s", attr == entity_attr_parent ? "parent" : "pred", type, label_of (other));

				break;
			}
			case entity_attr_vec3d:
			{
				int type = get_list_item (buffer, vec3d_types);
				float x = get_list_item (buffer, double);
				float y = get_list_item (buffer, double);
				float z = get_list_item (buffer, double);

				printf (" vec3d %d %08x %08x %08x", type, float_bits (x), float_bits (y), float_bits (z));

				break;
			}
			default:
			{
				harness_fail ("attribute kind not supported by the harness transport");
			}
		}
	}
}

/* transport: en_comms.c transmit_entity_comms_message */
void transmit_entity_comms_message (entity_comms_messages message, entity *en, ...)
{
	va_list
		pargs;

	va_start (pargs, en);

	if (message == ENTITY_COMMS_FLOAT_VALUE)
	{
		float_types type = va_arg (pargs, float_types);

		float value = va_arg (pargs, double);

		printf ("transmit %s %d %08x\n", label_of (en), (int) type, float_bits (value));
	}
	else if (message == ENTITY_COMMS_CREATE)
	{
		/* (entity_comms_messages message, entity *en, entity_types type, int index, char *pargs) */
		entity_types type = va_arg (pargs, entity_types);

		int index = va_arg (pargs, int);

		const char *buffer = va_arg (pargs, const char *);

		printf ("transmit-create %d %d", (int) type, index);

		print_attributes (buffer);

		printf ("\n");
	}
	else if (message == ENTITY_COMMS_DESTROY)
	{
		printf ("transmit-destroy %s\n", label_of (en));
	}
	else
	{
		harness_fail ("unexpected entity comms message");
	}

	va_end (pargs);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// keysite.c (slice 4) is compiled whole for update_keysite_cargo. Its other
// functions (FARP enabling, importance, attack notification, destruction,
// capture, repair, player suitability, dumps, landing-site checks, speech,
// MFD names) are not part of the port and the harness never calls them;
// nothing puts them in a dispatch table. What only they call is a fail-loud
// stub, and what only they read is defined here without an invented meaning.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/*
 * 3dobjvis.c :: get_object_3d_bounding_box: the 3D object database, loaded at
 * run time from the game's object files. The scenario declares the bounds of
 * each object it uses (the "bounds" line); any other object fails loudly.
 */
#define MAX_HARNESS_OBJECT_BOUNDS 8

static struct
{
	object_3d_index_numbers
		object;

	struct OBJECT_3D_BOUNDS
		bounds;
}
	harness_object_bounds [MAX_HARNESS_OBJECT_BOUNDS];

static int
	num_harness_object_bounds;

struct OBJECT_3D_BOUNDS *get_object_3d_bounding_box (object_3d_index_numbers object)
{
	int
		i;

	for (i = 0; i < num_harness_object_bounds; i++)
	{
		if (harness_object_bounds[i].object == object)
		{
			return &harness_object_bounds[i].bounds;
		}
	}

	harness_fail ("get_object_3d_bounding_box: the scenario declares no bounds for this object");

	return NULL;
}

/* global.c: the host's game status (gameflow.c / flight.c set it), zero
   (GAME_STATUS_UNINITIALISED) until the scenario's "game-status" line sets it */
game_status_types
	game_status;

/* read only by keysite.c functions the harness never calls */
int
	random_number_seed,
	command_line_capture_aircraft,
	speech_sector_coordinates [6];

const char
	*entity_side_names [NUM_ENTITY_SIDES],
	*entity_side_short_names [NUM_ENTITY_SIDES];

const char *(*fn_get_local_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type);

void add_default_entity_to_regen_queue (entity_sides side, entity_sub_types group_type) { NOT_REACHED ("add_default_entity_to_regen_queue"); }
int increment_regen_queue_size (entity_sides side, entity_types type, int shift) { NOT_REACHED ("increment_regen_queue_size"); return 0; }
entity *get_local_group_primary_task (entity *en) { NOT_REACHED ("get_local_group_primary_task"); return NULL; }
entity *get_local_group_member_landing_entity_from_task (entity *en) { NOT_REACHED ("get_local_group_member_landing_entity_from_task"); return NULL; }
void update_imap_sector_side (entity *en, int in_use) { NOT_REACHED ("update_imap_sector_side"); }
void update_imap_importance_level (entity *en, int in_use) { NOT_REACHED ("update_imap_importance_level"); }
void update_keysite_distance_to_friendly_base (entity *en, entity_sides side) { NOT_REACHED ("update_keysite_distance_to_friendly_base"); }
void restore_local_fixed_entity (entity *en) { NOT_REACHED ("restore_local_fixed_entity"); }
void group_kill_all_members (entity *en) { NOT_REACHED ("group_kill_all_members"); }
entity *get_local_landing_entity_route (entity *landing_en, entity_sub_types type) { NOT_REACHED ("get_local_landing_entity_route"); return NULL; }
entity *get_local_entity_landing_entity (entity *en, entity_sub_types landing_type) { NOT_REACHED ("get_local_entity_landing_entity"); return NULL; }
entity *get_local_entity_current_task (entity *member) { NOT_REACHED ("get_local_entity_current_task"); return NULL; }
int create_group_emergency_transfer_task (entity *en) { NOT_REACHED ("create_group_emergency_transfer_task"); return 0; }
void update_imap_distance_to_friendly_base (entity_sides side) { NOT_REACHED ("update_imap_distance_to_friendly_base"); }
void set_client_server_entity_parent (entity *en, list_types type, entity *parent) { NOT_REACHED ("set_client_server_entity_parent"); }
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
void assign_keysite_tasks (entity *keysite, task_category_types category) { NOT_REACHED ("assign_keysite_tasks"); }
int assign_group_callsign (entity *en) { NOT_REACHED ("assign_group_callsign"); return 0; }
task_completed_types assess_task_completeness (entity *en, task_terminated_types task_terminated) { NOT_REACHED ("assess_task_completeness"); return 0; }

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// fc_msgs.c (slice 5a) is compiled whole for response_to_force_low_on_supplies.
// Its other responses are not part of the port: initialise_tables keeps them
// out of the dispatch table, and what only they call is a fail-loud stub.
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/*
 * taskgen.c :: create_supply_task is Slice 5b (issue #12). Slice 5a's
 * boundary: the call is recorded with its arguments. It returns NULL: the
 * response uses the result only in its DEBUG_SUPPLY log, which is compiled out.
 * The call has no other effect, so whether a scenario prints it changes
 * nothing else in its output.
 */
/* scenarios written before Slice 5a do not observe the boundary: see the "observe-supply-tasks" line */
static int
	observe_supply_tasks = FALSE;

entity *create_supply_task (entity *requester, entity *supplier, entity *cargo, movement_types movement_type, float priority, entity *start_keysite, entity *end_keysite)
{
	if (observe_supply_tasks) printf ("create-supply-task %s %s %s %d %08x %s %s\n", label_of (requester), label_of (supplier), label_of (cargo), (int) movement_type, float_bits (priority), label_of (start_keysite), label_of (end_keysite));

	return NULL;
}

aircraft_data
	aircraft_database [NUM_ENTITY_SUB_TYPE_AIRCRAFT];

float get_local_sector_entity_enemy_surface_to_air_defence_level (entity *sector_en, entity_sides side) { NOT_REACHED ("get_local_sector_entity_enemy_surface_to_air_defence_level"); return 0.0f; }
float get_local_sector_entity_enemy_surface_to_surface_defence_level (entity *sector_en, entity_sides side) { NOT_REACHED ("get_local_sector_entity_enemy_surface_to_surface_defence_level"); return 0.0f; }
void play_mobile_under_attack_speech (entity *en, entity *aggressor) { NOT_REACHED ("play_mobile_under_attack_speech"); }
void play_client_server_radio_message_response (entity *en, int speech_index, float priority, float expire_time) { NOT_REACHED ("play_client_server_radio_message_response"); }
float get_sqr_2d_range (const vec3d *v1, const vec3d *v2) { NOT_REACHED ("get_sqr_2d_range"); return 0.0f; }
int engage_targets_in_group (entity *group, entity *target_group, int expire) { NOT_REACHED ("engage_targets_in_group"); return 0; }
void create_task_completed_reactionary_tasks (entity *task) { NOT_REACHED ("create_task_completed_reactionary_tasks"); }
void create_task_assigned_reactionary_tasks (entity *task) { NOT_REACHED ("create_task_assigned_reactionary_tasks"); }
int check_group_task_type (entity *group, entity_sub_types task_type) { NOT_REACHED ("check_group_task_type"); return 0; }
void campaign_completed (entity_sides side, campaign_completed_types complete) { NOT_REACHED ("campaign_completed"); }

/* wp_list.c and wp_int.c: reached only by the waypoint list maintenance the harness never runs */
void update_local_entity_waypoint_list_tags (entity *parent) { NOT_REACHED ("update_local_entity_waypoint_list_tags"); }
int get_formation_database_count (void) { NOT_REACHED ("get_formation_database_count"); return 0; }

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
static const char *unsupplied_get_string (entity *en, string_types type) { harness_fail ("string value not supplied"); return NULL; }
static void unsupplied_set_list (entity *en, list_types type, entity *other) { harness_fail ("list set not supplied"); }
static void unsupplied_update (entity *en) { harness_fail ("update function not supplied"); }
static void unsupplied_set_vec3d (entity *en, vec3d_types type, vec3d *v) { harness_fail ("vec3d set not supplied"); }
static void unsupplied_set_char (entity *en, char_types type, char value) { harness_fail ("char set not supplied"); }
static void unsupplied_set_string (entity *en, string_types type, const char *s) { harness_fail ("string set not supplied"); }
static void unsupplied_set_attitude_angles (entity *en, float heading, float pitch, float roll) { harness_fail ("attitude set not supplied"); }
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

/* ac_vec3d.c: physical position from the scenario (environment) */
static vec3d *shim_mobile_position (entity *en, vec3d_types type)
{
	return &((shim_mobile *) get_local_entity_data (en))->position;
}

/* the original fc_msgs.c :: response_to_force_low_on_supplies (static there) */
static int
	(*original_force_low_on_supplies) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

/*
 * Every delivery keeps the trace line Slices 1 and 4 recorded at this
 * boundary before the response was ported, then runs the original response.
 */
static int trace_force_low_on_supplies (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	va_list
		args;

	int
		sub_type;

	va_copy (args, pargs);

	sub_type = va_arg (args, int);

	va_end (args);

	printf ("message %s %s %d %d\n", label_of (receiver), label_of (sender), (int) message, sub_type);

	return original_force_low_on_supplies (message, receiver, sender, pargs);
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

	/* en_creat.c, en_dstry.c: the original defaults */
	initialise_entity_create_default_functions ();
	initialise_entity_destroy_default_functions ();

	overload_keysite_int_value_functions ();
	overload_keysite_float_value_functions ();
	overload_keysite_vec3d_functions ();
	overload_keysite_list_functions ();
	harness_overload_keysite_link_child_responses ();

	overload_force_int_value_functions ();
	overload_force_list_functions ();

	/* fc_msgs.c: only FORCE_LOW_ON_SUPPLIES is adopted; every other force row
	   keeps its fail-loud default */
	overload_force_message_responses ();

	original_force_low_on_supplies = message_responses[ENTITY_TYPE_FORCE][ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES];

	for (j = 0; j < NUM_ENTITY_MESSAGES; j++) message_responses[ENTITY_TYPE_FORCE][j] = unsupplied_message_response;

	message_responses[ENTITY_TYPE_FORCE][ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES] = trace_force_low_on_supplies;

	/* task accessors: the state a restored task holds (slice 5a) */
	overload_task_int_value_functions ();
	overload_task_float_value_functions ();
	overload_task_list_functions ();

	/* waypoint accessors: a route waypoint on a requester's LIST_TYPE_TASK_DEPENDENT
	   list (slice 5a); wp_float.c does not overload FLOAT_TYPE_TASK_USER_DATA, so the
	   en_float.c default answers for it */
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
	/* cg_msgs.c :: overload_cargo_message_responses -> overload_aircraft_message_responses, link parent rows */
	harness_overload_aircraft_link_parent_responses (ENTITY_TYPE_CARGO);

	overload_sector_create_functions ();
	overload_sector_int_value_functions ();
	overload_sector_list_functions ();
	overload_sector_message_responses ();

	/* hand-written rows */
	shim_root (ENTITY_TYPE_SESSION, LIST_TYPE_FORCE);

	shim_link (ENTITY_TYPE_GUIDE, LIST_TYPE_GUIDE_STACK);

	shim_link (ENTITY_TYPE_HELICOPTER, LIST_TYPE_MEMBER);
	fn_get_local_entity_vec3d_ptr[ENTITY_TYPE_HELICOPTER][VEC3D_TYPE_POSITION] = shim_mobile_position;

}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// scenario construction
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/* i386: the variadic create call below lays attributes out as 32-bit stack words */
typedef char harness_requires_ilp32[((sizeof (void *) == 4) && (sizeof (int) == 4) && (sizeof (double) == 8)) ? 1 : -1];

static int
	heap_ready = FALSE,
	heap_size = MAX_HARNESS_ENTITIES;

static entity
	*harness_session,
	*update_root;

static update
	update_data;

static entity *new_entity (entity_types type, void *data, const char *label)
{
	entity
		*en;

	/* en_heap.c, as a restored campaign allocates: the next free index */
	en = get_free_entity (ENTITY_INDEX_DONT_CARE);

	if (!en)
	{
		harness_fail ("the scenario exhausted the entity heap");
	}

	set_local_entity_type (en, type);
	set_local_entity_data (en, data);

	snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "%s", label);

	return en;
}

static void *new_raw (size_t size)
{
	void *raw = malloc (size);

	if (!raw) harness_fail ("out of memory");

	memset (raw, 0, size);

	return raw;
}

/* the heap, then the session and update entities, before any scenario entity */
static void ensure_heap (void)
{
	if (heap_ready)
	{
		return;
	}

	heap_ready = TRUE;

	in_operation = FALSE;

	initialise_entity_heap (heap_size);

	harness_session = new_entity (ENTITY_TYPE_SESSION, NULL, "session");

	update_root = new_entity (ENTITY_TYPE_UPDATE, &update_data, "update");

	/* up_update.c :: set_update_entity (not compiled whole; it only assigns and logs) */
	update_entity = update_root;
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

static double next_double (char **cursor)
{
	volatile double
		value;

	INPUT_FP_BEGIN
	value = strtod (next_token (cursor), NULL);
	INPUT_FP_END

	return value;
}

/* a scenario value stored in a float: narrowed as input, not by the code under test */
static float next_float (char **cursor)
{
	volatile float
		value;

	INPUT_FP_BEGIN
	value = (float) strtod (next_token (cursor), NULL);
	INPUT_FP_END

	return value;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static group
	harness_groups[MAX_HARNESS_ENTITIES];

static int
	num_groups;

static entity
	*groups[MAX_HARNESS_ENTITIES];

static void print_timeline_state (int step)
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

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// lifecycle scenarios
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int
	lifecycle = FALSE,
	map_complete = FALSE,
	num_keysites,
	num_created;

static entity
	*keysites[MAX_HARNESS_ENTITIES];

/* every entity a create line returned, by its scenario label (a destroyed entity keeps its pointer) */
static struct
{
	char
		label[32];

	entity
		*en;
} created[MAX_HARNESS_ENTITIES];

static entity *find_created (const char *label)
{
	entity
		*en;

	int
		i;

	if (strcmp (label, "NULL") == 0)
	{
		return NULL;
	}

	for (i = num_created - 1; i >= 0; i--)
	{
		if (strcmp (created[i].label, label) == 0)
		{
			return created[i].en;
		}
	}

	for (i = 0; i < num_keysites; i++)
	{
		if (strcmp (labels[get_local_entity_index (keysites[i])], label) == 0)
		{
			return keysites[i];
		}
	}

	/* any other live entity (e.g. a sector) */
	for (en = first_used_entity; en; en = en->succ)
	{
		if (strcmp (labels[get_local_entity_index (en)], label) == 0)
		{
			return en;
		}
	}

	harness_fail ("unknown entity label");

	return NULL;
}

static void print_list (list_types type, entity *parent)
{
	entity
		*en;

	en = get_local_entity_first_child (parent, type);

	if (!en)
	{
		printf (" -");
	}

	for (; en; en = get_local_entity_child_succ (en, type))
	{
		printf (" %s", label_of (en));
	}
}

/*
 * The entity graph after a lifecycle scenario:
 *
 *   heap free <index>...                     free list order (next allocation first)
 *   heap used <label>...                     used list order (most recent first)
 *   cargo <label> <index> <side> <sub_type> <alive> <x> <y> <z> <cargo parent> <sector parent>
 *   keysite <label> <cargo list>...
 *   sector <label> <index> <X_SECTOR> <Z_SECTOR> <sector list>...
 *
 * Values are read through the original accessors.
 */
static void print_lifecycle_state (void)
{
	entity
		*en;

	int
		i,
		x,
		z;

	printf ("heap free");

	for (en = first_free_entity; en; en = en->succ)
	{
		printf (" %d", get_local_entity_index (en));
	}

	printf ("\nheap used");

	for (en = first_used_entity; en; en = en->succ)
	{
		printf (" %s", label_of (en));
	}

	printf ("\n");

	for (en = first_used_entity; en; en = en->succ)
	{
		if (get_local_entity_type (en) == ENTITY_TYPE_CARGO)
		{
			vec3d *position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

			printf
			(
				"cargo %s %d %d %d %d %08x %08x %08x %s %s\n",
				label_of (en),
				get_local_entity_index (en),
				get_local_entity_int_value (en, INT_TYPE_SIDE),
				get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE),
				get_local_entity_int_value (en, INT_TYPE_ALIVE),
				float_bits (position->x),
				float_bits (position->y),
				float_bits (position->z),
				label_of (get_local_entity_parent (en, LIST_TYPE_CARGO)),
				label_of (get_local_entity_parent (en, LIST_TYPE_SECTOR))
			);
		}
	}

	for (i = 0; i < num_keysites; i++)
	{
		printf ("keysite %s", label_of (keysites[i]));

		print_list (LIST_TYPE_CARGO, keysites[i]);

		printf ("\n");
	}

	/* a map line that ended early leaves unassigned cells */
	if (map_complete)
	{
		for (z = MIN_MAP_Z_SECTOR; z <= MAX_MAP_Z_SECTOR; z++)
		{
			for (x = MIN_MAP_X_SECTOR; x <= MAX_MAP_X_SECTOR; x++)
			{
				en = entity_sector_map[x + (z * NUM_MAP_X_SECTORS)];

				printf
				(
					"sector %s %d %d %d",
					label_of (en),
					get_local_entity_index (en),
					get_local_entity_int_value (en, INT_TYPE_X_SECTOR),
					get_local_entity_int_value (en, INT_TYPE_Z_SECTOR)
				);

				print_list (LIST_TYPE_SECTOR, en);

				printf ("\n");
			}
		}
	}
}

/*
 * A create line's attributes as the i386 argument stack holds them after the
 * index argument (4-byte ints, enums and pointers; 8-byte doubles, floats
 * already promoted), so the original create_client_server_entity reads them
 * through its TX path: pargs_buffer = (char *) pargs.
 */
#define MAX_ATTRIBUTE_WORDS 64

static unsigned int
	attribute_words[MAX_ATTRIBUTE_WORDS];

static int
	num_attribute_words;

static void push_word (unsigned int word)
{
	if (num_attribute_words >= MAX_ATTRIBUTE_WORDS) harness_fail ("too many attributes");

	attribute_words[num_attribute_words++] = word;
}

static void push_double (double value)
{
	unsigned int
		words[2];

	memcpy (words, &value, sizeof (words));

	push_word (words[0]);
	push_word (words[1]);
}

static void push_entity (entity *en)
{
	uintptr_t
		address = (uintptr_t) en;

	push_word ((unsigned int) address);
}

static void parse_attributes (char **cursor)
{
	char
		*kind;

	num_attribute_words = 0;

	while (TRUE)
	{
		kind = next_token (cursor);

		if (strcmp (kind, "end") == 0)
		{
			push_word (entity_attr_end);

			return;
		}
		else if (strcmp (kind, "int") == 0)
		{
			push_word (entity_attr_int_value);
			push_word ((unsigned int) next_int (cursor));
			push_word ((unsigned int) next_int (cursor));
		}
		else if (strcmp (kind, "float") == 0)
		{
			push_word (entity_attr_float_value);
			push_word ((unsigned int) next_int (cursor));
			push_double (next_double (cursor));
		}
		else if ((strcmp (kind, "parent") == 0) || (strcmp (kind, "pred") == 0))
		{
			push_word ((strcmp (kind, "parent") == 0) ? entity_attr_parent : entity_attr_child_pred);
			push_word ((unsigned int) next_int (cursor));
			push_entity (find_created (next_token (cursor)));
		}
		else if (strcmp (kind, "vec3d") == 0)
		{
			push_word (entity_attr_vec3d);
			push_word ((unsigned int) next_int (cursor));
			push_double (next_double (cursor));
			push_double (next_double (cursor));
			push_double (next_double (cursor));
		}
		else
		{
			harness_fail ("unknown attribute kind");
		}
	}
}

#define W(N) attribute_words[(N)]

static entity *create_from_words (entity_types type, int index)
{
	/* 64 stack words: the attributes, then unread padding */
	return create_client_server_entity
	(
		type, index,
		W(0), W(1), W(2), W(3), W(4), W(5), W(6), W(7), W(8), W(9), W(10), W(11), W(12), W(13), W(14), W(15),
		W(16), W(17), W(18), W(19), W(20), W(21), W(22), W(23), W(24), W(25), W(26), W(27), W(28), W(29), W(30), W(31),
		W(32), W(33), W(34), W(35), W(36), W(37), W(38), W(39), W(40), W(41), W(42), W(43), W(44), W(45), W(46), W(47),
		W(48), W(49), W(50), W(51), W(52), W(53), W(54), W(55), W(56), W(57), W(58), W(59), W(60), W(61), W(62), W(63)
	);
}

static void label_sectors (void)
{
	int
		x,
		z;

	for (z = MIN_MAP_Z_SECTOR; z <= MAX_MAP_Z_SECTOR; z++)
	{
		for (x = MIN_MAP_X_SECTOR; x <= MAX_MAP_X_SECTOR; x++)
		{
			entity *en = entity_sector_map[x + (z * NUM_MAP_X_SECTORS)];

			snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "sector%d_%d", x, z);
		}
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int main (void)
{
	static shim_mobile leader_data;

	entity
		*forces[MAX_HARNESS_ENTITIES],
		*keysite_tail[NUM_ENTITY_SIDES] = { NULL },
		*group_en = NULL;

	int
		num_forces = 0,
		f32_lines = 0,
		timeline = FALSE,
		step = 0,
		i;

	char
		line[1024],
		*cursor,
		*word;

	/* unbuffered: output written before a fault is never lost, and the fault
	   handler never needs to flush */
	setvbuf (stdout, NULL, _IONBF, 0);

	initialise_tables ();

	install_segmentation_fault_handler ();

	install_fpu_environment ();

	atexit (check_fpu_environment);

	while (fgets (line, sizeof (line), stdin))
	{
		check_fpu_environment ();

		cursor = line;

		word = next_token (&cursor);

		if (strcmp (word, "fpu") == 0)
		{
			/* the floating-point environment the original code runs under */
			printf ("fpu cw %04x mxcsr-rc %x flt-eval-method %d\n", read_x87_control_word (), (read_mxcsr () & MXCSR_RC_MASK) >> 13, (int) FLT_EVAL_METHOD);

			return 0;
		}

		if (strcmp (word, "f32") == 0)
		{
			/*
			 * One C float operation under the canonical environment, the reference
			 * for src/core/float32.ts (test/c-reference/float32-rtz.cref.test.ts).
			 * Operands of mul, div and sqrt are floats; narrow, sum and dsum take doubles;
			 * crate-row takes the floats x, xmin and xmax.
			 */
			const char *kind = next_token (&cursor);
			volatile float result;

			if (strcmp (kind, "narrow") == 0)
			{
				volatile double d = next_double (&cursor);
				result = (float) d;
			}
			else if (strcmp (kind, "sum") == 0)
			{
				volatile double d1 = next_double (&cursor), d2 = next_double (&cursor);
				result = (float) (d1 + d2);
			}
			else if (strcmp (kind, "mul") == 0)
			{
				volatile float a = next_float (&cursor), b = next_float (&cursor);
				result = a * b;
			}
			else if (strcmp (kind, "div") == 0)
			{
				volatile float a = next_float (&cursor), b = next_float (&cursor);
				result = a / b;
			}
			else if (strcmp (kind, "sqrt") == 0)
			{
				volatile float a = next_float (&cursor);
				result = sqrt (a);
			}
			else if (strcmp (kind, "crate-row") == 0)
			{
				/* keysite.c :: update_keysite_cargo, lines 428 and 464, verbatim:
				   the crate row's step from float x, xmin and xmax to the stored x */
				struct OBJECT_3D_BOUNDS
					box,
					*bounding_box = &box;

				vec3d
					position;

				position.x = next_float (&cursor);
				box.xmin = next_float (&cursor);
				box.xmax = next_float (&cursor);

				position.x += (bounding_box->xmax - bounding_box->xmin) + 1.0;

				result = position.x;
			}
			else if (strcmp (kind, "dsum") == 0)
			{
				/* a double sum (e.g. keysite.c's (xmax - xmin) + 1.0), printed as double bits */
				volatile double d1 = next_double (&cursor), d2 = next_double (&cursor);
				volatile double sum = d1 + d2;
				unsigned long long bits;

				memcpy (&bits, (const void *) &sum, sizeof (bits));

				printf ("f32 %016llx\n", bits);

				f32_lines++;

				continue;
			}
			else
			{
				harness_fail ("unknown f32 operation");
			}

			printf ("f32 %08x\n", float_bits (result));

			f32_lines++;

			continue;
		}

		if (strcmp (word, "heap") == 0)
		{
			if (heap_ready) harness_fail ("heap line after the heap was initialised");

			heap_size = next_int (&cursor);

			if ((heap_size < 2) || (heap_size > MAX_HARNESS_ENTITIES)) harness_fail ("heap size out of the harness range");

			lifecycle = TRUE;

			continue;
		}

		ensure_heap ();

		if (strcmp (word, "session") == 0)
		{
			session_entity = next_int (&cursor) ? harness_session : NULL;
		}
		else if (strcmp (word, "force") == 0)
		{
			char label[32];

			force *raw = new_raw (sizeof (force));

			raw->side = (entity_sides) next_int (&cursor);

			snprintf (label, sizeof (label), "force%d", num_forces);

			forces[num_forces] = new_entity (ENTITY_TYPE_FORCE, raw, label);

			link_entity_raw (forces[num_forces], LIST_TYPE_FORCE, harness_session, num_forces ? forces[num_forces - 1] : NULL);

			num_forces++;
		}
		else if (strcmp (word, "keysite") == 0)
		{
			char label[32];

			keysite *raw = new_raw (sizeof (keysite));

			raw->side = (entity_sides) next_int (&cursor);
			raw->sub_type = next_int (&cursor);
			raw->in_use = next_int (&cursor);
			raw->position.x = next_float (&cursor);
			raw->position.y = 0.0;
			raw->position.z = next_float (&cursor);
			raw->supplies.ammo_supply_level = next_float (&cursor);
			raw->supplies.fuel_supply_level = next_float (&cursor);

			snprintf (label, sizeof (label), "keysite%d", num_keysites);

			keysites[num_keysites] = new_entity (ENTITY_TYPE_KEYSITE, raw, label);

			for (i = 0; i < num_forces; i++)
			{
				if (((force *) get_local_entity_data (forces[i]))->side == raw->side)
				{
					link_entity_raw (keysites[num_keysites], LIST_TYPE_KEYSITE_FORCE, forces[i], keysite_tail[raw->side]);

					keysite_tail[raw->side] = keysites[num_keysites];

					break;
				}
			}

			final_keysites[num_keysites] = raw;

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
			raw->supplies.ammo_supply_level = next_float (&cursor);
			raw->supplies.fuel_supply_level = next_float (&cursor);

			parent_kind = next_int (&cursor);
			parent_index = next_int (&cursor);
			busy = next_int (&cursor);
			has_leader = next_int (&cursor);

			leader_data.position.x = next_float (&cursor);
			leader_data.position.y = 0.0;
			leader_data.position.z = next_float (&cursor);

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
					if (((force *) get_local_entity_data (forces[i]))->side == raw->side)
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
			raw->sleep = next_float (&cursor);
			raw->assist_timer = next_float (&cursor);

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
			float value, delta = 0.0f;

			if (is_set)
			{
				group_index = next_int (&cursor);
				float_type = next_int (&cursor);
				value = next_float (&cursor);
			}
			else
			{
				delta = next_float (&cursor);
				locked = next_int (&cursor);
				count = next_int (&cursor);
			}

			if (setjmp (abort_operation) != 0)
			{
				return 0;
			}

			in_operation = TRUE;

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

			in_operation = FALSE;

			print_timeline_state (step);

			step++;
		}
		else if (strcmp (word, "game-status") == 0)
		{
			/* the host's game flow (global.c :: set_game_status) */
			game_status = (game_status_types) next_int (&cursor);
		}
		else if (strcmp (word, "bounds") == 0)
		{
			/* the 3D object database entry of one object (get_object_3d_bounding_box);
			   a later line for the same object replaces its entry */
			object_3d_index_numbers object = (object_3d_index_numbers) next_int (&cursor);
			int entry;

			for (entry = 0; (entry < num_harness_object_bounds) && (harness_object_bounds[entry].object != object); entry++)
			{
			}

			if (entry == num_harness_object_bounds)
			{
				if (num_harness_object_bounds == MAX_HARNESS_OBJECT_BOUNDS) harness_fail ("too many bounds lines");

				num_harness_object_bounds++;
			}

			harness_object_bounds[entry].object = object;
			harness_object_bounds[entry].bounds.xmin = next_float (&cursor);
			harness_object_bounds[entry].bounds.xmax = next_float (&cursor);
			harness_object_bounds[entry].bounds.ymin = next_float (&cursor);
			harness_object_bounds[entry].bounds.ymax = next_float (&cursor);
			harness_object_bounds[entry].bounds.zmin = next_float (&cursor);
			harness_object_bounds[entry].bounds.zmax = next_float (&cursor);
		}
		else if (strcmp (word, "keysite-state") == 0)
		{
			/* raw keysite state a saved game holds: the alive bit and the height */
			keysite *raw = (keysite *) get_local_entity_data (find_created (next_token (&cursor)));

			raw->alive = next_int (&cursor);
			raw->position.y = next_float (&cursor);
		}
		else if (strcmp (word, "observe-supply-tasks") == 0)
		{
			/* slice 5a: print the create_supply_task boundary from here on */
			observe_supply_tasks = TRUE;
		}
		else if (strcmp (word, "comms-model") == 0)
		{
			/* comms.c :: set_comms_model, as the host's session set-up sets it */
			system_comms_model = (comms_model_types) next_int (&cursor);
		}
		else if (strcmp (word, "restore-group") == 0)
		{
			/*
			 * slice 5a: a group as a restored campaign holds it, by label:
			 * restore-group <label> <sub_type> <side> <ammo> <fuel> <keysite label | NULL | independent> <busy> <has_leader> <x> <z>
			 */
			char label[32], parent_label[32], member_label[40];
			int busy, has_leader;
			entity *en;

			group *raw = new_raw (sizeof (group));

			shim_mobile *leader = new_raw (sizeof (shim_mobile));

			snprintf (label, sizeof (label), "%s", next_token (&cursor));

			raw->sub_type = next_int (&cursor);
			raw->side = (entity_sides) next_int (&cursor);
			raw->supplies.ammo_supply_level = next_float (&cursor);
			raw->supplies.fuel_supply_level = next_float (&cursor);

			snprintf (parent_label, sizeof (parent_label), "%s", next_token (&cursor));

			busy = next_int (&cursor);
			has_leader = next_int (&cursor);

			leader->position.x = next_float (&cursor);
			leader->position.y = 0.0;
			leader->position.z = next_float (&cursor);

			en = new_entity (ENTITY_TYPE_GROUP, raw, label);

			if (strcmp (parent_label, "independent") == 0)
			{
				for (i = 0; i < num_forces; i++)
				{
					if (((force *) get_local_entity_data (forces[i]))->side == raw->side)
					{
						link_entity_raw (en, LIST_TYPE_INDEPENDENT_GROUP, forces[i], last_child (forces[i], LIST_TYPE_INDEPENDENT_GROUP));

						break;
					}
				}
			}
			else if (strcmp (parent_label, "NULL") != 0)
			{
				entity *parent = find_created (parent_label);

				link_entity_raw (en, LIST_TYPE_KEYSITE_GROUP, parent, last_child (parent, LIST_TYPE_KEYSITE_GROUP));
			}

			if (busy)
			{
				snprintf (member_label, sizeof (member_label), "%s.guide", label);

				link_entity_raw (new_entity (ENTITY_TYPE_GUIDE, NULL, member_label), LIST_TYPE_GUIDE_STACK, en, NULL);
			}

			if (has_leader)
			{
				snprintf (member_label, sizeof (member_label), "%s.leader", label);

				link_entity_raw (new_entity (ENTITY_TYPE_HELICOPTER, leader, member_label), LIST_TYPE_MEMBER, en, NULL);
			}
		}
		else if (strcmp (word, "task") == 0)
		{
			/*
			 * slice 5a: a task as a restored campaign holds it, on the
			 * LIST_TYPE_TASK_DEPENDENT list of its objective (appended):
			 * task <label> <objective label> <sub_type> <side> <task_state> <task_user_data>
			 */
			char label[32];
			entity *en, *objective;

			task *raw = new_raw (sizeof (task));

			snprintf (label, sizeof (label), "%s", next_token (&cursor));

			objective = find_created (next_token (&cursor));

			raw->sub_type = next_int (&cursor);
			raw->side = next_int (&cursor);
			raw->task_state = (task_state_types) next_int (&cursor);
			raw->task_user_data = next_float (&cursor);

			en = new_entity (ENTITY_TYPE_TASK, raw, label);

			link_entity_raw (en, LIST_TYPE_TASK_DEPENDENT, objective, last_child (objective, LIST_TYPE_TASK_DEPENDENT));
		}
		else if (strcmp (word, "waypoint") == 0)
		{
			/*
			 * slice 5a: a route waypoint as a restored campaign holds it, on the
			 * LIST_TYPE_TASK_DEPENDENT list of its route dependent (appended), as
			 * croute.c links it: waypoint <label> <dependent label> <sub_type>
			 */
			char label[32];
			entity *en, *dependent;

			waypoint *raw = new_raw (sizeof (waypoint));

			snprintf (label, sizeof (label), "%s", next_token (&cursor));

			dependent = find_created (next_token (&cursor));

			raw->sub_type = next_int (&cursor);

			en = new_entity (ENTITY_TYPE_WAYPOINT, raw, label);

			link_entity_raw (en, LIST_TYPE_TASK_DEPENDENT, dependent, last_child (dependent, LIST_TYPE_TASK_DEPENDENT));
		}
		else if (strcmp (word, "assess-group") == 0)
		{
			/* group.c :: assess_group_supplies (en), within a lifecycle scenario */
			entity *target = find_created (next_token (&cursor));

			lifecycle = TRUE;

			if (setjmp (abort_operation) != 0)
			{
				print_lifecycle_state ();

				return 0;
			}

			in_operation = TRUE;

			assess_group_supplies (target);

			in_operation = FALSE;
		}
		else if (strcmp (word, "update-cargo") == 0)
		{
			/* keysite.c :: update_keysite_cargo (en, cargo_level, sub_type, cargo_size) */
			entity *target = find_created (next_token (&cursor));
			float level = next_float (&cursor);
			entity_sub_types sub_type = (entity_sub_types) next_int (&cursor);
			float size = next_float (&cursor);
			entity *en;

			lifecycle = TRUE;

			if (setjmp (abort_operation) != 0)
			{
				print_lifecycle_state ();

				return 0;
			}

			in_operation = TRUE;

			update_keysite_cargo (target, level, sub_type, size);

			in_operation = FALSE;

			/* crates the original created are labelled by index */
			for (en = first_used_entity; en; en = en->succ)
			{
				if ((get_local_entity_type (en) == ENTITY_TYPE_CARGO) && (labels[get_local_entity_index (en)][0] == '\0'))
				{
					snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "crate%d", get_local_entity_index (en));
				}
			}
		}
		else if ((strcmp (word, "map") == 0) || (strcmp (word, "create") == 0) || (strcmp (word, "destroy") == 0) || (strcmp (word, "allocate") == 0))
		{
			/* lifecycle operations: each runs the original code; an ASSERT or debug_fatal ends the scenario */
			char op[16], label[32];
			int x_sectors = 0, z_sectors = 0, side_length = 0, type = 0, index = 0;
			entity *target = NULL, *en;

			snprintf (op, sizeof (op), "%s", word);

			if (strcmp (op, "map") == 0)
			{
				x_sectors = next_int (&cursor);
				z_sectors = next_int (&cursor);
				side_length = next_int (&cursor);
			}
			else if (strcmp (op, "create") == 0)
			{
				snprintf (label, sizeof (label), "%s", next_token (&cursor));
				type = next_int (&cursor);
				index = next_int (&cursor);
				parse_attributes (&cursor);
			}
			else if (strcmp (op, "allocate") == 0)
			{
				snprintf (label, sizeof (label), "%s", next_token (&cursor));
				index = next_int (&cursor);
			}
			else
			{
				target = find_created (next_token (&cursor));
			}

			lifecycle = TRUE;

			if (setjmp (abort_operation) != 0)
			{
				print_lifecycle_state ();

				return 0;
			}

			in_operation = TRUE;

			if (strcmp (op, "map") == 0)
			{
				/* campaign script parser (parsgen.c) sets the map; en_creat.c ::
				   create_local_only_entities creates the sectors under SERVER/TX */
				map_complete = FALSE;

				set_entity_world_map_size (x_sectors, z_sectors, side_length);

				create_local_sector_entities ();

				label_sectors ();

				map_complete = TRUE;
			}
			else if (strcmp (op, "create") == 0)
			{
				en = create_from_words ((entity_types) type, index);

				if (en)
				{
					snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "%s", label);

					snprintf (created[num_created].label, sizeof (created[0].label), "%s", label);

					created[num_created].en = en;

					num_created++;

					printf ("created %s %d\n", label, get_local_entity_index (en));
				}
				else
				{
					printf ("created %s NULL\n", label);
				}
			}
			else if (strcmp (op, "allocate") == 0)
			{
				/* en_heap.c :: get_free_entity with a specific index, as restoring a
				   saved group does (en_pack.c); the scenario gives it raw data */
				en = get_free_entity (index);

				set_local_entity_type (en, ENTITY_TYPE_GROUP);

				set_local_entity_data (en, new_raw (sizeof (group)));

				snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "%s", label);

				printf ("allocated %s %d\n", label, get_local_entity_index (en));
			}
			else
			{
				destroy_client_server_entity_family (target);
			}

			in_operation = FALSE;
		}
		else if (strcmp (word, "end") == 0)
		{
			if (timeline || lifecycle)
			{
				printf ("result ok\n");

				if (lifecycle)
				{
					print_lifecycle_state ();
				}

				return 0;
			}
		}
		else if (strcmp (word, "op") == 0)
		{
			word = next_token (&cursor);

			if (setjmp (abort_operation) == 0)
			{
				in_operation = TRUE;

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

					pos.x = next_float (&cursor);
					pos.y = 0.0;
					pos.z = next_float (&cursor);
					min_range = next_float (&cursor);
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

					v1.x = next_float (&cursor);
					v1.y = 0.0;
					v1.z = next_float (&cursor);
					v2.x = next_float (&cursor);
					v2.y = 0.0;
					v2.z = next_float (&cursor);

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

	if (f32_lines > 0)
	{
		return 0;
	}

	harness_fail ("scenario has no op or end line");

	return 1;
}

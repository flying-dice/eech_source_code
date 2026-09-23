/*
 * Legacy scenario replay: runs an eech-core-ts C reference scenario (the text
 * c-reference/harness.c reads on stdin) through the native kernel and
 * produces the harness's output, line for line.
 *
 * Provenance: eech-core-ts/c-reference/harness.c (81ed32e), main () and the
 * scenario construction, lifecycle and output sections, adapted as follows:
 *
 *   - the whole replay is one kernel entry; each operation line installs its
 *     own abort point, as the harness's setjmp (abort_operation) does, and an
 *     ASSERT, debug_fatal or the boundary inside it is the reported outcome;
 *     anything else (an unported dependency, a host failure, a scenario
 *     error) fails the replay, as the harness's exit (2) does;
 *   - physical state the scenario states (mobile positions, 3D object
 *     bounds) is declared to the host, and the kernel reads it back through
 *     the host boundary like any other campaign would;
 *   - the i386 argument-stack construction of `create` lines is replaced by a
 *     real variadic call (the attribute list is marshalled by patch P1, as for
 *     every creation in the kernel);
 *   - output goes through the host's output callback.
 */

#include <fenv.h>
#include <stdint.h>

#include "project.h"

#include "eech_kernel_internal.h"

#pragma STDC FENV_ACCESS ON

#define MAX_LEGACY_ENTITIES 1024

extern void eech_link_entity_raw (entity *en, list_types type, entity *parent, entity *pred);
extern entity *eech_last_child (entity *parent, list_types type);
extern void *eech_new_raw (size_t size);
extern void eech_kernel_open_heap (int heap_size);
extern void eech_kernel_begin_replay (void);
extern void eech_kernel_end_replay (void);
extern void eech_kernel_initialise_tables (void);

int
	eech_legacy_active = FALSE,
	eech_legacy_observe_supply_tasks = FALSE;

static char
	labels[MAX_LEGACY_ENTITIES][32];

void eech_out (const char *format, ...)
{
	char
		text[4096];

	va_list
		pargs;

	if (!eech_legacy_active || !eech_current_host || !eech_current_host->output)
	{
		return;
	}

	va_start (pargs, format);

	vsnprintf (text, sizeof (text), format, pargs);

	va_end (pargs);

	if (eech_current_host->output (eech_current_host->context, text) != 0)
	{
		eech_abort (EECH_STATUS_HOST_ERROR, "output", "the host's output sink failed");
	}
}

const char *eech_legacy_label_of (entity *en)
{
	if (!eech_legacy_active)
	{
		return "";
	}

	return en ? labels[en - entities] : "NULL";
}

/* slice 5b: a task the original creates is labelled task<index> when it is first printed */
const char *eech_legacy_task_label_of (entity *en)
{
	if (eech_legacy_active && en && (labels[en - entities][0] == '\0'))
	{
		snprintf (labels[en - entities], sizeof (labels[0]), "task%d", (int) (en - entities));
	}

	return eech_legacy_label_of (en);
}

static unsigned int float_bits (float value)
{
	unsigned int
		bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

/*
 * An attribute buffer (en_attrs.c layout, read with get_list_item as
 * pack_entity_attributes does) in the scenario's create-line syntax, with
 * doubles narrowed to the floats the receiver stores.
 */
void eech_legacy_print_attributes (const char *buffer)
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
				eech_out (" end");

				return;
			}
			case entity_attr_int_value:
			{
				int type = get_list_item (buffer, int_types);
				int value = get_list_item (buffer, int);

				eech_out (" int %d %d", type, value);

				break;
			}
			case entity_attr_float_value:
			{
				int type = get_list_item (buffer, float_types);
				float value = get_list_item (buffer, double);

				eech_out (" float %d %08x", type, float_bits (value));

				break;
			}
			case entity_attr_parent:
			case entity_attr_child_pred:
			{
				int type = get_list_item (buffer, list_types);
				entity *other = get_list_item (buffer, entity *);

				eech_out (" %s %d %s", attr == entity_attr_parent ? "parent" : "pred", type, eech_legacy_label_of (other));

				break;
			}
			case entity_attr_vec3d:
			{
				int type = get_list_item (buffer, vec3d_types);
				float x = get_list_item (buffer, double);
				float y = get_list_item (buffer, double);
				float z = get_list_item (buffer, double);

				eech_out (" vec3d %d %08x %08x %08x", type, float_bits (x), float_bits (y), float_bits (z));

				break;
			}
			default:
			{
				eech_abort (EECH_STATUS_INVALID, "attribute kind", "attribute kind %d is not supported by the replay transport", (int) attr);
			}
		}
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// replay state (static: it must survive the operations' longjmps)
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int
	heap_ready,
	heap_size,
	lifecycle,
	map_complete,
	num_keysites,
	num_created,
	num_forces,
	num_groups,
	f32_lines,
	timeline,
	step,
	finished,
	observe_tasks;

static entity
	*keysites[MAX_LEGACY_ENTITIES],
	*forces[MAX_LEGACY_ENTITIES],
	*keysite_tail[NUM_ENTITY_SIDES],
	*groups[MAX_LEGACY_ENTITIES],
	*group_en;

static group
	*group_raws[MAX_LEGACY_ENTITIES],
	*final_group_raw;

static keysite
	*final_keysites[MAX_LEGACY_ENTITIES];

static int
	final_keysite_count;

static struct
{
	char
		label[32];

	entity
		*en;
} created[MAX_LEGACY_ENTITIES];

void eech_legacy_reset (void)
{
	eech_legacy_active = FALSE;
	eech_legacy_observe_supply_tasks = FALSE;
	memset (labels, 0, sizeof (labels));
	heap_ready = FALSE;
	heap_size = MAX_LEGACY_ENTITIES;
	lifecycle = FALSE;
	map_complete = FALSE;
	num_keysites = 0;
	num_created = 0;
	num_forces = 0;
	num_groups = 0;
	f32_lines = 0;
	timeline = FALSE;
	step = 0;
	finished = FALSE;
	observe_tasks = FALSE;
	memset (keysites, 0, sizeof (keysites));
	memset (forces, 0, sizeof (forces));
	memset (keysite_tail, 0, sizeof (keysite_tail));
	memset (groups, 0, sizeof (groups));
	memset (group_raws, 0, sizeof (group_raws));
	group_en = NULL;
	final_group_raw = NULL;
	memset (final_keysites, 0, sizeof (final_keysites));
	final_keysite_count = 0;
	memset (created, 0, sizeof (created));
}

static void fail (const char *what)
{
	eech_abort (EECH_STATUS_INVALID, what, "legacy scenario: %s", what);
}

static void set_label (entity *en, const char *label)
{
	snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "%s", label);
}

static entity *new_entity (entity_types type, void *data, const char *label)
{
	entity
		*en;

	en = get_free_entity (ENTITY_INDEX_DONT_CARE);

	if (!en)
	{
		fail ("the scenario exhausted the entity heap");
	}

	set_local_entity_type (en, type);
	set_local_entity_data (en, data);

	set_label (en, label);

	return en;
}

static void declare_position (entity *en, float x, float y, float z)
{
	float
		v[3];

	v[0] = x;
	v[1] = y;
	v[2] = z;

	eech_host_declare (EECH_DECLARE_MOBILE_POSITION, eech_ref_of (en), 0, v);
}

/* the heap, then the session and update entities, before any scenario entity */
static void ensure_heap (void)
{
	if (heap_ready)
	{
		return;
	}

	heap_ready = TRUE;

	eech_kernel_open_heap (heap_size);

	set_label (eech_session, "session");
	set_label (eech_update_root, "update");
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// tokens
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static char *next_token (char **cursor)
{
	char
		*token,
		*end;

	token = *cursor;

	while (*token == ' ' || *token == '\t' || *token == '\r')
	{
		token++;
	}

	if (*token == '\0')
	{
		fail ("truncated scenario line");
	}

	end = token;

	while (*end != '\0' && *end != ' ' && *end != '\t' && *end != '\r')
	{
		end++;
	}

	if (*end != '\0')
	{
		*end = '\0';
		end++;
	}

	*cursor = end;

	return token;
}

static int next_int (char **cursor) { return atoi (next_token (cursor)); }

/* scenario input is parsed and narrowed to nearest: it is the scenario's
   value, not a result of EECH arithmetic */
static double next_double (char **cursor)
{
	volatile double
		value;

	const char
		*token = next_token (cursor);

	int
		rounding = fegetround ();

	fesetround (FE_TONEAREST);
	value = strtod (token, NULL);
	fesetround (rounding);

	return value;
}

static float next_float (char **cursor)
{
	volatile float
		value;

	const char
		*token = next_token (cursor);

	int
		rounding = fegetround ();

	fesetround (FE_TONEAREST);
	value = (float) strtod (token, NULL);
	fesetround (rounding);

	return value;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// output of state
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static void emit_final_state (void)
{
	int
		i;

	if (final_group_raw)
	{
		eech_out ("final group %08x %08x\n", float_bits (final_group_raw->supplies.ammo_supply_level), float_bits (final_group_raw->supplies.fuel_supply_level));
	}

	for (i = 0; i < final_keysite_count; i++)
	{
		eech_out ("final keysite %08x %08x\n", float_bits (final_keysites[i]->supplies.ammo_supply_level), float_bits (final_keysites[i]->supplies.fuel_supply_level));
	}
}

static void print_timeline_state (int n)
{
	entity
		*en;

	int
		i;

	eech_out ("step %d %08x ", n, float_bits (get_delta_time ()));

	en = get_local_entity_first_child (eech_update_root, LIST_TYPE_UPDATE);

	if (!en)
	{
		eech_out ("-");
	}

	while (en)
	{
		eech_out ("%s%s", eech_legacy_label_of (en), get_local_entity_child_succ (en, LIST_TYPE_UPDATE) ? "," : "");

		en = get_local_entity_child_succ (en, LIST_TYPE_UPDATE);
	}

	eech_out ("\n");

	for (i = 0; i < num_groups; i++)
	{
		if (group_raws[i])
		{
			eech_out ("timer %08x %08x\n", float_bits (group_raws[i]->sleep), float_bits (group_raws[i]->assist_timer));
		}
	}
}

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

	for (en = first_used_entity; en; en = en->succ)
	{
		if (strcmp (labels[get_local_entity_index (en)], label) == 0)
		{
			return en;
		}
	}

	fail ("unknown entity label");

	return NULL;
}

static void label_new_tasks (void)
{
	entity
		*en;

	for (en = first_used_entity; en; en = en->succ)
	{
		if ((get_local_entity_type (en) == ENTITY_TYPE_TASK) && (labels[get_local_entity_index (en)][0] == '\0'))
		{
			snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "task%d", get_local_entity_index (en));
		}
	}
}

static void print_list (list_types type, entity *parent)
{
	entity
		*en;

	en = get_local_entity_first_child (parent, type);

	if (!en)
	{
		eech_out (" -");
	}

	for (; en; en = get_local_entity_child_succ (en, type))
	{
		eech_out (" %s", eech_legacy_label_of (en));
	}
}

static void print_lifecycle_state (void)
{
	entity
		*en;

	int
		i,
		x,
		z;

	eech_out ("heap free");

	for (en = first_free_entity; en; en = en->succ)
	{
		eech_out (" %d", get_local_entity_index (en));
	}

	eech_out ("\nheap used");

	for (en = first_used_entity; en; en = en->succ)
	{
		eech_out (" %s", eech_legacy_label_of (en));
	}

	eech_out ("\n");

	for (en = first_used_entity; en; en = en->succ)
	{
		if (get_local_entity_type (en) == ENTITY_TYPE_CARGO)
		{
			vec3d *position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

			eech_out
			(
				"cargo %s %d %d %d %d %08x %08x %08x %s %s\n",
				eech_legacy_label_of (en),
				get_local_entity_index (en),
				get_local_entity_int_value (en, INT_TYPE_SIDE),
				get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE),
				get_local_entity_int_value (en, INT_TYPE_ALIVE),
				float_bits (position->x),
				float_bits (position->y),
				float_bits (position->z),
				eech_legacy_label_of (get_local_entity_parent (en, LIST_TYPE_CARGO)),
				eech_legacy_label_of (get_local_entity_parent (en, LIST_TYPE_SECTOR))
			);
		}
	}

	for (i = 0; i < num_keysites; i++)
	{
		eech_out ("keysite %s", eech_legacy_label_of (keysites[i]));

		print_list (LIST_TYPE_CARGO, keysites[i]);

		eech_out ("\n");
	}

	if (observe_tasks)
	{
		for (en = first_used_entity; en; en = en->succ)
		{
			if (get_local_entity_type (en) == ENTITY_TYPE_FORCE)
			{
				eech_out ("force %s supply-tasks-created %d\n", eech_legacy_label_of (en), ((force *) get_local_entity_data (en))->task_generation[ENTITY_SUB_TYPE_TASK_SUPPLY].created);
			}
			else if (get_local_entity_type (en) == ENTITY_TYPE_TASK)
			{
				task *raw = (task *) get_local_entity_data (en);
				unsigned int loop;

				eech_out
				(
					"task %s %d sub %d side %d state %d id %d critical %d movement %d length %d difficulty %d expire %08x priority %08x user %08x objective %s keysite %s sector %s update %s\n",
					eech_legacy_label_of (en),
					get_local_entity_index (en),
					(int) raw->sub_type,
					(int) raw->side,
					(int) raw->task_state,
					(int) raw->task_id,
					(int) raw->critical_task,
					(int) raw->movement_type,
					(int) raw->route_length,
					(int) raw->difficulty,
					float_bits (raw->expire_timer),
					float_bits (raw->task_priority),
					float_bits (raw->task_user_data),
					eech_legacy_label_of (raw->task_dependent_link.parent),
					eech_legacy_label_of (raw->task_link.parent),
					eech_legacy_label_of (raw->sector_task_link.parent),
					eech_legacy_label_of (raw->update_link.parent)
				);

				if (raw->route_nodes)
				{
					eech_out ("route %s", eech_legacy_label_of (en));

					for (loop = 0; loop <= raw->route_length; loop ++)
					{
						eech_out (" %08x %08x %08x %d %d %s", float_bits (raw->route_nodes[loop].x), float_bits (raw->route_nodes[loop].y), float_bits (raw->route_nodes[loop].z), (int) raw->route_waypoint_types[loop], (int) raw->route_formation_types[loop], eech_legacy_label_of (raw->route_dependents[loop]));
					}

					eech_out (" return %s\n", eech_legacy_label_of (raw->return_keysite));
				}
			}
		}

		for (i = 0; i < num_keysites; i++)
		{
			eech_out ("unassigned %s", eech_legacy_label_of (keysites[i]));

			print_list (LIST_TYPE_UNASSIGNED_TASK, keysites[i]);

			eech_out ("\n");

			eech_out ("dependents %s", eech_legacy_label_of (keysites[i]));

			print_list (LIST_TYPE_TASK_DEPENDENT, keysites[i]);

			eech_out ("\n");
		}

		if (map_complete)
		{
			for (z = MIN_MAP_Z_SECTOR; z <= MAX_MAP_Z_SECTOR; z++)
			{
				for (x = MIN_MAP_X_SECTOR; x <= MAX_MAP_X_SECTOR; x++)
				{
					en = entity_sector_map[x + (z * NUM_MAP_X_SECTORS)];

					if (get_local_entity_first_child (en, LIST_TYPE_SECTOR_TASK))
					{
						eech_out ("sector-tasks %s", eech_legacy_label_of (en));

						print_list (LIST_TYPE_SECTOR_TASK, en);

						eech_out ("\n");
					}
				}
			}
		}
	}

	if (map_complete)
	{
		for (z = MIN_MAP_Z_SECTOR; z <= MAX_MAP_Z_SECTOR; z++)
		{
			for (x = MIN_MAP_X_SECTOR; x <= MAX_MAP_X_SECTOR; x++)
			{
				en = entity_sector_map[x + (z * NUM_MAP_X_SECTORS)];

				eech_out
				(
					"sector %s %d %d %d",
					eech_legacy_label_of (en),
					get_local_entity_index (en),
					get_local_entity_int_value (en, INT_TYPE_X_SECTOR),
					get_local_entity_int_value (en, INT_TYPE_Z_SECTOR)
				);

				print_list (LIST_TYPE_SECTOR, en);

				eech_out ("\n");
			}
		}
	}
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
//
// create lines: the attributes as a real variadic call
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define MAX_ATTRIBUTES 6

typedef struct
{
	int
		kind,
		type,
		value;

	double
		x,
		y,
		z;

	entity
		*other;
} scenario_attribute;

static scenario_attribute
	attributes[MAX_ATTRIBUTES + 1];

static int
	num_attributes;

static void parse_attributes (char **cursor)
{
	char
		*kind;

	num_attributes = 0;

	while (TRUE)
	{
		scenario_attribute
			*a;

		if (num_attributes > MAX_ATTRIBUTES)
		{
			fail ("too many attributes");
		}

		a = &attributes[num_attributes++];

		kind = next_token (cursor);

		if (strcmp (kind, "end") == 0)
		{
			a->kind = entity_attr_end;

			return;
		}
		else if (strcmp (kind, "int") == 0)
		{
			a->kind = entity_attr_int_value;
			a->type = next_int (cursor);
			a->value = next_int (cursor);
		}
		else if (strcmp (kind, "float") == 0)
		{
			a->kind = entity_attr_float_value;
			a->type = next_int (cursor);
			a->x = next_double (cursor);
		}
		else if ((strcmp (kind, "parent") == 0) || (strcmp (kind, "pred") == 0))
		{
			a->kind = (strcmp (kind, "parent") == 0) ? entity_attr_parent : entity_attr_child_pred;
			a->type = next_int (cursor);
			a->other = find_created (next_token (cursor));
		}
		else if (strcmp (kind, "vec3d") == 0)
		{
			a->kind = entity_attr_vec3d;
			a->type = next_int (cursor);
			a->x = next_double (cursor);
			a->y = next_double (cursor);
			a->z = next_double (cursor);
		}
		else
		{
			fail ("unknown attribute kind");
		}
	}
}

/* the arguments of one attribute, as the ENTITY_ATTR_* macros pass them */
#define A_INT(A) (A)->kind, (A)->type, (A)->value
#define A_FLOAT(A) (A)->kind, (A)->type, (A)->x
#define A_LINK(A) (A)->kind, (A)->type, (A)->other
#define A_VEC3D(A) (A)->kind, (A)->type, (A)->x, (A)->y, (A)->z

/*
 * A create line as a real call of the variadic create_client_server_entity:
 * the i386 harness laid its attributes out as stack words; here the compiler
 * passes them, and patch P1 marshals them like any other creation. The call
 * for each shape of attribute list (up to MAX_CREATE_ATTRIBUTES attributes of
 * the four argument patterns) is generated by eech-sys/build into
 * eech_legacy_create.inc.
 */
#define A_INT(I) attributes[I].kind, attributes[I].type, attributes[I].value
#define A_FLOAT(I) attributes[I].kind, attributes[I].type, attributes[I].x
#define A_VEC3D(I) attributes[I].kind, attributes[I].type, attributes[I].x, attributes[I].y, attributes[I].z
#define A_LINK(I) attributes[I].kind, attributes[I].type, attributes[I].other

static entity *create_n (entity_types type, int index)
{
	int
		n = num_attributes - 1,
		shape = 0,
		i;

	for (i = 0; i < n; i++)
	{
		int k = attributes[i].kind;

		shape = shape * 4 + (k == entity_attr_int_value ? 0 : k == entity_attr_float_value ? 1 : k == entity_attr_vec3d ? 2 : 3);
	}

	#include "eech_legacy_create.inc"

	fail ("create line: more attributes than the replay supports");

	return NULL;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// the replay
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/* operation abort point: an EECH outcome ends the scenario after `after` ran */
#define OPERATION_BEGIN(OUTCOME_AFTER) \
	{ \
		jmp_buf operation_point; \
		jmp_buf *outer_point = eech_abort_point; \
		eech_abort_point = &operation_point; \
		if (setjmp (operation_point) != 0) \
		{ \
			eech_abort_point = outer_point; \
			report_outcome (); \
			OUTCOME_AFTER; \
			finished = TRUE; \
			return; \
		}

#define OPERATION_END \
		eech_abort_point = outer_point; \
	}

extern const char *eech_k_last_message (void);

static void report_outcome (void)
{
	switch (eech_abort_status)
	{
		case EECH_STATUS_ASSERT:
		{
			eech_out ("result assert %s\n", eech_k_last_message ());

			break;
		}
		case EECH_STATUS_FATAL:
		{
			eech_out ("result fatal %s\n", eech_k_last_message ());

			break;
		}
		case EECH_STATUS_BOUNDARY:
		{
			eech_out ("result boundary %s\n", eech_k_last_message ());

			break;
		}
		default:
		{
			/* not an EECH outcome: fail the replay with the same status */
			char message[512];

			snprintf (message, sizeof (message), "%s", eech_k_last_message ());

			eech_abort (eech_abort_status, message, "%s", eech_k_last_detail ());
		}
	}
}

static void run_line (char *line);

static void replay_body (void *argument)
{
	char
		*text = (char *) argument,
		*line,
		*end;

	eech_kernel_initialise_tables ();

	for (line = text; *line && !finished; line = end)
	{
		end = strchr (line, '\n');

		if (end)
		{
			*end = '\0';
			end++;
		}
		else
		{
			end = line + strlen (line);
		}

		/* blank lines end the harness's fgets loop only at EOF; skip them */
		{
			char *p = line;

			while (*p == ' ' || *p == '\t' || *p == '\r') p++;

			if (*p == '\0') continue;
		}

		if (fegetround () != FE_TOWARDZERO)
		{
			eech_abort (EECH_STATUS_FPU_DRIFT, "floating-point environment drifted", "rounding mode %d before a scenario line", fegetround ());
		}

		run_line (line);
	}

	if (!finished && (f32_lines == 0))
	{
		fail ("scenario has no op or end line");
	}
}

static void run_line (char *line)
{
	char
		*cursor = line,
		*word;

	int
		i;

	word = next_token (&cursor);

	if (strcmp (word, "fpu") == 0)
	{
		/* the environment differs by construction (no x87 control word here) */
		eech_out ("fpu rounding %s flt-eval-method %d\n", fegetround () == FE_TOWARDZERO ? "toward-zero" : "other", (int) FLT_EVAL_METHOD);

		finished = TRUE;

		return;
	}

	if (strcmp (word, "aircraft-cruise-velocity") == 0)
	{
		for (i = 0; i < NUM_ENTITY_SUB_TYPE_AIRCRAFT; i++)
		{
			eech_out ("%d %08x\n", i, float_bits (aircraft_database[i].cruise_velocity));
		}

		finished = TRUE;

		return;
	}

	if (strcmp (word, "f32") == 0)
	{
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
			volatile double d1 = next_double (&cursor), d2 = next_double (&cursor);
			volatile double sum = d1 + d2;
			unsigned long long bits;

			memcpy (&bits, (const void *) &sum, sizeof (bits));

			eech_out ("f32 %016llx\n", bits);

			f32_lines++;

			return;
		}
		else
		{
			fail ("unknown f32 operation");
		}

		eech_out ("f32 %08x\n", float_bits (result));

		f32_lines++;

		return;
	}

	if (strcmp (word, "heap") == 0)
	{
		if (heap_ready) fail ("heap line after the heap was initialised");

		heap_size = next_int (&cursor);

		if ((heap_size < 2) || (heap_size > MAX_LEGACY_ENTITIES)) fail ("heap size out of the replay range");

		lifecycle = TRUE;

		return;
	}

	ensure_heap ();

	if (strcmp (word, "session") == 0)
	{
		session_entity = next_int (&cursor) ? eech_session : NULL;
	}
	else if (strcmp (word, "force") == 0)
	{
		char label[32];

		force *raw = eech_new_raw (sizeof (force));

		raw->side = (entity_sides) next_int (&cursor);

		snprintf (label, sizeof (label), "force%d", num_forces);

		forces[num_forces] = new_entity (ENTITY_TYPE_FORCE, raw, label);

		eech_link_entity_raw (forces[num_forces], LIST_TYPE_FORCE, eech_session, num_forces ? forces[num_forces - 1] : NULL);

		num_forces++;
	}
	else if (strcmp (word, "keysite") == 0)
	{
		char label[32];

		keysite *raw = eech_new_raw (sizeof (keysite));

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
				eech_link_entity_raw (keysites[num_keysites], LIST_TYPE_KEYSITE_FORCE, forces[i], keysite_tail[raw->side]);

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
		int parent_kind, parent_index, busy, has_leader;
		float x, z;

		group *raw = eech_new_raw (sizeof (group));

		group_raws[num_groups] = raw;

		raw->sub_type = next_int (&cursor);
		raw->side = (entity_sides) next_int (&cursor);
		raw->supplies.ammo_supply_level = next_float (&cursor);
		raw->supplies.fuel_supply_level = next_float (&cursor);

		parent_kind = next_int (&cursor);
		parent_index = next_int (&cursor);
		busy = next_int (&cursor);
		has_leader = next_int (&cursor);

		x = next_float (&cursor);
		z = next_float (&cursor);

		group_en = new_entity (ENTITY_TYPE_GROUP, raw, "group");

		final_group_raw = raw;

		/* slice 1's group is not a timeline group: no timer lines */
		group_raws[num_groups] = NULL;

		num_groups++;

		if (parent_kind == 1)
		{
			eech_link_entity_raw (group_en, LIST_TYPE_KEYSITE_GROUP, keysites[parent_index], NULL);
		}
		else if (parent_kind == 2)
		{
			for (i = 0; i < num_forces; i++)
			{
				if (((force *) get_local_entity_data (forces[i]))->side == raw->side)
				{
					eech_link_entity_raw (group_en, LIST_TYPE_INDEPENDENT_GROUP, forces[i], NULL);

					break;
				}
			}
		}

		if (busy)
		{
			eech_link_entity_raw (new_entity (ENTITY_TYPE_GUIDE, NULL, "guide"), LIST_TYPE_GUIDE_STACK, group_en, NULL);
		}

		if (has_leader)
		{
			entity *leader = new_entity (ENTITY_TYPE_HELICOPTER, eech_new_raw (sizeof (aircraft)), "leader");

			declare_position (leader, x, 0.0f, z);

			eech_link_entity_raw (leader, LIST_TYPE_MEMBER, group_en, NULL);
		}
	}
	else if (strcmp (word, "timeline") == 0)
	{
		timeline = TRUE;

		command_line_entity_update_frame_rate = next_int (&cursor);
	}
	else if (strcmp (word, "tgroup") == 0)
	{
		char label[32];

		group *raw = eech_new_raw (sizeof (group));

		group_raws[num_groups] = raw;

		raw->sub_type = next_int (&cursor);
		raw->side = (entity_sides) next_int (&cursor);
		raw->sleep = next_float (&cursor);
		raw->assist_timer = next_float (&cursor);

		snprintf (label, sizeof (label), "group%d", num_groups);

		groups[num_groups] = new_entity (ENTITY_TYPE_GROUP, raw, label);

		if (next_int (&cursor))
		{
			eech_link_entity_raw (groups[num_groups], LIST_TYPE_UPDATE, eech_update_root, eech_last_child (eech_update_root, LIST_TYPE_UPDATE));
		}

		num_groups++;
	}
	else if ((strcmp (word, "set") == 0) || (strcmp (word, "frame") == 0))
	{
		int is_set = (strcmp (word, "set") == 0);
		int group_index = 0, float_type = 0, locked = 0, count = 0, c;
		float value = 0.0f, delta = 0.0f;

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

		OPERATION_BEGIN ((void) 0)

		if (is_set)
		{
			set_client_server_entity_float_value (groups[group_index], (float_types) float_type, value);
		}
		else
		{
			system_delta_time = delta;
			system_one_over_delta_time = 1.0 / system_delta_time;
			locked_frame_rate = locked;

			for (c = 0; c < count; c++)
			{
				update_client_server_entities ();
			}
		}

		OPERATION_END

		print_timeline_state (step);

		step++;
	}
	else if (strcmp (word, "game-status") == 0)
	{
		game_status = (game_status_types) next_int (&cursor);
	}
	else if (strcmp (word, "bounds") == 0)
	{
		float b[6];
		int object = next_int (&cursor);

		for (i = 0; i < 6; i++)
		{
			b[i] = next_float (&cursor);
		}

		eech_host_declare (EECH_DECLARE_OBJECT_BOUNDS, eech_ref_of (NULL), object, b);
	}
	else if (strcmp (word, "keysite-state") == 0)
	{
		keysite *raw = (keysite *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->alive = next_int (&cursor);
		raw->position.y = next_float (&cursor);
	}
	else if (strcmp (word, "observe-supply-tasks") == 0)
	{
		eech_legacy_observe_supply_tasks = TRUE;
	}
	else if (strcmp (word, "observe-tasks") == 0)
	{
		observe_tasks = TRUE;
	}
	else if (strcmp (word, "single-player") == 0)
	{
		eech_single_player = TRUE;
	}
	else if (strcmp (word, "game-type") == 0)
	{
		game_type = (game_types) next_int (&cursor);
	}
	else if (strcmp (word, "keysite-landing") == 0)
	{
		keysite *raw = (keysite *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->landing_types = next_int (&cursor);
		raw->keysite_usable_state = next_int (&cursor);
	}
	else if (strcmp (word, "member-count") == 0)
	{
		group *raw = (group *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->member_count = next_int (&cursor);
	}
	else if (strcmp (word, "group-sleep") == 0)
	{
		group *raw = (group *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->sleep = next_float (&cursor);
	}
	else if (strcmp (word, "air-register") == 0)
	{
		entity *en = find_created (next_token (&cursor));
		entity *force_en = get_local_force_entity ((entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE));

		eech_link_entity_raw (en, LIST_TYPE_AIR_REGISTRY, force_en, eech_last_child (force_en, LIST_TYPE_AIR_REGISTRY));
	}
	else if (strcmp (word, "aircraft-type") == 0)
	{
		aircraft *raw = (aircraft *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->mob.sub_type = next_int (&cursor);
	}
	else if (strcmp (word, "unassigned-task") == 0)
	{
		char label[32];
		entity *en, *keysite_en, *objective;

		task *raw = eech_new_raw (sizeof (task));

		snprintf (label, sizeof (label), "%s", next_token (&cursor));

		keysite_en = find_created (next_token (&cursor));
		objective = find_created (next_token (&cursor));

		raw->sub_type = next_int (&cursor);
		raw->side = next_int (&cursor);
		raw->task_state = TASK_STATE_UNASSIGNED;
		raw->critical_task = next_int (&cursor);
		raw->task_priority = next_float (&cursor);
		raw->expire_timer = next_float (&cursor);

		en = new_entity (ENTITY_TYPE_TASK, raw, label);

		eech_link_entity_raw (en, LIST_TYPE_UNASSIGNED_TASK, keysite_en, eech_last_child (keysite_en, LIST_TYPE_UNASSIGNED_TASK));

		if (objective)
		{
			eech_link_entity_raw (en, LIST_TYPE_TASK_DEPENDENT, objective, eech_last_child (objective, LIST_TYPE_TASK_DEPENDENT));
		}
	}
	else if (strcmp (word, "add-member") == 0)
	{
		char label[32];
		entity *owner, *en;
		entity_types type;
		float x, z;
		aircraft *raw = eech_new_raw (sizeof (aircraft));

		snprintf (label, sizeof (label), "%s", next_token (&cursor));

		owner = find_created (next_token (&cursor));
		type = (entity_types) next_int (&cursor);

		if (type != ENTITY_TYPE_HELICOPTER && type != ENTITY_TYPE_FIXED_WING)
		{
			fail ("add-member: not an aircraft entity type");
		}

		raw->mob.sub_type = next_int (&cursor);
		x = next_float (&cursor);
		z = next_float (&cursor);

		en = new_entity (type, raw, label);

		declare_position (en, x, 0.0f, z);

		eech_link_entity_raw (en, LIST_TYPE_MEMBER, owner, eech_last_child (owner, LIST_TYPE_MEMBER));
	}
	else if (strcmp (word, "pilot") == 0)
	{
		char label[32];

		snprintf (label, sizeof (label), "%s", next_token (&cursor));

		new_entity (ENTITY_TYPE_PILOT, eech_new_raw (sizeof (pilot)), label);
	}
	else if (strcmp (word, "pilot-lock") == 0)
	{
		entity *en = find_created (next_token (&cursor));
		entity *pilot_en = find_created (next_token (&cursor));

		eech_link_entity_raw (en, LIST_TYPE_PILOT_LOCK, pilot_en, eech_last_child (pilot_en, LIST_TYPE_PILOT_LOCK));
	}
	else if (strcmp (word, "assign-tasks") == 0)
	{
		entity *target = find_created (next_token (&cursor));
		task_category_types category = (task_category_types) next_int (&cursor);

		lifecycle = TRUE;

		OPERATION_BEGIN (print_lifecycle_state ())

		assign_keysite_tasks (target, category);

		OPERATION_END
	}
	else if (strcmp (word, "task-counter") == 0)
	{
		force *raw = (force *) get_local_entity_data (find_created (next_token (&cursor)));

		int sub_type = next_int (&cursor);

		raw->task_generation[sub_type].created = next_int (&cursor);
	}
	else if (strcmp (word, "group-alive") == 0)
	{
		group *raw = (group *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->alive = next_int (&cursor);
	}
	else if (strcmp (word, "sector-state") == 0)
	{
		sector *raw = (sector *) get_local_entity_data (find_created (next_token (&cursor)));

		raw->sector_side[ENTITY_SIDE_BLUE_FORCE] = next_float (&cursor);
		raw->sector_side[ENTITY_SIDE_RED_FORCE] = next_float (&cursor);
		raw->surface_to_air_defence_level[ENTITY_SIDE_NEUTRAL] = next_float (&cursor);
		raw->surface_to_air_defence_level[ENTITY_SIDE_BLUE_FORCE] = next_float (&cursor);
		raw->surface_to_air_defence_level[ENTITY_SIDE_RED_FORCE] = next_float (&cursor);
	}
	else if (strcmp (word, "comms-model") == 0)
	{
		system_comms_model = (comms_model_types) next_int (&cursor);
	}
	else if (strcmp (word, "restore-group") == 0)
	{
		char label[32], parent_label[32], member_label[40];
		int busy, has_leader;
		float x, z;
		entity *en;

		group *raw = eech_new_raw (sizeof (group));

		snprintf (label, sizeof (label), "%s", next_token (&cursor));

		raw->sub_type = next_int (&cursor);
		raw->side = (entity_sides) next_int (&cursor);
		raw->supplies.ammo_supply_level = next_float (&cursor);
		raw->supplies.fuel_supply_level = next_float (&cursor);

		snprintf (parent_label, sizeof (parent_label), "%s", next_token (&cursor));

		busy = next_int (&cursor);
		has_leader = next_int (&cursor);

		x = next_float (&cursor);
		z = next_float (&cursor);

		en = new_entity (ENTITY_TYPE_GROUP, raw, label);

		if (strcmp (parent_label, "independent") == 0)
		{
			for (i = 0; i < num_forces; i++)
			{
				if (((force *) get_local_entity_data (forces[i]))->side == raw->side)
				{
					eech_link_entity_raw (en, LIST_TYPE_INDEPENDENT_GROUP, forces[i], eech_last_child (forces[i], LIST_TYPE_INDEPENDENT_GROUP));

					break;
				}
			}
		}
		else if (strcmp (parent_label, "NULL") != 0)
		{
			entity *parent = find_created (parent_label);

			eech_link_entity_raw (en, LIST_TYPE_KEYSITE_GROUP, parent, eech_last_child (parent, LIST_TYPE_KEYSITE_GROUP));
		}

		if (busy)
		{
			snprintf (member_label, sizeof (member_label), "%s.guide", label);

			eech_link_entity_raw (new_entity (ENTITY_TYPE_GUIDE, NULL, member_label), LIST_TYPE_GUIDE_STACK, en, NULL);
		}

		if (has_leader)
		{
			entity *leader;

			snprintf (member_label, sizeof (member_label), "%s.leader", label);

			leader = new_entity (ENTITY_TYPE_HELICOPTER, eech_new_raw (sizeof (aircraft)), member_label);

			declare_position (leader, x, 0.0f, z);

			eech_link_entity_raw (leader, LIST_TYPE_MEMBER, en, NULL);
		}
	}
	else if (strcmp (word, "task") == 0)
	{
		char label[32];
		entity *en, *objective;

		task *raw = eech_new_raw (sizeof (task));

		snprintf (label, sizeof (label), "%s", next_token (&cursor));

		objective = find_created (next_token (&cursor));

		raw->sub_type = next_int (&cursor);
		raw->side = next_int (&cursor);
		raw->task_state = (task_state_types) next_int (&cursor);
		raw->task_user_data = next_float (&cursor);

		en = new_entity (ENTITY_TYPE_TASK, raw, label);

		eech_link_entity_raw (en, LIST_TYPE_TASK_DEPENDENT, objective, eech_last_child (objective, LIST_TYPE_TASK_DEPENDENT));
	}
	else if (strcmp (word, "waypoint") == 0)
	{
		char label[32];
		entity *en, *dependent;

		waypoint *raw = eech_new_raw (sizeof (waypoint));

		snprintf (label, sizeof (label), "%s", next_token (&cursor));

		dependent = find_created (next_token (&cursor));

		raw->sub_type = next_int (&cursor);

		en = new_entity (ENTITY_TYPE_WAYPOINT, raw, label);

		eech_link_entity_raw (en, LIST_TYPE_TASK_DEPENDENT, dependent, eech_last_child (dependent, LIST_TYPE_TASK_DEPENDENT));
	}
	else if (strcmp (word, "assess-group") == 0)
	{
		entity *target = find_created (next_token (&cursor));

		lifecycle = TRUE;

		OPERATION_BEGIN (print_lifecycle_state ())

		assess_group_supplies (target);

		OPERATION_END

		label_new_tasks ();
	}
	else if (strcmp (word, "update-cargo") == 0)
	{
		entity *target = find_created (next_token (&cursor));
		float level = next_float (&cursor);
		entity_sub_types sub_type = (entity_sub_types) next_int (&cursor);
		float size = next_float (&cursor);
		entity *en;

		lifecycle = TRUE;

		OPERATION_BEGIN (print_lifecycle_state ())

		update_keysite_cargo (target, level, sub_type, size);

		OPERATION_END

		for (en = first_used_entity; en; en = en->succ)
		{
			if ((get_local_entity_type (en) == ENTITY_TYPE_CARGO) && (labels[get_local_entity_index (en)][0] == '\0'))
			{
				snprintf (labels[get_local_entity_index (en)], sizeof (labels[0]), "crate%d", get_local_entity_index (en));
			}
		}

		label_new_tasks ();
	}
	else if ((strcmp (word, "map") == 0) || (strcmp (word, "create") == 0) || (strcmp (word, "destroy") == 0) || (strcmp (word, "allocate") == 0))
	{
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

		OPERATION_BEGIN (print_lifecycle_state ())

		if (strcmp (op, "map") == 0)
		{
			map_complete = FALSE;

			set_entity_world_map_size (x_sectors, z_sectors, side_length);

			create_local_sector_entities ();

			label_sectors ();

			map_complete = TRUE;
		}
		else if (strcmp (op, "create") == 0)
		{
			en = create_n ((entity_types) type, index);

			if (en)
			{
				set_label (en, label);

				snprintf (created[num_created].label, sizeof (created[0].label), "%s", label);

				created[num_created].en = en;

				num_created++;

				eech_out ("created %s %d\n", label, get_local_entity_index (en));
			}
			else
			{
				eech_out ("created %s NULL\n", label);
			}
		}
		else if (strcmp (op, "allocate") == 0)
		{
			en = get_free_entity (index);

			set_local_entity_type (en, ENTITY_TYPE_GROUP);

			set_local_entity_data (en, eech_new_raw (sizeof (group)));

			set_label (en, label);

			eech_out ("allocated %s %d\n", label, get_local_entity_index (en));
		}
		else
		{
			destroy_client_server_entity_family (target);
		}

		OPERATION_END
	}
	else if (strcmp (word, "end") == 0)
	{
		if (timeline || lifecycle)
		{
			eech_out ("result ok\n");

			if (lifecycle)
			{
				print_lifecycle_state ();
			}

			finished = TRUE;
		}
	}
	else if (strcmp (word, "op") == 0)
	{
		word = next_token (&cursor);

		OPERATION_BEGIN (emit_final_state ())

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
				eech_out ("closest %s %08x\n", eech_legacy_label_of (closest), float_bits (actual_range));
			}
			else
			{
				eech_out ("closest %s -\n", eech_legacy_label_of (closest));
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

			eech_out ("range %08x %08x\n", float_bits (get_2d_range (&v1, &v2)), float_bits (get_approx_2d_range (&v1, &v2)));
		}
		else
		{
			fail ("unknown op (the harness self-tests fault-null and fault-unmapped are not replayed)");
		}

		eech_out ("result ok\n");

		OPERATION_END

		emit_final_state ();

		finished = TRUE;
	}
	else
	{
		fail ("unknown scenario line");
	}
}

int eech_k_legacy_replay (const eech_host *host, const char *scenario)
{
	char
		*text;

	int
		status;

	if (eech_in_entry ())
	{
		return EECH_STATUS_REENTRANT;
	}

	if (eech_kernel_open)
	{
		return EECH_STATUS_INVALID;
	}

	text = strdup (scenario);

	if (!text)
	{
		return EECH_STATUS_FATAL;
	}

	eech_kernel_begin_replay ();

	eech_legacy_active = TRUE;

	status = eech_enter (host, replay_body, text);

	free (text);

	eech_kernel_end_replay ();

	return status;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// a fault inside the NULL page is EECH's unguarded NULL dereference: the
// harness's "result null-dereference" outcome. Installed only by the replay
// process (eech-harness replay --process), never by the library. The
// handler's call graph reaches only write () and _exit ().
//
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#if defined (__linux__)

#include <signal.h>
#include <unistd.h>

#define LITERAL(TEXT) (TEXT), (sizeof (TEXT) - 1)

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

static void segmentation_fault (int signal_number, siginfo_t *info, void *context)
{
	int
		i;

	if ((uintptr_t) info->si_addr < 4096)
	{
		write_all (LITERAL ("result null-dereference\n"));

		if (final_group_raw)
		{
			write_final_pair (LITERAL ("final group "), final_group_raw->supplies.ammo_supply_level, final_group_raw->supplies.fuel_supply_level);
		}

		for (i = 0; i < final_keysite_count; i++)
		{
			write_final_pair (LITERAL ("final keysite "), final_keysites[i]->supplies.ammo_supply_level, final_keysites[i]->supplies.fuel_supply_level);
		}

		_exit (0);
	}
}

int eech_k_legacy_install_fault_handler (void)
{
	struct sigaction
		action;

	memset (&action, 0, sizeof (action));

	action.sa_sigaction = segmentation_fault;

	action.sa_flags = SA_SIGINFO | SA_RESETHAND;

	sigemptyset (&action.sa_mask);

	return sigaction (SIGSEGV, &action, NULL) == 0 ? EECH_STATUS_OK : EECH_STATUS_FATAL;
}

#else

int eech_k_legacy_install_fault_handler (void)
{
	return EECH_STATUS_UNPORTED;
}

#endif

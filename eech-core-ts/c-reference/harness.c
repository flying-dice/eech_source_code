/*
 * EECH C reference harness.
 *
 * Reads one scenario (the line format written by test/scenarios/campaign-scenario.ts ::
 * serialiseScenario) from stdin, builds the entity state, runs ONE operation
 * through the original EECH functions (extracted verbatim into
 * build/c-reference/eech_extracted.c) and prints the observable outcome.
 *
 * Output lines (floats as IEEE 754 single precision bit patterns, %08x):
 *
 *   transmit <entity> <float_type> <bits>        set_client_server_entity_float_value (server)
 *   message <receiver> <sender> <message> <arg>  delivery to FORCE/LOW_ON_SUPPLIES response
 *   closest <entity|NULL> <bits|->               result of op closest
 *   range <bits get_2d_range> <bits get_approx_2d_range>
 *   result ok | result assert <expression> | result null-dereference
 *   final group <ammo bits> <fuel bits>
 *   final keysite <ammo bits> <fuel bits>        one line per keysite, scenario order
 */

#include <setjmp.h>
#include <stdlib.h>
#include <string.h>

#include "eech_shim_prelude.h"

#include "eech_extracted.c"

#define MAX_ENTITIES 256

static entity
	entities[MAX_ENTITIES];

static int
	num_entities;

entity
	*session_entity;

int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

static jmp_buf
	abort_operation;

static char
	labels[MAX_ENTITIES][32];

static void harness_fail (const char *what)
{
	fprintf (stderr, "harness: %s\n", what);

	exit (2);
}

void harness_assert (const char *expression)
{
	printf ("result assert %s\n", expression);

	longjmp (abort_operation, 1);
}

static void null_dereference (void)
{
	printf ("result null-dereference\n");

	longjmp (abort_operation, 1);
}

static const char *label_of (entity *en)
{
	return en ? labels[en->index] : "NULL";
}

static unsigned int float_bits (float value)
{
	unsigned int
		bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

static entity *new_entity (entity_types type, void *data, const char *label)
{
	entity
		*en;

	if (num_entities >= MAX_ENTITIES)
	{
		harness_fail ("too many entities");
	}

	en = &entities[num_entities];

	memset (en, 0, sizeof (*en));

	en->type = type;
	en->index = num_entities;
	en->data = data;

	snprintf (labels[num_entities], sizeof (labels[num_entities]), "%s", label);

	num_entities++;

	return en;
}

/* LIST_TYPE_GROUP_LINK: one link serves three list types (en_list/get_prnt.h) */
static list_types link_slot (list_types type)
{
	if ((type == LIST_TYPE_BUILDING_GROUP) || (type == LIST_TYPE_INDEPENDENT_GROUP))
	{
		return LIST_TYPE_KEYSITE_GROUP;
	}

	return type;
}

/* pointer updates of insert_local_entity_into_parents_child_list, without notifications */
static void link_entity (entity *en, list_types type, entity *parent, entity *pred)
{
	entity
		*succ;

	list_types
		link = link_slot (type);

	succ = pred ? pred->child_succ[link] : parent->first_child[type];

	en->child_succ[link] = succ;
	en->child_pred[link] = pred;
	en->parent[link] = parent;

	if (succ)
	{
		succ->child_pred[link] = en;
	}

	if (pred)
	{
		pred->child_succ[link] = en;
	}
	else
	{
		parent->first_child[type] = en;
	}
}

void *get_local_entity_data (entity *en)
{
	if (!en) null_dereference ();

	return en->data;
}

entity_types get_local_entity_type (entity *en)
{
	if (!en) null_dereference ();

	return en->type;
}

entity *get_local_entity_first_child (entity *en, list_types type)
{
	if (!en) null_dereference ();

	return en->first_child[type];
}

entity *get_local_entity_parent (entity *en, list_types type)
{
	if (!en) null_dereference ();

	return en->parent[link_slot (type)];
}

entity *get_local_entity_child_succ (entity *en, list_types type)
{
	if (!en) null_dereference ();

	return en->child_succ[link_slot (type)];
}

int get_local_entity_int_value (entity *en, int_types type)
{
	if (!en) null_dereference ();

	switch (en->type)
	{
		case ENTITY_TYPE_GROUP:
		{
			group *raw = (group *) en->data;

			if (type == INT_TYPE_RESUPPLY_SOURCE) return raw->resupply_source;
			if (type == INT_TYPE_GROUP_MODE) return en->first_child[LIST_TYPE_GUIDE_STACK] ? GROUP_MODE_BUSY : GROUP_MODE_IDLE;
			if (type == INT_TYPE_SIDE) return raw->side;

			break;
		}
		case ENTITY_TYPE_KEYSITE:
		{
			keysite *raw = (keysite *) en->data;

			if (type == INT_TYPE_ENTITY_SUB_TYPE) return raw->sub_type;
			if (type == INT_TYPE_IN_USE) return raw->in_use;
			if (type == INT_TYPE_SIDE) return raw->side;

			break;
		}
		case ENTITY_TYPE_FORCE:
		{
			force *raw = (force *) en->data;

			if (type == INT_TYPE_SIDE) return raw->side;

			break;
		}
		default:
		{
			break;
		}
	}

	harness_fail ("int value not supplied by shim");

	return 0;
}

static supply_type *supplies_of (entity *en)
{
	if (en->type == ENTITY_TYPE_GROUP) return &((group *) en->data)->supplies;
	if (en->type == ENTITY_TYPE_KEYSITE) return &((keysite *) en->data)->supplies;

	harness_fail ("supplies not supplied by shim");

	return NULL;
}

float get_local_entity_float_value (entity *en, float_types type)
{
	if (!en) null_dereference ();

	if (type == FLOAT_TYPE_AMMO_SUPPLY_LEVEL) return supplies_of (en)->ammo_supply_level;
	if (type == FLOAT_TYPE_FUEL_SUPPLY_LEVEL) return supplies_of (en)->fuel_supply_level;

	harness_fail ("float value not supplied by shim");

	return 0.0f;
}

/* COMMS_MODEL_SERVER: set_server_float_value = set_local_float_value + transmit */
void set_client_server_entity_float_value (entity *en, float_types type, float value)
{
	if (!en) null_dereference ();

	if (type == FLOAT_TYPE_AMMO_SUPPLY_LEVEL) supplies_of (en)->ammo_supply_level = value;
	else if (type == FLOAT_TYPE_FUEL_SUPPLY_LEVEL) supplies_of (en)->fuel_supply_level = value;
	else harness_fail ("float set not supplied by shim");

	printf ("transmit %s %d %08x\n", label_of (en), (int) type, float_bits (value));
}

vec3d *get_local_entity_vec3d_ptr (entity *en, vec3d_types type)
{
	if (!en) null_dereference ();

	if (type != VEC3D_TYPE_POSITION) harness_fail ("vec3d not supplied by shim");

	switch (en->type)
	{
		case ENTITY_TYPE_KEYSITE: return &((keysite *) en->data)->position;
		case ENTITY_TYPE_HELICOPTER: return &((mobile *) en->data)->position;
		case ENTITY_TYPE_GROUP:
		{
			entity *leader = en->first_child[LIST_TYPE_MEMBER];		/* PTR_TYPE_GROUP_LEADER */

			return leader ? get_local_entity_vec3d_ptr (leader, type) : NULL;
		}
		default: break;
	}

	harness_fail ("position not supplied by shim");

	return NULL;
}

static int record_force_low_on_supplies (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	int
		sub_type = va_arg (pargs, int);

	printf ("message %s %s %d %d\n", label_of (receiver), label_of (sender), (int) message, sub_type);

	return FALSE;
}

static int unsupplied_message_response (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	harness_fail ("message response not supplied by shim");

	return FALSE;
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

int main (void)
{
	static force force_data[MAX_ENTITIES];
	static keysite keysite_data[MAX_ENTITIES];
	static group group_data;
	static mobile leader_data;

	entity
		*session,
		*forces[MAX_ENTITIES],
		*keysites[MAX_ENTITIES],
		*keysite_tail[NUM_ENTITY_SIDES] = { NULL },
		*group_en = NULL;

	int
		num_forces = 0,
		num_keysites = 0,
		i,
		j;

	char
		line[512],
		*cursor,
		*word;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_ENTITY_MESSAGES; j++)
		{
			message_responses[i][j] = unsupplied_message_response;
		}
	}

	message_responses[ENTITY_TYPE_FORCE][ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES] = record_force_low_on_supplies;

	session = new_entity (ENTITY_TYPE_SESSION, NULL, "session");

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

			link_entity (forces[num_forces], LIST_TYPE_FORCE, session, num_forces ? forces[num_forces - 1] : NULL);

			num_forces++;
		}
		else if (strcmp (word, "keysite") == 0)
		{
			char label[32];

			keysite *raw = &keysite_data[num_keysites];

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
					link_entity (keysites[num_keysites], LIST_TYPE_KEYSITE_FORCE, forces[i], keysite_tail[raw->side]);

					keysite_tail[raw->side] = keysites[num_keysites];

					break;
				}
			}

			num_keysites++;
		}
		else if (strcmp (word, "group") == 0)
		{
			int parent_kind, parent_index, busy, has_leader;

			next_int (&cursor);		/* group sub type: the shim receives its resupply_source directly */

			group_data.resupply_source = next_int (&cursor);
			group_data.side = (entity_sides) next_int (&cursor);
			group_data.supplies.ammo_supply_level = next_double (&cursor);
			group_data.supplies.fuel_supply_level = next_double (&cursor);

			parent_kind = next_int (&cursor);
			parent_index = next_int (&cursor);
			busy = next_int (&cursor);
			has_leader = next_int (&cursor);

			leader_data.position.x = next_double (&cursor);
			leader_data.position.y = 0.0;
			leader_data.position.z = next_double (&cursor);

			group_en = new_entity (ENTITY_TYPE_GROUP, &group_data, "group");

			if (parent_kind == 1)
			{
				link_entity (group_en, LIST_TYPE_KEYSITE_GROUP, keysites[parent_index], NULL);
			}
			else if (parent_kind == 2)
			{
				for (i = 0; i < num_forces; i++)
				{
					if (force_data[i].side == group_data.side)
					{
						link_entity (group_en, LIST_TYPE_INDEPENDENT_GROUP, forces[i], NULL);

						break;
					}
				}
			}

			if (busy)
			{
				link_entity (new_entity (ENTITY_TYPE_GUIDE, NULL, "guide"), LIST_TYPE_GUIDE_STACK, group_en, NULL);
			}

			if (has_leader)
			{
				link_entity (new_entity (ENTITY_TYPE_HELICOPTER, &leader_data, "leader"), LIST_TYPE_MEMBER, group_en, NULL);
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
				else
				{
					harness_fail ("unknown op");
				}

				printf ("result ok\n");
			}

			if (group_en)
			{
				printf ("final group %08x %08x\n", float_bits (group_data.supplies.ammo_supply_level), float_bits (group_data.supplies.fuel_supply_level));
			}

			for (i = 0; i < num_keysites; i++)
			{
				printf ("final keysite %08x %08x\n", float_bits (keysite_data[i].supplies.ammo_supply_level), float_bits (keysite_data[i].supplies.fuel_supply_level));
			}

			return 0;
		}
		else
		{
			harness_fail ("unknown scenario line");
		}
	}

	harness_fail ("scenario has no op line");

	return 1;
}

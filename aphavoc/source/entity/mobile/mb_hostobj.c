//
// 	Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	677 High Road, North Finchley, London N12 0DA
//
// 	Please see the document LICENSE.TXT for the full licence agreement
//
// 2. LICENCE
//  2.1
//  	Subject to the provisions of this Agreement we now grant to you the
//  	following rights in respect of the Source Code:
//   2.1.1
//   	the non-exclusive right to Exploit  the Source Code and Executable
//   	Code on any medium; and
//   2.1.2
//   	the non-exclusive right to create and distribute Derivative Works.
//  2.2
//  	Subject to the provisions of this Agreement we now grant you the
//  	following rights in respect of the Executable Code:
//   2.2.1
//   	the non-exclusive right to Exploit the Executable Code on any
//   	medium; and
//   2.2.2
//   	the non-exclusive right to distribute the Executable Code.
//  2.3
//  	You agree to Exploit the Source Code, Executable Code and Derivative Works
//  	only in accordance with the terms of this Agreement.
//

// See mb_hostobj.h: the structures' roster (EEDCS-46, R3) and the physical facts the host reports about an object it has the identity of.

#include "project.h"

#include "entity/mobile/mb_hostaim.h"

#include "entity/mobile/mb_hostobj.h"

typedef struct object_record
{
	char host_id[CAMPAIGN_OBJECT_ID_MAX + 1];
	// the entity the identity was issued for, while that incarnation lives (-1 once it was released); the index is looked up only to find the live entity, it is not what names the object
	int entity_index;
	int entity_type;
} object_record;

// every identity ever issued for a structure, in the order issued; none is ever reused or removed, an ended one is only marked
static object_record
	*records = NULL;

static size_t
	record_count = 0,
	record_capacity = 0;

// the record of the live incarnation in each entity slot (1-based; 0 none)
static int
	*record_of_entity = NULL;

// open-addressing table over the records by identity (record number + 1; 0 empty); records are never removed so no tombstones are needed
static int
	*table = NULL;

static size_t
	table_size = 0;

static int
	issued_count = 0;

int campaign_object_is_structure (int entity_type)
{
	switch (entity_type)
	{
		case ENTITY_TYPE_CITY_BUILDING:
		case ENTITY_TYPE_PYLON:
		case ENTITY_TYPE_SCENIC:
		case ENTITY_TYPE_SEGMENT:
		case ENTITY_TYPE_SITE:
		case ENTITY_TYPE_SITE_UPDATABLE:
		{
			return TRUE;
		}
		default:
		{
			return FALSE;
		}
	}
}

static size_t hash_id (const char *id)
{
	// FNV-1a
	size_t hash = 2166136261u;

	while (*id)
	{
		hash ^= (unsigned char) *id++;

		hash *= 16777619u;
	}

	return hash;
}

static int find_record (const char *id)
{
	size_t slot;

	if (!table)
	{
		return -1;
	}

	slot = hash_id (id) & (table_size - 1);

	while (table[slot])
	{
		if (strcmp (records[table[slot] - 1].host_id, id) == 0)
		{
			return table[slot] - 1;
		}

		slot = (slot + 1) & (table_size - 1);
	}

	return -1;
}

static void insert_into_table (int record)
{
	size_t slot = hash_id (records[record].host_id) & (table_size - 1);

	while (table[slot])
	{
		slot = (slot + 1) & (table_size - 1);
	}

	table[slot] = record + 1;
}

static void grow_table (void)
{
	size_t i;

	size_t wanted = table_size ? table_size : 1024;

	while ((record_count + 1) * 2 > wanted)
	{
		wanted *= 2;
	}

	if (wanted == table_size)
	{
		return;
	}

	free (table);

	table = (int *) calloc (wanted, sizeof (int));

	if (!table)
	{
		debug_fatal ("host objects: cannot allocate an identity table of %u entries", (unsigned int) wanted);
	}

	table_size = wanted;

	for (i = 0; i < record_count; i++)
	{
		insert_into_table ((int) i);
	}
}

void campaign_object_reset (void)
{
	free (records);
	free (record_of_entity);
	free (table);

	records = NULL;
	record_of_entity = NULL;
	table = NULL;
	record_count = 0;
	record_capacity = 0;
	table_size = 0;
	issued_count = 0;
}

const char *campaign_object_host_id (struct ENTITY *en)
{
	int type;

	if (!en)
	{
		return NULL;
	}

	type = (int) get_local_entity_type (en);

	if (campaign_object_is_structure (type))
	{
		int index = get_local_entity_index (en);

		if (record_of_entity && (index >= 0) && (index < MAX_NUM_ENTITIES) && record_of_entity[index])
		{
			return records[record_of_entity[index] - 1].host_id;
		}

		return NULL;
	}

	return campaign_roster_host_id (en);
}

// an identity is text the campaign carries and never reads: printable, and not already an identity of this registry or of the roster (whatever incarnation it named)
static void check_issued_identity (const char *id)
{
	const unsigned char *p;

	for (p = (const unsigned char *) id; *p; p++)
	{
		if ((*p < 0x20) || (*p == 0x7f))
		{
			debug_fatal ("host objects: the host issued an identity with a control character");
		}
	}

	if (find_record (id) >= 0)
	{
		debug_fatal ("host objects: the host issued the identity \"%.40s\", which it issued before", id);
	}

	if (campaign_roster_find_host_id (id, NULL) != 0)
	{
		debug_fatal ("host objects: the host issued the identity \"%.40s\" of a rostered member for a structure", id);
	}
}

const char *campaign_object_identify (struct ENTITY *en)
{
	const char *id;

	char issued [CAMPAIGN_OBJECT_ID_MAX + 1];

	campaign_identify_request request;

	vec3d *position;

	int type, index;

	id = campaign_object_host_id (en);

	if (id)
	{
		return id;
	}

	type = (int) get_local_entity_type (en);

	if (!campaign_object_is_structure (type))
	{
		return NULL;
	}

	index = get_local_entity_index (en);

	if ((index < 0) || (index >= MAX_NUM_ENTITIES))
	{
		debug_fatal ("host objects: entity %d is outside the entity heap", index);
	}

	memset (&request, 0, sizeof (request));

	request.entity_type = type;
	request.sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);
	request.side = get_local_entity_int_value (en, INT_TYPE_SIDE);
	request.alive = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? TRUE : FALSE;
	request.damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL);
	request.initial_damage_level = get_local_entity_int_value (en, INT_TYPE_INITIAL_DAMAGE_LEVEL);
	request.serial = issued_count + 1;

	position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	if (position)
	{
		request.position[0] = position->x;
		request.position[1] = position->y;
		request.position[2] = position->z;
	}

	campaign_aim_identify_object (&request, issued);

	check_issued_identity (issued);

	if (!record_of_entity)
	{
		record_of_entity = (int *) calloc ((size_t) MAX_NUM_ENTITIES, sizeof (int));

		if (!record_of_entity)
		{
			debug_fatal ("host objects: cannot allocate the identity map of the entity heap");
		}
	}

	if (record_count == record_capacity)
	{
		size_t capacity = record_capacity ? record_capacity * 2 : 256;

		object_record *grown = (object_record *) realloc (records, capacity * sizeof (object_record));

		if (!grown)
		{
			debug_fatal ("host objects: cannot allocate %u identity records", (unsigned int) capacity);
		}

		records = grown;

		record_capacity = capacity;
	}

	memset (&records[record_count], 0, sizeof (object_record));

	memcpy (records[record_count].host_id, issued, strlen (issued) + 1);

	records[record_count].entity_index = index;

	records[record_count].entity_type = type;

	record_count++;

	issued_count++;

	grow_table ();

	insert_into_table ((int) record_count - 1);

	record_of_entity[index] = (int) record_count;

	return records[record_count - 1].host_id;
}

void campaign_object_release_entity (struct ENTITY *en)
{
	int index, record;

	if (!record_of_entity)
	{
		return;
	}

	index = get_local_entity_index (en);

	if ((index < 0) || (index >= MAX_NUM_ENTITIES) || !record_of_entity[index])
	{
		return;
	}

	record = record_of_entity[index] - 1;

	records[record].entity_index = -1;

	record_of_entity[index] = 0;
}

// the live entity an identity names: the registry's structure, or the roster's member.  Stops with a clear error for an identity that was never issued or whose incarnation has ended
static struct ENTITY *resolve_report_target (const char *id, int *rostered)
{
	int record = find_record (id);

	struct ENTITY *member;

	*rostered = FALSE;

	if (record >= 0)
	{
		struct ENTITY *en;

		if (records[record].entity_index < 0)
		{
			debug_fatal ("host objects: the identity \"%.40s\" belongs to a structure that no longer exists", id);
		}

		en = get_local_entity_ptr (records[record].entity_index);

		if ((int) get_local_entity_type (en) != records[record].entity_type)
		{
			debug_fatal ("host objects: the identity \"%.40s\" no longer names the structure it was issued for", id);
		}

		return en;
	}

	switch (campaign_roster_find_host_id (id, &member))
	{
		case 1:
		{
			*rostered = TRUE;

			return member;
		}
		case 2:
		{
			debug_fatal ("host objects: the identity \"%.40s\" belongs to a member that no longer exists", id);

			break;
		}
		default:
		{
			debug_fatal ("host objects: the identity \"%.40s\" was never issued", id);

			break;
		}
	}

	return NULL;
}

void campaign_object_report_apply (const campaign_object_report *report, campaign_object_report_receipt *receipt)
{
	struct ENTITY *en;

	int rostered, level_before, level_after, kill;

	if (!memchr (report->host_id, '\0', CAMPAIGN_OBJECT_ID_MAX + 1) || (report->host_id[0] == '\0'))
	{
		debug_fatal ("host objects: a report without an identity");
	}

	if ((report->has_damage_level & ~1) || (report->destroyed & ~1) || (!report->has_damage_level && (report->damage_level != 0)) || (!report->has_damage_level && !report->destroyed))
	{
		debug_fatal ("host objects: a report of \"%.40s\" that states neither a damage level nor a destruction, or states them inconsistently", report->host_id);
	}

	en = resolve_report_target (report->host_id, &rostered);

	if (get_comms_model () != COMMS_MODEL_SERVER)
	{
		debug_fatal ("host objects: a physical report is applied by the server only");
	}

	if (!get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		debug_fatal ("host objects: \"%.40s\" is already destroyed: a fact is applied once", report->host_id);
	}

	level_before = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL);

	level_after = level_before;

	if (report->has_damage_level)
	{
		if ((report->damage_level < 0) || (report->damage_level > level_before))
		{
			debug_fatal ("host objects: a damage level of %d for \"%.40s\", which has %d and cannot be repaired by a report", report->damage_level, report->host_id, level_before);
		}

		level_after = report->damage_level;
	}

	kill = (report->destroyed || (report->has_damage_level && (level_after <= 0))) ? TRUE : FALSE;

	receipt->entity_type = (int) get_local_entity_type (en);

	receipt->rostered = rostered;

	receipt->damage_level_before = level_before;

	receipt->damage_level_after = level_after;

	receipt->killed = kill;

	// the two steps the collision response of every damaged object takes: the level is set, then the object is killed when it is destroyed.  Nothing about the object is read after the kill
	if (level_after != level_before)
	{
		set_client_server_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, level_after);
	}

	if (kill)
	{
		kill_client_server_entity (en);
	}
}

int campaign_object_structures (campaign_object_snapshot_fn callback, void *user)
{
	int index, count;

	count = 0;

	for (index = 0; index < MAX_NUM_ENTITIES; index++)
	{
		struct ENTITY *en = get_local_entity_ptr (index);

		campaign_object_snapshot snapshot;

		vec3d *position;

		if (!campaign_object_is_structure ((int) get_local_entity_type (en)))
		{
			continue;
		}

		memset (&snapshot, 0, sizeof (snapshot));

		snapshot.host_id = campaign_object_identify (en);

		snapshot.entity_type = (int) get_local_entity_type (en);

		snapshot.sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

		snapshot.side = get_local_entity_int_value (en, INT_TYPE_SIDE);

		snapshot.alive = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? TRUE : FALSE;

		snapshot.damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL);

		snapshot.initial_damage_level = get_local_entity_int_value (en, INT_TYPE_INITIAL_DAMAGE_LEVEL);

		position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

		if (position)
		{
			snapshot.position[0] = position->x;
			snapshot.position[1] = position->y;
			snapshot.position[2] = position->z;
		}

		callback (&snapshot, user);

		count++;
	}

	return count;
}

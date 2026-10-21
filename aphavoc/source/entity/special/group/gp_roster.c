#include "project.h"

#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "ai/taskgen/taskgen.h"
#include "gp_roster.h"

typedef struct campaign_member_record
{
	uint64_t member_key;
	uint64_t group_key;
	char host_id[CAMPAIGN_HOST_ID_MAX + 1];
	int origin_entity_index;
	int current_entity_index;
	int entity_type;
	int sub_type;
	int side;
	int initial_alive;
	int physical_present;
	int in_group;
} campaign_member_record;

static campaign_member_record *records;
static size_t record_count;
static size_t record_capacity;
static int *current_member_by_entity;
static uint64_t *current_group_by_entity;
static int map_size;
static uint64_t next_member_key = 1;
static uint64_t next_group_key = 1;
static uint64_t duplicate_comparisons;
static clock_t duplicate_scan_ticks;
static campaign_roster_host current_host;

static int entity_index (entity *en);

static campaign_member_record *record_for_member (entity *member)
{
	int index = entity_index (member);
	int record_index = current_member_by_entity[index] - 1;
	if (record_index < 0)
	{
		debug_fatal ("campaign roster: member %d has no host identity", index);
	}
	return &records[record_index];
}

static int child_count (entity *parent, list_types list_type)
{
	int count = 0;
	entity *child = get_local_entity_first_child (parent, list_type);
	while (child)
	{
		count++;
		child = get_local_entity_child_succ (child, list_type);
	}
	return count;
}

static int restricted_guide_count (entity *group)
{
	int count = 0;
	entity *guide = get_local_entity_first_child (group, LIST_TYPE_GUIDE_STACK);
	while (guide)
	{
		if (get_local_entity_int_value (guide, INT_TYPE_VALID_GUIDE_MEMBERS) != TASK_ASSIGN_ALL_MEMBERS)
		{
			count++;
		}
		guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE_STACK);
	}
	return count;
}

static int landing_release_can_change_state (entity *member, entity *group)
{
	entity *guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);
	entity *task = guide ? get_local_entity_parent (guide, LIST_TYPE_GUIDE) : NULL;
	if (get_local_entity_int_value (member, INT_TYPE_LANDED))
	{
		return TRUE;
	}
	if (task)
	{
		int sub_type = get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE);
		if ((sub_type == ENTITY_SUB_TYPE_TASK_LANDING) ||
			(sub_type == ENTITY_SUB_TYPE_TASK_LANDING_HOLDING) ||
			(sub_type == ENTITY_SUB_TYPE_TASK_TAKEOFF))
		{
			return TRUE;
		}
		if (get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP))
		{
			entity *primary_task = get_local_group_primary_task (group);
			return primary_task && get_local_entity_int_value (primary_task, INT_TYPE_ASSESS_LANDING);
		}
	}
	return FALSE;
}

static int observe_loss_event (entity *member, entity *group, campaign_loss_event_kind event_kind)
{
	campaign_loss_descriptor descriptor;
	campaign_member_record *record;
	entity *task;
	char error[CAMPAIGN_HOST_ERROR_MAX + 1];
	int decision = CAMPAIGN_LOSS_FORWARD;

	if (!current_host.observe_loss)
	{
		debug_fatal ("campaign roster: host observe_loss callback is unavailable");
	}
	record = record_for_member (member);
	task = get_local_group_primary_task (group);
	memset (&descriptor, 0, sizeof (descriptor));
	memset (error, 0, sizeof (error));
	descriptor.host_id = record->host_id;
	descriptor.member_key = record->member_key;
	descriptor.group_key = record->group_key;
	descriptor.event_kind = event_kind;
	descriptor.entity_type = record->entity_type;
	descriptor.sub_type = record->sub_type;
	descriptor.side = record->side;
	descriptor.group_sub_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);
	descriptor.primary_task_sub_type = task ? get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) : -1;
	descriptor.group_member_number = get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER);
	descriptor.campaign_member_count = get_local_entity_int_value (group, INT_TYPE_MEMBER_COUNT);
	descriptor.player = get_local_entity_int_value (member, INT_TYPE_PLAYER);
	descriptor.landed = get_local_entity_int_value (member, INT_TYPE_LANDED);
	descriptor.member_task_dependents = child_count (member, LIST_TYPE_TASK_DEPENDENT);
	descriptor.group_task_dependents = child_count (group, LIST_TYPE_TASK_DEPENDENT);
	descriptor.restricted_guides = restricted_guide_count (group);
	descriptor.landing_effect = landing_release_can_change_state (member, group);
	descriptor.movement_parent = get_local_entity_parent (member, LIST_TYPE_MOVEMENT_DEPENDENT) != NULL;
	descriptor.movement_dependents = child_count (member, LIST_TYPE_MOVEMENT_DEPENDENT);
	descriptor.takeoff_queue = get_local_entity_parent (member, LIST_TYPE_TAKEOFF_QUEUE) != NULL;

	if (current_host.observe_loss (&descriptor, &decision, error, CAMPAIGN_HOST_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("campaign roster: host observe_loss failed: %s", error[0] ? error : "unspecified error");
	}
	if ((decision != CAMPAIGN_LOSS_FORWARD) && (decision != CAMPAIGN_LOSS_WITHHOLD))
	{
		debug_fatal ("campaign roster: host observe_loss returned invalid decision %d", decision);
	}
	return decision;
}

static int entity_index (entity *en)
{
	int index = get_local_entity_index (en);
	if ((index < 0) || (index >= map_size))
	{
		debug_fatal ("campaign roster: entity index %d is outside the heap", index);
	}
	return index;
}

static uint64_t group_key (entity *group)
{
	int index = entity_index (group);
	if (!current_group_by_entity[index])
	{
		current_group_by_entity[index] = next_group_key++;
		if (!current_group_by_entity[index])
		{
			debug_fatal ("campaign roster: group key overflow");
		}
	}
	return current_group_by_entity[index];
}

static void ensure_record_capacity (void)
{
	if (record_count == record_capacity)
	{
		size_t capacity = record_capacity ? record_capacity * 2 : 1024;
		campaign_member_record *grown = (campaign_member_record *) realloc (records, capacity * sizeof (*records));
		if (!grown)
		{
			debug_fatal ("campaign roster: cannot allocate %u records", (unsigned int) capacity);
		}
		records = grown;
		record_capacity = capacity;
	}
}

static int find_duplicate_host_id (const char *host_id)
{
	clock_t started = clock ();
	size_t i;
	for (i = 0; i < record_count; ++i)
	{
		duplicate_comparisons++;
		if (strcmp (records[i].host_id, host_id) == 0)
		{
			duplicate_scan_ticks += clock () - started;
			return TRUE;
		}
	}
	duplicate_scan_ticks += clock () - started;
	return FALSE;
}

static int valid_utf8 (const unsigned char *text, size_t length)
{
	size_t i = 0;
	while (i < length)
	{
		unsigned char first = text[i++];
		unsigned int codepoint;
		int remaining;
		if (first < 0x80)
		{
			continue;
		}
		if ((first >= 0xC2) && (first <= 0xDF))
		{
			codepoint = first & 0x1F;
			remaining = 1;
		}
		else if ((first >= 0xE0) && (first <= 0xEF))
		{
			codepoint = first & 0x0F;
			remaining = 2;
		}
		else if ((first >= 0xF0) && (first <= 0xF4))
		{
			codepoint = first & 0x07;
			remaining = 3;
		}
		else
		{
			return FALSE;
		}
		if (i + (size_t) remaining > length)
		{
			return FALSE;
		}
		while (remaining--)
		{
			unsigned char continuation = text[i++];
			if ((continuation & 0xC0) != 0x80)
			{
				return FALSE;
			}
			codepoint = (codepoint << 6) | (continuation & 0x3F);
		}
		if (((codepoint < 0x800) && (first >= 0xE0)) ||
			((codepoint < 0x10000) && (first >= 0xF0)) ||
			((codepoint >= 0xD800) && (codepoint <= 0xDFFF)) ||
			(codepoint > 0x10FFFF))
		{
			return FALSE;
		}
	}
	return TRUE;
}

void campaign_roster_reset (void)
{
	free (records);
	free (current_member_by_entity);
	free (current_group_by_entity);
	records = NULL;
	record_count = 0;
	record_capacity = 0;
	map_size = number_of_entities;
	current_member_by_entity = (int *) calloc ((size_t) map_size, sizeof (*current_member_by_entity));
	current_group_by_entity = (uint64_t *) calloc ((size_t) map_size, sizeof (*current_group_by_entity));
	next_member_key = 1;
	next_group_key = 1;
	duplicate_comparisons = 0;
	duplicate_scan_ticks = 0;
	if (!current_member_by_entity || !current_group_by_entity)
	{
		debug_fatal ("campaign roster: cannot allocate maps for %d entities", map_size);
	}
}

void campaign_roster_set_host (const campaign_roster_host *host)
{
	if (host)
	{
		current_host = *host;
	}
	else
	{
		memset (&current_host, 0, sizeof (current_host));
	}
}

void campaign_roster_link_member (entity *group, entity *member)
{
	int member_index = entity_index (member);
	int record_index = current_member_by_entity[member_index] - 1;
	uint64_t key = group_key (group);

	if (record_index >= 0)
	{
		campaign_member_record *record = &records[record_index];
		if (!record->physical_present || (record->current_entity_index != member_index))
		{
			debug_fatal ("campaign roster: stale current member mapping for entity %d", member_index);
		}
		record->group_key = key;
		record->in_group = TRUE;
		return;
	}

	if (!current_host.bind_member)
	{
		debug_fatal ("campaign roster: host bind_mobile callback is unavailable");
	}

	{
		campaign_member_descriptor descriptor;
		char host_id[CAMPAIGN_HOST_ID_MAX + 2];
		char error[CAMPAIGN_HOST_ERROR_MAX + 1];
		size_t length;
		campaign_member_record *record;

		memset (&descriptor, 0, sizeof (descriptor));
		memset (host_id, 0, sizeof (host_id));
		memset (error, 0, sizeof (error));
		descriptor.member_key = next_member_key;
		descriptor.group_key = key;
		descriptor.entity_index = member_index;
		descriptor.entity_type = get_local_entity_type (member);
		descriptor.sub_type = get_local_entity_int_value (member, INT_TYPE_ENTITY_SUB_TYPE);
		descriptor.side = get_local_entity_int_value (member, INT_TYPE_SIDE);

		if (current_host.bind_member (&descriptor, host_id, CAMPAIGN_HOST_ID_MAX + 1, error, CAMPAIGN_HOST_ERROR_MAX, current_host.user) != 0)
		{
			debug_fatal ("campaign roster: host bind_mobile failed: %s", error[0] ? error : "unspecified error");
		}
		length = strnlen (host_id, CAMPAIGN_HOST_ID_MAX + 1);
		if ((length == 0) || (length > CAMPAIGN_HOST_ID_MAX))
		{
			debug_fatal ("campaign roster: host bind_mobile returned an invalid identity length");
		}
		if (!valid_utf8 ((const unsigned char *) host_id, length))
		{
			debug_fatal ("campaign roster: host bind_mobile returned a non-UTF-8 identity");
		}
		if (find_duplicate_host_id (host_id))
		{
			debug_fatal ("campaign roster: host identity %s was reused for a new physical incarnation", host_id);
		}

		ensure_record_capacity ();
		record = &records[record_count];
		memset (record, 0, sizeof (*record));
		record->member_key = next_member_key++;
		if (!next_member_key)
		{
			debug_fatal ("campaign roster: member key overflow");
		}
		record->group_key = key;
		memcpy (record->host_id, host_id, length + 1);
		record->origin_entity_index = member_index;
		record->current_entity_index = member_index;
		record->entity_type = descriptor.entity_type;
		record->sub_type = descriptor.sub_type;
		record->side = descriptor.side;
		record->initial_alive = get_local_entity_int_value (member, INT_TYPE_ALIVE) ? TRUE : FALSE;
		record->physical_present = TRUE;
		record->in_group = TRUE;
		current_member_by_entity[member_index] = (int) record_count + 1;
		record_count++;
	}
}

void campaign_roster_unlink_member (entity *group, entity *member)
{
	int index = entity_index (member);
	int record_index = current_member_by_entity[index] - 1;
	(void) group;
	if (record_index >= 0)
	{
		records[record_index].in_group = FALSE;
	}
}

void campaign_roster_release_entity (entity *en)
{
	int index;
	int record_index;
	if (!current_member_by_entity || !current_group_by_entity || !entities)
	{
		return;
	}
	index = get_local_entity_index (en);
	if ((index < 0) || (index >= map_size))
	{
		return;
	}
	record_index = current_member_by_entity[index] - 1;
	if (record_index >= 0)
	{
		campaign_member_record *record = &records[record_index];
		record->physical_present = FALSE;
		record->in_group = FALSE;
		record->current_entity_index = ENTITY_INDEX_DONT_CARE;
		current_member_by_entity[index] = 0;
	}
	current_group_by_entity[index] = 0;
}

int campaign_roster_observe_credit (entity *member)
{
	entity *group = get_local_entity_parent (member, LIST_TYPE_MEMBER);
	if (!group)
	{
		debug_fatal ("campaign roster: credited member %d has no group", get_local_entity_index (member));
	}
	return observe_loss_event (member, group, CAMPAIGN_LOSS_EVENT_CREDIT);
}

int campaign_roster_observe_physical_loss (entity *member, entity *group)
{
	return observe_loss_event (member, group, CAMPAIGN_LOSS_EVENT_PHYSICAL);
}

int campaign_roster_has_pending_loss (void)
{
	return FALSE;
}

int campaign_roster_report_loss (const char *host_id)
{
	(void) host_id;
	debug_fatal ("campaign roster: no loss is pending");
	return FALSE;
}

int campaign_roster_members (campaign_member_snapshot_fn callback, void *user)
{
	size_t i;
	for (i = 0; i < record_count; ++i)
	{
		campaign_member_snapshot snapshot;
		campaign_member_record *record = &records[i];
		snapshot.member_key = record->member_key;
		snapshot.group_key = record->group_key;
		snapshot.host_id = record->host_id;
		snapshot.origin_entity_index = record->origin_entity_index;
		snapshot.current_entity_index = record->current_entity_index;
		snapshot.entity_type = record->entity_type;
		snapshot.sub_type = record->sub_type;
		snapshot.side = record->side;
		snapshot.initial_alive = record->initial_alive;
		snapshot.physical_present = record->physical_present;
		snapshot.in_group = record->in_group;
		callback (&snapshot, user);
	}
	return (int) record_count;
}

void campaign_roster_get_stats (campaign_roster_stats *stats)
{
	stats->duplicate_comparisons = duplicate_comparisons;
	stats->duplicate_scan_seconds = (double) duplicate_scan_ticks / CLOCKS_PER_SEC;
}

#include "project.h"

#include <stdlib.h>
#include <string.h>
#include <time.h>

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

static campaign_member_record *current_member_record (entity *member)
{
	int index = entity_index (member);
	int record_index = current_member_by_entity[index] - 1;
	if (record_index < 0)
	{
		return NULL;
	}
	if (!records[record_index].physical_present || (records[record_index].current_entity_index != index))
	{
		debug_fatal ("campaign roster: stale current member mapping for entity %d", index);
	}
	return &records[record_index];
}

uint64_t campaign_roster_member_key (entity *member)
{
	campaign_member_record *record = current_member_record (member);
	return record ? record->member_key : 0;
}

const char *campaign_roster_host_id (entity *member)
{
	campaign_member_record *record = current_member_record (member);
	return record ? record->host_id : NULL;
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

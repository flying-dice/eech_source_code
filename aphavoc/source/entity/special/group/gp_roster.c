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
	int credit_observed;
	int loss_pending;
} campaign_member_record;

typedef struct campaign_entity_reference_record
{
	uint64_t key;
	int entity_index;
	int entity_type;
	int active;
} campaign_entity_reference_record;

typedef struct campaign_pending_task_action
{
	uint64_t task_key;
	int sub_type;
	int state;
	int termination_reason;
} campaign_pending_task_action;

typedef struct campaign_pending_loss
{
	char host_id[CAMPAIGN_HOST_ID_MAX + 1];
	uint64_t member_key;
	uint64_t group_reference_key;
	uint64_t primary_task_key;
	int side;
	int entity_type;
	int sub_type;
	int group_sub_type;
	int group_member_number;
	int force_info_category;
	int task_count;
	campaign_pending_task_action tasks[CAMPAIGN_LOSS_TASK_DEPENDENT_MAX];
	uint64_t landing_key;
	int landing_count;
	int state;
} campaign_pending_loss;

enum
{
	CAMPAIGN_PENDING_LOSS_ACTIVE = 1,
	CAMPAIGN_PENDING_LOSS_REPORTED = 2
};

enum
{
	CAMPAIGN_PENDING_TASK_ACTIVE = 1,
	CAMPAIGN_PENDING_TASK_RETIRED_EXPIRE = 2,
	CAMPAIGN_PENDING_TASK_REPORTING = 3,
	CAMPAIGN_PENDING_TASK_REPORTED = 4
};

static campaign_member_record *records;
static size_t record_count;
static size_t record_capacity;
static int *current_member_by_entity;
static uint64_t *current_group_by_entity;
static int *current_reference_by_entity;
static int map_size;
static uint64_t next_member_key = 1;
static uint64_t next_group_key = 1;
static campaign_entity_reference_record *reference_records;
static size_t reference_record_count;
static size_t reference_record_capacity;
static uint64_t next_reference_key = 1;
static campaign_pending_loss *pending_losses;
static size_t pending_loss_count;
static size_t pending_loss_capacity;
static uint64_t duplicate_comparisons;
static clock_t duplicate_scan_ticks;
static campaign_roster_host current_host;

static int entity_index (entity *en);

static void ensure_reference_capacity (void)
{
	if (reference_record_count == reference_record_capacity)
	{
		size_t capacity = reference_record_capacity ? reference_record_capacity * 2 : 1024;
		campaign_entity_reference_record *grown = (campaign_entity_reference_record *) realloc (reference_records, capacity * sizeof (*reference_records));
		if (!grown)
		{
			debug_fatal ("campaign roster: cannot allocate %u entity references", (unsigned int) capacity);
		}
		reference_records = grown;
		reference_record_capacity = capacity;
	}
}

static uint64_t campaign_reference_key (entity *en)
{
	int index = entity_index (en);
	int record_index = current_reference_by_entity[index] - 1;
	if (record_index >= 0)
	{
		campaign_entity_reference_record *record = &reference_records[record_index];
		if (!record->active || (record->entity_index != index) || (record->entity_type != get_local_entity_type (en)))
		{
			debug_fatal ("campaign roster: stale entity reference for slot %d", index);
		}
		return record->key;
	}

	ensure_reference_capacity ();
	{
		campaign_entity_reference_record *record = &reference_records[reference_record_count];
		record->key = next_reference_key++;
		if (!next_reference_key)
		{
			debug_fatal ("campaign roster: entity reference key overflow");
		}
		record->entity_index = index;
		record->entity_type = get_local_entity_type (en);
		record->active = TRUE;
		current_reference_by_entity[index] = (int) reference_record_count + 1;
		reference_record_count++;
		return record->key;
	}
}

static entity *campaign_reference_resolve (uint64_t key, entity_types expected_type)
{
	campaign_entity_reference_record *record;
	entity *en;
	int record_index;

	if (!key || (key > reference_record_count))
	{
		debug_fatal ("campaign roster: unknown entity reference %llu", (unsigned long long) key);
	}
	record_index = (int) key - 1;
	record = &reference_records[record_index];
	if ((record->key != key) || !record->active || (record->entity_type != expected_type) ||
		(current_reference_by_entity[record->entity_index] != record_index + 1))
	{
		debug_fatal ("campaign roster: stale entity reference %llu", (unsigned long long) key);
	}
	en = get_local_entity_ptr (record->entity_index);
	if (get_local_entity_type (en) != expected_type)
	{
		debug_fatal ("campaign roster: entity reference %llu changed type", (unsigned long long) key);
	}
	return en;
}

static void campaign_reference_require_retired (uint64_t key, entity_types expected_type)
{
	campaign_entity_reference_record *record;

	if (!key || (key > reference_record_count))
	{
		debug_fatal ("campaign roster: unknown retired entity reference %llu", (unsigned long long) key);
	}
	record = &reference_records[(int) key - 1];
	if ((record->key != key) || record->active || (record->entity_type != expected_type))
	{
		debug_fatal ("campaign roster: entity reference %llu was not explicitly retired", (unsigned long long) key);
	}
}

static void ensure_pending_loss_capacity (void)
{
	if (pending_loss_count == pending_loss_capacity)
	{
		size_t capacity = pending_loss_capacity ? pending_loss_capacity * 2 : 16;
		campaign_pending_loss *grown = (campaign_pending_loss *) realloc (pending_losses, capacity * sizeof (*pending_losses));
		if (!grown)
		{
			debug_fatal ("campaign roster: cannot allocate %u pending losses", (unsigned int) capacity);
		}
		pending_losses = grown;
		pending_loss_capacity = capacity;
	}
}

static campaign_pending_loss *find_pending_loss (const char *host_id)
{
	size_t i;
	for (i = 0; i < pending_loss_count; ++i)
	{
		if (!strcmp (pending_losses[i].host_id, host_id))
		{
			return &pending_losses[i];
		}
	}
	return NULL;
}

static campaign_pending_loss *require_pending_loss (const char *host_id)
{
	campaign_pending_loss *pending = find_pending_loss (host_id);
	if (!pending)
	{
		debug_fatal ("campaign loss: no loss is pending for %s", host_id);
	}
	return pending;
}

static campaign_member_record *find_member_record (uint64_t member_key)
{
	size_t i;
	for (i = 0; i < record_count; ++i)
	{
		if (records[i].member_key == member_key)
		{
			return &records[i];
		}
	}
	return NULL;
}

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

static void capture_task_dependents (entity *member, campaign_loss_descriptor *descriptor)
{
	entity *child = get_local_entity_first_child (member, LIST_TYPE_TASK_DEPENDENT);
	while (child)
	{
		int copied = descriptor->member_task_dependent_copied;
		descriptor->member_task_dependent_count++;
		if (copied < CAMPAIGN_LOSS_TASK_DEPENDENT_MAX)
		{
			campaign_loss_task_dependent *dependent = &descriptor->member_task_dependents[copied];
			entity *objective;
			dependent->task_key = campaign_reference_key (child);
			dependent->entity_type = get_local_entity_type (child);
			dependent->sub_type = get_local_entity_int_value (child, INT_TYPE_ENTITY_SUB_TYPE);
			dependent->task_state = -1;
			dependent->task_completed = -1;
			if (dependent->entity_type == ENTITY_TYPE_TASK)
			{
				objective = get_local_entity_parent (child, LIST_TYPE_TASK_DEPENDENT);
				dependent->task_state = get_local_entity_int_value (child, INT_TYPE_TASK_STATE);
				dependent->task_completed = get_local_entity_int_value (child, INT_TYPE_TASK_COMPLETED);
				dependent->primary_task = task_database[dependent->sub_type].primary_task;
				dependent->persistent_task = task_database[dependent->sub_type].persistent_task;
				dependent->objective_is_member = objective == member;
				dependent->task_dependent_children = child_count (child, LIST_TYPE_TASK_DEPENDENT);
				dependent->guide_children = child_count (child, LIST_TYPE_GUIDE);
				dependent->expire_timer = get_local_entity_float_value (child, FLOAT_TYPE_EXPIRE_TIMER);
				dependent->stop_timer = get_local_entity_float_value (child, FLOAT_TYPE_STOP_TIMER);
			}
			descriptor->member_task_dependent_copied++;
		}
		child = get_local_entity_child_succ (child, LIST_TYPE_TASK_DEPENDENT);
	}
}

static void append_landing_action (campaign_loss_descriptor *descriptor, entity *landing_en, int action_kind, int count)
{
	int copied = descriptor->landing_action_copied;
	descriptor->landing_action_count++;
	if (copied < CAMPAIGN_LOSS_LANDING_ACTION_MAX)
	{
		campaign_loss_landing_action *action = &descriptor->landing_actions[copied];
		action->landing_key = campaign_reference_key (landing_en);
		action->action_kind = action_kind;
		action->count = count;
		action->landing_sub_type = get_local_entity_int_value (landing_en, INT_TYPE_ENTITY_SUB_TYPE);
		action->reserved_landing_sites = get_local_entity_int_value (landing_en, INT_TYPE_RESERVED_LANDING_SITES);
		action->free_landing_sites = get_local_entity_int_value (landing_en, INT_TYPE_FREE_LANDING_SITES);
		action->landing_lock = get_local_entity_int_value (landing_en, INT_TYPE_LANDING_LOCK);
		action->landed_lock = get_local_entity_int_value (landing_en, INT_TYPE_LANDED_LOCK);
		action->takeoff_lock = get_local_entity_int_value (landing_en, INT_TYPE_TAKEOFF_LOCK);
		descriptor->landing_action_copied++;
	}
}

static void capture_landing_actions (entity *member, entity *group, campaign_loss_descriptor *descriptor)
{
	entity *guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);
	entity *task = guide ? get_local_entity_parent (guide, LIST_TYPE_GUIDE) : NULL;
	entity *keysite;
	entity *landing_en;
	int landing_flag = FALSE;

	if (task)
	{
		int sub_type = get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE);
		switch (sub_type)
		{
			case ENTITY_SUB_TYPE_TASK_LANDING:
				landing_en = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);
				if (landing_en)
				{
					append_landing_action (descriptor, landing_en, CAMPAIGN_LOSS_LANDING_UNLOCK_ROUTE, 0);
				}
				landing_flag = TRUE;
				break;
			case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
				landing_flag = TRUE;
				break;
			case ENTITY_SUB_TYPE_TASK_TAKEOFF:
				landing_en = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);
				if (landing_en)
				{
					append_landing_action (descriptor, landing_en, CAMPAIGN_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE, 0);
				}
				/* The original switch deliberately falls through to reservation release. */
			default:
				keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);
				if (keysite && (get_local_entity_type (keysite) == ENTITY_TYPE_KEYSITE))
				{
					entity *primary_task;
					landing_en = get_local_entity_landing_entity (keysite, group_database[get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].default_landing_type);
					primary_task = get_local_group_primary_task (group);
					if (landing_en && primary_task && get_local_entity_int_value (primary_task, INT_TYPE_ASSESS_LANDING))
					{
						append_landing_action (descriptor, landing_en, CAMPAIGN_LOSS_LANDING_UNRESERVE_SITE, 1);
					}
				}
				break;
		}
	}

	if (get_local_entity_int_value (member, INT_TYPE_LANDED) || landing_flag)
	{
		keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);
		if (keysite && (get_local_entity_type (keysite) == ENTITY_TYPE_KEYSITE))
		{
			landing_en = get_local_entity_landing_entity (keysite, group_database[get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].default_landing_type);
			if (landing_en)
			{
				append_landing_action (descriptor, landing_en, CAMPAIGN_LOSS_LANDING_UNLOCK_SITE, 0);
			}
		}
	}
}

static void begin_pending_loss (entity *group, campaign_member_record *member_record, const campaign_loss_descriptor *descriptor)
{
	campaign_pending_loss *pending;
	int task_count = 0;
	int waypoint_count = 0;
	int i;

	if ((descriptor->entity_type != ENTITY_TYPE_HELICOPTER) ||
		(descriptor->primary_task_sub_type != ENTITY_SUB_TYPE_TASK_SUPPLY) ||
		(descriptor->player != ENTITY_PLAYER_AI) || descriptor->landed ||
		(descriptor->campaign_member_count <= 1) || descriptor->group_task_dependents ||
		descriptor->restricted_guides || descriptor->movement_parent || descriptor->movement_dependents ||
		descriptor->takeoff_queue)
	{
		debug_fatal ("campaign loss: WITHHOLD supports only a non-final AI helicopter on a bounded SUPPLY loss");
	}
	if ((descriptor->force_info_category < 0) || (descriptor->force_info_category >= NUM_FORCE_INFO_CATAGORIES))
	{
		debug_fatal ("campaign loss: invalid force information category %d", descriptor->force_info_category);
	}
	if ((descriptor->member_task_dependent_count != descriptor->member_task_dependent_copied) ||
		(descriptor->member_task_dependent_count != 8))
	{
		debug_fatal ("campaign loss: supported SUPPLY loss requires eight copied task-dependent entries");
	}
	for (i = 0; i < descriptor->member_task_dependent_copied; ++i)
	{
		const campaign_loss_task_dependent *dependent = &descriptor->member_task_dependents[i];
		if ((dependent->entity_type == ENTITY_TYPE_WAYPOINT) &&
			(dependent->sub_type == ENTITY_SUB_TYPE_WAYPOINT_TARGET))
		{
			waypoint_count++;
			continue;
		}
		if ((dependent->entity_type != ENTITY_TYPE_TASK) ||
			(dependent->sub_type != ENTITY_SUB_TYPE_TASK_ENGAGE) ||
			(dependent->task_state != TASK_STATE_UNASSIGNED) ||
			(dependent->task_completed != TASK_INCOMPLETE) ||
			dependent->primary_task || dependent->persistent_task || !dependent->objective_is_member ||
			dependent->task_dependent_children || (dependent->expire_timer <= 0.0) ||
			(dependent->stop_timer != 0.0) || (dependent->guide_children <= 0))
		{
			debug_fatal ("campaign loss: unsupported SUPPLY task-dependent entry %d", i);
		}
		task_count++;
	}
	if ((task_count != 4) || (waypoint_count != 4))
	{
		debug_fatal ("campaign loss: supported SUPPLY loss requires four ENGAGE tasks and four target waypoints");
	}
	if ((descriptor->landing_action_count != 1) || (descriptor->landing_action_copied != 1) ||
		(descriptor->landing_actions[0].action_kind != CAMPAIGN_LOSS_LANDING_UNRESERVE_SITE) ||
		(descriptor->landing_actions[0].count != 1) ||
		(descriptor->landing_actions[0].landing_sub_type != ENTITY_SUB_TYPE_LANDING_HELICOPTER) ||
		!descriptor->landing_actions[0].landing_key)
	{
		debug_fatal ("campaign loss: supported SUPPLY loss requires one landing-site unreserve action");
	}
	if (!descriptor->primary_task_key || find_pending_loss (descriptor->host_id) || member_record->loss_pending)
	{
		debug_fatal ("campaign loss: duplicate or incomplete pending loss for %s", descriptor->host_id);
	}
	if (member_record->credit_observed)
	{
		debug_fatal ("campaign loss: WITHHOLD does not support a member with credited aggression");
	}

	ensure_pending_loss_capacity ();
	pending = &pending_losses[pending_loss_count++];
	memset (pending, 0, sizeof (*pending));
	strcpy (pending->host_id, descriptor->host_id);
	pending->member_key = descriptor->member_key;
	pending->group_reference_key = campaign_reference_key (group);
	pending->primary_task_key = descriptor->primary_task_key;
	pending->side = descriptor->side;
	pending->entity_type = descriptor->entity_type;
	pending->sub_type = descriptor->sub_type;
	pending->group_sub_type = descriptor->group_sub_type;
	pending->group_member_number = descriptor->group_member_number;
	pending->force_info_category = descriptor->force_info_category;
	pending->landing_key = descriptor->landing_actions[0].landing_key;
	pending->landing_count = descriptor->landing_actions[0].count;
	pending->state = CAMPAIGN_PENDING_LOSS_ACTIVE;
	for (i = 0; i < descriptor->member_task_dependent_copied; ++i)
	{
		const campaign_loss_task_dependent *dependent = &descriptor->member_task_dependents[i];
		if (dependent->entity_type == ENTITY_TYPE_TASK)
		{
			pending->tasks[pending->task_count].task_key = dependent->task_key;
			pending->tasks[pending->task_count].sub_type = dependent->sub_type;
			pending->tasks[pending->task_count].state = CAMPAIGN_PENDING_TASK_ACTIVE;
			pending->task_count++;
		}
	}
	member_record->loss_pending = TRUE;
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
	descriptor.force_info_category = get_local_entity_int_value (member, INT_TYPE_FORCE_INFO_CATAGORY);
	descriptor.group_sub_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);
	descriptor.primary_task_sub_type = task ? get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) : -1;
	descriptor.primary_task_key = task ? campaign_reference_key (task) : 0;
	descriptor.group_member_number = get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER);
	descriptor.campaign_member_count = get_local_entity_int_value (group, INT_TYPE_MEMBER_COUNT);
	descriptor.player = get_local_entity_int_value (member, INT_TYPE_PLAYER);
	descriptor.landed = get_local_entity_int_value (member, INT_TYPE_LANDED);
	capture_task_dependents (member, &descriptor);
	descriptor.group_task_dependents = child_count (group, LIST_TYPE_TASK_DEPENDENT);
	descriptor.restricted_guides = restricted_guide_count (group);
	capture_landing_actions (member, group, &descriptor);
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
	if (decision == CAMPAIGN_LOSS_WITHHOLD)
	{
		if (event_kind != CAMPAIGN_LOSS_EVENT_PHYSICAL)
		{
			debug_fatal ("campaign loss: CREDIT withholding is unsupported");
		}
		begin_pending_loss (group, record, &descriptor);
	}
	if (event_kind == CAMPAIGN_LOSS_EVENT_CREDIT)
	{
		record->credit_observed = TRUE;
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
	free (current_reference_by_entity);
	free (reference_records);
	free (pending_losses);
	records = NULL;
	record_count = 0;
	record_capacity = 0;
	map_size = number_of_entities;
	current_member_by_entity = (int *) calloc ((size_t) map_size, sizeof (*current_member_by_entity));
	current_group_by_entity = (uint64_t *) calloc ((size_t) map_size, sizeof (*current_group_by_entity));
	current_reference_by_entity = (int *) calloc ((size_t) map_size, sizeof (*current_reference_by_entity));
	reference_records = NULL;
	reference_record_count = 0;
	reference_record_capacity = 0;
	pending_losses = NULL;
	pending_loss_count = 0;
	pending_loss_capacity = 0;
	next_reference_key = 1;
	next_member_key = 1;
	next_group_key = 1;
	duplicate_comparisons = 0;
	duplicate_scan_ticks = 0;
	if (!current_member_by_entity || !current_group_by_entity || !current_reference_by_entity)
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
	if (!current_member_by_entity || !current_group_by_entity || !current_reference_by_entity || !entities)
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
	if (current_reference_by_entity[index])
	{
		campaign_entity_reference_record *reference = &reference_records[current_reference_by_entity[index] - 1];
		reference->active = FALSE;
		current_reference_by_entity[index] = 0;
	}
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

void campaign_roster_record_task_termination (entity *task, int termination_reason)
{
	campaign_entity_reference_record *reference;
	campaign_pending_task_action *action = NULL;
	int index;
	int reference_index;
	size_t i;
	int j;

	index = entity_index (task);
	reference_index = current_reference_by_entity[index] - 1;
	if (reference_index < 0)
	{
		return;
	}
	reference = &reference_records[reference_index];
	if (!reference->active || (reference->entity_type != ENTITY_TYPE_TASK) ||
		(reference->entity_index != index) || (reference->key == 0))
	{
		debug_fatal ("campaign loss: invalid task reference during termination");
	}
	for (i = 0; i < pending_loss_count; ++i)
	{
		campaign_pending_loss *pending = &pending_losses[i];
		if (pending->state != CAMPAIGN_PENDING_LOSS_ACTIVE)
		{
			continue;
		}
		for (j = 0; j < pending->task_count; ++j)
		{
			if (pending->tasks[j].task_key == reference->key)
			{
				if (action)
				{
					debug_fatal ("campaign loss: task reference belongs to multiple pending losses");
				}
				action = &pending->tasks[j];
			}
		}
	}
	if (!action)
	{
		return;
	}
	if (action->state == CAMPAIGN_PENDING_TASK_REPORTING)
	{
		if (termination_reason != TASK_TERMINATED_OBJECTIVE_MESSAGE)
		{
			debug_fatal ("campaign loss: report terminated ENGAGE task for an unexpected reason %d", termination_reason);
		}
		return;
	}
	if (action->state != CAMPAIGN_PENDING_TASK_ACTIVE)
	{
		debug_fatal ("campaign loss: ENGAGE task terminated more than once");
	}
	if ((termination_reason != TASK_TERMINATED_EXPIRE_TIME_REACHED) ||
		(get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != action->sub_type) ||
		(get_local_entity_int_value (task, INT_TYPE_TASK_STATE) != TASK_STATE_UNASSIGNED) ||
		(get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED) != TASK_INCOMPLETE) ||
		get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT))
	{
		debug_fatal ("campaign loss: pending ENGAGE task changed before natural expiry");
	}
	action->termination_reason = termination_reason;
	action->state = CAMPAIGN_PENDING_TASK_RETIRED_EXPIRE;
}

int campaign_roster_has_pending_loss (void)
{
	size_t i;
	for (i = 0; i < pending_loss_count; ++i)
	{
		if (pending_losses[i].state == CAMPAIGN_PENDING_LOSS_ACTIVE)
		{
			return TRUE;
		}
	}
	return FALSE;
}

int campaign_roster_report_loss (const char *host_id, campaign_loss_report_receipt *receipt)
{
	/*
	 * Lifetime rule: the campaign actions below can free entities (the group
	 * itself when this was its last physical member, tasks through their guide
	 * cascades) and can grow the roster registries. Nothing that points into
	 * an entity or a registry is held across an action. The plan is a value
	 * copy of the pending record; entities are re-resolved from their private
	 * generation keys immediately before each use, and a stale key is fatal.
	 */
	campaign_pending_loss plan;
	campaign_pending_loss *pending;
	campaign_member_record *member_record;
	entity *group;
	entity *primary_task;
	entity *landing;
	entity *task;
	int i;
	int landing_reserved_before;
	int landing_reserved_after;
	int group_losses_before;
	int group_losses_after;
	int task_losses_before;
	int task_losses_after;
	int retired_expire_count = 0;
	int completed_on_report_count = 0;

	if (!host_id || !host_id[0])
	{
		debug_fatal ("campaign loss: report_loss requires an opaque host identity");
	}
	if (!receipt)
	{
		debug_fatal ("campaign loss: report_loss requires a receipt");
	}
	pending = require_pending_loss (host_id);
	if (pending->state == CAMPAIGN_PENDING_LOSS_REPORTED)
	{
		debug_fatal ("campaign loss: loss for %s was already reported", host_id);
	}
	if (pending->state != CAMPAIGN_PENDING_LOSS_ACTIVE)
	{
		debug_fatal ("campaign loss: invalid pending state for %s", host_id);
	}
	plan = *pending;
	pending = NULL;

	group = campaign_reference_resolve (plan.group_reference_key, ENTITY_TYPE_GROUP);
	primary_task = campaign_reference_resolve (plan.primary_task_key, ENTITY_TYPE_TASK);
	landing = campaign_reference_resolve (plan.landing_key, ENTITY_TYPE_LANDING);
	if (get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE) != plan.group_sub_type)
	{
		debug_fatal ("campaign loss: group changed before report for %s", host_id);
	}
	if (get_local_entity_int_value (primary_task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_SUPPLY)
	{
		debug_fatal ("campaign loss: primary SUPPLY task changed before report for %s", host_id);
	}
	if (get_local_entity_int_value (landing, INT_TYPE_RESERVED_LANDING_SITES) < plan.landing_count)
	{
		debug_fatal ("campaign loss: landing reservation changed before report for %s", host_id);
	}
	if ((plan.group_member_number < 0) ||
		(plan.group_member_number >= (int) (sizeof (unsigned int) * 8)))
	{
		debug_fatal ("campaign loss: invalid group member number %d for %s", plan.group_member_number, host_id);
	}
	for (i = 0; i < plan.task_count; ++i)
	{
		campaign_pending_task_action *action = &plan.tasks[i];
		if (action->state == CAMPAIGN_PENDING_TASK_RETIRED_EXPIRE)
		{
			if (action->termination_reason != TASK_TERMINATED_EXPIRE_TIME_REACHED)
			{
				debug_fatal ("campaign loss: ENGAGE task has an invalid retirement reason for %s", host_id);
			}
			campaign_reference_require_retired (action->task_key, ENTITY_TYPE_TASK);
			retired_expire_count++;
			continue;
		}
		if (action->state != CAMPAIGN_PENDING_TASK_ACTIVE)
		{
			debug_fatal ("campaign loss: ENGAGE task has an invalid pending state for %s", host_id);
		}
		task = campaign_reference_resolve (action->task_key, ENTITY_TYPE_TASK);
		if ((get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != action->sub_type) ||
			(get_local_entity_int_value (task, INT_TYPE_TASK_STATE) != TASK_STATE_UNASSIGNED) ||
			(get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED) != TASK_INCOMPLETE) ||
			(get_local_entity_int_value (task, INT_TYPE_TASK_TERMINATED) != TASK_TERMINATED_IN_PROGRESS) ||
			get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT) ||
			get_local_entity_first_child (task, LIST_TYPE_TASK_DEPENDENT))
		{
			debug_fatal ("campaign loss: ENGAGE task changed before report for %s", host_id);
		}
	}
	landing_reserved_before = get_local_entity_int_value (landing, INT_TYPE_RESERVED_LANDING_SITES);
	group_losses_before = get_local_entity_int_value (group, INT_TYPE_LOSSES);
	task_losses_before = get_local_entity_int_value (primary_task, INT_TYPE_LOSSES);
	member_record = find_member_record (plan.member_key);
	if (!member_record || member_record->physical_present || member_record->in_group || !member_record->loss_pending)
	{
		debug_fatal ("campaign loss: physical member is not durably released for %s", host_id);
	}
	group = NULL;
	primary_task = NULL;
	landing = NULL;
	task = NULL;
	member_record = NULL;

	remove_from_force_info_values
	(
		get_local_force_entity ((entity_sides) plan.side),
		(entity_sub_types) plan.group_sub_type,
		plan.force_info_category
	);
	add_entity_to_regen_queue
	(
		(entity_sides) plan.side,
		(entity_types) plan.entity_type,
		(entity_sub_types) plan.sub_type,
		(entity_sub_types) plan.group_sub_type
	);
	for (i = 0; i < plan.task_count; ++i)
	{
		if (plan.tasks[i].state == CAMPAIGN_PENDING_TASK_RETIRED_EXPIRE)
		{
			continue;
		}
		require_pending_loss (host_id)->tasks[i].state = CAMPAIGN_PENDING_TASK_REPORTING;
		task = campaign_reference_resolve (plan.tasks[i].task_key, ENTITY_TYPE_TASK);
		if (!complete_non_primary_task_for_lost_objective (task))
		{
			debug_fatal ("campaign loss: ENGAGE task could not be completed for %s", host_id);
		}
		require_pending_loss (host_id)->tasks[i].state = CAMPAIGN_PENDING_TASK_REPORTED;
		completed_on_report_count++;
	}
	task = NULL;

	landing = campaign_reference_resolve (plan.landing_key, ENTITY_TYPE_LANDING);
	unreserve_local_landing_sites (landing, plan.landing_count);
	landing_reserved_after = get_local_entity_int_value (landing, INT_TYPE_RESERVED_LANDING_SITES);

	group = campaign_reference_resolve (plan.group_reference_key, ENTITY_TYPE_GROUP);
	increment_group_losses (group);
	group_losses_after = get_local_entity_int_value (group, INT_TYPE_LOSSES);

	primary_task = campaign_reference_resolve (plan.primary_task_key, ENTITY_TYPE_TASK);
	increment_task_losses (primary_task);
	task_losses_after = get_local_entity_int_value (primary_task, INT_TYPE_LOSSES);

	/* Frees the group when this was its last physical member; group is dead afterwards. */
	group = campaign_reference_resolve (plan.group_reference_key, ENTITY_TYPE_GROUP);
	apply_group_member_killed (group, 1U << plan.group_member_number);
	group = NULL;
	primary_task = NULL;
	landing = NULL;

	check_all_force_campaign_objectives ();
	debug_log
	(
		"campaign loss: reported %s landing_reserved=%d->%d group_losses=%d->%d primary_task_losses=%d->%d tasks_retired_expire=%d tasks_completed_on_report=%d",
		host_id,
		landing_reserved_before,
		landing_reserved_after,
		group_losses_before,
		group_losses_after,
		task_losses_before,
		task_losses_after,
		retired_expire_count,
		completed_on_report_count
	);

	require_pending_loss (host_id)->state = CAMPAIGN_PENDING_LOSS_REPORTED;
	member_record = find_member_record (plan.member_key);
	if (!member_record)
	{
		debug_fatal ("campaign loss: member record vanished during report for %s", host_id);
	}
	member_record->loss_pending = FALSE;

	/* Filled only now, after every fatal check: a failed report leaves the caller's receipt untouched. */
	receipt->landing_reserved_before = landing_reserved_before;
	receipt->landing_reserved_after = landing_reserved_after;
	receipt->group_losses_before = group_losses_before;
	receipt->group_losses_after = group_losses_after;
	receipt->primary_task_losses_before = task_losses_before;
	receipt->primary_task_losses_after = task_losses_after;
	receipt->tasks_retired_expire = retired_expire_count;
	receipt->tasks_completed_on_report = completed_on_report_count;
	return TRUE;
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

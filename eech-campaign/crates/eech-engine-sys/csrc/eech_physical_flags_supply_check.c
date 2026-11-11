/*
 * TEST-ONLY (EEDCS-49, D3, Task49, feature `physical-flags-supply-check`): the world of the SUPPLY ARRIVAL DESCRIPTOR control of the per-entity UPDATED word, the next immediate consumer after the original guide branch
 * (csrc/eech_physical_flags_consumer_check.c) and the moving-mobile collision pass-through (csrc/eech_physical_flags_collision_check.c).
 *
 * THE CONSUMER (the shipping arrival, not the original guide branch): `campaign_supply_arrival_check (member, guide)` (special/guide/gd_supply_arrival.c). For a FIXED-WING member it reads
 * `descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED)` UNCONDITIONALLY (the raw getter, destructive, once per call) and hands the whole descriptor to the host through
 * `campaign_supply_arrival_host.observe_reach` BEFORE any gate. The host function is the engine's own Rust boundary (eech-engine `supply_reached`, which converts the descriptor with `checked_bool ("updated", ...)`).
 * THIS FILE builds only the world around the real producer; it decides nothing about the descriptor and checks nothing about it: the test (eech-engine/tests/physical_flags_supply.rs) hands in the REAL Rust
 * boundary as `observe_reach` and asserts what that boundary received and answered.
 *
 * WHAT IS REAL: `campaign_supply_arrival_check`, its roster identity (`campaign_roster_link_member` and `campaign_roster_host_id`), the real mobile/aircraft/fixed-wing getters, the real guide criteria accessors,
 * the real list functions (`insert_local_entity_into_parents_child_list`) linking member, guide, group, waypoint and task, the real update reset (`update_client_server_entities`), and in the HOSTED profile the real
 * getter/setter/reset over the HOST's words (eech_physical_flags_hosted.h), in the ORIGINAL profile (feature collision-fixture) over the engine's own global arrays.
 *
 * WHAT IS HAND-MADE, AND DECLARED (nothing else is scripted):
 *   - the entities: slots of a calloc'd pool of the engine's own `entity` struct (the flag words are keyed by `entity - entities`); a member is a slot typed FIXED_WING with a calloc'd raw `fixed_wing` (position, motion,
 *     distance, velocity); one GROUP, one GUIDE (RADIUS criterion valid, no other), one WAYPOINT (sub type PICK_UP) and one TASK (sub type SUPPLY) with calloc'd raw records of the engine's own types;
 *   - the lists: member in the group's MEMBER list (so the group leader is the first member) and the guide's FOLLOWER list, the guide in the group's GUIDE_STACK list and the waypoint's CURRENT_WAYPOINT list: the
 *     producer's own preconditions, linked by the real list functions; the waypoint's PARENT in the task's WAYPOINT list is set in the raw record (the real insertion notifies the task and stopped this world);
 *   - the roster identity: the campaign's own roster links each member and a PRIVATE TEST COLLABORATOR (`bind_member` below) issues `eech-world:<n>`, as the reference host's bind_mobile does; the test never asserts on it;
 *   - the delta time (`system_delta_time`) and the comms model (SERVER), set directly and given back; one ENTITY_TYPE_UPDATE entity with an empty list so the real `update_client_server_entities` runs its reset.
 *
 * NOT CLAIMED: the helicopter (its descriptor does NOT read the getter by the source text; not run), the other mobile kinds, the weapon-vector path, a full save or the global state beyond the tables below, -O2, any
 * lifetime of the host's store: a test-only fresh binding resets the store and forgets the group keys between scenarios; the whole of this is a dedicated test process.
 *
 * STATE THE CONTROL CHANGES AND GIVES BACK (`eech_supply_close`): the int, float, vec3d, ptr, list and message tables, the three flag arrays (original profile, copied whole), `entities`, `number_of_entities`,
 * update_entity, update_succ, locked_frame_rate, system_delta_time, the comms model, the campaign roster and supply arrival hosts (removed) and every raw record it allocated. NOT restored: float/attitude setter tables the
 * overload initialisers also fill, and the roster and supply arrival private state, which keep what the control's last scenario left (their arrays are sized from the heap of the run and index only); the process is dedicated to this control.
 */

#ifdef EECH_PHYSICAL_FLAGS_SUPPLY_CHECK

#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "project.h"

#include "eech_headless.h"
#include "eech_physical_flags_hosted.h"

#define WORDS (MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8))

#define GROUP_INDEX 400
#define GUIDE_INDEX 401
#define WAYPOINT_INDEX 402
#define TASK_INDEX 403

#ifdef EECH_CAMPAIGN_HOST_ENTITY

static void clear_flags (void)
{
	hosted_clear ();
}

#else

static unsigned int
	saved_moved [WORDS],
	saved_rotated [WORDS],
	saved_updated [WORDS];

static void clear_flags (void)
{
	memset (moved_entities, 0, sizeof (moved_entities [0]) * WORDS);
	memset (rotated_entities, 0, sizeof (rotated_entities [0]) * WORDS);
	memset (updated_entities, 0, sizeof (updated_entities [0]) * WORDS);
}

#endif

struct saved_table
{
	void *table;
	size_t size;
	void *copy;
};

#define TABLE(T) { (void *) (T), sizeof (T), NULL }

static struct saved_table
	saved_tables [] =
{
	TABLE (fn_get_local_entity_int_value),
	TABLE (fn_set_local_entity_int_value),
	TABLE (fn_set_local_entity_raw_int_value),
	TABLE (fn_set_client_server_entity_int_value),
	TABLE (fn_get_local_entity_float_value),
	TABLE (fn_set_local_entity_float_value),
	TABLE (fn_set_local_entity_raw_float_value),
	TABLE (fn_set_client_server_entity_float_value),
	TABLE (fn_get_local_entity_vec3d),
	TABLE (fn_get_local_entity_vec3d_ptr),
	TABLE (fn_set_local_entity_vec3d),
	TABLE (fn_set_local_entity_raw_vec3d),
	TABLE (fn_set_client_server_entity_vec3d),
	TABLE (fn_get_local_entity_ptr_value),
	TABLE (fn_set_local_entity_ptr_value),
	TABLE (fn_set_local_entity_first_child),
	TABLE (fn_get_local_entity_first_child),
	TABLE (fn_set_local_entity_parent),
	TABLE (fn_get_local_entity_parent),
	TABLE (fn_set_local_entity_child_succ),
	TABLE (fn_get_local_entity_child_succ),
	TABLE (fn_set_local_entity_child_pred),
	TABLE (fn_get_local_entity_child_pred),
	TABLE (message_responses),
};

#define SAVED_TABLES ((int) (sizeof (saved_tables) / sizeof (saved_tables [0])))

static void save_tables (void)
{
	int i;

	for (i = 0; i < SAVED_TABLES; i++)
	{
		saved_tables [i].copy = malloc (saved_tables [i].size);

		memcpy (saved_tables [i].copy, saved_tables [i].table, saved_tables [i].size);
	}
}

static void restore_tables (void)
{
	int i;

	for (i = 0; i < SAVED_TABLES; i++)
	{
		memcpy (saved_tables [i].table, saved_tables [i].copy, saved_tables [i].size);

		free (saved_tables [i].copy);

		saved_tables [i].copy = NULL;
	}
}

#define MAX_RAW 64

static entity
	*pool,
	*saved_entities,
	*saved_update_entity,
	*saved_update_succ,
	update_en;

static int
	opened,
	saved_number_of_entities,
	saved_locked,
	saved_comms,
	raw_count,
	member_count,
	next_host_id;

static float
	saved_dt;

static fixed_wing
	*raw_records [MAX_RAW];

static entity
	*members [MAX_RAW];

static int
	in_member_list [MAX_RAW];

static update
	*update_raw;

static guide
	*guide_raw;

static group
	*group_raw;

static waypoint
	*waypoint_raw;

static task
	*task_raw;

#ifdef EECH_CAMPAIGN_HOST_ENTITY

/* the model provider, the fresh-binding entry and the host-word observation are the test's (tests/support/flags_host.rs): the REAL store of eech-world; HOSTED profile only */
void eech_supply_set_flags_host (campaign_model_fn model, flags_host_fresh_fn fresh, flags_host_observe_fn observe)
{
	hosted_install (model, fresh, observe);
}

#endif

/* the private test collaborator of the campaign's own roster: the host's supported registration of a movable member (the reference host's bind_mobile) */
static int bind_member (const campaign_member_descriptor *descriptor, char *host_id, size_t host_id_size, char *error, size_t error_size, void *user)
{
	(void) descriptor;
	(void) error;
	(void) error_size;
	(void) user;

	snprintf (host_id, host_id_size, "eech-world:%d", ++next_host_id);

	return 0;
}

static void link_entity (entity *child, list_types type, entity *parent)
{
	insert_local_entity_into_parents_child_list (child, type, parent, NULL);
}

/* the producer's own world: one group, one guide, one PICK_UP waypoint of one SUPPLY task, linked as the campaign's task machinery links them */
static void build_fixed_world (void)
{
	memset (&pool [GROUP_INDEX], 0, sizeof (entity));
	memset (&pool [GUIDE_INDEX], 0, sizeof (entity));
	memset (&pool [WAYPOINT_INDEX], 0, sizeof (entity));
	memset (&pool [TASK_INDEX], 0, sizeof (entity));

	pool [GROUP_INDEX].type = ENTITY_TYPE_GROUP;
	pool [GUIDE_INDEX].type = ENTITY_TYPE_GUIDE;
	pool [WAYPOINT_INDEX].type = ENTITY_TYPE_WAYPOINT;
	pool [TASK_INDEX].type = ENTITY_TYPE_TASK;

	group_raw = (group *) calloc (1, sizeof (group));
	guide_raw = (guide *) calloc (1, sizeof (guide));
	waypoint_raw = (waypoint *) calloc (1, sizeof (waypoint));
	task_raw = (task *) calloc (1, sizeof (task));

	set_local_entity_data (&pool [GROUP_INDEX], group_raw);
	set_local_entity_data (&pool [GUIDE_INDEX], guide_raw);
	set_local_entity_data (&pool [WAYPOINT_INDEX], waypoint_raw);
	set_local_entity_data (&pool [TASK_INDEX], task_raw);

	waypoint_raw->sub_type = ENTITY_SUB_TYPE_WAYPOINT_PICK_UP;
	task_raw->sub_type = ENTITY_SUB_TYPE_TASK_SUPPLY;

	/* RADIUS is the one valid criterion (the shipping selection refuses any other for a supply waypoint) */
	guide_raw->criteria [GUIDE_CRITERIA_RADIUS].valid = TRUE;
	guide_raw->criteria [GUIDE_CRITERIA_RADIUS].value = 6.0f;

	link_entity (&pool [GUIDE_INDEX], LIST_TYPE_GUIDE_STACK, &pool [GROUP_INDEX]);
	link_entity (&pool [GUIDE_INDEX], LIST_TYPE_CURRENT_WAYPOINT, &pool [WAYPOINT_INDEX]);
	/* the one link set in the raw record and not by the real list insertion: the insertion of a waypoint into a task's list notifies the task (ENTITY_MESSAGE_LINK_CHILD) and stopped the process in this world, which has no task state beyond its sub type; the producer reads only the waypoint's parent */
	waypoint_raw->waypoint_link.parent = &pool [TASK_INDEX];
}

int eech_supply_open (campaign_observe_supply_reach_fn observe_reach, void *user)
{
	campaign_roster_host
		roster_host;

	campaign_supply_arrival_host
		arrival_host;

	if (opened)
	{
		return 1;
	}

	pool = (entity *) calloc (MAX_NUM_ENTITIES, sizeof (entity));

	if (!pool)
	{
		return 2;
	}

	saved_entities = entities;
	saved_number_of_entities = number_of_entities;
	saved_update_entity = update_entity;
	saved_update_succ = update_succ;
	saved_locked = locked_frame_rate;
	saved_dt = system_delta_time;
	saved_comms = get_comms_model ();

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (saved_moved, moved_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_rotated, rotated_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_updated, updated_entities, sizeof (unsigned int) * WORDS);
#endif

	save_tables ();

	entities = pool;
	number_of_entities = MAX_NUM_ENTITIES;
	raw_count = 0;
	member_count = 0;
	next_host_id = 0;

	/* the engine's own table initialisation, for the types the producer and its collaborators send to */
	initialise_entity_int_value_default_functions ();
	initialise_entity_float_value_default_functions ();
	initialise_entity_vec3d_default_functions ();
	initialise_entity_ptr_value_default_functions ();
	initialise_entity_list_default_functions ();
	initialise_entity_message_responses ();
	overload_mobile_float_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_vec3d_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_list_functions (ENTITY_TYPE_FIXED_WING);
	overload_aircraft_float_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_aircraft_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_aircraft_list_functions (ENTITY_TYPE_FIXED_WING);
	overload_fixed_wing_float_value_functions ();
	overload_fixed_wing_int_value_functions ();
	overload_fixed_wing_list_functions ();
	overload_guide_int_value_functions ();
	overload_guide_ptr_value_functions ();
	overload_guide_list_functions ();
	overload_group_ptr_value_functions ();
	overload_group_list_functions ();
	overload_waypoint_int_value_functions ();
	overload_waypoint_list_functions ();
	overload_task_int_value_functions ();
	overload_task_list_functions ();
	overload_update_list_functions ();

	set_comms_model (COMMS_MODEL_SERVER);

	system_delta_time = 1.0f;

	memset (&update_en, 0, sizeof (update_en));

	update_en.type = ENTITY_TYPE_UPDATE;

	update_raw = (update *) calloc (1, sizeof (update));

	set_local_entity_data (&update_en, update_raw);

	set_update_entity (&update_en);

	memset (&roster_host, 0, sizeof (roster_host));

	roster_host.bind_member = bind_member;

	campaign_roster_set_host (&roster_host);

	memset (&arrival_host, 0, sizeof (arrival_host));

	arrival_host.observe_reach = observe_reach;
	arrival_host.user = user;

	campaign_roster_reset ();
	campaign_supply_arrival_reset ();
	campaign_supply_arrival_set_host (&arrival_host);

	build_fixed_world ();

	opened = 1;

	return 0;
}

/* a scenario starts from clear flags (a fresh host binding in the hosted profile) and a fresh roster; every member of the previous scenario is taken out of the lists */
void eech_supply_begin (void)
{
	int i;

	for (i = 0; i < member_count; i++)
	{
		delete_local_entity_from_parents_child_list (members [i], LIST_TYPE_FOLLOWER);
		if (in_member_list [i])
		{
			delete_local_entity_from_parents_child_list (members [i], LIST_TYPE_MEMBER);
		}
	}

	member_count = 0;

	campaign_roster_reset ();
	next_host_id = 0;

	clear_flags ();
}

/* a member: the pool slot `index` typed FIXED_WING with a FRESH raw record on the coherent trajectory (not landed, distance 10, velocity 20, position (10, 5, 0), motion (-20, 0, 0)), in the group and the guide's
   lists, linked into the campaign's roster (unless `bound` is 0). The FIRST member of a scenario is the group leader (the first child of MEMBER). */
void eech_supply_member (int index, int bound)
{
	fixed_wing
		*raw;

	memset (&pool [index], 0, sizeof (entity));

	pool [index].type = ENTITY_TYPE_FIXED_WING;

	raw = (fixed_wing *) calloc (1, sizeof (fixed_wing));

	raw_records [raw_count++] = raw;

	raw->ac.operational_state = OPERATIONAL_STATE_NAVIGATING;
	raw->ac.distance = 10.0f;
	raw->ac.mob.velocity = 20.0f;
	raw->ac.mob.position.x = 10.0f;
	raw->ac.mob.position.y = 5.0f;
	raw->ac.mob.position.z = 0.0f;
	raw->ac.mob.motion_vector.x = -20.0f;
	raw->ac.mob.motion_vector.y = 0.0f;
	raw->ac.mob.motion_vector.z = 0.0f;

	set_local_entity_data (&pool [index], raw);

	/* the real group message response links a member that enters the MEMBER list into the roster (gp_msgs.c); a member left out of that list (bound 0) has no host identity */
	if (bound)
	{
		link_entity (&pool [index], LIST_TYPE_MEMBER, &pool [GROUP_INDEX]);
	}
	link_entity (&pool [index], LIST_TYPE_FOLLOWER, &pool [GUIDE_INDEX]);

	/* `bound` 0 leaves the member out of the campaign's roster: it has no host identity (the matched NEGATIVE, never a positive) */
	if (bound)
	{
		campaign_roster_link_member (&pool [GROUP_INDEX], &pool [index]);
	}

	in_member_list [member_count] = bound;
	members [member_count++] = &pool [index];
}

void eech_supply_set_updated (int index)
{
	set_local_entity_int_value (&pool [index], INT_TYPE_UPDATED, TRUE);
}

/* the real raw getter of the member (destructive: it clears the other entities' bits of the shared word) */
int eech_supply_raw_updated (int index)
{
	return get_local_entity_int_value (&pool [index], INT_TYPE_UPDATED);
}

unsigned int eech_supply_word (int index)
{
#ifdef EECH_CAMPAIGN_HOST_ENTITY
	return hosted_word (INT_TYPE_UPDATED, index);
#else
	return updated_entities [index >> 5];
#endif
}

/* the real update iteration's reset of the UPDATED array (the update list is empty) */
void eech_supply_update (void)
{
	update_client_server_entities ();
}

/* THE CONSUMER, through its public entry: the engine's own fatal (the host's refusal, as an engine entry catches it) is caught here. Returns 1 when the engine stopped (the message is copied), else 0 and the
   arrival (0 or 1) in *result */
#define TRAP(STATEMENT, CAUGHT) \
	do { \
		jmp_buf trap_; \
		jmp_buf *saved_target_ = eech_fatal_target; \
		int saved_poisoned_ = eech_engine_poisoned; \
		(CAUGHT) = 0; \
		eech_fatal_target = &trap_; \
		if (setjmp (trap_) == 0) { STATEMENT; } else { (CAUGHT) = 1; } \
		eech_fatal_target = saved_target_; \
		eech_engine_poisoned = saved_poisoned_; \
	} while (0)

int eech_supply_check (int index, int *result, char *fatal, int size)
{
	int caught;

	*result = -1;

	TRAP (*result = campaign_supply_arrival_check (&pool [index], &pool [GUIDE_INDEX]), caught);

	if (size > 0)
	{
		fatal [0] = 0;

		if (caught)
		{
			snprintf (fatal, (size_t) size, "%s", eech_fatal_message);
		}
	}

	return caught;
}

void eech_supply_close (void)
{
	int i;

	if (!opened)
	{
		return;
	}

	campaign_supply_arrival_set_host (NULL);
	campaign_roster_set_host (NULL);

	set_update_entity (NULL);

	free (update_raw);
	free (group_raw);
	free (guide_raw);
	free (waypoint_raw);
	free (task_raw);

	for (i = 0; i < raw_count; i++)
	{
		free (raw_records [i]);
	}

	set_comms_model ((comms_model_types) saved_comms);

	system_delta_time = saved_dt;
	locked_frame_rate = saved_locked;
	update_entity = saved_update_entity;
	update_succ = saved_update_succ;
	entities = saved_entities;
	number_of_entities = saved_number_of_entities;

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (moved_entities, saved_moved, sizeof (unsigned int) * WORDS);
	memcpy (rotated_entities, saved_rotated, sizeof (unsigned int) * WORDS);
	memcpy (updated_entities, saved_updated, sizeof (unsigned int) * WORDS);
#endif

	restore_tables ();

	free (pool);

	pool = NULL;
	opened = 0;
}

#endif

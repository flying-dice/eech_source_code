/*
 * TEST-ONLY (EEDCS-49, D3, Task49, feature `physical-flags-motion-check`): the world of the MOTION-RESPONSE control of the per-entity MOVED / ROTATED words: the one live path by which the shipping headless campaign SETS them.
 *
 * THE PATH (special/../vehicle/routed/rv_hostmv.c, under EECH_CAMPAIGN_HOST_MOVEMENT, which the shipping build defines): the AI HELICOPTER's two guided steps, `campaign_movement_helicopter_guided_begin` and
 * `campaign_movement_helicopter_guided_end`, ask the host to move the helicopter (`campaign_movement_host.move_vehicle`) and then apply the answer: `if (result.moved) set_local_entity_int_value (en, INT_TYPE_MOVED, TRUE)`
 * after the begin and `if (result.rotated) set_local_entity_int_value (en, INT_TYPE_ROTATED, TRUE)` after the end. Nothing in the headless shipping path READS either word (the only readers sit behind the Tacview log
 * file, which the headless boot never opens: physical_flags_guard.rs), so the effect of these two statements is a SET on the host's store (hosted profile) or the engine's array (original profile).
 * THIS FILE builds only the world around the real bodies; the host's physical answer is the one environmental input it scripts (moved, rotated, pending: what the host's physics decided) and it decides nothing
 * about the flag: the test (tests/physical_flags_motion.rs) asserts which words the REAL bodies and the REAL setter left, through the real getter and the host's own observation.
 *
 * WHAT IS REAL: `campaign_movement_helicopter_guided_begin` and `_guided_end` with their host id, step filling, answer validation (`ask_host`: moved, rotated and pending must be 0 or 1), the apply guard and
 * the pose setters; the mobile/aircraft/helicopter int, float, vec3d and list functions (the real flag setter and getter, the real position setter with its sector check and dependent notification); the campaign's roster
 * link (`campaign_roster_link_member`); and in the HOSTED profile the flag setter/getter over the HOST's words (eech_physical_flags_hosted.h), in the ORIGINAL profile over the engine's own global arrays.
 *
 * WHAT IS HAND-MADE, AND DECLARED (nothing else is scripted):
 *   - the entities: slots of a calloc'd pool of the engine's own `entity` struct (the flag words are keyed by `entity - entities`); a HELICOPTER slot with a calloc'd raw `helicopter` (position inside the one sector,
 *     identity attitude, no dependents); one GROUP slot with a calloc'd raw `group` (the roster's group key); one SECTOR slot;
 *   - the sector world: `world_map` gets one sector (side 1000, bounds 0..999) and `entity_sector_map` its one cell, so the real position setter finds the helicopter's own sector unchanged (the helicopter's sector
 *     parent is set in its raw record, as the supply control sets a waypoint's parent: the real sector insertion belongs to the creation path and is not run);
 *   - the roster identity: the campaign's own roster links each helicopter and a PRIVATE TEST COLLABORATOR (`bind_member`) issues `eech-world:<n>`, as the reference host's bind_mobile does;
 *   - the host's movement answer (`scripted_host`): GUIDED_BEGIN answers pending TRUE, the scripted `moved`, a velocity and a motion vector; GUIDED_END answers pending FALSE, the scripted `rotated`, a position inside the
 *     sector, the identity attitude and angles; every other mode is refused by name (a placement question here would mean a setter ran outside the apply guard);
 *   - the delta time (`system_delta_time`) and the comms model (SERVER), set directly and given back.
 *
 * NOT CLAIMED: the other motion responses of rv_hostmv.c (the vehicle and fixed-wing sets are behind the per-entity Tacview logging bit, never set headless: pinned by the guard, not run here), the ship and the
 * weapon-vector (they name neither word), the Rust boundary of the movement host, a native or Windows run, the engine boot, any lifetime of the host's store, the global/FPU state beyond what is declared.
 *
 * STATE THE CONTROL CHANGES AND GIVES BACK (`eech_motion_close`): the int, float, vec3d, ptr, list and message tables, the three flag arrays (original profile, copied whole), `entities`, `number_of_entities`,
 * `world_map`, `entity_sector_map`, `system_delta_time`, the comms model, the campaign roster and movement hosts (removed) and every raw record it allocated. The overload initialisers also fill float/attitude
 * tables the control does not save: a dedicated test process.
 */

#ifdef EECH_PHYSICAL_FLAGS_MOTION_CHECK

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
#define SECTOR_INDEX 404

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
#define MODES 128

static entity
	*pool,
	*saved_entities,
	**saved_sector_map,
	**sector_map;

static world_map_data
	saved_world_map;

static int
	opened,
	saved_number_of_entities,
	saved_comms,
	raw_count,
	next_host_id,
	script_moved,
	script_rotated,
	questions [MODES];

static float
	saved_dt;

static helicopter
	*raw_records [MAX_RAW];

static group
	*group_raw;

#ifdef EECH_CAMPAIGN_HOST_ENTITY

/* the model provider, the fresh-binding entry and the host-word observation are the test's (tests/support/flags_host.rs): the REAL store of eech-world; HOSTED profile only */
void eech_motion_set_flags_host (campaign_model_fn model, flags_host_fresh_fn fresh, flags_host_observe_fn observe)
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

/* the host's physical answer for the two guided steps; any other question is refused by name */
static int scripted_host (const campaign_movement_step *step, campaign_movement_result *result, char *error, size_t error_size, void *user)
{
	(void) user;

	if ((step->mode >= 0) && (step->mode < MODES))
	{
		questions [step->mode]++;
	}

	memset (result, 0, sizeof (*result));

	result->attitude [0] = 1.0f;
	result->attitude [4] = 1.0f;
	result->attitude [8] = 1.0f;

	if (step->mode == CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_BEGIN)
	{
		result->pending = TRUE;
		result->moved = script_moved;
		result->velocity = 3.0f;
		result->motion_vector [0] = 1.0f;
		result->motion_vector [2] = 2.0f;

		return 0;
	}

	if (step->mode == CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_END)
	{
		result->pending = FALSE;
		result->rotated = script_rotated;
		result->position [0] = 20.0f;
		result->position [1] = 5.0f;
		result->position [2] = 30.0f;
		result->motion_vector [0] = 1.0f;
		result->motion_vector [2] = 2.0f;

		return 0;
	}

	snprintf (error, error_size, "the motion control answers only the two guided helicopter steps, not mode %d", step->mode);

	return 1;
}

int eech_motion_open (void)
{
	campaign_roster_host
		roster_host;

	campaign_movement_host
		movement_host;

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
	saved_dt = system_delta_time;
	saved_comms = get_comms_model ();
	saved_world_map = world_map;
	saved_sector_map = entity_sector_map;

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (saved_moved, moved_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_rotated, rotated_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_updated, updated_entities, sizeof (unsigned int) * WORDS);
#endif

	save_tables ();

	entities = pool;
	number_of_entities = MAX_NUM_ENTITIES;
	raw_count = 0;
	next_host_id = 0;

	/* the engine's own table initialisation, for the types the real bodies and their collaborators send to */
	initialise_entity_int_value_default_functions ();
	initialise_entity_float_value_default_functions ();
	initialise_entity_vec3d_default_functions ();
	initialise_entity_ptr_value_default_functions ();
	initialise_entity_list_default_functions ();
	initialise_entity_message_responses ();
	overload_mobile_float_value_functions (ENTITY_TYPE_HELICOPTER);
	overload_mobile_int_value_functions (ENTITY_TYPE_HELICOPTER);
	overload_mobile_vec3d_functions (ENTITY_TYPE_HELICOPTER);
	overload_mobile_list_functions (ENTITY_TYPE_HELICOPTER);
	overload_aircraft_float_value_functions (ENTITY_TYPE_HELICOPTER);
	overload_aircraft_int_value_functions (ENTITY_TYPE_HELICOPTER);
	overload_aircraft_vec3d_functions (ENTITY_TYPE_HELICOPTER);
	overload_aircraft_list_functions (ENTITY_TYPE_HELICOPTER);
	overload_helicopter_float_value_functions ();
	overload_helicopter_int_value_functions ();
	overload_helicopter_vec3d_functions ();
	overload_helicopter_list_functions ();
	overload_group_ptr_value_functions ();
	overload_group_list_functions ();

	set_comms_model (COMMS_MODEL_SERVER);

	system_delta_time = 1.0f;

	/* the one-sector world: the real position setter finds the helicopter's own sector unchanged */
	memset (&world_map, 0, sizeof (world_map));

	world_map.sector_side_length = 1000;
	world_map.num_map_x_sectors = 1;
	world_map.num_map_z_sectors = 1;
	world_map.num_map_sectors = 1;
	world_map.min_map_x = 0.0f;
	world_map.min_map_y = -100.0f;
	world_map.min_map_z = 0.0f;
	world_map.max_map_x = 999.0f;
	world_map.max_map_y = 1000.0f;
	world_map.max_map_z = 999.0f;

	sector_map = (entity **) calloc (1, sizeof (entity *));

	memset (&pool [SECTOR_INDEX], 0, sizeof (entity));

	pool [SECTOR_INDEX].type = ENTITY_TYPE_SECTOR;

	sector_map [0] = &pool [SECTOR_INDEX];

	entity_sector_map = sector_map;

	memset (&pool [GROUP_INDEX], 0, sizeof (entity));

	pool [GROUP_INDEX].type = ENTITY_TYPE_GROUP;

	group_raw = (group *) calloc (1, sizeof (group));

	set_local_entity_data (&pool [GROUP_INDEX], group_raw);

	memset (&roster_host, 0, sizeof (roster_host));

	roster_host.bind_member = bind_member;

	campaign_roster_set_host (&roster_host);

	memset (&movement_host, 0, sizeof (movement_host));

	movement_host.move_vehicle = scripted_host;

	campaign_movement_set_host (&movement_host);

	campaign_roster_reset ();

	opened = 1;

	return 0;
}

/* a scenario starts from clear flags (a fresh host binding in the hosted profile), a fresh roster and no scripted answer or counted question */
void eech_motion_begin (void)
{
	campaign_roster_reset ();

	next_host_id = 0;
	script_moved = 0;
	script_rotated = 0;

	memset (questions, 0, sizeof (questions));

	clear_flags ();
}

/* the host's physical answer: moved (GUIDED_BEGIN) and rotated (GUIDED_END), 0 or 1, or anything else to make the host's answer malformed */
void eech_motion_script (int moved, int rotated)
{
	script_moved = moved;
	script_rotated = rotated;
}

/* a helicopter: the pool slot `index` typed HELICOPTER with a FRESH raw record inside the sector (position (10, 5, 10), identity attitude), linked into the campaign's roster unless `bound` is 0 (the matched
   NEGATIVE: no host identity) */
void eech_motion_helicopter (int index, int bound)
{
	helicopter
		*raw;

	memset (&pool [index], 0, sizeof (entity));

	pool [index].type = ENTITY_TYPE_HELICOPTER;

	raw = (helicopter *) calloc (1, sizeof (helicopter));

	raw_records [raw_count++] = raw;

	raw->ac.mob.position.x = 10.0f;
	raw->ac.mob.position.y = 5.0f;
	raw->ac.mob.position.z = 10.0f;
	raw->ac.mob.attitude [0][0] = 1.0f;
	raw->ac.mob.attitude [1][1] = 1.0f;
	raw->ac.mob.attitude [2][2] = 1.0f;
	raw->ac.mob.sector_link.parent = &pool [SECTOR_INDEX];

	set_local_entity_data (&pool [index], raw);

	if (bound)
	{
		campaign_roster_link_member (&pool [GROUP_INDEX], &pool [index]);
	}
}

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

static int finish (int caught, char *fatal, int size)
{
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

/* THE REAL BODIES, through their public entries: the engine's own fatal (the host's refusal, as an engine entry catches it) is caught here; 1 when the engine stopped */
int eech_motion_guided_begin (int index, char *fatal, int size)
{
	int caught;

	TRAP (campaign_movement_helicopter_guided_begin (&pool [index]), caught);

	return finish (caught, fatal, size);
}

int eech_motion_guided_end (int index, char *fatal, int size)
{
	campaign_hc_steer
		steer;

	float
		angles [3];

	int caught;

	memset (&steer, 0, sizeof (steer));

	steer.delta_seconds = 1.0f;
	steer.task_leader_is_self = TRUE;

	TRAP (campaign_movement_helicopter_guided_end (&pool [index], &steer, angles), caught);

	return finish (caught, fatal, size);
}

/* the real raw getter of the helicopter (destructive: it clears the other entities' bits of the shared word); type 1 MOVED, 2 ROTATED */
int eech_motion_raw (int type, int index)
{
	return get_local_entity_int_value (&pool [index], (type == 1) ? INT_TYPE_MOVED : INT_TYPE_ROTATED);
}

/* the real raw setter: the SAME call the bodies make, used by the test to prepare a neighbour's bit */
void eech_motion_set (int type, int index)
{
	set_local_entity_int_value (&pool [index], (type == 1) ? INT_TYPE_MOVED : INT_TYPE_ROTATED, TRUE);
}

/* the whole word of the array (the host's own observation in the hosted profile, the global array in the original) */
unsigned int eech_motion_word (int type, int index)
{
#ifdef EECH_CAMPAIGN_HOST_ENTITY
	return hosted_word ((type == 1) ? INT_TYPE_MOVED : INT_TYPE_ROTATED, index);
#else
	return ((type == 1) ? moved_entities : rotated_entities) [index >> 5];
#endif
}

/* how often the host was asked the question of one movement mode since the last begin */
int eech_motion_questions (int mode)
{
	return ((mode >= 0) && (mode < MODES)) ? questions [mode] : -1;
}

int eech_motion_mode_begin (void)
{
	return CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_BEGIN;
}

int eech_motion_mode_end (void)
{
	return CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_END;
}

float eech_motion_position_x (int index)
{
	return ((helicopter *) get_local_entity_data (&pool [index]))->ac.mob.position.x;
}

float eech_motion_velocity (int index)
{
	return ((helicopter *) get_local_entity_data (&pool [index]))->ac.mob.velocity;
}

void eech_motion_close (void)
{
	int i;

	if (!opened)
	{
		return;
	}

	campaign_movement_set_host (NULL);
	campaign_roster_set_host (NULL);

	free (group_raw);
	free (sector_map);

	for (i = 0; i < raw_count; i++)
	{
		free (raw_records [i]);
	}

	set_comms_model ((comms_model_types) saved_comms);

	system_delta_time = saved_dt;
	entities = saved_entities;
	number_of_entities = saved_number_of_entities;
	world_map = saved_world_map;
	entity_sector_map = saved_sector_map;

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

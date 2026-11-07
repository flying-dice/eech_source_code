/*
 * TEST-ONLY (EEDCS-49, D3, Task49, feature `physical-flags-collision-check`): the second REAL immediate-consumer control of the per-entity UPDATED word: the moving-mobile PASS-THROUGH of the collision query, the very statement
 *
 *     if (get_local_entity_int_value (target, INT_TYPE_UPDATED)) { new = position; old = new - motion * dt } else { old = position; new = old + motion * dt }
 *
 * of get_line_of_sight_collision_entity (en_cllsn.c, MOBILE TARGET, moving mobile), run through the REAL public function, the real sector lists and sector map, the real mobile getters (the raw UPDATED word included), the real
 * get_delta_time, the broad phase and the engine's own accurate_moving_bounding_box_collision_test, over hand-made entities of the engine's own types. It extends, and does not replace, the guide-consumer control
 * (csrc/eech_physical_flags_consumer_check.c): that control is a different consumer (the fixed-wing arrival response); this one is the collision consumer.
 *
 * TWO PROFILES, one world, the SAME scenarios. In the ORIGINAL profile (feature `collision-fixture`: EECH_CAMPAIGN_HOST_ENTITY is not defined) the body above is compiled and this control pins it (15 expectations). In the HOSTED profile (the default
 * build) that body is not compiled: get_line_of_sight_collision_entity returns campaign_scan_entities, and the decision is the HOST's (ENTITY_SCAN, ENTITY_CANDIDATE, ENTITY_COLLISION), which decides a MOBILE candidate from its OWN physical record of
 * it. So in the hosted profile each mobile is first LINKED to the campaign's own roster group (gp_roster.c campaign_roster_link_member) and the host issues its identity and starts its mover record through the host's supported registration
 * (the private test collaborator `bind_member`, tests/support/lua_provider.rs, as the reference host's bind_mobile does it); the real roster placement, scan, candidate (which asks for the UPDATED word once, when the host needs it) and narrow-phase
 * questions are the real provider's, and the flag word is the host's store. The same 15 expectations hold, and a 16th is the matched NEGATIVE: a mobile that was never linked (no host identity, no record) is refused by name by the real host.
 *
 * BOTH profiles are served by the REAL reference provider (tests/support/lua_provider.rs: a small TEST bridge, not the shipped Engine::boot bridge, over the real roster, flag store, mover registry and model store of the minimal installation).
 *
 * WHAT IS HAND-MADE, AND DECLARED (nothing else is scripted):
 *   - the entity pool (the engine's own `entity` array; the flag word is keyed by `entity - entities`), ONE sector entity of a 1 x 1 map (side 8192), the sector's lists (the real list functions) and the two moving mobiles,
 *     slots of the pool typed FIXED_WING with a FRESH raw fixed_wing record each (position, motion vector (20, 0, 0), identity attitude, shape 1); no other field is read;
 *   - the model data: the REAL provider's catalogue and scenes (the minimal installation: scene 1 is a loaded box of half-width 1, here with NO collision object, the original's -1, so the narrow phase tests the body's bounding box and the
 *     host answers no triangle), installed through the real campaign_model_install_scenes; the original solver reads the host's tables in this build (EECH_CAMPAIGN_HOST_MODEL);
 *   - the delta time: system_delta_time is set directly (1.0) and given back; the comms model is not needed.
 *
 * WHAT IT PINS (observed facts of the ORIGINAL code, none a design): the unset flag selects the FUTURE position (the segment through the position ahead hits, the one behind misses); the SET flag (raw mask, read once per
 * visit, destructively) selects the PREVIOUS position (the two results swap); a read of one mobile destroys the other's bit of the shared word, so the consumed result of the second depends on the ORDER of the list (the
 * scan visits the tail first); the own bit survives its own read. Positive coherent fixture FIRST, then single changes.
 *
 * NOT CLAIMED: the stationary-mobile branch, the fixed branch, the city, the narrow phase's own arithmetic beyond its hit or miss, any hosted profile, -O2, signalling NaN, the Tacview writers, the supply descriptor.
 *
 * STATE THE CONTROL CHANGES AND GIVES BACK: the dispatch tables (copied whole), the `entities` pointer, world_map, entity_sector_map, system_delta_time, and the three flag arrays (copied whole).
 *
 * eech_physical_flags_collision_run returns the number of failed expectations (0: all held) and writes one line per failure and the observed facts; *checks is the number of expectations made.
 */

#ifdef EECH_PHYSICAL_FLAGS_COLLISION_CHECK

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include <unistd.h>

#include "project.h"

#include "eech_headless.h"

#include "entity/mobile/mb_hostmodel.h"

struct check
{
	char *text;
	int capacity, used, failures, checks;
};

static void note (struct check *c, const char *format, ...)
{
	va_list args;
	int n;

	if (c->used >= c->capacity - 1)
	{
		return;
	}

	va_start (args, format);
	n = vsnprintf (c->text + c->used, (size_t) (c->capacity - c->used), format, args);
	va_end (args);

	if (n > 0)
	{
		c->used += n;

		if (c->used > c->capacity - 1)
		{
			c->used = c->capacity - 1;
		}
	}
}

#define EXPECT(C, CONDITION, WHAT) do { (C)->checks++; if (!(CONDITION)) { (C)->failures++; note ((C), "FAIL %s:%d %s\n", __FILE__, __LINE__, (WHAT)); } } while (0)
#define EXPECT_INT(C, ACTUAL, WANTED, WHAT) do { long long a_ = (long long) (ACTUAL), w_ = (long long) (WANTED); (C)->checks++; if (a_ != w_) { (C)->failures++; note ((C), "FAIL %s:%d %s: got %lld, wanted %lld\n", __FILE__, __LINE__, (WHAT), a_, w_); } } while (0)

#include "eech_physical_flags_hosted.h"

#ifdef EECH_CAMPAIGN_HOST_ENTITY
#define PROFILE "hosted"
#else
#define PROFILE "original"

/* the ORIGINAL profile has no hosted header content: the provider is installed here the same way */
typedef void (*flags_host_fresh_fn) (void);
typedef unsigned int (*flags_host_observe_fn) (const char *key, int array);

static flags_host_fresh_fn
	hosted_fresh_fn = NULL;

static flags_host_observe_fn
	hosted_observe_fn = NULL;
#endif

#define SCENE 1

/* the model provider, the fresh-binding entry and the host-word observation are the test's (tests/support/lua_provider.rs): the REAL reference provider over the minimal installation (scene 1 is a box of half-width 1) */
void eech_physical_flags_collision_set_host (campaign_model_fn model, flags_host_fresh_fn fresh, flags_host_observe_fn observe, campaign_bind_member_fn bind)
{
	campaign_model_host
		host;

	campaign_roster_host
		roster_host;

	memset (&host, 0, sizeof (host));

	host.model = model;
	hosted_fresh_fn = fresh;
	hosted_observe_fn = observe;

	campaign_model_set_host (&host);

#ifdef EECH_CAMPAIGN_HOST_ENTITY
	/* the private test collaborator of the campaign's own roster: the host's supported registration of a movable member (the reference host's bind_mobile), tests/support/lua_provider.rs bind_member */
	memset (&roster_host, 0, sizeof (roster_host));

	roster_host.bind_member = bind;

	campaign_roster_set_host (&roster_host);
#else
	(void) roster_host;
	(void) bind;
#endif
}

static entity
	sector_entity,

	*sector_map [1];

static entity *world_sector (void)
{
	return &sector_entity;
}

static struct saved_table
{
	void *table;
	size_t size;
	void *copy;
} saved_tables [] =
{
#define TABLE(T) { (void *) (T), sizeof (T), NULL }
	TABLE (message_responses),
	TABLE (fn_get_local_entity_int_value),
	TABLE (fn_set_local_entity_int_value),
	TABLE (fn_set_local_entity_raw_int_value),
	TABLE (fn_set_client_server_entity_int_value),
	TABLE (fn_get_local_entity_float_value),
	TABLE (fn_get_local_entity_vec3d),
	TABLE (fn_get_local_entity_vec3d_ptr),
	TABLE (fn_get_local_entity_attitude_matrix_ptr),
	TABLE (fn_get_local_entity_attitude_matrix),
	TABLE (fn_set_local_entity_first_child),
	TABLE (fn_get_local_entity_first_child),
	TABLE (fn_set_local_entity_parent),
	TABLE (fn_get_local_entity_parent),
	TABLE (fn_set_local_entity_child_succ),
	TABLE (fn_get_local_entity_child_succ),
	TABLE (fn_set_local_entity_child_pred),
	TABLE (fn_get_local_entity_child_pred),
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

#define WORDS (MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8))

#ifndef EECH_CAMPAIGN_HOST_ENTITY

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

/* a scenario starts from a clear flag word AND an empty sector list: a slot given a new mobile must not stay linked under the old one */
static entity *world_sector (void);

#define MAX_RAW 8

static fixed_wing
	*raw_records [MAX_RAW];

static int
	raw_count;

/* the word of the UPDATED array a slot belongs to, as the profile holds it: the original's global array, the host's store asked of the host */
#ifdef EECH_CAMPAIGN_HOST_ENTITY
#define UPDATED_WORD(SLOT) hosted_word (INT_TYPE_UPDATED, (SLOT))
#else
#define UPDATED_WORD(SLOT) (updated_entities [(SLOT) >> 5])
#endif

static void reset_world (void)
{
	memset (get_local_entity_data (world_sector ()), 0, sizeof (sector));

#ifdef EECH_CAMPAIGN_HOST_ENTITY
	/* the campaign's mirror of the host's placement keys ends (its boot event: the host's roster is cleared) and the host's flag store is a fresh binding, with the campaign's group keys forgotten (the test's scenario isolation) */
	campaign_place_reset ();

	/* the campaign's roster of members (the host's identities of its mobiles) is fresh too */
	campaign_roster_reset ();

	hosted_clear ();
#else
	clear_flags ();
#endif
}

/* a moving mobile: the pool slot `index` typed FIXED_WING with a FRESH raw record at (x, 100, z), motion (20, 0, 0), identity attitude, scene 1, linked into the sector's list (the real list functions: the newest insertion is the head) */
static entity *roster_group;

/* `bound` links the mobile to the campaign's roster group before it is placed: the host then issues its identity and starts its mover record (the hosted profile; the original profile has no roster here) */
static entity *make_mobile_as (entity *pool, int index, float x, float z, int bound)
{
	fixed_wing
		*raw;

	memset (&pool [index], 0, sizeof (entity));

	pool [index].type = ENTITY_TYPE_FIXED_WING;

	raw = (fixed_wing *) calloc (1, sizeof (fixed_wing));

	raw_records [raw_count++] = raw;

	raw->ac.object_3d_shape = SCENE;
	raw->ac.mob.position.x = x;
	raw->ac.mob.position.y = 100.0f;
	raw->ac.mob.position.z = z;
	raw->ac.mob.motion_vector.x = 20.0f;
	raw->ac.mob.motion_vector.y = 0.0f;
	raw->ac.mob.motion_vector.z = 0.0f;
	raw->ac.mob.velocity = 20.0f;
	raw->ac.mob.alive = TRUE;
	raw->ac.mob.attitude [0][0] = 1.0f;
	raw->ac.mob.attitude [1][1] = 1.0f;
	raw->ac.mob.attitude [2][2] = 1.0f;

	set_local_entity_data (&pool [index], raw);

#ifdef EECH_CAMPAIGN_HOST_ENTITY
	if (bound)
	{
		campaign_roster_link_member (roster_group, &pool [index]);
	}
#else
	(void) bound;
#endif

	insert_local_entity_into_parents_child_list (&pool [index], LIST_TYPE_SECTOR, &sector_entity, NULL);

	return &pool [index];
}

static entity *make_mobile (entity *pool, int index, float x, float z)
{
	return make_mobile_as (pool, index, x, z, TRUE);
}

static void set_updated (entity *en)
{
	set_local_entity_int_value (en, INT_TYPE_UPDATED, TRUE);
}

/* the REAL query along one straight segment: the entity it returns (NULL: the line of sight is clear) */
static entity *query (float x0, float y0, float z0, float x1, float y1, float z1)
{
	vec3d
		old_position,
		new_position,
		intercept,
		normal;

	memset (&intercept, 0, sizeof (intercept));
	memset (&normal, 0, sizeof (normal));

	old_position.x = x0;
	old_position.y = y0;
	old_position.z = z0;
	new_position.x = x1;
	new_position.y = y1;
	new_position.z = z1;

	return get_line_of_sight_collision_entity (NULL, NULL, &old_position, &new_position, &intercept, &normal);
}

#define X 4000.0f
#define Z 4000.0f
/* a vertical segment through the box of a mobile at (X, 100, Z) at its NEW position of the future path (X + 20 +- 2: the segment stands at X + 21, off the box centre and off the diagonal of the face it crosses, which the original's face test does not report) and one through its OLD position of the previous path (X - 20 +- 2: X - 19). The collision test is made with the body at the old, the middle and the new position (the speed 20 is above the body length 6), so the future path has boxes at X, X + 10 and X + 20 and the previous one at X - 20, X - 10 and X */
#define AHEAD(MOBILE_Z) query (X + 20.4f, 130.0f, (MOBILE_Z) + 0.2f, X + 20.4f, 70.0f, (MOBILE_Z) + 0.2f)
#define BEHIND(MOBILE_Z) query (X - 19.6f, 130.0f, (MOBILE_Z) + 0.2f, X - 19.6f, 70.0f, (MOBILE_Z) + 0.2f)

int eech_physical_flags_collision_run (char *text, int capacity, int *checks)
{
	struct check
		c;
	entity
		*pool,
		*saved_entities,
		*a,
		*b,
		**saved_sector_map;
	int
		saved_heap;
	world_map_data
		saved_map;
	float
		saved_dt;
	int
		i = 0;

	/* a safety net of the TEST only: a defect in the hand-made world (a list that loops) ends the process after two minutes instead of hanging the check */
	alarm (120);

	c.text = text;
	c.capacity = capacity;
	c.used = 0;
	c.failures = 0;
	c.checks = 0;
	text [0] = 0;

	pool = (entity *) calloc (MAX_NUM_ENTITIES, sizeof (entity));

	if (!pool)
	{
		note (&c, "FAIL the entity pool could not be allocated\n");

		*checks = 0;

		return 1;
	}

	saved_entities = entities;
	saved_dt = system_delta_time;
	saved_map = world_map;
	saved_sector_map = entity_sector_map;

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (saved_moved, moved_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_rotated, rotated_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_updated, updated_entities, sizeof (unsigned int) * WORDS);
#endif

	if ((hosted_fresh_fn == NULL) || (hosted_observe_fn == NULL))
	{
		note (&c, "FAIL no provider was installed before the run\n");

		free (pool);

		*checks = 0;

		return 1;
	}

	saved_heap = number_of_entities;

	/* the campaign's placement table is sized from the heap of the run (mb_hostplace.c: the heap this run allocated) */
	number_of_entities = MAX_NUM_ENTITIES;

	save_tables ();

	entities = pool;
	raw_count = 0;

	initialise_entity_int_value_default_functions ();
	initialise_entity_float_value_default_functions ();
	initialise_entity_vec3d_default_functions ();
	initialise_entity_attitude_default_functions ();
	initialise_entity_list_default_functions ();
	initialise_entity_message_responses ();
	overload_mobile_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_vec3d_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_attitude_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_list_functions (ENTITY_TYPE_FIXED_WING);
	overload_aircraft_list_functions (ENTITY_TYPE_FIXED_WING);
	overload_fixed_wing_list_functions ();
	overload_aircraft_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_sector_list_functions ();
	overload_sector_float_value_functions ();

	campaign_model_install_scenes ();

	/* the sector: one entity of a 1 x 1 map, side 8192 */
	memset (&sector_entity, 0, sizeof (sector_entity));

	set_local_entity_type (&sector_entity, ENTITY_TYPE_SECTOR);
	set_local_entity_data (&sector_entity, calloc (1, sizeof (sector)));

	memset (&world_map, 0, sizeof (world_map));

	world_map.sector_side_length = 8192;
	world_map.sector_side_length_mask = 8191;
	world_map.num_map_x_sectors = 1;
	world_map.num_map_z_sectors = 1;
	world_map.num_map_sectors = 1;
	world_map.min_map_x_sector = 0;
	world_map.min_map_z_sector = 0;
	world_map.max_map_x_sector = 0;
	world_map.max_map_z_sector = 0;

	sector_map [0] = &sector_entity;

	entity_sector_map = sector_map;

	system_delta_time = 1.0f;

	memset (&pool [400], 0, sizeof (entity));

	pool [400].type = ENTITY_TYPE_GROUP;

	roster_group = &pool [400];

	/* ---- POSITIVE FIRST: ONE moving mobile, flag unset: the future position is the one the query uses ---- */
	reset_world ();

	a = make_mobile (pool, 3, X, Z);

	EXPECT (&c, BEHIND (Z) == NULL, "unset flag: a segment behind the mobile (the previous path) is clear");
	EXPECT (&c, AHEAD (Z) == a, "unset flag: a segment ahead of the mobile (the future path) hits it");
	EXPECT_INT (&c, UPDATED_WORD (3) & 8, 0, "the read of an unset entity leaves its bit clear");

	/* ---- ONE CHANGED INPUT: the flag set through the real setter: the previous position is used: the two results swap ---- */
	set_updated (a);

	EXPECT (&c, BEHIND (Z) == a, "flag set: a segment behind the mobile (the previous path) now hits it");
	EXPECT (&c, AHEAD (Z) == NULL, "flag set: a segment ahead of the mobile is now clear");
	EXPECT_INT (&c, UPDATED_WORD (3) & 8, 8, "the consumer's destructive read kept the entity's own bit (raw mask, no boolean)");
	EXPECT (&c, BEHIND (Z) == a, "a second query reads the same raw mask again: the previous path still hits");

	/* ---- ORDER: two mobiles of one word, both set; the scan visits the list's TAIL first, so the OLDER entity is read first and destroys the newer one's bit ---- */
	reset_world ();

	a = make_mobile (pool, 3, X, Z);
	b = make_mobile (pool, 9, X, Z + 100.0f);

	set_updated (a);
	set_updated (b);

	/* one query that reaches both sectors' entries reads both flags: the first read (a, the tail) keeps a's bit and clears b's, then b's own read finds it clear and keeps it clear */
	EXPECT (&c, BEHIND (Z) == a, "both set: the first visited (the older, slot 3) keeps its flag: its previous path hits");
	EXPECT_INT (&c, UPDATED_WORD (3) & 0x208, 8, "the first read cleared the newer mobile's bit of the shared word");
	EXPECT (&c, BEHIND (Z + 100.0f) == NULL, "the newer mobile lost its flag to the older one's read: its previous path is clear");
	EXPECT (&c, AHEAD (Z + 100.0f) == b, "and its future path hits (it is read as unset)");

	/* ---- the reverse insertion order: the mirror result (the older mobile is the one that loses its flag) ---- */
	reset_world ();

	a = make_mobile (pool, 9, X, Z);
	b = make_mobile (pool, 3, X, Z + 100.0f);

	set_updated (a);
	set_updated (b);

	EXPECT (&c, BEHIND (Z) == a, "reverse order: the first visited is the older (slot 9) and keeps its flag: its previous path hits");
	EXPECT_INT (&c, UPDATED_WORD (3) & 0x208, 0x200, "its read cleared the newer mobile's bit (slot 3) and kept its own (slot 9)");
	EXPECT (&c, BEHIND (Z + 100.0f) == NULL, "the newer mobile lost its flag whichever bit it holds: its previous path is clear");
	EXPECT_INT (&c, UPDATED_WORD (3) & 0x208, 0, "and its own read of the cleared word destroyed the older mobile's bit too: an unset consumer destroys a set neighbour");

#ifdef EECH_CAMPAIGN_HOST_ENTITY
	/* ---- the matched NEGATIVE: the host decides a MOBILE candidate from ITS OWN physical record of it, so a mobile the campaign never linked to its roster (no host identity, no record) is refused by name by the real provider, which the
	   engine turns into its fatal (caught here as an engine entry catches it). The identity refusal is not relaxed to make the positive case pass: the positive mobiles above were bound first. ---- */
	reset_world ();

	a = make_mobile_as (pool, 3, X, Z, FALSE);

	{
		jmp_buf
			trap;

		jmp_buf
			*saved_target = eech_fatal_target;

		int
			saved_poisoned = eech_engine_poisoned,
			caught = 0;

		eech_fatal_target = &trap;

		if (setjmp (trap) == 0)
		{
			(void) AHEAD (Z);
		}
		else
		{
			caught = 1;
		}

		eech_fatal_target = saved_target;
		eech_engine_poisoned = saved_poisoned;

		EXPECT (&c, caught && (strstr (eech_fatal_message, "the host holds no physical record for this mobile candidate") != NULL), "a mobile with no host identity is refused by name by the real host");
	}
#endif

	note (&c, "physical-flags-collision: consumer=get_line_of_sight_collision_entity (moving mobile pass-through, original profile)\n");

	/* give everything back */
	for (i = 0; i < raw_count; i++)
	{
		free (raw_records [i]);
	}

	free (get_local_entity_data (&sector_entity));

	entity_sector_map = saved_sector_map;
	world_map = saved_map;
	system_delta_time = saved_dt;
	entities = saved_entities;
	number_of_entities = saved_heap;

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (moved_entities, saved_moved, sizeof (unsigned int) * WORDS);
	memcpy (rotated_entities, saved_rotated, sizeof (unsigned int) * WORDS);
	memcpy (updated_entities, saved_updated, sizeof (unsigned int) * WORDS);
#endif

	restore_tables ();

	free (pool);

	alarm (0);

	*checks = c.checks;

	return c.failures;
}

#endif

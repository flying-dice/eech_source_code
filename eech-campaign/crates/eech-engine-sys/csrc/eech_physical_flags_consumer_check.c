/*
 * TEST-ONLY (EEDCS-49, D3, feature `physical-flags-consumer-check`): the first REAL IMMEDIATE-CONSUMER control of the per-entity UPDATED word (ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1). It extends the sociable control of csrc/eech_physical_flags_check.c (199 expectations over the real
 * getter, setter, arrays and reset, left as it is) with the smallest consumer the source supports: the ORIGINAL `response_to_check_mobile_reached_guide` of the fixed wing (aircraft/fixwing/fw_msgs.c 396..518), called through the REAL message dispatch
 * (notify_local_entity -> message_responses [ENTITY_TYPE_FIXED_WING][ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE], installed by the engine's own overload_fixed_wing_message_responses), reading its ACTUAL collaborators: the real getters for LANDED, DISTANCE, VELOCITY, the motion
 * vector and the position (through the real dispatch tables the engine's own overload_* initialisers fill), the real guide criterion accessors (guide.c), the real get_delta_time () and the real get_3d_perp_dist_of_point_from_line (modules/maths/vector.c). Nothing of the response, the
 * getters or the vector helper is rewritten, mocked or replaced.
 *
 * WHAT IS HAND-MADE, AND DECLARED (nothing else is scripted):
 *   - the entities: slots of a calloc'd pool of the engine's own `entity` struct (the flag arrays are keyed by `entity - entities`); a receiver is a slot typed FIXED_WING whose raw `fixed_wing` record is calloc'd and set field by field (operational state NAVIGATING = not landed, distance, velocity,
 *     position, motion vector); the sender is ONE `entity` of type GUIDE (outside the pool) with a calloc'd raw `guide` whose RADIUS criterion is valid and whose WEAPON_VECTOR criterion is off (so the response never reaches its frand1 path); no other criterion is read;
 *   - the delta time: the engine's variable `system_delta_time` (what get_delta_time () reads) is set directly and given back (set_manual_delta_time also appends to a private history that cannot be restored);
 *   - the comms model: SERVER, through the engine's set_comms_model (the response asserts it), given back;
 *   - the update entity: one ENTITY_TYPE_UPDATE entity with an empty update list, so the real update_client_server_entities runs its reset (as in the first control).
 *
 * A SOURCE FACT THIS CONTROL MEASURES, not a design: the response reads the guide position with `get_local_entity_vec3d (receiver, VEC3D_TYPE_GUIDE_POSITION, ...)` on the FIXED-WING RECEIVER. Only a GUIDE registers that vec3d type; the receiver's table keeps the engine's DEFAULT getter, which writes the ZERO vector. So in the pass-through branch the
 * "guide position" the original measures against is the world origin, not the guide (the control observes that through the returned value; it does not repair it).
 *
 * WHAT IT PINS (the consumer's immediate contract, observed with the ORIGINAL code):
 *   - the raw UPDATED word is read ONLY on the pass-through branch (not landed, radius valid, range > radius, range - velocity <= radius); the early returns (landed, range - velocity > radius, range <= radius) leave every flag word untouched;
 *   - on that branch an UNSET flag selects the future position (pos2 = pos1 + motion * dt) and a SET flag (any raw mask, bit 31 = INT_MIN included) the previous position (pos2 = pos1 - motion * dt); with the coherent trajectory below the unset case is TRUE and the set case FALSE
 *     (the origin projects onto the finite segment only in the first), and the real delta time and radius change the result;
 *   - the consumer's read is the destructive `&=` read: it clears the other entities' bits of the shared word and keeps its own, so the result of the SECOND consumer call of two entities in one word depends on the order of the calls (A then B differs from B then A);
 *   - above the reset extent (index >= 131072) the same holds: a set bit survives update_client_server_entities and the consumer still reads it, a consumer read of another entity of its word clears it (so the bit persists only until a destructive neighbour read or another overwrite), below the extent the reset clears it;
 *   - a slot given a fresh raw record keeps the flag bit of the slot index (no claim about the real heap lifecycle).
 *
 * NOT CLAIMED: the other consumers (en_cllsn.c moving-mobile pass-through, gd_supply_arrival.c descriptor, the Tacview writers) are NOT run; the shipping profile routes the arrival to campaign_supply_arrival_check / campaign_radius_arrival_check first (guide.c check_mobile_reached_waypoint), so this
 * consumer is the ORIGINAL arrival the profile retains for the other cases; the weapon-vector path (frand1) is not run; helicopters, vehicles and ships (their own responses) are not run; the setter with FALSE; -O2, strict-C or signalling-NaN behaviour (the bit-31 observation is the supported compiler profile's, not a portable guarantee).
 *
 * STATE THE CONTROL CHANGES AND GIVES BACK: the int, float, vec3d, list and message tables, the three flag arrays (copied whole), `entities`, update_entity, update_succ, locked_frame_rate, system_delta_time and the comms model are saved first and restored on every path (no early return).
 *
 * TWO PROFILES (Task49, the host-owned flag store): the SAME scenarios run in the ORIGINAL profile (feature collision-fixture: the words are the engine's global arrays, unchanged) and in the HOSTED profile (the default build: the words are
 * the HOST's, a scenario starts from a fresh host binding and a word is observed by asking the host; eech_physical_flags_hosted.h, tests/support/flags_host.rs).
 *
 * eech_physical_flags_consumer_run returns the number of failed expectations and writes one line per failure and the observed facts; *checks is the number of expectations made.
 */

#ifdef EECH_PHYSICAL_FLAGS_CONSUMER_CHECK

#include <float.h>
#include <limits.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "project.h"

#include "eech_physical_flags_hosted.h"

#define WORDS (MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8))

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

#ifdef EECH_CAMPAIGN_HOST_ENTITY

/* the hosted profile starts a scenario from a fresh host binding and observes a word by asking the HOST (see eech_physical_flags_hosted.h) */
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

/* the hand-made world: the receivers' raw records (freed at the end), the guide and its raw record */
#define MAX_RAW 64

static fixed_wing
	*raw_records [MAX_RAW];

static int
	raw_count;

static entity
	guide_en;

static guide
	*guide_raw;

static const float
	RADIUS = 6.0f;

/* a receiver: the pool slot `index` typed FIXED_WING with a FRESH raw record on the coherent trajectory: not landed, distance 10, velocity 20, position (10, 5, 0), motion (-20, 0, 0) */
static entity *receiver (entity *pool, int index)
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

	return &pool [index];
}

static fixed_wing *raw_of (entity *en)
{
	return (fixed_wing *) get_local_entity_data (en);
}

static void set_radius (float radius)
{
	guide_raw->criteria [GUIDE_CRITERIA_RADIUS].valid = TRUE;
	guide_raw->criteria [GUIDE_CRITERIA_RADIUS].value = radius;
	guide_raw->criteria [GUIDE_CRITERIA_WEAPON_VECTOR].valid = FALSE;
}

/* the ORIGINAL response, through the real message dispatch */
static int consume (entity *en)
{
	return notify_local_entity (ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE, en, &guide_en);
}

static unsigned int mask_of (int index)
{
	return 1u << (index & 31);
}

static unsigned int word_of_index (int index)
{
#ifdef EECH_CAMPAIGN_HOST_ENTITY
	return hosted_word (INT_TYPE_UPDATED, index);
#else
	return updated_entities [index >> 5];
#endif
}

static void set_updated (entity *en)
{
	set_local_entity_int_value (en, INT_TYPE_UPDATED, TRUE);
}

static void run_cases (struct check *c, entity *pool, entity *update_en)
{
	static const int offsets [] = { 0, 1, 2, 30, 31 };
	int
		i,
		word = 7,
		base = word * 32,
		first,
		second;
	entity
		*a,
		*b;
	float
		saved_dt = system_delta_time;

	system_delta_time = 1.0f;

	set_radius (RADIUS);

	/* POSITIVE FIRST: the coherent trajectory with no flag set: the pass-through branch (10 - 20 <= 6 < 10) is reached, the unset flag selects the FUTURE position (-10, 5, 0), the origin projects onto the finite segment at distance 5 <= 6: TRUE, the word untouched */
	clear_flags ();

	a = receiver (pool, base + 3);

	EXPECT_INT (c, consume (a), TRUE, "POSITIVE: an unset flag, the future position passes within the radius: TRUE");
	EXPECT_INT (c, word_of_index (base + 3), 0, "the read of an unset entity leaves the word zero");

	/* ONE CHANGED INPUT: the same entity with its flag set: the PREVIOUS position (30, 5, 0) is selected, the origin projects off the segment (FLT_MAX): FALSE; the entity's own bit is kept */
	set_updated (a);

	EXPECT_INT (c, consume (a), FALSE, "the flag set: the previous position, no pass-through: FALSE");
	EXPECT_INT (c, word_of_index (base + 3), mask_of (base + 3), "the consumer's destructive read kept the entity's own bit");
	EXPECT_INT (c, consume (a), FALSE, "a second call reads the same raw mask again: FALSE");

	/* every raw mask: unset TRUE, set FALSE, whatever the mask (INT_MIN for bit 31 included): the consumer sees a non-zero RAW value, never a normalised boolean */
	for (i = 0; i < (int) (sizeof (offsets) / sizeof (offsets [0])); i++)
	{
		clear_flags ();

		a = receiver (pool, base + offsets [i]);

		EXPECT_INT (c, consume (a), TRUE, "unset at this offset: TRUE");

		set_updated (a);

		EXPECT_INT (c, get_local_entity_int_value (a, INT_TYPE_UPDATED), (long long) (int) mask_of (offsets [i]), "the raw value the consumer reads is the mask, not 1");
		EXPECT_INT (c, consume (a), FALSE, "set at this offset: FALSE");
		EXPECT_INT (c, word_of_index (base + offsets [i]), mask_of (offsets [i]), "the consumer left exactly the entity's own bit");
	}

	/* a changed radius and a changed delta time change the REAL arithmetic's result with the flag unset */
	clear_flags ();

	a = receiver (pool, base + 3);

	set_radius (4.0f);

	EXPECT_INT (c, consume (a), FALSE, "radius 4: the future position's perpendicular distance 5 exceeds it: FALSE");

	set_radius (RADIUS);

	system_delta_time = 0.25f;

	EXPECT_INT (c, consume (a), FALSE, "delta time 0.25: the future position (5, 5, 0) ends before the origin projects onto the segment: FALSE");

	system_delta_time = 1.0f;

	EXPECT_INT (c, consume (a), TRUE, "back at delta time 1: TRUE again");

	/* the raw word is read ONLY on the pass-through branch: the three early returns leave every flag word untouched even with the receiver's and a neighbour's bits set */
	clear_flags ();

	a = receiver (pool, base + 3);
	b = receiver (pool, base + 9);

	set_updated (a);
	set_updated (b);

	raw_of (a)->ac.distance = 5.0f; /* range <= radius: the in-radius early TRUE */

	EXPECT_INT (c, consume (a), TRUE, "range inside the radius: TRUE without the pass-through");
	EXPECT_INT (c, word_of_index (base + 3), mask_of (base + 3) | mask_of (base + 9), "the in-radius return did not read the flag: the neighbour's bit is still set");

	raw_of (a)->ac.distance = 100.0f; /* range - velocity > radius: the early FALSE */

	EXPECT_INT (c, consume (a), FALSE, "range - velocity beyond the radius: FALSE");
	EXPECT_INT (c, word_of_index (base + 3), mask_of (base + 3) | mask_of (base + 9), "the early FALSE did not read the flag either");

	raw_of (a)->ac.distance = 10.0f;
	raw_of (a)->ac.operational_state = OPERATIONAL_STATE_LANDED; /* landed: the first early FALSE */

	EXPECT_INT (c, consume (a), FALSE, "landed: FALSE");
	EXPECT_INT (c, word_of_index (base + 3), mask_of (base + 3) | mask_of (base + 9), "the landed return did not read the flag");

	/* ORDER: two entities of one word, both set: the consumer's read of the first clears the second's bit, so the second's answer changes; the reverse order gives the mirror result */
	for (first = 0; first < 2; first++)
	{
		second = 1 - first;

		clear_flags ();

		a = receiver (pool, base + 3);
		b = receiver (pool, base + 9);

		set_updated (a);
		set_updated (b);

		{
			entity *order [2];
			int result [2];

			order [0] = first ? b : a;
			order [1] = first ? a : b;

			result [0] = consume (order [0]);
			result [1] = consume (order [1]);

			EXPECT_INT (c, result [0], FALSE, "the first consumed entity sees its own set flag: FALSE");
			EXPECT_INT (c, result [1], TRUE, "the second consumed entity lost its flag to the first's destructive read: TRUE");
		}

		(void) second;
	}

	/* an UNSET receiver with a set neighbour: the consumer answers TRUE and DESTROYS the neighbour's flag */
	clear_flags ();

	a = receiver (pool, base + 3);
	b = receiver (pool, base + 9);

	set_updated (b);

	EXPECT_INT (c, consume (a), TRUE, "unset receiver: TRUE");
	EXPECT_INT (c, word_of_index (base + 3), 0, "the consumer's read of the unset receiver cleared the neighbour's flag");
	EXPECT_INT (c, consume (b), TRUE, "the neighbour now reads unset: TRUE (its flag was destroyed by another entity's consumer)");

	/* the reset extent seen through the consumer: below 131072 the real reset clears the bit, so the consumer sees it unset */
	locked_frame_rate = TRUE;

	clear_flags ();

	a = receiver (pool, 100);

	set_updated (a);

	EXPECT_INT (c, consume (a), FALSE, "low index, flag set: FALSE");

	update_client_server_entities ();

	EXPECT_INT (c, consume (a), TRUE, "low index after the real reset: TRUE");

	/* ABOVE the extent: the bit SURVIVES the quarter-memset and the consumer still reads it, repeated own-bit reads change nothing; a destructive read of a NEIGHBOUR clears it (so it persists only until then) */
	clear_flags ();

	a = receiver (pool, 400000);
	b = receiver (pool, 400001);

	set_updated (a);

	EXPECT_INT (c, consume (a), FALSE, "high index, flag set: FALSE");

	update_client_server_entities ();
	update_client_server_entities ();

	EXPECT_INT (c, word_of_index (400000), mask_of (400000), "the high-index bit survived two real resets");
	EXPECT_INT (c, consume (a), FALSE, "after the resets the consumer still reads it: FALSE");
	EXPECT_INT (c, consume (a), FALSE, "repeated own-bit reads change nothing");
	EXPECT_INT (c, consume (b), TRUE, "a neighbour's consumer read (it is unset: TRUE) ...");
	EXPECT_INT (c, word_of_index (400000), 0, "... cleared the high-index bit above the reset extent");
	EXPECT_INT (c, consume (a), TRUE, "after the neighbour's destructive read the high-index entity is unset: TRUE (not permanent)");

	set_updated (a);
	set_updated (b);

	update_client_server_entities ();

	EXPECT_INT (c, consume (a), FALSE, "both set above the extent survive the reset: the first consumed sees its flag: FALSE");
	EXPECT_INT (c, consume (b), TRUE, "and its read destroyed the neighbour's: TRUE");

	/* a slot given a FRESH raw record keeps the flag bit of its index (the bit is keyed by the index alone; this is NOT a model of the real heap's reuse) */
	clear_flags ();

	a = receiver (pool, 5000);

	set_updated (a);

	a = receiver (pool, 5000);

	EXPECT_INT (c, consume (a), FALSE, "the same slot with a fresh record inherits the set bit: FALSE");

	system_delta_time = saved_dt;

	(void) update_en;
}

#ifdef EECH_CAMPAIGN_HOST_ENTITY

/* the hosted profile's provider, fresh-binding and observation entries (eech_physical_flags_hosted.h), installed by the test before the run */
void eech_physical_flags_consumer_set_host (campaign_model_fn model, flags_host_fresh_fn fresh, flags_host_observe_fn observe)
{
	hosted_install (model, fresh, observe);
}

#endif

int eech_physical_flags_consumer_run (char *text, int capacity, int *checks)
{
	struct check
		c;
	entity
		*pool,
		*saved_entities,
		*saved_update_entity,
		*saved_update_succ,
		update_en;
	update
		*update_raw;
	int
		i,
		saved_locked,
		saved_comms;
	float
		saved_dt;

	c.text = text;
	c.capacity = capacity;
	c.used = 0;
	c.failures = 0;
	c.checks = 0;
	text [0] = 0;

#ifdef EECH_CAMPAIGN_HOST_ENTITY
	if ((hosted_fresh_fn == NULL) || (hosted_observe_fn == NULL))
	{
		note (&c, "FAIL no hosted flag provider was installed before the run\n");

		*checks = 0;

		return 1;
	}
#endif

	pool = (entity *) calloc (MAX_NUM_ENTITIES, sizeof (entity));

	if (!pool)
	{
		note (&c, "FAIL the entity pool could not be allocated\n");

		*checks = 0;

		return 1;
	}

	saved_entities = entities;
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
	raw_count = 0;

	/* the engine's own table initialisation, for the types this control sends to */
	initialise_entity_int_value_default_functions ();
	initialise_entity_float_value_default_functions ();
	initialise_entity_vec3d_default_functions ();
	initialise_entity_list_default_functions ();
	initialise_entity_message_responses ();
	overload_mobile_float_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_vec3d_functions (ENTITY_TYPE_FIXED_WING);
	overload_aircraft_float_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_aircraft_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_fixed_wing_float_value_functions ();
	overload_fixed_wing_int_value_functions ();
	overload_fixed_wing_message_responses ();
	overload_update_list_functions ();

	/* the family and the response: the consumer is the engine's own installed response, and the fixed wing has NO getter of its own for the guide position (the engine's default stays) */
	EXPECT (&c, message_responses [ENTITY_TYPE_FIXED_WING][ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE] != message_responses [ENTITY_TYPE_OBJECT][ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE], "the fixed wing has its own CHECK_MOBILE_REACHED_GUIDE response (the other types keep the default)");
	EXPECT (&c, fn_get_local_entity_vec3d [ENTITY_TYPE_FIXED_WING][VEC3D_TYPE_GUIDE_POSITION] == fn_get_local_entity_vec3d [ENTITY_TYPE_OBJECT][VEC3D_TYPE_GUIDE_POSITION], "the fixed wing registers no GUIDE_POSITION getter: the engine's default (the zero vector) answers");
	EXPECT (&c, fn_get_local_entity_vec3d [ENTITY_TYPE_FIXED_WING][VEC3D_TYPE_POSITION] != fn_get_local_entity_vec3d [ENTITY_TYPE_OBJECT][VEC3D_TYPE_POSITION], "the fixed wing registers a POSITION getter (the real mobile one)");

	memset (&guide_en, 0, sizeof (guide_en));

	guide_en.type = ENTITY_TYPE_GUIDE;

	guide_raw = (guide *) calloc (1, sizeof (guide));

	set_local_entity_data (&guide_en, guide_raw);

	set_comms_model (COMMS_MODEL_SERVER);

	memset (&update_en, 0, sizeof (update_en));

	update_en.type = ENTITY_TYPE_UPDATE;

	update_raw = (update *) calloc (1, sizeof (update));

	set_local_entity_data (&update_en, update_raw);

	set_update_entity (&update_en);

	run_cases (&c, pool, &update_en);

	note (&c, "physical-flags-consumer: consumer=response_to_check_mobile_reached_guide via message_responses[FIXED_WING][CHECK_MOBILE_REACHED_GUIDE]\n");

	/* give everything back */
	set_update_entity (NULL);
	free (update_raw);
	free (guide_raw);

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

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (moved_entities, saved_moved, sizeof (unsigned int) * WORDS);
	memcpy (rotated_entities, saved_rotated, sizeof (unsigned int) * WORDS);
	memcpy (updated_entities, saved_updated, sizeof (unsigned int) * WORDS);
#endif

	restore_tables ();

	free (pool);

	*checks = c.checks;

	return c.failures;
}

#endif

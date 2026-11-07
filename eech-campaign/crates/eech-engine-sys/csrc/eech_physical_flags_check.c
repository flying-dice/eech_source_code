/*
 * TEST-ONLY (EEDCS-49, D3, feature `physical-flags-check`): the CHARACTERISATION of the per-entity UPDATED / MOVED / ROTATED word, the next required physical contract (ENV-PHYSICAL-FLAGS-RAWWORD-CONTRACT-1).
 *
 * It calls the ORIGINAL code the engine library links, not a rewrite: the mobile family's int getter and setter for INT_TYPE_UPDATED / INT_TYPE_MOVED / INT_TYPE_ROTATED (mobile/mb_int.c, installed by the
 * engine's own overload_mobile_int_value_functions and reached through the real dispatch tables and the real get_local_entity_int_value / set_local_entity_int_value macros), the three global bit arrays
 * they work on (special/update/up_update.c) and the REAL update_client_server_entities (the only function that resets them) over an empty update list.
 *
 * WHAT IS HAND-MADE, AND DECLARED (nothing else is scripted):
 *   - the entity pool: the global `entities` pointer is pointed at a calloc'd array of MAX_NUM_ENTITIES real `entity` structs (the engine's own type; the flag arrays are keyed by `entity - entities`),
 *     a slot is typed (HELICOPTER, FIXED_WING, ROUTED_VEHICLE, OBJECT) only to select the dispatch table entry; no slot has raw data (the flag words never read it);
 *   - the update entity: one ENTITY_TYPE_UPDATE entity whose raw `update` record is zero (an empty LIST_TYPE_UPDATE), installed through the engine's own overload_update_list_functions, so the real
 *     update_client_server_entities runs its reset and its (empty) update loop; locked_frame_rate is set so exactly one iteration runs and no frame time is needed;
 *   - nothing else: no campaign, no map, no host, no random stream, no floating-point arithmetic.
 *
 * WHAT IT PINS (every line is an observed fact of the ORIGINAL code; none is a design):
 *   - the three words are three separate global bit arrays of MAX_NUM_ENTITIES / 32 unsigned words, one bit per entity index (word = index >> 5, mask = 1 << (index & 31)); the setter ORs the bit (it is TRUE-only);
 *   - the getter is `array[word] &= mask` and returns that: it returns the entity's MASK (a power of two, INT_MIN for bit 31), NOT 1, it CLEARS EVERY OTHER entity's bit in the same 32-bit word, and it
 *     does NOT clear the entity's own bit (a second read returns the mask again); a getter of an unset entity clears its set neighbours;
 *   - the key is the slot index alone: retyping a slot (an entity index reused by another type) keeps the bit, and nothing in the getter or setter looks at the type, the identity or the life of the entity;
 *   - the family domain: only the mobile family installs the three words; another type (here OBJECT) keeps the default int function for them;
 *   - the RESET: update_client_server_entities clears UPDATED once per iteration with memset (updated_entities, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)): that is a count of BYTES that is a QUARTER of
 *     the array (16384 of 65536 bytes), so entity indices 0..131071 are reset and 131072..524287 are NEVER reset (a legacy defect, preserved as a compatibility fact; a bit above the extent SURVIVES the reset, but the shared &= getter still clears it when it reads ANOTHER entity of its word); MOVED and ROTATED are not reset by the
 *     call with the Tacview logging off (their reset is behind tacview_is_logging () && tacview_reset_frame (), not exercised here).
 *
 * NOT CLAIMED: the consumers' bodies (en_cllsn.c, fw_msgs.c, gd_supply_arrival.c, the Tacview writers) are not run (their census is docs/d3-physical-flags-contract.md and tests/physical_flags_guard.rs); the setter with FALSE
 * (it is ASSERT-guarded: its behaviour with assertions off is a source fact, not run); the Tacview reset path; -O2 or any non-default engine C level; every other entity type.
 *
 * STATE THE CONTROL CHANGES AND GIVES BACK: the int and list dispatch tables, the three flag arrays (copied whole), the `entities` pointer, update_entity, update_succ and locked_frame_rate are saved first and
 * restored on every path (there is no early return).
 *
 * TWO PROFILES (Task49, the host-owned flag store). The scenarios below are the SAME text in both. In the ORIGINAL profile (feature collision-fixture, EECH_CAMPAIGN_HOST_ENTITY not defined) the words are the engine's global arrays, clear_flags () clears them and a word is read from them: this is the accepted
 * characterisation, unchanged. In the HOSTED profile (the default build) the words are the HOST's: the getter, setter and reset ask the model provider, the arrays are not defined in the build, a scenario starts from a fresh
 * host binding and a word is observed by asking the host (eech_physical_flags_hosted.h); the test installs the REAL store of eech-world as the provider (tests/support/flags_host.rs).
 *
 * eech_physical_flags_check_run returns the number of failed expectations (0: all held) and writes one line per failure and a short report of the observed facts; *checks is the number of expectations made.
 */

#ifdef EECH_PHYSICAL_FLAGS_CHECK

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

/* the word of an array as the profile holds it: the ORIGINAL profile's global array, the HOSTED profile's store asked of the host (eech_physical_flags_hosted.h); `WORD` is a word index, `SLOT` an entity slot */
#ifdef EECH_CAMPAIGN_HOST_ENTITY
#define FLAG_WORD(TYPE, WORD) hosted_word ((TYPE), (WORD) * 32)
#else
#define FLAG_WORD(TYPE, WORD) ((((TYPE) == INT_TYPE_UPDATED) ? updated_entities : (((TYPE) == INT_TYPE_MOVED) ? moved_entities : rotated_entities)) [(WORD)])
#endif

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

/* the hosted profile starts a scenario from a fresh host binding (see eech_physical_flags_hosted.h) */
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

/* the engine's own tables for the types this control sends to, written by the engine's own initialisers */
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

static entity *slot (entity *pool, int index, entity_types type)
{
	memset (&pool [index], 0, sizeof (entity));

	pool [index].type = type;

	return &pool [index];
}

static unsigned int mask_of (int index)
{
	return 1u << (index & 31);
}

/* ---- the getter/setter word semantics, through the real dispatch ---- */
static void run_word_cases (struct check *c, entity *pool, int type, const char *name)
{
	static const int offsets [] = { 0, 1, 2, 15, 30, 31 };
	int
		i,
		word = 5,
		base = word * 32,
		value;
	entity
		*a,
		*b,
		*other;

	clear_flags ();

	/* initial: nothing set, the getter answers 0 and leaves the word 0 */
	a = slot (pool, base + 3, ENTITY_TYPE_HELICOPTER);

	EXPECT_INT (c, get_local_entity_int_value (a, type), 0, "an untouched entity reads 0");
	EXPECT_INT (c, FLAG_WORD (type, word), 0, "the getter of an untouched entity leaves the word 0");

	/* one entity at every interesting bit offset: the getter returns the MASK (not 1), INT_MIN for bit 31; any non-zero value is true to a consumer */
	for (i = 0; i < (int) (sizeof (offsets) / sizeof (offsets [0])); i++)
	{
		clear_flags ();

		a = slot (pool, base + offsets [i], ENTITY_TYPE_HELICOPTER);

		set_local_entity_int_value (a, type, TRUE);

		EXPECT_INT (c, FLAG_WORD (type, word), (long long) mask_of (offsets [i]), "the setter ORs exactly this entity's bit (the word is unsigned)");

		value = get_local_entity_int_value (a, type);

		EXPECT_INT (c, value, (long long) (int) mask_of (offsets [i]), "the getter returns the entity's MASK, not TRUE");
		EXPECT (c, (value != 0) == TRUE, "a mask is non-zero, hence true to an if () consumer");
		EXPECT (c, (value == TRUE) == (offsets [i] == 0), "only bit 0 equals TRUE: every other mask is not the boolean 1");
		EXPECT (c, (offsets [i] != 31) || (value < 0), "bit 31 reads as a NEGATIVE int (INT_MIN)");
		EXPECT_INT (c, FLAG_WORD (type, word), (long long) mask_of (offsets [i]), "reading does not change the entity's own bit");
		EXPECT_INT (c, get_local_entity_int_value (a, type), value, "a second read returns the same mask: the getter does not consume the entity's own bit");
	}

	/* two entities in one word: the getter of one DESTROYS the other's flag */
	clear_flags ();

	a = slot (pool, base + 3, ENTITY_TYPE_HELICOPTER);
	b = slot (pool, base + 9, ENTITY_TYPE_FIXED_WING);

	set_local_entity_int_value (a, type, TRUE);
	set_local_entity_int_value (b, type, TRUE);

	EXPECT_INT (c, FLAG_WORD (type, word), (long long) (int) (mask_of (3) | mask_of (9)), "both bits are set");
	EXPECT_INT (c, get_local_entity_int_value (a, type), (long long) (int) mask_of (3), "the first entity reads its own mask");
	EXPECT_INT (c, FLAG_WORD (type, word), (long long) (int) mask_of (3), "its getter cleared the other entity's bit in the shared word");
	EXPECT_INT (c, get_local_entity_int_value (b, type), 0, "the other entity now reads 0: a getter destroys its neighbours' flags");
	EXPECT_INT (c, FLAG_WORD (type, word), 0, "and the getter of the now-unset entity cleared the first entity's bit too");
	EXPECT_INT (c, get_local_entity_int_value (a, type), 0, "the first entity has lost its flag to the neighbour's read: reads are order dependent");

	/* an entity in ANOTHER word is untouched by all of it */
	clear_flags ();

	a = slot (pool, base + 3, ENTITY_TYPE_HELICOPTER);
	other = slot (pool, base + 32 + 3, ENTITY_TYPE_HELICOPTER);

	set_local_entity_int_value (a, type, TRUE);
	set_local_entity_int_value (other, type, TRUE);

	EXPECT_INT (c, get_local_entity_int_value (a, type), (long long) (int) mask_of (3), "word 5, bit 3");
	EXPECT_INT (c, FLAG_WORD (type, word + 1), (long long) (int) mask_of (3), "word 6 is another entity's and is untouched");
	EXPECT_INT (c, get_local_entity_int_value (other, type), (long long) (int) mask_of (3), "the entity in the next word reads its own mask");

	/* the key is the SLOT INDEX alone: a reused slot of another type keeps the bit, a different slot of the same word does not own it */
	clear_flags ();

	a = slot (pool, base + 4, ENTITY_TYPE_HELICOPTER);

	set_local_entity_int_value (a, type, TRUE);

	b = slot (pool, base + 4, ENTITY_TYPE_ROUTED_VEHICLE);

	EXPECT_INT (c, get_local_entity_int_value (b, type), (long long) (int) mask_of (4), "a slot reused by another type reads the previous occupant's flag");

	(void) name;
}

static int sample_indices [] = { 0, 100, 131071, 131072, 400000, 524287 };
#define SAMPLES ((int) (sizeof (sample_indices) / sizeof (sample_indices [0])))

#ifdef EECH_CAMPAIGN_HOST_ENTITY

/* the hosted profile's provider, fresh-binding and observation entries (eech_physical_flags_hosted.h), installed by the test before the run */
void eech_physical_flags_check_set_host (campaign_model_fn model, flags_host_fresh_fn fresh, flags_host_observe_fn observe)
{
	hosted_install (model, fresh, observe);
}

#endif

int eech_physical_flags_check_run (char *text, int capacity, int *checks)
{
	struct check
		c;
	entity
		*pool,
		*saved_entities,
		*saved_update_entity,
		*saved_update_succ,
		*e,
		update_en;
	update
		*update_raw;
	int
		i,
		saved_locked;

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

#ifndef EECH_CAMPAIGN_HOST_ENTITY
	memcpy (saved_moved, moved_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_rotated, rotated_entities, sizeof (unsigned int) * WORDS);
	memcpy (saved_updated, updated_entities, sizeof (unsigned int) * WORDS);
#endif

	save_tables ();

	entities = pool;

	/* the engine's own table initialisation, for the types this control sends to */
	initialise_entity_int_value_default_functions ();
	initialise_entity_list_default_functions ();
	overload_mobile_int_value_functions (ENTITY_TYPE_HELICOPTER);
	overload_mobile_int_value_functions (ENTITY_TYPE_FIXED_WING);
	overload_mobile_int_value_functions (ENTITY_TYPE_ROUTED_VEHICLE);
	overload_update_list_functions ();

	/* the family domain: the three words exist for the mobile family only; another type keeps the engine's default for them */
	EXPECT (&c, fn_get_local_entity_int_value [ENTITY_TYPE_HELICOPTER][INT_TYPE_UPDATED] == fn_get_local_entity_int_value [ENTITY_TYPE_FIXED_WING][INT_TYPE_UPDATED], "the mobile types share one UPDATED getter");
	EXPECT (&c, fn_get_local_entity_int_value [ENTITY_TYPE_HELICOPTER][INT_TYPE_MOVED] == fn_get_local_entity_int_value [ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_MOVED], "the mobile types share one MOVED getter");
	EXPECT (&c, fn_get_local_entity_int_value [ENTITY_TYPE_OBJECT][INT_TYPE_UPDATED] != fn_get_local_entity_int_value [ENTITY_TYPE_HELICOPTER][INT_TYPE_UPDATED], "a fixed OBJECT has no UPDATED word (the default function stays)");
	EXPECT (&c, fn_get_local_entity_int_value [ENTITY_TYPE_OBJECT][INT_TYPE_ROTATED] != fn_get_local_entity_int_value [ENTITY_TYPE_HELICOPTER][INT_TYPE_ROTATED], "a fixed OBJECT has no ROTATED word");
	EXPECT (&c, fn_set_local_entity_int_value [ENTITY_TYPE_OBJECT][INT_TYPE_MOVED] != fn_set_local_entity_int_value [ENTITY_TYPE_HELICOPTER][INT_TYPE_MOVED], "a fixed OBJECT has no MOVED setter");
	EXPECT (&c, fn_set_client_server_entity_int_value [ENTITY_TYPE_HELICOPTER][INT_TYPE_UPDATED][COMMS_MODEL_SERVER] == fn_set_client_server_entity_int_value [ENTITY_TYPE_OBJECT][INT_TYPE_UPDATED][COMMS_MODEL_SERVER], "no client/server setter is installed for UPDATED: only the local setter exists");

	/* the word semantics, once per word */
	run_word_cases (&c, pool, INT_TYPE_UPDATED, "UPDATED");
	run_word_cases (&c, pool, INT_TYPE_MOVED, "MOVED");
	run_word_cases (&c, pool, INT_TYPE_ROTATED, "ROTATED");

	/* the three words are separate arrays: one entity set in each, read in each */
	clear_flags ();

	e = slot (pool, 200, ENTITY_TYPE_HELICOPTER);

	set_local_entity_int_value (e, INT_TYPE_MOVED, TRUE);

	EXPECT_INT (&c, get_local_entity_int_value (e, INT_TYPE_UPDATED), 0, "MOVED does not set UPDATED");
	EXPECT_INT (&c, get_local_entity_int_value (e, INT_TYPE_ROTATED), 0, "MOVED does not set ROTATED");
	EXPECT_INT (&c, get_local_entity_int_value (e, INT_TYPE_MOVED), (long long) (int) mask_of (200), "MOVED reads its own mask");

	/* the reset: the REAL update_client_server_entities over an empty update list (one iteration) */
	memset (&update_en, 0, sizeof (update_en));

	update_en.type = ENTITY_TYPE_UPDATE;

	update_raw = (update *) calloc (1, sizeof (update));

	set_local_entity_data (&update_en, update_raw);

	set_update_entity (&update_en);

	locked_frame_rate = TRUE;

	clear_flags ();

	for (i = 0; i < SAMPLES; i++)
	{
		e = slot (pool, sample_indices [i], ENTITY_TYPE_HELICOPTER);

		set_local_entity_int_value (e, INT_TYPE_UPDATED, TRUE);

		set_local_entity_int_value (e, INT_TYPE_MOVED, TRUE);

		set_local_entity_int_value (e, INT_TYPE_ROTATED, TRUE);
	}

	for (i = 0; i < SAMPLES; i++)
	{
		EXPECT_INT (&c, FLAG_WORD (INT_TYPE_UPDATED, sample_indices [i] >> 5) & mask_of (sample_indices [i]), mask_of (sample_indices [i]), "before the reset every sampled UPDATED bit is set");
	}

	update_client_server_entities ();

	for (i = 0; i < SAMPLES; i++)
	{
		int
			bit_set = (FLAG_WORD (INT_TYPE_UPDATED, sample_indices [i] >> 5) & mask_of (sample_indices [i])) != 0;

		/* the memset count is MAX_NUM_ENTITIES / 32 BYTES = a quarter of the array: indices below 131072 are reset, the rest never are */
		EXPECT (&c, bit_set == (sample_indices [i] >= 131072), "the reset clears indices below 131072 and never the rest (quarter-memset)");
		EXPECT (&c, (FLAG_WORD (INT_TYPE_MOVED, sample_indices [i] >> 5) & mask_of (sample_indices [i])) != 0, "MOVED is not reset by the call with Tacview logging off");
		EXPECT (&c, (FLAG_WORD (INT_TYPE_ROTATED, sample_indices [i] >> 5) & mask_of (sample_indices [i])) != 0, "ROTATED is not reset by the call with Tacview logging off");
	}

	/* a flag above the reset extent SURVIVES the resets (a second reset does not clear it) and the getter keeps reading its own bit; it is NOT permanent: a getter read of another entity of its word clears it (measured by the consumer control) */
	update_client_server_entities ();

	e = &pool [400000];

	EXPECT_INT (&c, get_local_entity_int_value (e, INT_TYPE_UPDATED), (long long) (int) mask_of (400000), "index 400000 is still 'updated' after two resets");
	EXPECT_INT (&c, get_local_entity_int_value (&pool [100], INT_TYPE_UPDATED), 0, "index 100 was reset");

	/* the reset boundary at the word edge: the last reset word and the first unreset word */
	clear_flags ();

	set_local_entity_int_value (slot (pool, 131071, ENTITY_TYPE_HELICOPTER), INT_TYPE_UPDATED, TRUE);
	set_local_entity_int_value (slot (pool, 131072, ENTITY_TYPE_HELICOPTER), INT_TYPE_UPDATED, TRUE);

	update_client_server_entities ();

	EXPECT_INT (&c, get_local_entity_int_value (&pool [131071], INT_TYPE_UPDATED), 0, "131071 (the last reset index) reads 0 after the reset");
	EXPECT_INT (&c, get_local_entity_int_value (&pool [131072], INT_TYPE_UPDATED), (long long) (int) mask_of (131072), "131072 (the first unreset index) keeps its flag");

	note (&c, "physical-flags-check: words=%d array_bytes=%d reset_bytes=%d reset_indices=%d\n", (int) WORDS, (int) (WORDS * sizeof (unsigned int)), (int) WORDS, (int) (WORDS * 8));

	/* give everything back */
	set_update_entity (NULL);
	free (update_raw);

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

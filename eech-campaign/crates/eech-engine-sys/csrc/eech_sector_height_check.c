/*
 * TEST-ONLY (EEDCS-49, D3, feature `sector-height-check`): the CHARACTERISATION of a sector's tallest-structure height, the scalar the next physical move to the host must reproduce.
 *
 * It calls the ORIGINAL code the engine library links, not a rewrite of it: the sector's LINK_CHILD and UNLINK_CHILD responses (sc_msgs.c, reached through message_responses, the table
 * en_list.c's funnel notifies through), the sector's float getter (sc_float.c, through the dispatch table and the real get_local_entity_float_value macro), the fixed entity's own int and
 * position getters (fx_int.c, fx_vec3d.c), the bounding-box accessor of the shipping profile (mb_hostmodel.c, the host-owned scene tables the original accessor was replaced by), and the
 * session's sector pack and unpack (sc_pack.c) over the real pack stream (en_pack.c).
 *
 * WHAT IS HAND-MADE, AND DECLARED (nothing else is scripted):
 *   - the entities: structs of the engine's own types. The sector is zero filled exactly as sc_seccreat.c fills it; a fixed sender is an `object` with a position and a scene index; the sender
 *     that is not fixed is an entity of another type whose tables are the engine's defaults;
 *   - the model provider: a scripted callback answers the catalogue and each scene with a bounding box (the host-authored data the shipping accessor serves; one scene per distinct top),
 *     installed through the real campaign_model_install_scenes. The host refuses a non-finite bound, so a top is always finite;
 *   - the sector map the session pack walks (the engine's world_map and entity_sector_map).
 * THE MESSAGE TABLE IS NOT THE PUBLIC FUNNEL. en_list.c's insert_local_entity_into_parents_child_list (the roster hook, the list write and then these notifications) needs a host roster and is
 * not run here; the real trigger and consumer for a native run are named in docs/d3-sector-height-contract.md.
 *
 * WHAT IT PINS (the immediate contract of the scalar):
 *   - it starts at the all-zero word and is a HIGH-WATER value, not the maximum of the current members: a higher fixed arrival raises it, a lower or an equal one leaves it, a removal does
 *     not lower it, a reinsertion of the same or a lower structure leaves it, a structure that moves sector raises the new sector and does not lower the old one;
 *   - only an entity whose IDENTIFY_FIXED is true and whose shape is not OBJECT_3D_INVALID_OBJECT_INDEX can raise it, by position.y + the scene's bounding-box ymax (a float addition, compared
 *     with a strict `>` against the word ACTUALLY held): over a zero-initialised history a sum that is not greater than zero (negative, +0, -0, NaN, -infinity) never changes it, but after a RAW restore a larger negative sum can raise a negative word and a restored NaN makes every comparison false (source-derived, not measured); an invalid shape never reads the bounding box;
 *   - the float words: the subnormal edges, the smallest normal, the largest finite and the infinity are stored exactly; a sum that is not exact is rounded by the mode in force (the engine
 *     entry runs toward zero) and is run under both modes;
 *   - the getter returns the stored word and reads nothing else;
 *   - the session pack writes the word as the LAST 32 raw bits of the sector's record (PACK_MODE_SERVER_SESSION only; FLOAT_TYPE_PROTOTYPE_1 is FLOAT_PACK_TYPE_UNPACKED), the unpack restores
 *     it bit for bit for the 13 representative patterns the control measures (save_words: zero, negative zero, the subnormal and normal edges, three ordinary values, both extremes, both infinities and one quiet NaN; the general domain is source-derived: FLOAT_PACK_TYPE_UNPACKED copies 32 raw bits), a client or browse session carries and restores nothing, and each sector gets its own word.
 *
 * NOT CLAIMED: a signalling-NaN payload (an x87 load can quieten it), an exhaustive enumeration of finite words (13 representative patterns were run, not every word), any -O2 or non-default engine-C level (the default level 1 only), the mobile side branch of the link
 * response (a sender that is not fixed reads the default tables: only IDENTIFY_FIXED is exercised), the sector creation (the memset is a source fact asserted by tests/sector_height_guard.rs), the full floating-point control and status state (only the rounding mode is restored and checked), and
 * any real world or host.
 *
 * STATE THE CONTROL CHANGES AND GIVES BACK: the engine's int, float and vec3d dispatch tables and message_responses are saved before the initialisers run and copied back on every path of the
 * function (it has no early return); world_map, entity_sector_map and the rounding mode are restored the same way. WHAT IT CANNOT GIVE BACK: mb_hostmodel.c keeps its installed scenes and its
 * model callback in private statics with no reset, so the control installs them once, and the safety bound is the process: this is its own test binary with one test function, so no other test
 * inherits the synthetic scenes, and any later boot in a process reinstalls them from its own provider (the module's own lifecycle rule).
 *
 * TWO PROFILES (Task49, the host-owned sector height). The ORIGINAL body of the response, the getter and the pack is compiled in every build that keeps the original entity solver (feature `collision-fixture`): there this control is the
 * accepted characterisation above, unchanged (the same scenarios, the same expectations), plus a DIFFERENTIAL of the host's raise expression (eech-world csrc/sector_height_raise.h, the same text its kernel runs) against the original response over
 * every domain case, under both rounding modes. In the hosted profile (the default: EECH_CAMPAIGN_HOST_ENTITY) the height is the HOST's word: the campaign's response raises and writes nothing, the getter and the pack read the host's word (a
 * scripted provider answers the sector height question), and the unpack states each saved word to the host raw; the hosted cases pin exactly that, and the legacy field (a canary) is never read nor written.
 *
 * eech_sector_height_check_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */

#ifdef EECH_SECTOR_HEIGHT_CHECK

#include <fenv.h>
#include <math.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "project.h"

#include "entity/mobile/mb_hostmodel.h"

/* the host's raise expression (eech-world model_step.c, `eech_model_sector_height_raise`): the same text, run here beside the original response over the same inputs */
#include "../../eech-world/csrc/sector_height_raise.h"

#define SECTOR_COUNT 4
#define STREAM_BYTES 512
#define FIXED_COUNT 4

struct check
{
	char *text;
	int capacity, used, failures, checks;
};

static void expect (struct check *c, int condition, const char *label, int line)
{
	c->checks++;
	if (!condition)
	{
		int n;
		c->failures++;
		n = snprintf (c->text + c->used, (size_t) (c->capacity - c->used), "FAIL line %d: %s\n", line, label);
		if (n > 0 && n < c->capacity - c->used)
		{
			c->used += n;
		}
	}
}

static void expect_word (struct check *c, uint32_t actual, uint32_t wanted, const char *label, int line)
{
	c->checks++;
	if (actual != wanted)
	{
		int n;
		c->failures++;
		n = snprintf (c->text + c->used, (size_t) (c->capacity - c->used), "FAIL line %d: %s: word %08x, expected %08x\n", line, label, (unsigned) actual, (unsigned) wanted);
		if (n > 0 && n < c->capacity - c->used)
		{
			c->used += n;
		}
	}
}

#define EXPECT(C, CONDITION) expect ((C), (CONDITION), #CONDITION, __LINE__)
#define EXPECT_WORD(C, ACTUAL, WANTED, LABEL) expect_word ((C), (ACTUAL), (WANTED), (LABEL), __LINE__)

static uint32_t float_bits (float value)
{
	uint32_t bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

static float bits_float (uint32_t bits)
{
	float value;

	memcpy (&value, &bits, sizeof (value));

	return value;
}

/* ---- the host-authored scenes: one scene per distinct top, registered BEFORE the one install ---- */

static uint32_t
	scene_top_bits [OBJECT_3D_LAST];

static int
	scenes_registered,
	scenes_installed,
	scene_lookup_failures;

static int scene_for (uint32_t top_bits)
{
	int
		scene;

	for (scene = 1; scene <= scenes_registered; scene++)
	{
		if (scene_top_bits [scene] == top_bits)
		{
			return scene;
		}
	}

	if (scenes_installed || (scenes_registered + 1 >= OBJECT_3D_LAST))
	{
		scene_lookup_failures++;

		return 0;
	}

	scenes_registered++;

	scene_top_bits [scenes_registered] = top_bits;

	return scenes_registered;
}

/* the host's words for the sectors of the 2 x 2 map (index x + 2 * z), the questions it was asked (the phase, the sector, and for a restore the word stated) and how many */
static uint32_t
	host_height_words [SECTOR_COUNT];

static int
	host_height_log_phase [64],
	host_height_log_x [64],
	host_height_log_z [64],
	host_height_asks;

static uint32_t
	host_height_log_word [64];

static int scripted_model (const campaign_model_request *request, campaign_model_reply *reply, char *error, size_t error_size, void *user)
{
	(void) user;

	if (request->op == CAMPAIGN_MODEL_ENTITY_SCAN)
	{
		int phase, x, z;
		uint32_t word, stated;

		phase = (int) request->pose [CAMPAIGN_SCAN_PHASE];
		x = (int) request->pose [CAMPAIGN_SCAN_CURSOR];
		z = (int) request->pose [CAMPAIGN_SCAN_CURSOR + 1];

		if ((phase != CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT && phase != CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE) || x < 0 || x > 1 || z < 0 || z > 1)
		{
			snprintf (error, error_size, "[sector-height-check] the control answers only the sector height read and restore of its 2 x 2 map");

			return 1;
		}

		memcpy (&stated, &request->pose [CAMPAIGN_SECTOR_HEIGHT_WORD], sizeof (stated));

		if (host_height_asks < 64)
		{
			host_height_log_phase [host_height_asks] = phase;
			host_height_log_x [host_height_asks] = x;
			host_height_log_z [host_height_asks] = z;
			host_height_log_word [host_height_asks] = stated;
		}

		host_height_asks++;

		if (phase == CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE)
		{
			host_height_words [x + 2 * z] = stated;
		}

		word = host_height_words [x + 2 * z];

		reply->candidate_state = (phase == CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE) ? CAMPAIGN_MODEL_SCAN_RESTORED : CAMPAIGN_MODEL_SCAN_HEIGHT;
		reply->candidate_count = CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS;

		memcpy (&reply->candidate [0], &request->pose [CAMPAIGN_SCAN_CURSOR], 2 * sizeof (float));
		memcpy (&reply->candidate [2], &word, sizeof (word));

		return 0;
	}

	if (request->op == CAMPAIGN_MODEL_CATALOGUE)
	{
		reply->scene_count = OBJECT_3D_LAST;

		return 0;
	}

	if (request->op == CAMPAIGN_MODEL_SCENE && request->scene >= 0 && request->scene < OBJECT_3D_LAST)
	{
		const char
			*name;

		float
			top;

		name = (object_3d_scene_names [request->scene]) ? object_3d_scene_names [request->scene] : "";

		strncpy (reply->name, name, sizeof (reply->name) - 1);

		/* the host authors the invalid scene as zeros and not loaded; every other scene is loaded, without a collider */
		reply->loaded = (request->scene != 0);

		reply->radius = 1.0f;

		top = (request->scene > 0 && request->scene <= scenes_registered) ? bits_float (scene_top_bits [request->scene]) : 0.0f;

		if (request->scene != 0)
		{
			reply->bounding_box [0] = -1.0f;
			reply->bounding_box [1] = 1.0f;
			reply->bounding_box [2] = 0.0f;
			reply->bounding_box [3] = top;
			reply->bounding_box [4] = -1.0f;
			reply->bounding_box [5] = 1.0f;

			memcpy (reply->bounding_box_without_lines, reply->bounding_box, sizeof (reply->bounding_box));
		}

		return 0;
	}

	snprintf (error, error_size, "[sector-height-check] the control answers only the catalogue and the scenes");

	return 1;
}

/* ---- the world this file makes of the engine's own types ---- */

static entity
	sector_entities [SECTOR_COUNT],
	*sector_map [SECTOR_COUNT],
	fixed_entities [FIXED_COUNT],
	mobile_entity;

static void make_sector (entity *en)
{
	sector
		*raw;

	raw = (sector *) calloc (1, sizeof (sector));

	memset (en, 0, sizeof (entity));

	set_local_entity_type (en, ENTITY_TYPE_SECTOR);

	set_local_entity_data (en, raw);
}

static void free_sector (entity *en)
{
	free (get_local_entity_data (en));

	set_local_entity_data (en, NULL);
}

static void make_fixed (entity *en, uint32_t y_bits, int scene)
{
	object
		*raw;

	raw = (object *) calloc (1, sizeof (object));

	raw->fix.position.y = bits_float (y_bits);

	raw->fix.object_3d_shape = scene;

	memset (en, 0, sizeof (entity));

	set_local_entity_type (en, ENTITY_TYPE_OBJECT);

	set_local_entity_data (en, raw);
}

static void free_fixed (entity *en)
{
	free (get_local_entity_data (en));

	set_local_entity_data (en, NULL);
}

static void set_scene (entity *en, int scene)
{
	((object *) get_local_entity_data (en))->fix.object_3d_shape = scene;
}

static uint32_t stored_word (entity *en)
{
	sector
		*raw;

	uint32_t
		bits;

	raw = (sector *) get_local_entity_data (en);

	memcpy (&bits, &raw->tallest_structure_height, sizeof (bits));

	return bits;
}

static void store_word (entity *en, uint32_t bits)
{
	sector
		*raw;

	raw = (sector *) get_local_entity_data (en);

	memcpy (&raw->tallest_structure_height, &bits, sizeof (bits));
}

/* the message as en_list.c's funnel sends it: the engine's own response table, the receiver's type, the list type as the one variadic */
static int deliver (entity_messages message, entity *receiver, entity *sender, ...)
{
	va_list
		pargs;

	int
		result;

	va_start (pargs, sender);

	result = message_responses [get_local_entity_type (receiver)][message] (message, receiver, sender, pargs);

	va_end (pargs);

	return result;
}

static int link_child (entity *sector_en, entity *child)
{
	return deliver (ENTITY_MESSAGE_LINK_CHILD, sector_en, child, LIST_TYPE_SECTOR);
}

static int unlink_child (entity *sector_en, entity *child)
{
	return deliver (ENTITY_MESSAGE_UNLINK_CHILD, sector_en, child, LIST_TYPE_SECTOR);
}

/* the getter as every reader reaches it: the real macro over the real dispatch table */
static uint32_t served_word (entity *sector_en)
{
	return float_bits (get_local_entity_float_value (sector_en, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT));
}

/* one arrival of one fixed structure at a sector that holds `before`; the word after it */
static uint32_t arrival (struct check *c, uint32_t before, uint32_t y_bits, uint32_t ymax_bits, const char *label)
{
	entity
		*sector_en,
		*fixed_en;

	sector_en = &sector_entities [0];

	fixed_en = &fixed_entities [1];

	store_word (sector_en, before);

	make_fixed (fixed_en, y_bits, scene_for (ymax_bits));

	expect (c, link_child (sector_en, fixed_en) == TRUE, label, __LINE__);

	free_fixed (fixed_en);

	return stored_word (sector_en);
}

struct domain_case
{
	const char *label;
	uint32_t before, y, ymax, after_to_nearest, after_toward_zero;
};

/* sums that are exact in float, so the rounding mode cannot matter (and neither can an x87 evaluation): the stored word is the same under both. A top is always finite (the host refuses a
   bound that is not); the position is the campaign's own word and may be anything */
static const struct domain_case exact_cases [] =
{
	{ "100 + 10 over zero is stored", 0x00000000u, 0x42c80000u, 0x41200000u, 0x42dc0000u, 0x42dc0000u },
	{ "a lower sum leaves the word", 0x42dc0000u, 0x42480000u, 0x41200000u, 0x42dc0000u, 0x42dc0000u },
	{ "an equal sum leaves the word", 0x42dc0000u, 0x42c80000u, 0x41200000u, 0x42dc0000u, 0x42dc0000u },
	{ "a higher sum is stored", 0x42dc0000u, 0x43480000u, 0x40a00000u, 0x434d0000u, 0x434d0000u },
	{ "a negative sum over zero stays zero (the floor is the zero word, not minus infinity)", 0x00000000u, 0xc2480000u, 0x41200000u, 0x00000000u, 0x00000000u },
	{ "a negative sum over a stored word leaves it", 0x42dc0000u, 0xc2480000u, 0x41200000u, 0x42dc0000u, 0x42dc0000u },
	{ "positive zero plus positive zero is not greater than zero", 0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u, 0x00000000u },
	{ "negative zero plus negative zero never stores the negative zero word", 0x00000000u, 0x80000000u, 0x80000000u, 0x00000000u, 0x00000000u },
	{ "negative zero plus positive zero is positive zero and is not greater", 0x00000000u, 0x80000000u, 0x00000000u, 0x00000000u, 0x00000000u },
	{ "the smallest subnormal is greater than zero and is stored exactly", 0x00000000u, 0x00000001u, 0x00000000u, 0x00000001u, 0x00000001u },
	{ "the largest subnormal plus the smallest subnormal is the smallest normal, exactly", 0x00000000u, 0x007fffffu, 0x00000001u, 0x00800000u, 0x00800000u },
	{ "a subnormal under a stored larger word leaves it", 0x3f800000u, 0x00000001u, 0x00000000u, 0x3f800000u, 0x3f800000u },
	{ "the largest finite plus zero is stored exactly", 0x00000000u, 0x7f7fffffu, 0x00000000u, 0x7f7fffffu, 0x7f7fffffu },
	{ "an infinite position plus a finite top is stored as infinity", 0x00000000u, 0x7f800000u, 0x41200000u, 0x7f800000u, 0x7f800000u },
	{ "a negative infinite position is not greater", 0x00000000u, 0xff800000u, 0x41200000u, 0x00000000u, 0x00000000u },
	{ "a quiet NaN position is not greater and the stored word stays", 0x42dc0000u, 0x7fc00000u, 0x41200000u, 0x42dc0000u, 0x42dc0000u },
	{ "a quiet NaN position over zero stays zero", 0x00000000u, 0x7fc00000u, 0x41200000u, 0x00000000u, 0x00000000u },
	{ "a stored infinity is never exceeded", 0x7f800000u, 0x7f7fffffu, 0x00000000u, 0x7f800000u, 0x7f800000u },
};

/* sums that are NOT exact: the stored word is the sum rounded by the mode in force. The engine entry runs toward zero. */
static const struct domain_case rounded_cases [] =
{
	/* 1 + 1.5 * 2^-24 is three quarters of an ulp: nearest rounds up, toward zero down. Over a stored one the same sum is stored only where it rounds above it (x87 compares the exact sum, the
	   store rounds it again: the word is the same either way) */
	{ "three quarters of an ulp over one", 0x00000000u, 0x3f800000u, 0x33c00000u, 0x3f800001u, 0x3f800000u },
	{ "the same sum over a stored one", 0x3f800000u, 0x3f800000u, 0x33c00000u, 0x3f800001u, 0x3f800000u },
	/* the largest finite plus itself overflows: nearest gives infinity, toward zero the largest finite */
	{ "an overflowing sum", 0x00000000u, 0x7f7fffffu, 0x7f7fffffu, 0x7f800000u, 0x7f7fffffu },
};

#define EXACT_COUNT ((int) (sizeof (exact_cases) / sizeof (exact_cases [0])))
#define ROUNDED_COUNT ((int) (sizeof (rounded_cases) / sizeof (rounded_cases [0])))

static void run_cases (struct check *c, const struct domain_case *cases, int count, int mode)
{
	int
		i,
		previous;

	previous = fegetround ();

	fesetround (mode);

	for (i = 0; i < count; i++)
	{
		uint32_t
			after;

		after = arrival (c, cases [i].before, cases [i].y, cases [i].ymax, cases [i].label);

		expect_word (c, after, (mode == FE_TOWARDZERO) ? cases [i].after_toward_zero : cases [i].after_to_nearest, cases [i].label, __LINE__);

#ifndef EECH_CAMPAIGN_HOST_ENTITY
		/* the DIFFERENTIAL: the host's raise expression (sector_height_raise.h), run here under the same rounding mode over the same words, leaves exactly the word the original response left */
		{
			float
				position_y,
				box_ymax,
				tallest;

			uint32_t
				host_word;

			position_y = bits_float (cases [i].y);
			box_ymax = bits_float (cases [i].ymax);
			tallest = bits_float (cases [i].before);

			sector_height_raise_body (&position_y, &box_ymax, &tallest);

			host_word = float_bits (tallest);

			expect_word (c, host_word, after, "the host's raise leaves the word the original response left", __LINE__);
		}
#endif
	}

	fesetround (previous);
}

/* the high-water sequence, the removal and the other senders (all with exact sums) */
static void run_sequence (struct check *c)
{
	entity
		*sector_a,
		*sector_b,
		*low,
		*mid,
		*high;

	sector_a = &sector_entities [0];

	sector_b = &sector_entities [1];

	low = &fixed_entities [1];

	mid = &fixed_entities [2];

	high = &fixed_entities [3];

	store_word (sector_a, 0);

	store_word (sector_b, 0);

	make_fixed (low, float_bits (50.0f), scene_for (float_bits (10.0f)));

	make_fixed (mid, float_bits (100.0f), scene_for (float_bits (10.0f)));

	make_fixed (high, float_bits (200.0f), scene_for (float_bits (5.0f)));

	/* the first arrival raises the empty sector from the zero word */
	EXPECT_WORD (c, stored_word (sector_a), 0u, "a sector starts at the all-zero word");
	EXPECT (c, link_child (sector_a, mid) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (110.0f), "first arrival");

	/* lower, equal (the same structure listed again) and higher */
	EXPECT (c, link_child (sector_a, low) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (110.0f), "lower arrival");
	EXPECT (c, link_child (sector_a, mid) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (110.0f), "equal arrival");
	EXPECT (c, link_child (sector_a, high) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (205.0f), "higher arrival");

	/* removal is not a recomputation: neither the highest member's nor the last member's removal lowers it, and a sector that never held it is not touched */
	EXPECT (c, unlink_child (sector_a, high) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (205.0f), "the highest member removed: still the high-water");
	EXPECT (c, unlink_child (sector_a, mid) == TRUE);
	EXPECT (c, unlink_child (sector_a, low) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (205.0f), "every member removed: still the high-water");
	EXPECT_WORD (c, stored_word (sector_b), 0u, "another sector is independent");

	/* reinsertion of a lower structure after the removal leaves it, the same highest structure leaves it, a taller shape raises it */
	EXPECT (c, link_child (sector_a, low) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (205.0f), "a lower reinsertion after a removal");
	EXPECT (c, link_child (sector_a, high) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (205.0f), "the same highest structure reinserted");
	set_scene (high, scene_for (float_bits (6.0f)));
	EXPECT (c, link_child (sector_a, high) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), float_bits (206.0f), "the same entity with a taller shape raises it (the shape is read at arrival)");

	/* the same structure moved to another sector (fx_vec3d.c: a removal, then an insertion into the new list): the new sector is raised, the old one is not lowered */
	EXPECT (c, unlink_child (sector_a, high) == TRUE);
	EXPECT (c, link_child (sector_b, high) == TRUE);
	EXPECT_WORD (c, stored_word (sector_b), float_bits (206.0f), "the sector a structure moves into is raised");
	EXPECT_WORD (c, stored_word (sector_a), float_bits (206.0f), "the sector it left is not lowered");

	free_fixed (low);
	free_fixed (mid);
	free_fixed (high);

	/* a fixed entity with the invalid shape never reads the bounding box: the host authors scene 0 as zeros, so a read would store the position (10) */
	store_word (sector_a, 0);

	make_fixed (low, float_bits (10.0f), OBJECT_3D_INVALID_OBJECT_INDEX);

	EXPECT (c, link_child (sector_a, low) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), 0u, "an invalid shape raises nothing");

	free_fixed (low);

	/* a sender that is not fixed (the default IDENTIFY_FIXED, no position read: its data is NULL) raises nothing */
	memset (&mobile_entity, 0, sizeof (entity));

	set_local_entity_type (&mobile_entity, ENTITY_TYPE_FIXED_WING);

	EXPECT (c, link_child (sector_a, &mobile_entity) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), 0u, "a sender that is not fixed raises nothing");
	EXPECT (c, unlink_child (sector_a, &mobile_entity) == TRUE);
	EXPECT_WORD (c, stored_word (sector_a), 0u, "its removal changes nothing");
}

/* the words the save must carry bit for bit: zero, negative zero, the subnormal edges, the normal edges, both extremes, both infinities and a quiet NaN */
static const uint32_t save_words [] =
{
	0x00000000u, 0x80000000u, 0x00000001u, 0x007fffffu, 0x00800000u, 0x3f800000u, 0x42dc0000u, 0x461c4000u, 0x7f7fffffu, 0xff7fffffu, 0x7f800000u, 0xff800000u, 0x7fc00000u,
};

#define SAVE_WORDS ((int) (sizeof (save_words) / sizeof (save_words [0])))

static void set_map (int x_count, int z_count)
{
	int
		i;

	memset (&world_map, 0, sizeof (world_map));

	world_map.sector_side_length = 8192;
	world_map.sector_side_length_mask = 8191;
	world_map.num_map_x_sectors = x_count;
	world_map.num_map_z_sectors = z_count;
	world_map.num_map_sectors = x_count * z_count;
	world_map.min_map_x_sector = 0;
	world_map.min_map_z_sector = 0;
	world_map.max_map_x_sector = x_count - 1;
	world_map.max_map_z_sector = z_count - 1;

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		sector_map [i] = &sector_entities [i];

		if (get_local_entity_data (&sector_entities [i]) != NULL)
		{
			((sector *) get_local_entity_data (&sector_entities [i]))->x_sector = (unsigned) (i % x_count);
			((sector *) get_local_entity_data (&sector_entities [i]))->z_sector = (unsigned) (i / x_count);
		}
	}

	entity_sector_map = sector_map;
}

/* packs the map in `mode`; returns the bit size of the record (-1 on an overflow), the bytes are in `stream` */
static int pack_map (unsigned char *stream, pack_modes mode)
{
	int
		bits;

	memset (stream, 0, STREAM_BYTES);

	open_pack_buffer (stream, STREAM_BYTES);

	pack_local_sector_data (mode);

	bits = get_pack_buffer_overflow () ? -1 : get_current_pack_buffer_bit_size ();

	close_pack_buffer ();

	return bits;
}

static void unpack_map (unsigned char *stream, pack_modes mode)
{
	open_unpack_buffer (stream, STREAM_BYTES);

	unpack_local_sector_data (mode);

	close_unpack_buffer ();
}

/* the last 32 bits of a record, decoded with the engine's own unpack primitive. The primitive consumes the bytes it reads (it shifts them in place), so it reads a copy. */
static uint32_t last_word (const unsigned char *stream, int bits)
{
	unsigned char
		copy [STREAM_BYTES];

	uint32_t
		word;

	int
		i;

	memcpy (copy, stream, STREAM_BYTES);

	open_unpack_buffer (copy, STREAM_BYTES);

	for (i = 0; i < bits - 32; i++)
	{
		unpack_unsigned_data (1);
	}

	word = unpack_unsigned_data (32);

	close_unpack_buffer ();

	return word;
}

static void run_save (struct check *c)
{
	unsigned char
		stream [STREAM_BYTES],
		other [STREAM_BYTES];

	int
		i,
		bits,
		first_bits,
		client_bits;

	/* one sector: the word is the last 32 raw bits of the record, whatever the word, and the record has the same size for every word */
	set_map (1, 1);

	first_bits = 0;

	for (i = 0; i < SAVE_WORDS; i++)
	{
		store_word (&sector_entities [0], save_words [i]);

		bits = pack_map (stream, PACK_MODE_SERVER_SESSION);

		if (i == 0)
		{
			first_bits = bits;
		}

		EXPECT (c, bits > 32 && bits == first_bits);
		EXPECT_WORD (c, last_word (stream, bits), save_words [i], "the packed record ends with the stored word");

		store_word (&sector_entities [0], ~save_words [i]);

		unpack_map (stream, PACK_MODE_SERVER_SESSION);
		EXPECT_WORD (c, stored_word (&sector_entities [0]), save_words [i], "the unpack restores the word bit for bit");
	}

	/* a client session and a browse session carry no tallest height: a client keeps what its own insertions made, and the record is shorter by at least the word */
	store_word (&sector_entities [0], 0x42dc0000u);

	client_bits = pack_map (other, PACK_MODE_CLIENT_SESSION);
	EXPECT (c, client_bits > 0 && client_bits + 32 <= first_bits);

	store_word (&sector_entities [0], 0x43520000u);

	unpack_map (other, PACK_MODE_CLIENT_SESSION);
	EXPECT_WORD (c, stored_word (&sector_entities [0]), 0x43520000u, "a client session restores nothing");

	bits = pack_map (other, PACK_MODE_BROWSE_SESSION);
	EXPECT (c, bits == 0);

	/* four sectors: each record has its own word and each word lands in its own sector */
	set_map (2, 2);

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		store_word (&sector_entities [i], save_words [(i * 3 + 1) % SAVE_WORDS]);
	}

	bits = pack_map (stream, PACK_MODE_SERVER_SESSION);
	EXPECT (c, bits > 0);

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		store_word (&sector_entities [i], 0x12345678u + (uint32_t) i);
	}

	unpack_map (stream, PACK_MODE_SERVER_SESSION);

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		EXPECT_WORD (c, stored_word (&sector_entities [i]), save_words [(i * 3 + 1) % SAVE_WORDS], "each sector restores its own word");
	}

	/* the getter serves the stored word and only it (a NaN is compared by class: an x87 return may quieten a payload) */
	for (i = 0; i < SAVE_WORDS; i++)
	{
		store_word (&sector_entities [0], save_words [i]);

		if (isnan (bits_float (save_words [i])))
		{
			EXPECT (c, isnan (get_local_entity_float_value (&sector_entities [0], FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT)));
		}
		else
		{
			EXPECT_WORD (c, served_word (&sector_entities [0]), save_words [i], "the getter serves the stored word");
		}
	}
}

#ifdef EECH_CAMPAIGN_HOST_ENTITY

/* ---- the HOSTED profile (EECH_CAMPAIGN_HOST_ENTITY, Task49): the sector's height is the HOST's word. The campaign's response raises nothing and writes nothing, the getter and the pack read the host's word and the restore states it ---- */

static const uint32_t hosted_words [] =
{
	0x00000000u, 0x80000000u, 0x00000001u, 0x007fffffu, 0x00800000u, 0x3f800000u, 0x42dc0000u, 0x461c4000u, 0x7f7fffffu, 0xff7fffffu, 0x7f800000u, 0xff800000u, 0x7fc00000u,
	/* raw words a restore makes valid history: a negative word and a quiet NaN with a payload */
	0xbf800000u, 0xc2dc0000u, 0x7fc0f00du
};

#define HOSTED_WORDS ((int) (sizeof (hosted_words) / sizeof (hosted_words [0])))

#define LEGACY_CANARY 0xc0ffee11u

static void hosted_canaries (void)
{
	int
		i;

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		store_word (&sector_entities [i], LEGACY_CANARY + (uint32_t) i);
	}
}

static int hosted_legacy_untouched (void)
{
	int
		i;

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		if (stored_word (&sector_entities [i]) != LEGACY_CANARY + (uint32_t) i)
		{
			return FALSE;
		}
	}

	return TRUE;
}

static void run_hosted (struct check *c)
{
	unsigned char
		stream [STREAM_BYTES],
		other [STREAM_BYTES];

	int
		i,
		bits,
		first_bits;

	set_map (2, 2);

	/* the campaign's LINK_CHILD response raises nothing and writes nothing, whatever arrives: every domain case leaves the legacy word as it was (the host raises it, at the arrival's state statement) */
	for (i = 0; i < EXACT_COUNT; i++)
	{
		uint32_t
			after;

		after = arrival (c, exact_cases [i].before, exact_cases [i].y, exact_cases [i].ymax, exact_cases [i].label);

		expect_word (c, after, exact_cases [i].before, "the hosted link response writes nothing", __LINE__);
	}

	for (i = 0; i < ROUNDED_COUNT; i++)
	{
		uint32_t
			after;

		after = arrival (c, rounded_cases [i].before, rounded_cases [i].y, rounded_cases [i].ymax, rounded_cases [i].label);

		expect_word (c, after, rounded_cases [i].before, "the hosted link response writes nothing", __LINE__);
	}

	/* the getter is the host's word, for every word, for each sector by its own coordinates: the legacy field (a canary here) is never read */
	hosted_canaries ();

	for (i = 0; i < HOSTED_WORDS; i++)
	{
		int
			sector_index,
			before_asks;

		sector_index = i % SECTOR_COUNT;
		host_height_words [sector_index] = hosted_words [i];
		before_asks = host_height_asks;

		if (isnan (bits_float (hosted_words [i])))
		{
			EXPECT (c, isnan (get_local_entity_float_value (&sector_entities [sector_index], FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT)));
		}
		else
		{
			EXPECT_WORD (c, served_word (&sector_entities [sector_index]), hosted_words [i], "the getter serves the host's word");
		}

		EXPECT (c, host_height_asks == before_asks + 1 && host_height_log_phase [before_asks] == CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT);
		EXPECT (c, host_height_log_x [before_asks] == sector_index % 2 && host_height_log_z [before_asks] == sector_index / 2);
	}

	EXPECT (c, hosted_legacy_untouched ());

	/* the pack carries the host's word as the last 32 raw bits of each sector's record, whatever the word, and the record keeps its size */
	set_map (1, 1);
	hosted_canaries ();
	first_bits = 0;

	for (i = 0; i < HOSTED_WORDS; i++)
	{
		host_height_words [0] = hosted_words [i];

		bits = pack_map (stream, PACK_MODE_SERVER_SESSION);

		if (i == 0)
		{
			first_bits = bits;
		}

		EXPECT (c, bits > 32 && bits == first_bits);
		EXPECT_WORD (c, last_word (stream, bits), hosted_words [i], "the packed record ends with the host's word");
	}

	/* a client session and a browse session carry no height and ask the host for none */
	{
		int
			asks;

		asks = host_height_asks;

		EXPECT (c, pack_map (other, PACK_MODE_CLIENT_SESSION) > 0);
		EXPECT (c, pack_map (other, PACK_MODE_BROWSE_SESSION) == 0);
		EXPECT (c, host_height_asks == asks);
	}

	/* four sectors: each record carries its own host word, in the original's order, and the unpack states each word to ITS sector (a restore for each sector, the word RAW) and writes the legacy field nowhere */
	set_map (2, 2);

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		host_height_words [i] = hosted_words [(i * 3 + 1) % HOSTED_WORDS];
	}

	bits = pack_map (stream, PACK_MODE_SERVER_SESSION);
	EXPECT (c, bits > 0);

	hosted_canaries ();

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		host_height_words [i] = 0x12345678u + (uint32_t) i;
	}

	{
		int
			before_asks;

		before_asks = host_height_asks;

		unpack_map (stream, PACK_MODE_SERVER_SESSION);

		EXPECT (c, host_height_asks == before_asks + SECTOR_COUNT);

		for (i = 0; i < SECTOR_COUNT; i++)
		{
			EXPECT (c, host_height_log_phase [before_asks + i] == CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE);
			EXPECT (c, host_height_log_x [before_asks + i] == i % 2 && host_height_log_z [before_asks + i] == i / 2);
			EXPECT_WORD (c, host_height_log_word [before_asks + i], hosted_words [(i * 3 + 1) % HOSTED_WORDS], "the unpack states each sector's saved word to the host");
			EXPECT_WORD (c, host_height_words [i], hosted_words [(i * 3 + 1) % HOSTED_WORDS], "the host holds exactly the saved word");
		}
	}

	EXPECT (c, hosted_legacy_untouched ());

	/* a client session restores nothing and states nothing */
	{
		int
			asks;

		asks = host_height_asks;

		unpack_map (other, PACK_MODE_CLIENT_SESSION);

		EXPECT (c, host_height_asks == asks);
	}
}

#endif

/* every dispatch table the initialisers write, saved whole and copied back */
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
	TABLE (message_responses),
};

#define SAVED_TABLES ((int) (sizeof (saved_tables) / sizeof (saved_tables [0])))

static void save_tables (void)
{
	int
		i;

	for (i = 0; i < SAVED_TABLES; i++)
	{
		saved_tables [i].copy = malloc (saved_tables [i].size);

		memcpy (saved_tables [i].copy, saved_tables [i].table, saved_tables [i].size);
	}
}

static void restore_tables (void)
{
	int
		i;

	for (i = 0; i < SAVED_TABLES; i++)
	{
		memcpy (saved_tables [i].table, saved_tables [i].copy, saved_tables [i].size);

		free (saved_tables [i].copy);

		saved_tables [i].copy = NULL;
	}
}

int eech_sector_height_check_run (char *text, int capacity, int *checks)
{
	struct check
		c;

	campaign_model_host
		host;

	world_map_data
		saved_map;

	entity
		**saved_sector_map;

	pack_buffer
		*saved_pack_buffer;

	unpack_buffer
		*saved_unpack_buffer;

	int
		i,
		saved_rounding;

	c.text = text;
	c.capacity = capacity;
	c.used = 0;
	c.failures = 0;
	c.checks = 0;
	text [0] = 0;

	saved_rounding = fegetround ();
	saved_map = world_map;
	saved_sector_map = entity_sector_map;
	saved_pack_buffer = current_pack_buffer;
	saved_unpack_buffer = current_unpack_buffer;

	save_tables ();

	/* the engine's own table initialisation, for the types this control sends to */
	initialise_entity_int_value_default_functions ();
	initialise_entity_float_value_default_functions ();
	initialise_entity_vec3d_default_functions ();
	overload_fixed_int_value_functions (ENTITY_TYPE_OBJECT);
	overload_fixed_vec3d_functions (ENTITY_TYPE_OBJECT);
	overload_sector_float_value_functions ();
	overload_sector_message_responses ();

	/* every top the cases use becomes one host-authored scene, then the real install asks the scripted provider once */
	for (i = 0; i < EXACT_COUNT; i++)
	{
		scene_for (exact_cases [i].ymax);
	}

	for (i = 0; i < ROUNDED_COUNT; i++)
	{
		scene_for (rounded_cases [i].ymax);
	}

	scene_for (float_bits (10.0f));
	scene_for (float_bits (5.0f));
	scene_for (float_bits (6.0f));

	memset (&host, 0, sizeof (host));

	host.model = scripted_model;

	campaign_model_set_host (&host);

	campaign_model_install_scenes ();

	scenes_installed = TRUE;

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		make_sector (&sector_entities [i]);
	}

	set_map (2, 2);

#ifdef EECH_CAMPAIGN_HOST_ENTITY

	/* the hosted profile: the word is the host's, and the original's raise, getter and pack lines are compiled out (see the header of this file) */
	run_hosted (&c);

#else

	/* the engine entry runs toward zero: that is the profile of record; the other mode shows what the rounding changes */
	run_cases (&c, exact_cases, EXACT_COUNT, FE_TOWARDZERO);
	run_cases (&c, exact_cases, EXACT_COUNT, FE_TONEAREST);
	run_cases (&c, rounded_cases, ROUNDED_COUNT, FE_TOWARDZERO);
	run_cases (&c, rounded_cases, ROUNDED_COUNT, FE_TONEAREST);

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		store_word (&sector_entities [i], 0);
	}

	run_sequence (&c);

	run_save (&c);

#endif

	EXPECT (&c, scene_lookup_failures == 0);

	for (i = 0; i < SECTOR_COUNT; i++)
	{
		free_sector (&sector_entities [i]);
	}

	/* the control's own state is given back (the installed scenes cannot be: see the header) */
	EXPECT (&c, current_pack_buffer == saved_pack_buffer);
	EXPECT (&c, current_unpack_buffer == saved_unpack_buffer);
	EXPECT (&c, fegetround () == saved_rounding);

	fesetround (saved_rounding);
	world_map = saved_map;
	entity_sector_map = saved_sector_map;

	restore_tables ();

	*checks = c.checks;

	return c.failures;
}

#endif

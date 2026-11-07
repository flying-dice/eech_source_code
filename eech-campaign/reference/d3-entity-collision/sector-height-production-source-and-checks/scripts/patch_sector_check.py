"""The real-C sector height control: the hosted profile's cases (the Core raises nothing, every reader and the pack use the host's word, the restore states it to the host), and in the original-solver profile
the unchanged 179 characterisation plus a differential of the host's raise expression against the original response over the same inputs."""
import sys

ROOT = sys.argv[1]
P = ROOT + "/eech-campaign/crates/eech-engine-sys/csrc/eech_sector_height_check.c"
raw = open(P, "rb").read()
assert b"\r\n" not in raw
d = raw.decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep("""#include "entity/mobile/mb_hostmodel.h"
""", """#include "entity/mobile/mb_hostmodel.h"

/* the host's raise expression (eech-world model_step.c, `eech_model_sector_height_raise`): the same text, run here beside the original response over the same inputs */
#include "../../eech-world/csrc/sector_height_raise.h"
""")

# the scripted provider answers the host's sector height question and restore (inert where the campaign does not ask)
rep("""static int scripted_model (const campaign_model_request *request, campaign_model_reply *reply, char *error, size_t error_size, void *user)
{
	(void) user;
""", """/* the host's words for the sectors of the 2 x 2 map (index x + 2 * z), the questions it was asked (the phase, the sector, and for a restore the word stated) and how many */
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
""")

# sector coordinates, so that the hosted getter and the pack ask for the sector they stand in
rep("""	for (i = 0; i < SECTOR_COUNT; i++)
	{
		sector_map [i] = &sector_entities [i];
	}

	entity_sector_map = sector_map;
}""", """	for (i = 0; i < SECTOR_COUNT; i++)
	{
		sector_map [i] = &sector_entities [i];

		if (get_local_entity_data (&sector_entities [i]) != NULL)
		{
			((sector *) get_local_entity_data (&sector_entities [i]))->x_sector = (unsigned) (i % x_count);
			((sector *) get_local_entity_data (&sector_entities [i]))->z_sector = (unsigned) (i / x_count);
		}
	}

	entity_sector_map = sector_map;
}""")

# the differential, in the profile that has the original response
rep("""		after = arrival (c, cases [i].before, cases [i].y, cases [i].ymax, cases [i].label);

		expect_word (c, after, (mode == FE_TOWARDZERO) ? cases [i].after_toward_zero : cases [i].after_to_nearest, cases [i].label, __LINE__);
	}
""", """		after = arrival (c, cases [i].before, cases [i].y, cases [i].ymax, cases [i].label);

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
""")

# the hosted cases and the profile switch
rep("""/* every dispatch table the initialisers write, saved whole and copied back */""", """#ifdef EECH_CAMPAIGN_HOST_ENTITY

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

/* every dispatch table the initialisers write, saved whole and copied back */""")

rep("""	/* the engine entry runs toward zero: that is the profile of record; the other mode shows what the rounding changes */
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
""", """#ifdef EECH_CAMPAIGN_HOST_ENTITY

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
""")

rep(""" * eech_sector_height_check_run returns the number of failed expectations""", """ * TWO PROFILES (Task49, the host-owned sector height). The ORIGINAL body of the response, the getter and the pack is compiled in every build that keeps the original entity solver (feature `collision-fixture`): there this control is the
 * accepted characterisation above, unchanged (the same scenarios, the same expectations), plus a DIFFERENTIAL of the host's raise expression (eech-world csrc/sector_height_raise.h, the same text its kernel runs) against the original response over
 * every domain case, under both rounding modes. In the hosted profile (the default: EECH_CAMPAIGN_HOST_ENTITY) the height is the HOST's word: the campaign's response raises and writes nothing, the getter and the pack read the host's word (a
 * scripted provider answers the sector height question), and the unpack states each saved word to the host raw; the hosted cases pin exactly that, and the legacy field (a canary) is never read nor written.
 *
 * eech_sector_height_check_run returns the number of failed expectations""")

open(P, "wb").write(d.encode("utf-8"))
print("ok")

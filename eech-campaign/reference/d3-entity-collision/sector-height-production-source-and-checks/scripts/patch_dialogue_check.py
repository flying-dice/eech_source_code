"""The isolated control of the campaign's half of the host dialogue (feature entity-dialogue-check): the sector height read and restore, and the city state's new shape word."""
import sys

ROOT = sys.argv[1]
P = ROOT + "/eech-campaign/crates/eech-engine-sys/csrc/eech_entity_dialogue_check.c"
raw = open(P, "rb").read()
assert b"\r\n" not in raw
d = raw.decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep("""static campaign_model_request expand_seen;
""", """static campaign_model_request expand_seen;
/* the SECTOR HEIGHT read and restore (phases 17 and 18, Task49): the word the scripted host holds (a restore replaces it with the word stated), the question the campaign asked last and how many, and an answer forced instead of the
   coherent one (the state, the name, the count and the words) when `height_override` is set. The phase numbers are written out, like the others, so that the control compiles against a build that has none */
#define CHECK_PHASE_SECTOR_HEIGHT 17
#define CHECK_PHASE_SECTOR_RESTORE 18
typedef char check_phase_height_is_the_contracts [((CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT == CHECK_PHASE_SECTOR_HEIGHT) && (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE == CHECK_PHASE_SECTOR_RESTORE) &&
	(CAMPAIGN_SECTOR_HEIGHT_WORD == 0) && (CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS == 3)) ? 1 : -1];
static int height_asks, height_override, height_state, height_count;
static const char *height_name;
static uint32_t height_held, height_words[CAMPAIGN_MODEL_CANDIDATE_POSE_MAX];
static campaign_model_request height_seen;
""")

rep("""	if ((request->op == CAMPAIGN_MODEL_ENTITY_SCAN) && ((int) request->pose[CAMPAIGN_SCAN_PHASE] == CHECK_PHASE_FIXED_STATE))
	{
		fixed_seen = *request;
""", """	if ((request->op == CAMPAIGN_MODEL_ENTITY_SCAN) && (((int) request->pose[CAMPAIGN_SCAN_PHASE] == CHECK_PHASE_SECTOR_HEIGHT) || ((int) request->pose[CAMPAIGN_SCAN_PHASE] == CHECK_PHASE_SECTOR_RESTORE)))
	{
		const int restore = ((int) request->pose[CAMPAIGN_SCAN_PHASE] == CHECK_PHASE_SECTOR_RESTORE);
		uint32_t coherent[3];

		height_seen = *request;
		height_asks++;

		if (refuse_the_exchange)
		{
			debug_fatal ("host model: [model-provider:callback] the scripted host refused");
		}

		if (restore)
		{
			memcpy (&height_held, &request->pose[CAMPAIGN_SECTOR_HEIGHT_WORD], sizeof (height_held));
		}

		memcpy (&coherent[0], &request->pose[CAMPAIGN_SCAN_CURSOR], 2 * sizeof (uint32_t));
		coherent[2] = height_held;

		reply->candidate_state = height_override ? height_state : (restore ? CAMPAIGN_MODEL_SCAN_RESTORED : CAMPAIGN_MODEL_SCAN_HEIGHT);
		strncpy (reply->name, height_override ? height_name : "", sizeof (reply->name) - 1);
		reply->candidate_count = height_override ? height_count : CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS;

		if (height_override)
		{
			memcpy (reply->candidate, height_words, sizeof (reply->candidate));
		}
		else
		{
			memcpy (reply->candidate, coherent, sizeof (coherent));
		}

		return;
	}

	if ((request->op == CAMPAIGN_MODEL_ENTITY_SCAN) && ((int) request->pose[CAMPAIGN_SCAN_PHASE] == CHECK_PHASE_FIXED_STATE))
	{
		fixed_seen = *request;
""")

rep("""	(CAMPAIGN_CITY_STATE_WORDS == 10)) ? 1 : -1];
""", """	(CAMPAIGN_CITY_STATE_WORDS == 11) && (CAMPAIGN_CITY_SHAPE == 9)) ? 1 : -1];
""")

rep("""memcpy ((OUT) + 6, (E)->box_max, 3 * sizeof (float)); (OUT)[9] = (E)->sub_type; } while (0)""",
    """memcpy ((OUT) + 6, (E)->box_max, 3 * sizeof (float)); (OUT)[9] = (E)->sub_type; (OUT)[10] = (E)->shape; } while (0)""")

rep("""static void city_expand_cases (struct tc *t)
{""", """/* ---- the SECTOR HEIGHT read and restore (phases 17 and 18, Task49): the campaign asks the host for a sector's word by its coordinates and states a restored word to it, and checks every answer completely ---- */

/* one exchange under a stop catcher: TRUE when the engine stopped. `stated` is a restore's word (NULL for a read); `out` receives the word the campaign took */
static int height_do (int phase, int x_sector, int z_sector, const uint32_t *stated, uint32_t *out)
{
	float word_out, word_stated;

	height_asks = 0;
	fatal_text[0] = '\\0';

	if (setjmp (fatal_jump) == 0)
	{
		if (stated != NULL)
		{
			memcpy (&word_stated, stated, sizeof (word_stated));
		}

		memcpy (&word_out, out, sizeof (word_out));
		campaign_entity_sector_height_exchange (phase, x_sector, z_sector, (stated != NULL) ? &word_stated : NULL, &word_out);
		memcpy (out, &word_out, sizeof (word_out));

		return (FALSE);
	}

	return (TRUE);
}

/* the question the campaign sent: the phase, the sector in the cursor facts, a restore's word in fact 0 and nothing else, with no identity place */
static int height_request_is (const campaign_model_request *q, int phase, int x_sector, int z_sector, const uint32_t *stated)
{
	int index;

	if ((q->op != CAMPAIGN_MODEL_ENTITY_SCAN) || (q->scene != 0) || (q->pose_count != CAMPAIGN_MODEL_SCAN_FACTS) || (q->host_id[0] != 0))
	{
		return (FALSE);
	}

	for (index = 0; index < CAMPAIGN_MODEL_SCAN_FACTS; index++)
	{
		uint32_t word, wanted = 0;
		const float wanted_float = (index == CAMPAIGN_SCAN_PHASE) ? (float) phase : (index == CAMPAIGN_SCAN_CURSOR) ? (float) x_sector : (index == CAMPAIGN_SCAN_CURSOR + 1) ? (float) z_sector : 0.0f;

		memcpy (&word, &q->pose[index], sizeof (word));

		if ((index == CAMPAIGN_SCAN_PHASE) || (index == CAMPAIGN_SCAN_CURSOR) || (index == CAMPAIGN_SCAN_CURSOR + 1))
		{
			memcpy (&wanted, &wanted_float, sizeof (wanted));
		}
		else if ((index == CAMPAIGN_SECTOR_HEIGHT_WORD) && (stated != NULL))
		{
			wanted = *stated;
		}

		if (word != wanted)
		{
			return (FALSE);
		}
	}

	return (TRUE);
}

static void height_script (uint32_t held)
{
	height_held = held;
	height_override = FALSE;
	height_state = CAMPAIGN_MODEL_SCAN_HEIGHT;
	height_count = CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS;
	height_name = "";
	memset (height_words, 0, sizeof (height_words));
}

/* a forced answer: `state`, no name, `count` words (the sector 5 and 9 as asked, then `word`), and `extra` in the first word beyond the three */
static void height_force (int state, int count, uint32_t x_word, uint32_t z_word, uint32_t word, uint32_t extra)
{
	height_override = TRUE;
	height_state = state;
	height_count = count;
	height_name = "";
	memset (height_words, 0, sizeof (height_words));
	height_words[0] = x_word;
	height_words[1] = z_word;
	height_words[2] = word;
	height_words[3] = extra;
}

static void sector_height_cases (struct tc *t)
{
	/* the words a sector's height can be: the thirteen the original control measured through pack and unpack, and the raw words a restore makes valid history (a negative word, both infinities, a quiet NaN with a payload) */
	static const uint32_t WORDS[] = {
		0x00000000u, 0x80000000u, 0x00000001u, 0x007fffffu, 0x00800000u, 0x3f800000u, 0x42dc0000u, 0x461c4000u, 0x7f7fffffu, 0xff7fffffu, 0x7f800000u, 0xff800000u, 0x7fc00000u,
		0xbf800000u, 0xc2dc0000u, 0x7fc0f00du, 0xffc00001u
	};
	static const uint32_t X5 = 0x40a00000u, Z9 = 0x41100000u;
	uint32_t out, canary, before;
	size_t index;
	int stopped;

	/* a read of every word the host can hold crosses bit for bit, and the question states nothing but the phase and the sector (x in fact 12, z in fact 13) */
	for (index = 0; index < sizeof (WORDS) / sizeof (WORDS[0]); index++)
	{
		height_script (WORDS[index]);
		out = 0x13572468u;
		stopped = height_do (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, 5, 9, NULL, &out);
		EXPECT (t, !stopped && height_asks == 1 && out == WORDS[index] && height_request_is (&height_seen, CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, 5, 9, NULL));
	}

	/* a restore states the word RAW in fact 0, and the host's stored word comes back as stated, for every word */
	for (index = 0; index < sizeof (WORDS) / sizeof (WORDS[0]); index++)
	{
		height_script (0x3f000000u);
		out = 0x13572468u;
		stopped = height_do (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, 5, 9, &WORDS[index], &out);
		EXPECT (t, !stopped && height_asks == 1 && out == WORDS[index] && height_held == WORDS[index] && height_request_is (&height_seen, CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, 5, 9, &WORDS[index]));
	}

	/* the largest sector indices cross as whole numbers */
	height_script (0x3f800000u);
	out = 0;
	stopped = height_do (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, 255, 255, NULL, &out);
	EXPECT (t, !stopped && out == 0x3f800000u && height_request_is (&height_seen, CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, 255, 255, NULL));

	/* every refusal is a stop with the host's marker and the caller's word untouched: the wrong state (a read answered as a restore and the reverse, a state of another question), a placement named, a short or long
	   answer, an answer for another sector (either coordinate), a word stored for a restore that is not the word stated, and a word beyond the three */
#define REFUSED(PHASE,STATED,TEXT) \\
	do { \\
		out = canary = 0xa5a5a5a5u; before = out; \\
		stopped = height_do ((PHASE), 5, 9, (STATED), &out); \\
		EXPECT (t, stopped && height_asks == 1 && strstr (fatal_text, "[model-provider:entity_scan.") != NULL && strstr (fatal_text, (TEXT)) != NULL && out == before && canary == before); \\
	} while (0)

	height_script (0x3f800000u);
	height_force (CAMPAIGN_MODEL_SCAN_RESTORED, 3, X5, Z9, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "decided state");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 3, X5, Z9, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, &WORDS[5], "decided state");
	height_force (CAMPAIGN_MODEL_SCAN_PLACED, 3, X5, Z9, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "decided state");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 3, X5, Z9, 0x3f800000u, 0);
	height_name = "12";
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "names none");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 2, X5, Z9, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "answered 2 words");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 4, X5, Z9, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "answered 4 words");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 3, X5 + 1, Z9, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "another sector");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 3, X5, Z9 + 1, 0x3f800000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "another sector");
	height_force (CAMPAIGN_MODEL_SCAN_RESTORED, 3, X5, Z9, 0x3f800001u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, &WORDS[5], "not the word stated");
	/* a restored NaN word that comes back with another payload, a signed zero that comes back as the other zero: not the word stated */
	height_force (CAMPAIGN_MODEL_SCAN_RESTORED, 3, X5, Z9, 0x7fc00001u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, &WORDS[12], "not the word stated");
	height_force (CAMPAIGN_MODEL_SCAN_RESTORED, 3, X5, Z9, 0x00000000u, 0);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, &WORDS[1], "not the word stated");
	height_force (CAMPAIGN_MODEL_SCAN_HEIGHT, 3, X5, Z9, 0x3f800000u, 0x00000001u);
	REFUSED (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, NULL, "more than 3");
	/* a negative sector index is refused before anything is asked */
	out = before = 0xa5a5a5a5u;
	height_script (0x3f800000u);
	stopped = height_do (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, -1, 9, NULL, &out);
	EXPECT (t, stopped && height_asks == 0 && strstr (fatal_text, "index is negative") != NULL && out == before);
	/* a host that refuses the exchange stops the engine with its own reason: nothing falls back to a word the campaign could hold */
	height_script (0x3f800000u);
	refuse_the_exchange = TRUE;
	out = before = 0xa5a5a5a5u;
	stopped = height_do (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, 5, 9, NULL, &out);
	EXPECT (t, stopped && strstr (fatal_text, "the scripted host refused") != NULL && out == before);
	refuse_the_exchange = FALSE;
#undef REFUSED
	height_script (0);
}

static void city_expand_cases (struct tc *t)
{""")

rep("""	city_expand_cases (&t);

	if (checks)""", """	city_expand_cases (&t);
	sector_height_cases (&t);

	if (checks)""")

open(P, "wb").write(d.encode("utf-8"))
print("ok")

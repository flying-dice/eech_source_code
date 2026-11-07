"""Production move of the sector high-water word: the C core (campaign side). Byte-exact, count-checked patches; run with -I from outside the repo."""
import sys

ROOT = sys.argv[1]


def patch(rel, pairs):
    p = ROOT + "/" + rel
    d = open(p, "rb").read()
    assert b"\r\n" not in d, rel + " has CRLF"
    d = d.decode("utf-8")
    for old, new in pairs:
        assert d.count(old) == 1, (rel, old[:80], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))


M = "aphavoc/source/entity/mobile/"

patch(M + "mb_hostmodel.h", [
    ("""	CAMPAIGN_SCAN_PHASE_CITY_RELEASE
};
""", """	CAMPAIGN_SCAN_PHASE_CITY_RELEASE,
	/* the sector's HIGH-WATER HEIGHT (Task49, the host-owned sector height): the campaign asks the host for the word of a sector by its coordinates (every reader of the original's tallest structure height), and states a
	   session's restored word to it (see campaign_model_sector_height and campaign_model_sector_height_restore) */
	CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT,
	CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE
};
"""),
    ("""	CAMPAIGN_MODEL_SCAN_EXPANDED
};
""", """	CAMPAIGN_MODEL_SCAN_EXPANDED,
	/* the answers to the sector height read and restore: the reply's candidate holds the sector's x and z as asked, then the word the host holds (the restored word as stored) */
	CAMPAIGN_MODEL_SCAN_HEIGHT,
	CAMPAIGN_MODEL_SCAN_RESTORED
};
"""),
    ("""#define CAMPAIGN_CITY_SUBTYPE 10
/* the floats of the state the campaign reads from a city (campaign_model_entity_city_state): position (3), box minimum (3), box maximum (3), sub type */
#define CAMPAIGN_CITY_STATE_WORDS 10
""", """#define CAMPAIGN_CITY_SUBTYPE 10
/* the city's SHAPE (a whole scene index; 0, OBJECT_3D_INVALID_OBJECT_INDEX, is what a city is created with): the one fact the host needs of a city to decide the sector's height raise the original makes at the city's arrival
   when the shape is valid. Fact 9 is the fixed state report's withdrawal word and was stated by no city report, so it carries the shape there */
#define CAMPAIGN_CITY_SHAPE 9
/* the floats of the state the campaign reads from a city (campaign_model_entity_city_state): position (3), box minimum (3), box maximum (3), sub type, shape */
#define CAMPAIGN_CITY_STATE_WORDS 11
"""),
    ("""#define CAMPAIGN_CITY_EXPAND_WORDS 6
""", """#define CAMPAIGN_CITY_EXPAND_WORDS 6

/* the SECTOR HEIGHT read and restore (the same question, phases CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT and _SECTOR_RESTORE; Task49): the sector's x and z in the cursor facts (whole numbers), every other fact zero but the phase and,
   for a restore, the saved word RAW in fact 0; the identity place is empty. The reply (state HEIGHT or RESTORED) holds exactly CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS words: the x and z echoed, then the word the host holds. The word is
   NEVER validated as finite, non-negative or non-NaN: a negative, infinite or quiet-NaN word the original saved is valid history and crosses exactly */
#define CAMPAIGN_SECTOR_HEIGHT_WORD 0
#define CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS 3
"""),
    ("""/* ENTITY_SCAN, the CITY_EXPAND question""", """/* ENTITY_SCAN, the sector's HIGH-WATER HEIGHT (phase CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT; Task49): the word the HOST holds for the sector (`x_sector`, `z_sector`), copied into `word` as the 32 bits it holds (never computed with
   here). The campaign holds no word of its own in a hosted build: this is the one read behind the sector's float getter, so the scan, the fixed-wing clearance and the session pack all read the host's. The reply is checked
   completely before the word is written: the state HEIGHT, no placement named, exactly three words, the sector echoed as asked and nothing beyond; a refusal, a missing provider or an answer that does not hold stops the engine by marker, and there is no fallback. */
void campaign_model_sector_height (int x_sector, int z_sector, float *word);

/* ENTITY_SCAN, the session RESTORE of a sector's height (phase CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE; Task49): the saved word (`word`, the 32 raw bits) is stated to the host, which holds exactly these bits afterwards. The reply is
   checked completely: the state RESTORED, the sector echoed as asked and the word stored equal, bit for bit, to the word stated. */
void campaign_model_sector_height_restore (int x_sector, int z_sector, const float *word);

/* ENTITY_SCAN, the CITY_EXPAND question"""),
])

patch(M + "mb_hostentity.h", [
    ("""	request.pose [CAMPAIGN_CITY_SUBTYPE] = state [9];
""", """	request.pose [CAMPAIGN_CITY_SUBTYPE] = state [9];
	request.pose [CAMPAIGN_CITY_SHAPE] = state [10];
"""),
    ("""// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// ENTITY_POSE (EEDCS-49, D3, fifth slice): an OBSERVATION""", """// The SECTOR HEIGHT read and restore (the same question; phases CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT and CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, Task49; see mb_hostmodel.h). The sector is stated by its whole-number coordinates (a
// sector's index pair is never negative), and the host's reply is checked COMPLETELY before one word of the caller's is written: the state the phase allows, no placement named, exactly CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS words
// and none beyond, the sector's x and z echoed bit for bit as asked (an answer for another sector is refused, never used) and, for a restore, the word the host stored equal bit for bit to the word stated. The height word
// itself is read as bytes and is NEVER classified: a negative, infinite or NaN word is valid, and no floating-point value is loaded or compared here.
static void check_sector_height_reply (const campaign_model_reply *reply, int phase, const float *asked, const float *stated)
{
	int
		index,
		wanted;

	wanted = (phase == CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT) ? CAMPAIGN_MODEL_SCAN_HEIGHT : CAMPAIGN_MODEL_SCAN_RESTORED;

	if (reply->candidate_state != wanted)
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a sector height %s, which allows only state %d", reply->candidate_state, (phase == CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT) ? "read" : "restore", wanted);
	}

	check_reply_key (reply, "entity_scan");

	if (reply->name [0] != 0)
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the host named the placement \\"%.40s\\" for a sector height, which names none", reply->name);
	}

	if (reply->candidate_count != CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS)
	{
		debug_fatal ("host model: [model-provider:entity_scan.sector_height] the host answered %d words for a sector height, which carries %d", reply->candidate_count, CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS);
	}

	if (memcmp (&reply->candidate [0], asked, 2 * sizeof (float)) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_scan.sector_height] the host answered a sector height for another sector than the one asked about");
	}

	if ((stated != NULL) && (memcmp (&reply->candidate [2], stated, sizeof (float)) != 0))
	{
		debug_fatal ("host model: [model-provider:entity_scan.sector_height] the host stored a word for the restore of a sector's height that is not the word stated");
	}

	for (index = CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS; index < CAMPAIGN_MODEL_CANDIDATE_POSE_MAX; index++)
	{
		if (!scan_word_is_zero (&reply->candidate [index]))
		{
			debug_fatal ("host model: [model-provider:entity_scan.sector_height] the host answered more than %d words for a sector height", CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS);
		}
	}
}

// `stated` is the saved word of a restore (NULL for a read); `word` receives the 32 bits the host answered, after the whole answer was checked
static void campaign_entity_sector_height_exchange (int phase, int x_sector, int z_sector, const float *stated, float *word)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	float
		asked [2];

	if ((x_sector < 0) || (z_sector < 0))
	{
		debug_fatal ("host model: [model-provider:entity_scan.sector_height] a sector height names the sector (%d, %d), whose index is negative", x_sector, z_sector);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) phase;

	asked [0] = (float) x_sector;
	asked [1] = (float) z_sector;

	memcpy (&request.pose [CAMPAIGN_SCAN_CURSOR], asked, sizeof (asked));

	if (stated != NULL)
	{
		memcpy (&request.pose [CAMPAIGN_SECTOR_HEIGHT_WORD], stated, sizeof (float));
	}

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_sector_height_reply (&reply, phase, asked, stated);

	memcpy (word, &reply.candidate [2], sizeof (float));
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// ENTITY_POSE (EEDCS-49, D3, fifth slice): an OBSERVATION"""),
])

patch(M + "mb_hostmodel.c", [
    ("""	long entity_candidate, candidate_skip, candidate_updated, candidate_normal, candidate_buildings, candidate_test, candidate_fixed, candidate_placement;
};
""", """	long entity_candidate, candidate_skip, candidate_updated, candidate_normal, candidate_buildings, candidate_test, candidate_fixed, candidate_placement;
	/* the SECTOR HEIGHT reads and restores (the same question, phases 17 and 18): the sector words the host answered and the saved words it stored */
	long sector_height_read, sector_height_restored;
};
"""),
    ("""// ENTITY_CANDIDATE (mb_hostmodel.h; EEDCS-49, D3, third slice): what the host decides about one candidate of the scan. The dialogue is mb_hostentity.h; the facts are read by the caller.
int campaign_model_entity_candidate""", """// ENTITY_SCAN, the sector's HIGH-WATER HEIGHT read (mb_hostmodel.h; Task49): the word the host holds for a sector. The dialogue is mb_hostentity.h.
void campaign_model_sector_height (int x_sector, int z_sector, float *word)
{
	campaign_entity_sector_height_exchange (CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT, x_sector, z_sector, NULL, word);

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.sector_height_read++;
#endif
}

// ENTITY_SCAN, the session RESTORE of a sector's height (mb_hostmodel.h; Task49): the saved word is stated to the host, which holds exactly these bits. The dialogue is mb_hostentity.h.
void campaign_model_sector_height_restore (int x_sector, int z_sector, const float *word)
{
	float
		stored;

	campaign_entity_sector_height_exchange (CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, x_sector, z_sector, word, &stored);

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.sector_height_restored++;
#endif
}

// ENTITY_CANDIDATE (mb_hostmodel.h; EEDCS-49, D3, third slice): what the host decides about one candidate of the scan. The dialogue is mb_hostentity.h; the facts are read by the caller.
int campaign_model_entity_candidate"""),
])

patch(M + "mb_hostplace.c", [
    ("""	state [9] = (float) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);
}""", """	state [9] = (float) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

	// the city's shape (a city is created with the invalid index, 0): the host raises the sector's height at the city's arrival only when it is valid, as the original's link response does
	state [10] = (float) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
}"""),
])

S = "aphavoc/source/entity/special/sector/"

patch(S + "sc_float.c", [
    ("""		case FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT:
		////////////////////////////////////////
		{
			value = raw->tallest_structure_height;

			break;
		}""", """		case FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT:
		////////////////////////////////////////
		{
#ifdef EECH_CAMPAIGN_HOST_ENTITY

			// the sector's high-water height is the HOST's word (Task49): it is raised, reset and restored there and this is the ONE read of it, so the scan, the fixed-wing clearance and the session pack all use it.
			// The legacy field is not read (and is not written) in a hosted build, and nothing falls back to it.
			campaign_model_sector_height ((int) raw->x_sector, (int) raw->z_sector, &value);

#else

			value = raw->tallest_structure_height;

#endif

			break;
		}"""),
])

patch(S + "sc_msgs.c", [
    ("""	if (list_type == LIST_TYPE_SECTOR)
	{
		float
			height;

		vec3d
			*position;

		object_3d_index_numbers
			object_3d_index;

		object_3d_bounds
			*bounding_box;

		//
		// check for tallest fixed structure
		//

		if (get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED))""", """	if (list_type == LIST_TYPE_SECTOR)
	{
#ifndef EECH_CAMPAIGN_HOST_ENTITY

		float
			height;

		vec3d
			*position;

		object_3d_index_numbers
			object_3d_index;

		object_3d_bounds
			*bounding_box;

#endif

#ifndef EECH_CAMPAIGN_HOST_ENTITY

		//
		// check for tallest fixed structure
		//

		if (get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED))"""),
    ("""				if (height > raw->tallest_structure_height)
				{
					raw->tallest_structure_height = height;
				}
			}
		}

		//
		// set defence levels / activity array""", """				if (height > raw->tallest_structure_height)
				{
					raw->tallest_structure_height = height;
				}
			}
		}

#else

		// the sector's tallest-structure height is the HOST's word (Task49): the host raises it at the arrival of a fixed sender, from the state the campaign stated to its record of the placement just before this
		// response (campaign_place_insert, en_list.c), and the legacy field is neither written nor read in a hosted build. The original's order is kept: the height, then the mobile values, were this block's first steps.

#endif

		//
		// set defence levels / activity array"""),
])

patch(S + "sc_pack.c", [
    ("""				pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, raw->tallest_structure_height);
""", """#ifdef EECH_CAMPAIGN_HOST_ENTITY

				// the sector's height is the HOST's word, packed as the same raw 32 bits (the one read behind the sector's float getter)
				pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, get_local_entity_float_value (en, FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT));

#else

				pack_float_value (en, FLOAT_TYPE_PROTOTYPE_1, raw->tallest_structure_height);

#endif
"""),
    ("""				raw->tallest_structure_height = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1);
""", """#ifdef EECH_CAMPAIGN_HOST_ENTITY

				// the saved word is stated to the HOST, which holds exactly these bits afterwards (the acknowledgement is checked there, before the load goes on); sectors are unpacked before entities, so the
				// insertions of the load can only raise it. The legacy field is not written
				{
					float
						saved_word;

					saved_word = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1);

					campaign_model_sector_height_restore ((int) raw->x_sector, (int) raw->z_sector, &saved_word);
				}

#else

				raw->tallest_structure_height = unpack_float_value (en, FLOAT_TYPE_PROTOTYPE_1);

#endif
"""),
])
print("ok")

for rel in ("sc_float.c", "sc_pack.c"):
    patch(S + rel, [('#include "project.h"\n', '#include "project.h"\n\n#ifdef EECH_CAMPAIGN_HOST_ENTITY\n#include "entity/mobile/mb_hostmodel.h"\n#endif\n')])
print("includes ok")

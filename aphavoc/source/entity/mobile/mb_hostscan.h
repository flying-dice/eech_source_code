/*
 * The campaign's side of the host-owned ENUMERATION of the entity scan (EEDCS-49, D3, fourth and sixth slices): see mb_hostmodel.h, ENTITY_SCAN.  The host decides which sectors are visited and in what
 * order, which entries are skipped and when the scan ends; this decides nothing physical.  The host also owns WHICH entities stand in a sector and in what order (the roster, which the campaign
 * keeps informed from the one funnel every sector-list insertion and removal goes through, mb_hostplace.c): it names the entry to serve by the key it issued, and this resolves the key to the
 * campaign's entity, states the one fact only the entity heap can (that the entry is the source or the target), offers the entry for its decisions (mb_hostoffer.h) and returns the result of that offer.
 *
 * THE CAMPAIGN'S LISTS ARE A CONFORMANCE MIRROR, never the answer.  The reads are still the original's, in its order (the sector entity, its tallest structure height, its first child, the walk to the end of the
 * list and the predecessor of an entry), because the campaign's own lists remain its entity state; the tallest-structure height is read through the sector's getter, which asks the HOST (the host-owned sector height, Task49);
 * but the entry served is the one the HOST names, and
 * an entry the campaign's list does not agree with stops the engine by name (entity_scan.roster_mirror), as does a key that names no entity.  Nothing falls back to the campaign's list and a
 * disagreement is never read as an end or a miss.  A list cannot change inside one scan, because nothing in the scan links, unlinks or releases an entity (the two effectful getters it
 * performs, UPDATED and the terrain normal, change a flag word and the shared terrain record, never a list).  Still the campaign's, declared and required to move: a
 * city's buildings and every entity's shape, flags and static pose.
 *
 * The reads are the original's, in its order: for each sector the host names, the sector entity, its tallest structure height, its first child and the walk to the end of the list; for each
 * entry the comparison with the source and the target; then the offer.  Nothing is read for a sector the host does not name.
 *
 * It is a header, included by en_cllsn.c (bound to the real sector lists) and by the isolated control csrc/eech_entity_dialogue_check.c (bound to scripted sectors and the real dialogue,
 * mb_hostentity.h), so the control exercises THIS code and not a copy.  The includer provides FALSE, TRUE, NULL, memset, memcpy, debug_fatal, mb_hostoffer.h's campaign_offer_candidate,
 * campaign_model_entity_scan (mb_hostmodel.h), and
 *
 *   CAMPAIGN_SCAN_ENTITY                        the entity type
 *   CAMPAIGN_SCAN_SIDE_LENGTH (), CAMPAIGN_SCAN_MAP_MIN_X (), _MIN_Z (), _MAX_X (), _MAX_Z ()   the campaign's world map, as floats
 *   CAMPAIGN_SCAN_SECTOR_VALID (X, Z)           whether a sector index pair lies inside the map's sectors
 *   CAMPAIGN_SCAN_SECTOR (X, Z)                 the sector entity
 *   CAMPAIGN_SCAN_TALLEST (S)                   the sector's tallest structure height
 *   CAMPAIGN_SCAN_FIRST (S), CAMPAIGN_SCAN_SUCC (E), CAMPAIGN_SCAN_PRED (E)   the sector list's first child, a child's successor and predecessor (NULL at the ends): the mirror
 *   CAMPAIGN_SCAN_ENTITY_OF (KEY)               the entity the host's roster key names, or NULL when the campaign holds none for it
 */
#ifndef MB_HOSTSCAN_H
#define MB_HOSTSCAN_H

// the most questions one scan may ask the host: the map's sectors and the entries in them are far fewer, and a host that does not end the scan is stopped by name
#define CAMPAIGN_SCAN_MAX_QUESTIONS 4000000

// The entity the host served by `key` (empty: the sector's list has no more), checked against the campaign's own list, which stands at `mirror`. The host's answer is the entry; a key that names no entity
// of the campaign, or an entry the campaign's list is not at, is a named refusal, never an end of the list and never a fallback to the mirror.
static CAMPAIGN_SCAN_ENTITY *campaign_scan_served (const char *key, CAMPAIGN_SCAN_ENTITY *mirror)
{
	CAMPAIGN_SCAN_ENTITY
		*entry;

	entry = NULL;

	if (key [0] != 0)
	{
		entry = CAMPAIGN_SCAN_ENTITY_OF (key);

		if (entry == NULL)
		{
			debug_fatal ("host model: [model-provider:entity_scan.roster_key] the host served the entry \"%.40s\", which names no entity of the campaign", key);
		}
	}

	if (entry != mirror)
	{
		debug_fatal ("host model: [model-provider:entity_scan.roster_mirror] the host served %s but the campaign's own sector list is %s: the roster and the list disagree", (entry != NULL) ? "an entry" : "the end of the list", (mirror != NULL) ? "at another entry" : "at its end");
	}

	return (entry);
}

// Scans the sector lists for the first candidate that hits, in the order the host decides. `source` and `target` are the entities that are never candidates (either may be NULL), `source_old` and
// `source_new` the segment's ends (three floats each), `point` and `normal` the caller's outputs (neither read nor sent here). Returns the entity that was hit, with the intercept point and face
// normal written as the original left them, or NULL.
static CAMPAIGN_SCAN_ENTITY *campaign_scan_entities
(
	CAMPAIGN_SCAN_ENTITY *source,
	CAMPAIGN_SCAN_ENTITY *target,
	const float *source_old,
	const float *source_new,
	float *point,
	float *normal
)
{
	float
		facts [CAMPAIGN_MODEL_SCAN_FACTS],
		cursor [2],
		sector_tallest_structure_height;

	int
		phase,
		state,
		x_sector,
		z_sector,
		questions;

	char
		current [CAMPAIGN_ROSTER_KEY_MAX + 1],
		served [CAMPAIGN_ROSTER_KEY_MAX + 1];

	CAMPAIGN_SCAN_ENTITY
		*sector,
		*entry,
		*mirror,
		*successor,
		*hit;

	memset (facts, 0, sizeof (facts));

	memcpy (&facts [CAMPAIGN_SCAN_SEGMENT_OLD], source_old, 3 * sizeof (float));

	memcpy (&facts [CAMPAIGN_SCAN_SEGMENT_NEW], source_new, 3 * sizeof (float));

	facts [CAMPAIGN_SCAN_SIDE] = CAMPAIGN_SCAN_SIDE_LENGTH ();

	facts [CAMPAIGN_SCAN_MIN_X] = CAMPAIGN_SCAN_MAP_MIN_X ();

	facts [CAMPAIGN_SCAN_MIN_Z] = CAMPAIGN_SCAN_MAP_MIN_Z ();

	facts [CAMPAIGN_SCAN_MAX_X] = CAMPAIGN_SCAN_MAP_MAX_X ();

	facts [CAMPAIGN_SCAN_MAX_Z] = CAMPAIGN_SCAN_MAP_MAX_Z ();

	phase = CAMPAIGN_SCAN_PHASE_BEGIN;

	sector_tallest_structure_height = 0.0f;

	sector = NULL;

	entry = NULL;

	mirror = NULL;

	current [0] = 0;

	hit = NULL;

	questions = 0;

	for (;;)
	{
		questions++;

		if (questions > CAMPAIGN_SCAN_MAX_QUESTIONS)
		{
			debug_fatal ("host model: [model-provider:entity_scan.runaway] the host asked for more than %d steps of one scan without ending it", CAMPAIGN_SCAN_MAX_QUESTIONS);
		}

		facts [CAMPAIGN_SCAN_PHASE] = (float) phase;

		state = campaign_model_entity_scan (facts, current, cursor, served);

		switch (state)
		{
			case CAMPAIGN_MODEL_SCAN_FIRST:
			{
				// the sector the host named: its entity, its tallest structure, and its list from the TAIL (the original walks to the end so that static objects are checked before mobile ones)

				x_sector = (int) cursor [0];

				z_sector = (int) cursor [1];

				if (!CAMPAIGN_SCAN_SECTOR_VALID (x_sector, z_sector))
				{
					debug_fatal ("host model: [model-provider:entity_scan.cursor] the sector (%d, %d) the host named is outside the map", x_sector, z_sector);
				}

				facts [CAMPAIGN_SCAN_CURSOR + 0] = cursor [0];

				facts [CAMPAIGN_SCAN_CURSOR + 1] = cursor [1];

				sector = CAMPAIGN_SCAN_SECTOR (x_sector, z_sector);

				sector_tallest_structure_height = CAMPAIGN_SCAN_TALLEST (sector);

				mirror = CAMPAIGN_SCAN_FIRST (sector);

				if (mirror)
				{
					successor = CAMPAIGN_SCAN_SUCC (mirror);

					while (successor)
					{
						mirror = successor;

						successor = CAMPAIGN_SCAN_SUCC (mirror);
					}
				}

				entry = campaign_scan_served (served, mirror);

				break;
			}
			case CAMPAIGN_MODEL_SCAN_NEXT:
			{
				// the host's exchange only allows this after an entry was served, so there is one: the campaign's list is read the way the original read it, and it is the host that names the entry

				mirror = CAMPAIGN_SCAN_PRED (mirror);

				entry = campaign_scan_served (served, mirror);

				break;
			}
			case CAMPAIGN_MODEL_SCAN_OFFER:
			{
				// the entry's decisions are the host's (mb_hostoffer.h); the result goes back to it, and the first hit ends the scan

				hit = campaign_offer_candidate (entry, FALSE, sector_tallest_structure_height, source_old, source_new, point, normal);

				facts [CAMPAIGN_SCAN_HIT] = (hit != NULL) ? 1.0f : 0.0f;

				phase = CAMPAIGN_SCAN_PHASE_RESOLVED;

				continue;
			}
			case CAMPAIGN_MODEL_SCAN_DONE_HIT:
			{
				return (hit);
			}
			default:
			{
				return (NULL);
			}
		}

		// an entry was served (FIRST or NEXT): the host is told whether there is one and whether it is the source or the target, which only the entity heap can compare

		facts [CAMPAIGN_SCAN_HIT] = 0.0f;

		phase = (entry != NULL) ? CAMPAIGN_SCAN_PHASE_ENTRY : CAMPAIGN_SCAN_PHASE_END;

		memcpy (current, served, sizeof (current));

		facts [CAMPAIGN_SCAN_EXCLUDED] = ((entry != NULL) && ((entry == source) || (entry == target))) ? 1.0f : 0.0f;
	}
}

#endif

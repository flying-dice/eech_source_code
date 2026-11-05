/*
 * The campaign's side of the host-owned ENUMERATION of the entity scan (EEDCS-49, D3, fourth slice): see mb_hostmodel.h, ENTITY_SCAN.  The host decides which sectors are visited and in what
 * order, which entries are skipped and when the scan ends; this decides nothing physical.  It SERVES the host's cursor over the campaign's own sector lists: the tail of the list of the sector
 * the host names, the predecessor of the entry it is at, the one fact only the entity heap can state (that the entry is the source or the target), the offer of an entry for its decisions
 * (mb_hostoffer.h) and the result of that offer.
 *
 * THE TRANSITIONAL BACKING DEPENDENCY, named and REQUIRED to move.  The sector lists, their insertion history and the entities' live state stay the campaign's heap: the host is given the
 * order the heap has, never a copy of it, so a farther entry in an earlier sector or list position still wins over a nearer one exactly as before and there is no second store whose
 * synchronisation could be wrong.  A roster the host holds would have to follow every position write, sector relink and release, and its entities would need semantic identities that
 * outlive a scan; until that exists an entry is named only by the host's cursor.  A list cannot change inside one scan, because nothing in the scan links, unlinks or releases an entity
 * (the two effectful getters it performs, UPDATED and the terrain normal, change a flag word and the shared terrain record, never a list).
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
 *   CAMPAIGN_SCAN_FIRST (S), CAMPAIGN_SCAN_SUCC (E), CAMPAIGN_SCAN_PRED (E)   the sector list's first child, a child's successor and predecessor (NULL at the ends)
 */
#ifndef MB_HOSTSCAN_H
#define MB_HOSTSCAN_H

// the most questions one scan may ask the host: the map's sectors and the entries in them are far fewer, and a host that does not end the scan is stopped by name
#define CAMPAIGN_SCAN_MAX_QUESTIONS 4000000

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

	CAMPAIGN_SCAN_ENTITY
		*sector,
		*entry,
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

		state = campaign_model_entity_scan (facts, cursor);

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

				entry = CAMPAIGN_SCAN_FIRST (sector);

				if (entry)
				{
					successor = CAMPAIGN_SCAN_SUCC (entry);

					while (successor)
					{
						entry = successor;

						successor = CAMPAIGN_SCAN_SUCC (entry);
					}
				}

				break;
			}
			case CAMPAIGN_MODEL_SCAN_NEXT:
			{
				// the host's exchange only allows this after an entry was served, so there is one

				entry = CAMPAIGN_SCAN_PRED (entry);

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

		facts [CAMPAIGN_SCAN_EXCLUDED] = ((entry != NULL) && ((entry == source) || (entry == target))) ? 1.0f : 0.0f;
	}
}

#endif

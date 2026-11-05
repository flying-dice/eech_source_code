/*
 * The campaign's OFFER of one scan candidate to the host (EEDCS-49, D3, third slice): see mb_hostmodel.h, ENTITY_CANDIDATE.  The host decides everything about a candidate that the original
 * scan decided from its state; this is the campaign's side of that dialogue, and it decides nothing physical.  It reads the entity's own state, in stages and only when the host asks
 * for the next stage (the order the original read it in), performs the two getters that have effects of their own exactly when the host names them, offers a city's buildings in the
 * campaign's list order, and sends the narrow phase the pose the host derived.
 *
 * WHAT IS READ WHEN (the original's reads, not more):
 *   - for every candidate in the sector list: the collision-test-mobile flag, and for a mobile candidate its shape, position and motion vector;
 *   - for a building listed under a city whose box was met: its shape and position;
 *   - NOTHING ELSE until the host asks.  A candidate that is not mobile in a sector whose tallest structure is below the segment is skipped by the host on these facts alone: its fixed flag,
 *     shape and position are never read, which is how the original behaved, and a direct city building (which has no shape) needs none;
 *   - NEED_FIXED: the fixed flag and, for a fixed candidate, whether it is a city or a city building, a city's first building, position and box, or any other candidate's shape and position;
 *   - NEED_PLACEMENT (the broad phase passed): the attitude of a mobile candidate, or the heading and align flag of a fixed one (for a building under a city, the flags too);
 *   - NEED_UPDATED and NEED_NORMAL: the destructive UPDATED getter and the terrain face-normal question.
 *
 * It is a header, included by en_cllsn.c (which binds it to the real entity getters) and by the isolated control csrc/eech_entity_dialogue_check.c (which binds it to scripted entities and the
 * real dialogue, mb_hostentity.h), so the control exercises THIS code and not a copy.  The includer provides FALSE, TRUE, memset, memcpy, campaign_model_entity_candidate and
 * campaign_model_entity_collision (mb_hostmodel.h), and
 *
 *   CAMPAIGN_OFFER_ENTITY                       the entity type
 *   CAMPAIGN_OFFER_MOBILE_FLAG (E), CAMPAIGN_OFFER_FIXED_FLAG (E)     the collision-test flags (any non-zero value is set)
 *   CAMPAIGN_OFFER_IS_CITY (E), CAMPAIGN_OFFER_IS_CITY_BUILDING (E)   the entity's type
 *   CAMPAIGN_OFFER_SHAPE (E)                    the entity's 3d shape index (the scene)
 *   CAMPAIGN_OFFER_POSITION (E, OUT), CAMPAIGN_OFFER_MOTION (E, OUT)  three floats, written to OUT
 *   CAMPAIGN_OFFER_ATTITUDE (E, OUT)            nine floats, row-major
 *   CAMPAIGN_OFFER_HEADING (E), CAMPAIGN_OFFER_ALIGN (E)              the heading and the align-with-terrain flag
 *   CAMPAIGN_OFFER_FIRST_BUILDING (E), CAMPAIGN_OFFER_NEXT_BUILDING (B)   a city's buildings in its list order (NULL at the end)
 *   CAMPAIGN_OFFER_BOX (E, MIN, MAX)            a city's bounding box (three floats each)
 *   CAMPAIGN_OFFER_UPDATED (E)                  the destructive UPDATED getter
 *   CAMPAIGN_OFFER_TERRAIN_NORMAL (X, Z, OUT)   the terrain's face normal at a point (the terrain question and its effects)
 *   CAMPAIGN_OFFER_DELTA_TIME ()                the frame time
 *   CAMPAIGN_OFFER_TYPE (E), CAMPAIGN_OFFER_ALIVE (E)   the entity's type and whether it is alive, and
 *   CAMPAIGN_OFFER_HOST_ID (E)                  its semantic host identity (NULL when the roster has none): the ENTITY_POSE observation of a mobile candidate (mb_hostmodel.h)
 *
 * The ENTITY_POSE observations of a mobile candidate of a sector list (the position and motion vector the original read, and the attitude when the broad phase passed) are asked BESIDE those reads: the words
 * are copied from the facts already read, so they add no physical read, but each observation ADDS THREE read-only metadata reads the original did not make (TYPE, ALIVE and the roster HOST_ID; mb_hostmodel.h states
 * what each is). They skip nothing, reorder nothing and no decision depends on the answer. Non-mobile candidates, a city and every building listed under it (even one whose flags read as mobile in its placement)
 * are not observed.
 */
#ifndef MB_HOSTOFFER_H
#define MB_HOSTOFFER_H

// The observation of a mobile candidate's words (mb_hostmodel.h, ENTITY_POSE): the position and motion vector, and for STAGE_PLACEMENT the attitude, exactly as they were read into `facts`. The
// answer is observed by the host; nothing here depends on it.
static void campaign_offer_observe_pose (int stage, CAMPAIGN_OFFER_ENTITY *target, const char *host_id, const float *facts)
{
	float
		words [CAMPAIGN_MODEL_POSE_FACTS];

	memset (words, 0, sizeof (words));

	memcpy (&words [0], &facts [CAMPAIGN_CANDIDATE_POSITION], 3 * sizeof (float));

	memcpy (&words [3], &facts [CAMPAIGN_CANDIDATE_MOTION], 3 * sizeof (float));

	if (stage == CAMPAIGN_POSE_STAGE_PLACEMENT)
	{
		memcpy (&words [6], &facts [CAMPAIGN_CANDIDATE_ATTITUDE], 9 * sizeof (float));
	}

	campaign_model_entity_pose (stage, (int) CAMPAIGN_OFFER_TYPE (target), (CAMPAIGN_OFFER_ALIVE (target) != 0), host_id, words);
}

// Offers one candidate to the host. `of_city` is TRUE for a building listed under a city whose box the host reported as met (the host does not apply the sector skip to it again).
// `source_old` and `source_new` are the segment's ends (three floats each) and `point` and `normal` the caller's outputs (neither read nor sent here). Returns the entity that was hit, with
// the intercept point and face normal written as the original left them, or NULL.
static CAMPAIGN_OFFER_ENTITY *campaign_offer_candidate
(
	CAMPAIGN_OFFER_ENTITY *target,
	int of_city,
	float sector_tallest_structure_height,
	const float *source_old,
	const float *source_new,
	float *point,
	float *normal
)
{
	float
		facts [CAMPAIGN_MODEL_CANDIDATE_FACTS],
		pose [CAMPAIGN_MODEL_CANDIDATE_POSE_MAX],
		terrain_face_normal [3],
		box_min [3],
		box_max [3];

	const char
		*host_id;

	int
		mobile,
		fixed,
		scene,
		state,
		pose_count;

	CAMPAIGN_OFFER_ENTITY
		*first_building,
		*building,
		*hit;

	first_building = NULL;

	host_id = NULL;

	memset (facts, 0, sizeof (facts));

	memcpy (&facts [CAMPAIGN_CANDIDATE_SEGMENT_OLD], source_old, 3 * sizeof (float));

	memcpy (&facts [CAMPAIGN_CANDIDATE_SEGMENT_NEW], source_new, 3 * sizeof (float));

	facts [CAMPAIGN_CANDIDATE_TALLEST] = sector_tallest_structure_height;

	facts [CAMPAIGN_CANDIDATE_DELTA_TIME] = CAMPAIGN_OFFER_DELTA_TIME ();

	scene = 0;

	if (of_city)
	{
		// a building under a city that passed its box: the original read its shape and position before its broad phase, and nothing else

		facts [CAMPAIGN_CANDIDATE_OF_CITY] = 1.0f;

		scene = CAMPAIGN_OFFER_SHAPE (target);

		CAMPAIGN_OFFER_POSITION (target, &facts [CAMPAIGN_CANDIDATE_POSITION]);
	}
	else
	{
		// the original read the mobile flag of every candidate first; a mobile candidate's shape, position and motion vector follow, and everything else waits for the host

		mobile = (CAMPAIGN_OFFER_MOBILE_FLAG (target) != 0);

		facts [CAMPAIGN_CANDIDATE_MOBILE] = (float) mobile;

		if (mobile)
		{
			scene = CAMPAIGN_OFFER_SHAPE (target);

			CAMPAIGN_OFFER_POSITION (target, &facts [CAMPAIGN_CANDIDATE_POSITION]);

			CAMPAIGN_OFFER_MOTION (target, &facts [CAMPAIGN_CANDIDATE_MOTION]);

			// the mobile's semantic host identity (a read-only metadata read, once per offer): the host decides this candidate's physical pose from its own record of it

			host_id = CAMPAIGN_OFFER_HOST_ID (target);

			campaign_offer_observe_pose (CAMPAIGN_POSE_STAGE_OFFER, target, host_id, facts);
		}
	}

	for (;;)
	{
		state = campaign_model_entity_candidate (scene, facts, host_id, pose, &pose_count);

		switch (state)
		{
			case CAMPAIGN_MODEL_CANDIDATE_SKIP:
			{
				return (NULL);
			}
			case CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED:
			{
				// the destructive getter, once, exactly when the host needs it (the original read it for every moving candidate it visited)

				facts [CAMPAIGN_CANDIDATE_UPDATED] = CAMPAIGN_OFFER_UPDATED (target) ? 2.0f : 1.0f;

				break;
			}
			case CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED:
			{
				// the sector skip did not remove the candidate: the original now read its fixed flag and the facts of the branch that flag opens

				fixed = (CAMPAIGN_OFFER_FIXED_FLAG (target) != 0);

				facts [CAMPAIGN_CANDIDATE_FIXED] = (float) fixed;

				facts [CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 1.0f;

				if (fixed)
				{
					if (CAMPAIGN_OFFER_IS_CITY (target))
					{
						facts [CAMPAIGN_CANDIDATE_CITY] = 1.0f;

						first_building = CAMPAIGN_OFFER_FIRST_BUILDING (target);

						facts [CAMPAIGN_CANDIDATE_HAS_BUILDINGS] = (first_building != NULL);

						if (first_building)
						{
							CAMPAIGN_OFFER_POSITION (target, &facts [CAMPAIGN_CANDIDATE_POSITION]);

							CAMPAIGN_OFFER_BOX (target, box_min, box_max);

							memcpy (&facts [CAMPAIGN_CANDIDATE_BOX_MIN], box_min, 3 * sizeof (float));

							memcpy (&facts [CAMPAIGN_CANDIDATE_BOX_MAX], box_max, 3 * sizeof (float));
						}
					}
					else if (CAMPAIGN_OFFER_IS_CITY_BUILDING (target))
					{
						// a building listed directly in a sector: the original skipped it without a shape lookup, and so does the host

						facts [CAMPAIGN_CANDIDATE_CITY_BUILDING] = 1.0f;
					}
					else
					{
						scene = CAMPAIGN_OFFER_SHAPE (target);

						CAMPAIGN_OFFER_POSITION (target, &facts [CAMPAIGN_CANDIDATE_POSITION]);
					}
				}

				break;
			}
			case CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT:
			{
				// the broad phase passed: the live placement the original read in its narrow phase

				if (of_city)
				{
					mobile = (CAMPAIGN_OFFER_MOBILE_FLAG (target) != 0);

					fixed = ((!mobile) && (CAMPAIGN_OFFER_FIXED_FLAG (target) != 0));

					facts [CAMPAIGN_CANDIDATE_MOBILE] = (float) mobile;

					facts [CAMPAIGN_CANDIDATE_FIXED] = (float) fixed;
				}

				if (facts [CAMPAIGN_CANDIDATE_MOBILE] != 0.0f)
				{
					CAMPAIGN_OFFER_ATTITUDE (target, &facts [CAMPAIGN_CANDIDATE_ATTITUDE]);

					if (!of_city)
					{
						campaign_offer_observe_pose (CAMPAIGN_POSE_STAGE_PLACEMENT, target, host_id, facts);
					}
				}
				else if (facts [CAMPAIGN_CANDIDATE_FIXED] != 0.0f)
				{
					facts [CAMPAIGN_CANDIDATE_HEADING] = CAMPAIGN_OFFER_HEADING (target);

					facts [CAMPAIGN_CANDIDATE_ALIGN] = (CAMPAIGN_OFFER_ALIGN (target) != 0);
				}

				facts [CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 1.0f;

				break;
			}
			case CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL:
			{
				// the terrain question of a terrain-aligned fixed object, asked once the broad phase has passed, as the original asked it

				CAMPAIGN_OFFER_TERRAIN_NORMAL (facts [CAMPAIGN_CANDIDATE_POSITION + 0], facts [CAMPAIGN_CANDIDATE_POSITION + 2], terrain_face_normal);

				facts [CAMPAIGN_CANDIDATE_NORMAL_GIVEN] = 1.0f;

				memcpy (&facts [CAMPAIGN_CANDIDATE_NORMAL], terrain_face_normal, 3 * sizeof (float));

				break;
			}
			case CAMPAIGN_MODEL_CANDIDATE_BUILDINGS:
			{
				// the city's box is met: its buildings, in the campaign's list order, the first one that is hit ends the scan

				building = first_building;

				while (building)
				{
					hit = campaign_offer_candidate (building, TRUE, sector_tallest_structure_height, source_old, source_new, point, normal);

					if (hit)
					{
						return (hit);
					}

					building = CAMPAIGN_OFFER_NEXT_BUILDING (building);
				}

				return (NULL);
			}
			default:
			{
				// the narrow phase, at the pose the host derived

				if (campaign_model_entity_collision (scene, source_old, source_new, pose, pose_count, point, normal))
				{
					return (target);
				}

				return (NULL);
			}
		}
	}
}

#endif

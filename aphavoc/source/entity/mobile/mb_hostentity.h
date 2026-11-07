/*
 * The campaign's half of the host-owned entity collision narrow phase (EEDCS-49, D3): see mb_hostmodel.h, ENTITY_COLLISION.  It sends the segment and the candidate's live pose,
 * checks the host's reply completely and only then writes the caller's buffers.  It holds no model, transform or intersection.
 *
 * THE OUTPUT CONTRACT.  The caller's intercept point and face normal are OUTPUTS.  The original's intersection helpers (modules/maths/vector.c, get_3d_line_triangle_intersection and
 * get_3d_line_quad_intersection) never read them before assigning them: they assign the whole normal (x, y, z) first, whatever the segment does, and the whole point only after the
 * segment has crossed the face's plane.  A real caller (answer_weapon_step_lookup in wn_move.c) passes an automatic face normal it never initialised, so this exchange does NOT read
 * either buffer and sends neither to the host: an indeterminate word is never loaded, copied or forwarded.  The host says which components the original's arithmetic assigned (a six
 * bit write mask: bits 0 to 2 the point, bits 3 to 5 the normal, a vector being written whole) and gives their computed words; only those components are written, normal first, after
 * every check has passed.  A component that was not assigned keeps exactly the words it held, without being read.
 *
 * It is a header, included by mb_hostmodel.c (which binds it to the real host callback and the real scene tables) and by the isolated control csrc/eech_entity_dialogue_check.c
 * (which binds it to a scripted host), so the control exercises THIS code and not a copy.  The includer provides campaign_model_request and campaign_model_reply (mb_hostmodel.h),
 * FALSE, TRUE, uint32_t, debug_fatal, memcpy, memset, and
 *
 *   CAMPAIGN_ENTITY_REQUIRE_SCENE (scene)       stops the engine unless `scene` is a scene a candidate can have
 *   CAMPAIGN_ENTITY_ASK (request, reply)         one host exchange, which stops the engine on a missing provider or a refusal
 *   CAMPAIGN_ENTITY_IDENTITY (reply, scene)      stops the engine unless the reply names the campaign's own name for `scene`
 *   CAMPAIGN_ENTITY_CANDIDATE_ASK (request, reply)  the same exchange for an ENTITY_CANDIDATE question (it may be counted separately)
 *   CAMPAIGN_ENTITY_CANDIDATE_IDENTITY (reply, scene)  the same identity check for an ENTITY_CANDIDATE reply, which names its own operation (entity_candidate.identity)
 *   CAMPAIGN_ENTITY_SCAN_ASK (request, reply)  the same exchange for an ENTITY_SCAN question (it may be counted separately)
 *   CAMPAIGN_ENTITY_POSE_ASK (request, reply)  the same exchange for an ENTITY_POSE observation (it may be counted separately)
 *
 * The same header holds the campaign's half of the host's per-candidate decisions (ENTITY_CANDIDATE, mb_hostmodel.h): the facts layout, the exchange and the complete check of the host's
 * decision against the facts it was given.  It reads no entity: the caller (en_cllsn.c) fills the facts from the entity it visits and performs the two services the host may ask for.
 */
#ifndef MB_HOSTENTITY_H
#define MB_HOSTENTITY_H

// the write mask of a reply: the point is words 0 to 2 of `points` and the normal words 3 to 5
#define CAMPAIGN_ENTITY_POINT_WRITTEN 0x07
#define CAMPAIGN_ENTITY_NORMAL_WRITTEN 0x38
#define CAMPAIGN_ENTITY_EVERYTHING_WRITTEN 0x3f

// a computed word of the reply, read as bytes (the reply is the host's, fully initialised by the exchange)
static uint32_t entity_reply_word (const campaign_model_reply *reply, int component)
{
	uint32_t
		word;

	memcpy (&word, &reply->points[component], sizeof (word));

	return (word);
}

// Everything about a reply that can be wrong, before any word of the caller's buffers is written.
static void check_entity_reply (const campaign_model_reply *reply, int scene)
{
	int
		mask,
		component;

	if ((reply->result != CAMPAIGN_MODEL_TRIANGLE_FOUND) && (reply->result != CAMPAIGN_MODEL_TRIANGLE_END))
	{
		debug_fatal ("host model: [model-provider:entity_collision.result] the host answered neither a hit nor a miss for scene %d (result %d)", scene, reply->result);
	}

	mask = reply->write_mask;

	if ((mask < 0) || (mask > CAMPAIGN_ENTITY_EVERYTHING_WRITTEN))
	{
		debug_fatal ("host model: [model-provider:entity_collision.write_mask] the host's write mask %d for scene %d is outside 0 to %d", mask, scene, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	}

	if (((mask & CAMPAIGN_ENTITY_POINT_WRITTEN) != 0) && ((mask & CAMPAIGN_ENTITY_POINT_WRITTEN) != CAMPAIGN_ENTITY_POINT_WRITTEN))
	{
		debug_fatal ("host model: [model-provider:entity_collision.write_mask] the host's write mask %d for scene %d writes the intercept point in part: the original assigns it whole", mask, scene);
	}

	if (((mask & CAMPAIGN_ENTITY_NORMAL_WRITTEN) != 0) && ((mask & CAMPAIGN_ENTITY_NORMAL_WRITTEN) != CAMPAIGN_ENTITY_NORMAL_WRITTEN))
	{
		debug_fatal ("host model: [model-provider:entity_collision.write_mask] the host's write mask %d for scene %d writes the face normal in part: the original assigns it whole", mask, scene);
	}

	if (((mask & CAMPAIGN_ENTITY_POINT_WRITTEN) != 0) && ((mask & CAMPAIGN_ENTITY_NORMAL_WRITTEN) == 0))
	{
		debug_fatal ("host model: [model-provider:entity_collision.write_mask] the host's write mask %d for scene %d writes the intercept point without the normal, which the original assigns first", mask, scene);
	}

	if ((reply->result == CAMPAIGN_MODEL_TRIANGLE_FOUND) && (mask != CAMPAIGN_ENTITY_EVERYTHING_WRITTEN))
	{
		debug_fatal ("host model: [model-provider:entity_collision.write_mask] the host reports a hit for scene %d with write mask %d: a hit has assigned the point and the normal", scene, mask);
	}

	// a component that was not assigned carries no word: the host gives zero for it, so nothing the host did not compute can reach a buffer or be mistaken for a computed value
	for (component = 0; component < 6; component++)
	{
		if (((mask & (1 << component)) == 0) && (entity_reply_word (reply, component) != 0))
		{
			debug_fatal ("host model: [model-provider:entity_collision.unwritten] the host gave a word for component %d of scene %d, which its write mask %d says was not assigned", component, scene, mask);
		}
	}
}

// One collision candidate against the segment.  `candidate` has 12 floats (a stationary candidate: position, attitude) or 16 (a moving one: old position, new position, attitude,
// speed).  Returns TRUE for a hit.  `point` and `normal` are written, never read.
static int campaign_entity_collision_exchange
(
	int scene,
	const float *segment_old,
	const float *segment_new,
	const float *candidate,
	int candidate_count,
	float *point,
	float *normal
)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	CAMPAIGN_ENTITY_REQUIRE_SCENE (scene);

	if ((candidate_count != CAMPAIGN_MODEL_ENTITY_STATIONARY_POSE - 6) && (candidate_count != CAMPAIGN_MODEL_ENTITY_MOVING_POSE - 6))
	{
		debug_fatal ("host model: [model-provider:entity_collision.pose] a pose of %d candidate floats was given for scene %d", candidate_count, scene);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_COLLISION;
	request.scene = scene;
	request.pose_count = 6 + candidate_count;
	memcpy (&request.pose[0], segment_old, 3 * sizeof (float));
	memcpy (&request.pose[3], segment_new, 3 * sizeof (float));
	memcpy (&request.pose[6], candidate, candidate_count * sizeof (float));

	CAMPAIGN_ENTITY_ASK (&request, &reply);

	CAMPAIGN_ENTITY_IDENTITY (&reply, scene);

	check_entity_reply (&reply, scene);

	// only now are the caller's buffers touched, and only the components the original assigned, the normal first
	if ((reply.write_mask & CAMPAIGN_ENTITY_NORMAL_WRITTEN) != 0)
	{
		memcpy (normal, &reply.points[3], 3 * sizeof (float));
	}

	if ((reply.write_mask & CAMPAIGN_ENTITY_POINT_WRITTEN) != 0)
	{
		memcpy (point, &reply.points[0], 3 * sizeof (float));
	}

	return (reply.result == CAMPAIGN_MODEL_TRIANGLE_FOUND);
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// ENTITY_CANDIDATE (EEDCS-49, D3, third slice): the exchange. The facts are laid out as in mb_hostmodel.h (CAMPAIGN_CANDIDATE_*).
// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

// whether the float at `value` is a finite number, classified from its raw bytes: the untrusted word is never passed or loaded as a floating value before it is refused (an infinity or a
// NaN, signalling or quiet, has every exponent bit set)
static int candidate_word_is_finite (const float *value)
{
	uint32_t
		word;

	memcpy (&word, value, sizeof (word));

	return ((word & 0x7f800000u) != 0x7f800000u);
}

// Everything about a decision that can be wrong given the facts it answers, before any of it is used.
static void check_candidate_reply (const campaign_model_reply *reply, int scene, const float *facts)
{
	int
		state,
		count,
		index;

	state = reply->candidate_state;

	count = reply->candidate_count;

	if ((state < CAMPAIGN_MODEL_CANDIDATE_SKIP) || (state > CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT))
	{
		debug_fatal ("host model: [model-provider:entity_candidate.state] the host decided state %d for scene %d, which is not one of the seven", state, scene);
	}

	if ((state != CAMPAIGN_MODEL_CANDIDATE_TEST) && (count != 0))
	{
		debug_fatal ("host model: [model-provider:entity_candidate.pose] the host gave a pose of %d floats for scene %d with state %d, which carries none", count, scene, state);
	}

	if (state == CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED)
	{
		if ((facts [CAMPAIGN_CANDIDATE_MOBILE] == 0.0f) || (facts [CAMPAIGN_CANDIDATE_OF_CITY] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_UPDATED] != 0.0f))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host asked for the UPDATED flag of scene %d, which is not a mobile candidate whose flag was not asked yet", scene);
		}
	}

	if (state == CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED)
	{
		if ((facts [CAMPAIGN_CANDIDATE_MOBILE] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_OF_CITY] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_FIXED_GIVEN] != 0.0f))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host asked for the fixed facts of scene %d, which is a mobile candidate, a building under a city or one whose fixed facts were read", scene);
		}
	}

	if (state == CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT)
	{
		// the placement is read by the campaign's getters (heading, align flag, attitude), so it is refused HERE, before any of them, unless the facts describe a branch that has one: a mobile candidate,
		// a building listed under a city (its collision flags are read DURING the placement, so its fixed facts are not given), or an ordinary fixed candidate whose fixed facts say it is one (not a city,
		// not a building listed directly in a sector, not an entity that is neither mobile nor fixed). Each needs a scene, and none may have its placement already.
		int
			mobile,
			of_city,
			ordinary_fixed;

		mobile = (facts [CAMPAIGN_CANDIDATE_MOBILE] != 0.0f);

		of_city = (facts [CAMPAIGN_CANDIDATE_OF_CITY] != 0.0f);

		ordinary_fixed = ((!mobile) && (!of_city) && (facts [CAMPAIGN_CANDIDATE_FIXED_GIVEN] != 0.0f) && (facts [CAMPAIGN_CANDIDATE_FIXED] != 0.0f) &&
			(facts [CAMPAIGN_CANDIDATE_CITY] == 0.0f) && (facts [CAMPAIGN_CANDIDATE_CITY_BUILDING] == 0.0f));

		if ((facts [CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] != 0.0f) || (scene == 0) || (!(mobile || of_city || ordinary_fixed)))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host asked for the placement facts of scene %d, which have been read already, or which belongs to no branch that has a placement (a city, a building listed directly in a sector, an entity that is neither mobile nor fixed, or a candidate that has no scene)", scene);
		}
	}

	if (state == CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL)
	{
		if ((facts [CAMPAIGN_CANDIDATE_ALIGN] == 0.0f) || (facts [CAMPAIGN_CANDIDATE_NORMAL_GIVEN] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_MOBILE] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_FIXED] == 0.0f) ||
			(facts [CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] == 0.0f))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host asked for the terrain face normal of scene %d, which is not a fixed candidate with its placement aligned with the terrain whose normal was not given yet", scene);
		}
	}

	if (state == CAMPAIGN_MODEL_CANDIDATE_BUILDINGS)
	{
		if ((facts [CAMPAIGN_CANDIDATE_CITY] == 0.0f) || (facts [CAMPAIGN_CANDIDATE_HAS_BUILDINGS] == 0.0f))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host named buildings for scene %d, which is not a city with a first building", scene);
		}
	}

	if (state == CAMPAIGN_MODEL_CANDIDATE_TEST)
	{
		// a moving candidate has a pose of 16 floats and every other 12; only a mobile candidate can be moving
		if ((count != 12) && (count != 16))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.pose] the host gave a pose of %d floats for scene %d: a stationary candidate has 12 and a moving one 16", count, scene);
		}

		if ((count == 16) && (facts [CAMPAIGN_CANDIDATE_MOBILE] == 0.0f))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.pose] the host gave a moving pose for scene %d, which is not a mobile candidate", scene);
		}

		// the narrow phase needs the candidate's scene: a pose for a candidate whose shape was never read cannot be tested
		if (scene == 0)
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host required a narrow phase for a candidate that has no scene");
		}

		for (index = 0; index < count; index++)
		{
			if (!candidate_word_is_finite (&reply->candidate [index]))
			{
				debug_fatal ("host model: [model-provider:entity_candidate.pose] float %d of the pose the host gave for scene %d is not a finite number", index, scene);
			}
		}
	}
}

// The host identity a candidate question carries (a MOBILE candidate's: the host decides its physical pose from its own record of that identity): the roster's whole domain of 1 to CAMPAIGN_MODEL_HOST_ID_MAX
// bytes, never shortened. One beyond the domain cannot exist and is refused by marker rather than skipped.
static size_t candidate_host_id_length (const char *host_id)
{
	size_t
		length;

	length = (host_id != NULL) ? strlen (host_id) : 0;

	if (length > CAMPAIGN_MODEL_HOST_ID_MAX)
	{
		debug_fatal ("host model: [model-provider:entity_candidate.request] a host identity of %d bytes is outside the roster's domain of 1 to %d", (int) length, CAMPAIGN_MODEL_HOST_ID_MAX);
	}

	return (length);
}

// One question about one candidate: the facts in, the host's checked decision out. `scene` is the candidate's scene, and 0 for a city. Returns a campaign_model_candidate_state and, for TEST,
// the narrow phase's pose in `pose` (`*pose_count` floats).
static int campaign_entity_candidate_exchange (int scene, const float *facts, const char *host_id, float *pose, int *pose_count)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	int
		index;

	// scene 0 means no shape has been read yet: a skipped candidate, a city and a direct city building have none, and the host refuses by name a branch that needs one. A scene that is named
	// must be one a candidate can have.
	if ((facts [CAMPAIGN_CANDIDATE_CITY] != 0.0f) && (scene != 0))
	{
		debug_fatal ("host model: [model-provider:entity_candidate.request] a city has no scene of its own, but scene %d was named", scene);
	}

	if (scene != 0)
	{
		CAMPAIGN_ENTITY_REQUIRE_SCENE (scene);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_CANDIDATE;
	request.scene = scene;
	request.pose_count = CAMPAIGN_MODEL_CANDIDATE_FACTS;
	memcpy (&request.pose [0], facts, CAMPAIGN_MODEL_CANDIDATE_FACTS * sizeof (float));

	if (host_id != NULL)
	{
		memcpy (request.host_id, host_id, candidate_host_id_length (host_id));
	}

	CAMPAIGN_ENTITY_CANDIDATE_ASK (&request, &reply);

	if (scene == 0)
	{
		if (reply.name [0] != 0)
		{
			debug_fatal ("host model: [model-provider:entity_candidate.identity] the host named a scene \"%.60s\" for a candidate that has none", reply.name);
		}
	}
	else
	{
		CAMPAIGN_ENTITY_CANDIDATE_IDENTITY (&reply, scene);
	}

	check_candidate_reply (&reply, scene, facts);

	*pose_count = 0;

	if (reply.candidate_state == CAMPAIGN_MODEL_CANDIDATE_TEST)
	{
		for (index = 0; index < reply.candidate_count; index++)
		{
			pose [index] = reply.candidate [index];
		}

		*pose_count = reply.candidate_count;
	}

	return (reply.candidate_state);
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// ENTITY_SCAN (EEDCS-49, D3, fourth slice): the enumeration. The host's cursor over the campaign's lists, with a complete check of its answer against the facts it answers.
// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

// whether the float at `value` has every bit clear: an unassigned cursor, read as bytes
static int scan_word_is_zero (const float *value)
{
	uint32_t
		word;

	memcpy (&word, value, sizeof (word));

	return (word == 0);
}

// Whether `key` is a key of the host's roster: one to CAMPAIGN_ROSTER_KEY_MAX decimal digits without a leading zero (the host issues them from 1 and never again). The campaign carries a key whole and reads it
// as nothing else.
static int roster_key_is_valid (const char *key)
{
	size_t
		length,
		index;

	length = strlen (key);

	if ((length < 1) || (length > CAMPAIGN_ROSTER_KEY_MAX) || (key [0] == '0'))
	{
		return (FALSE);
	}

	for (index = 0; index < length; index++)
	{
		if ((key [index] < '0') || (key [index] > '9'))
		{
			return (FALSE);
		}
	}

	return (TRUE);
}

// The key a reply names, checked BEFORE it is copied: the name place is text that ends inside the reply, and is empty, or a valid key.
static void check_reply_key (const campaign_model_reply *reply, const char *operation)
{
	if (memchr (reply->name, 0, sizeof (reply->name)) == NULL)
	{
		debug_fatal ("host model: [model-provider:%s.key] the key the host named does not end inside the reply", operation);
	}

	if ((reply->name [0] != 0) && (!roster_key_is_valid (reply->name)))
	{
		debug_fatal ("host model: [model-provider:%s.key] the key \"%.40s\" the host named is not a roster key", operation, reply->name);
	}
}

// The answer to a roster event: the state the phase allows, and a key only for an insertion (and then always).
static void check_roster_reply (const campaign_model_reply *reply, int phase)
{
	int
		wanted;

	wanted = (phase == CAMPAIGN_SCAN_PHASE_INSERT) ? CAMPAIGN_MODEL_SCAN_PLACED : ((phase == CAMPAIGN_SCAN_PHASE_REMOVE) ? CAMPAIGN_MODEL_SCAN_REMOVED : CAMPAIGN_MODEL_SCAN_CLEARED);

	if (reply->candidate_state != wanted)
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a roster event of phase %d, which allows only state %d", reply->candidate_state, phase, wanted);
	}

	check_reply_key (reply, "entity_scan");

	// a removal may name the CITY record the host keeps for the box it built (a valid key, checked above; the caller refuses it for an entity that is no city); every other answer is a key for an insertion and none otherwise
	if ((phase != CAMPAIGN_SCAN_PHASE_REMOVE) && ((wanted == CAMPAIGN_MODEL_SCAN_PLACED) != (reply->name [0] != 0)))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the host %s a key for a roster event of phase %d", (wanted == CAMPAIGN_MODEL_SCAN_PLACED) ? "issued no" : "issued", phase);
	}

	if ((!scan_word_is_zero (&reply->candidate [0])) || (!scan_word_is_zero (&reply->candidate [1])))
	{
		debug_fatal ("host model: [model-provider:entity_scan.cursor] the host named a sector for a roster event, which carries none");
	}
}

// Everything about a scan decision that can be wrong given the facts it answers, before any of it is used. The cursor is checked from its raw words: finite first, then a whole number inside
// 0 to 16777216 (the largest whole number a float holds exactly), and only then compared. The key of the entry to serve is the host's roster key, or empty when the sector's list has no more.
static void check_scan_reply (const campaign_model_reply *reply, const float *facts)
{
	int
		state,
		phase;

	state = reply->candidate_state;

	phase = (int) facts [CAMPAIGN_SCAN_PHASE];

	if (phase >= CAMPAIGN_SCAN_PHASE_INSERT)
	{
		check_roster_reply (reply, phase);

		return;
	}

	if ((state < CAMPAIGN_MODEL_SCAN_FIRST) || (state > CAMPAIGN_MODEL_SCAN_DONE_HIT))
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d, which is not one of the five", state);
	}

	if (phase == CAMPAIGN_SCAN_PHASE_BEGIN)
	{
		if ((state != CAMPAIGN_MODEL_SCAN_FIRST) && (state != CAMPAIGN_MODEL_SCAN_DONE_NONE))
		{
			debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d at the start of a scan, which only a first sector or an empty window allows", state);
		}
	}
	else if (phase == CAMPAIGN_SCAN_PHASE_ENTRY)
	{
		// an entry that is the source or the target is skipped, every other is offered
		if (state != ((facts [CAMPAIGN_SCAN_EXCLUDED] != 0.0f) ? CAMPAIGN_MODEL_SCAN_NEXT : CAMPAIGN_MODEL_SCAN_OFFER))
		{
			debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a served entry (excluded %d), which must be skipped when excluded and offered otherwise", state, (int) facts [CAMPAIGN_SCAN_EXCLUDED]);
		}
	}
	else if (phase == CAMPAIGN_SCAN_PHASE_END)
	{
		if ((state != CAMPAIGN_MODEL_SCAN_FIRST) && (state != CAMPAIGN_MODEL_SCAN_DONE_NONE))
		{
			debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d at the end of a list, which only the next sector or the end of the scan allows", state);
		}
	}
	else
	{
		// the first candidate that hits ends the scan, and a miss goes on
		if (state != ((facts [CAMPAIGN_SCAN_HIT] != 0.0f) ? CAMPAIGN_MODEL_SCAN_DONE_HIT : CAMPAIGN_MODEL_SCAN_NEXT))
		{
			debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d after an offered entry (hit %d), which must end the scan on a hit and go on after a miss", state, (int) facts [CAMPAIGN_SCAN_HIT]);
		}
	}

	if (state == CAMPAIGN_MODEL_SCAN_FIRST)
	{
		int
			index;

		for (index = 0; index < 2; index++)
		{
			if (!candidate_word_is_finite (&reply->candidate [index]))
			{
				debug_fatal ("host model: [model-provider:entity_scan.cursor] the sector index %d the host named is not a finite number", index);
			}

			if ((reply->candidate [index] < 0.0f) || (reply->candidate [index] > 16777216.0f) || ((float) (int) reply->candidate [index] != reply->candidate [index]))
			{
				debug_fatal ("host model: [model-provider:entity_scan.cursor] the sector index %d the host named is not a whole number within 0 to 16777216", index);
			}
		}
	}
	else if ((!scan_word_is_zero (&reply->candidate [0])) || (!scan_word_is_zero (&reply->candidate [1])))
	{
		debug_fatal ("host model: [model-provider:entity_scan.cursor] the host named a sector with state %d, which carries none", state);
	}

	check_reply_key (reply, "entity_scan");

	// only an entry that is served carries a key
	if (((state != CAMPAIGN_MODEL_SCAN_FIRST) && (state != CAMPAIGN_MODEL_SCAN_NEXT)) && (reply->name [0] != 0))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the host named the entry \"%.40s\" with state %d, which serves none", reply->name, state);
	}
}

// One question about the enumeration: the facts in, the host's checked decision out (a campaign_model_scan_state, for FIRST the two sector indices in `cursor`, and for FIRST and NEXT the key of the entry to
// serve in `key`, empty when the sector's list has no more). `entry_key` is the key of the entry the campaign is standing at: the host serves what comes after IT, so the question that follows a served entry
// always names it, and the others never do.
static int campaign_entity_scan_exchange (const float *facts, const char *entry_key, float *cursor, char *key)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	int
		phase,
		standing;

	phase = (int) facts [CAMPAIGN_SCAN_PHASE];

	standing = (entry_key != NULL) && (entry_key [0] != 0);

	if (standing != ((phase == CAMPAIGN_SCAN_PHASE_ENTRY) || (phase == CAMPAIGN_SCAN_PHASE_RESOLVED)))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the scan asked in phase %d with %s, which only a served entry has", phase, standing ? "the key of an entry" : "no entry key");
	}

	if (standing && !roster_key_is_valid (entry_key))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the entry key \"%.40s\" the scan is standing at is not a roster key", entry_key);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	memcpy (&request.pose [0], facts, CAMPAIGN_MODEL_SCAN_FACTS * sizeof (float));

	if (standing)
	{
		memcpy (request.host_id, entry_key, strlen (entry_key));
	}

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_scan_reply (&reply, facts);

	cursor [0] = 0.0f;

	cursor [1] = 0.0f;

	key [0] = 0;

	if (reply.candidate_state == CAMPAIGN_MODEL_SCAN_FIRST)
	{
		cursor [0] = reply.candidate [0];

		cursor [1] = reply.candidate [1];
	}

	if ((reply.candidate_state == CAMPAIGN_MODEL_SCAN_FIRST) || (reply.candidate_state == CAMPAIGN_MODEL_SCAN_NEXT))
	{
		memcpy (key, reply.name, strlen (reply.name) + 1);
	}

	return (reply.candidate_state);
}

// The roster's events (the same question, phases 4 to 6; see mb_hostmodel.h): `phase` is CAMPAIGN_SCAN_PHASE_INSERT, _REMOVE or _RESET. An insertion states the entity's `entity_type`, the sector side length and the
// `position` words (3 floats) the campaign holds, with a hosted mobile's semantic `identity` (NULL for every other entity), and receives the host's key in `key`; a removal names the key in `identity`.
static void campaign_entity_roster_exchange (int phase, int entity_type, float side, const float *position, const char *identity, char *key)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	size_t
		length;

	length = (identity != NULL) ? strlen (identity) : 0;

	if (length > CAMPAIGN_MODEL_HOST_ID_MAX)
	{
		debug_fatal ("host model: [model-provider:entity_scan.request] a host identity of %d bytes is outside the roster's domain of 1 to %d", (int) length, CAMPAIGN_MODEL_HOST_ID_MAX);
	}

	if ((phase == CAMPAIGN_SCAN_PHASE_REMOVE) && !roster_key_is_valid ((identity != NULL) ? identity : ""))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] a removal names \"%.40s\", which is not a roster key", (identity != NULL) ? identity : "");
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) phase;

	if (phase == CAMPAIGN_SCAN_PHASE_INSERT)
	{
		memcpy (&request.pose [CAMPAIGN_ROSTER_POSITION], position, 3 * sizeof (float));
		request.pose [CAMPAIGN_ROSTER_TYPE] = (float) entity_type;
		request.pose [CAMPAIGN_SCAN_SIDE] = side;
	}

	if (length > 0)
	{
		memcpy (request.host_id, identity, length);
	}

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_scan_reply (&reply, request.pose);

	memcpy (key, reply.name, strlen (reply.name) + 1);
}


// The CITY events (the same question; phases CAMPAIGN_SCAN_PHASE_CHILD_INSERT to CAMPAIGN_SCAN_PHASE_NAME, EEDCS-49, D3, seventh slice; see mb_hostmodel.h): a building's place in its city's list
// (LIST_TYPE_CITY_BUILDING) is inserted, removed, asked for its first or next entry; a sector placement and a city child are declared the same entity; and a placement is named by the legacy object-report
// identity the host issued for its entity. `identity` is the identity place the caller built from keys (and, for a NAME, the object identity after one space): it is validated HERE, before anything is sent, against
// the grammar of the phase. The host's reply is checked completely: the state the phase allows, a key for an insertion, the end of a list as an empty key, and for a LINK or a NAME the key it echoes.
static int city_split (const char *identity, int *first_length)
{
	const char
		*space;

	space = strchr (identity, ' ');

	*first_length = (space != NULL) ? (int) (space - identity) : (int) strlen (identity);

	return (space != NULL);
}

// The identity place is a key and, after one space, a second part: a key, or for a NAME the object identity, which has its own domain (CAMPAIGN_MODEL_OBJECT_ID_MAX) beside the key's, so the whole place is sized for both
// (CAMPAIGN_MODEL_SCAN_IDENTITY_MAX). Every part is measured against its own maximum before anything is copied or sent.
static void city_identity_check (int phase, const char *identity)
{
	char
		first [CAMPAIGN_ROSTER_KEY_MAX + 2];

	const char
		*second;

	int
		first_length,
		has_second,
		wants_second;

	size_t
		length;

	length = strlen (identity);

	if (length > CAMPAIGN_MODEL_SCAN_IDENTITY_MAX)
	{
		debug_fatal ("host model: [model-provider:entity_scan.request] the identity place of a city event of phase %d holds %d bytes, outside the request's identity field of 1 to %d", phase, (int) length, CAMPAIGN_MODEL_SCAN_IDENTITY_MAX);
	}

	has_second = city_split (identity, &first_length);

	wants_second = (phase == CAMPAIGN_SCAN_PHASE_CHILD_REMOVE) || (phase == CAMPAIGN_SCAN_PHASE_CHILD_NEXT) || (phase == CAMPAIGN_SCAN_PHASE_LINK) || (phase == CAMPAIGN_SCAN_PHASE_NAME);

	if ((first_length < 1) || (first_length > CAMPAIGN_ROSTER_KEY_MAX))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the identity place of a city event of phase %d begins with a key of %d bytes", phase, first_length);
	}

	memcpy (first, identity, (size_t) first_length);

	first [first_length] = 0;

	if (!roster_key_is_valid (first))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the key \"%.40s\" of a city event of phase %d is not a roster key", first, phase);
	}

	if ((phase == CAMPAIGN_SCAN_PHASE_CHILD_FIRST) && has_second)
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] a request for the first building of a city names the city only");
	}

	if ((phase == CAMPAIGN_SCAN_PHASE_CITY_RELEASE) && has_second)
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the release of a city names the key of its record only");
	}

	if (wants_second && !has_second)
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the identity place of a city event of phase %d names one key where it needs two", phase);
	}

	if (has_second)
	{
		second = identity + first_length + 1;

		if (phase == CAMPAIGN_SCAN_PHASE_NAME)
		{
			if (second [0] == 0)
			{
				debug_fatal ("host model: [model-provider:entity_scan.request] a NAME carries an empty object identity");
			}

			if (strlen (second) > CAMPAIGN_MODEL_OBJECT_ID_MAX)
			{
				debug_fatal ("host model: [model-provider:entity_scan.request] a NAME carries an object identity of %d bytes, outside its domain of 1 to %d", (int) strlen (second), CAMPAIGN_MODEL_OBJECT_ID_MAX);
			}

			{
				const unsigned char
					*byte;

				for (byte = (const unsigned char *) second; *byte; byte++)
				{
					if ((*byte < 0x20) || (*byte == 0x7f))
					{
						debug_fatal ("host model: [model-provider:entity_scan.request] a NAME carries an object identity with a control character");
					}
				}
			}
		}
		else if (!roster_key_is_valid (second))
		{
			debug_fatal ("host model: [model-provider:entity_scan.key] the second key \"%.40s\" of a city event of phase %d is not a roster key", second, phase);
		}
	}
}

static void check_city_reply (const campaign_model_reply *reply, int phase, const char *identity)
{
	int
		matches,
		wanted;

	const char
		*space;

	wanted = ((phase == CAMPAIGN_SCAN_PHASE_CHILD_REMOVE) || (phase == CAMPAIGN_SCAN_PHASE_CITY_RELEASE)) ? CAMPAIGN_MODEL_SCAN_REMOVED : (((phase == CAMPAIGN_SCAN_PHASE_CHILD_FIRST) || (phase == CAMPAIGN_SCAN_PHASE_CHILD_NEXT)) ? CAMPAIGN_MODEL_SCAN_NEXT : CAMPAIGN_MODEL_SCAN_PLACED);

	if (reply->candidate_state != wanted)
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a city event of phase %d, which allows only state %d", reply->candidate_state, phase, wanted);
	}

	check_reply_key (reply, "entity_scan");

	if (phase == CAMPAIGN_SCAN_PHASE_CHILD_INSERT)
	{
		if (reply->name [0] == 0)
		{
			debug_fatal ("host model: [model-provider:entity_scan.key] the host issued no key for a city insertion");
		}
	}
	else if ((phase == CAMPAIGN_SCAN_PHASE_LINK) || (phase == CAMPAIGN_SCAN_PHASE_NAME))
	{
		// the host echoes the key it linked (the child's, the second) or named (the placement's, the first): the campaign never accepts an acknowledgement of another entry
		space = strchr (identity, ' ');

		if (phase == CAMPAIGN_SCAN_PHASE_LINK)
		{
			matches = (strcmp (reply->name, space + 1) == 0);
		}
		else
		{
			matches = (strlen (reply->name) == (size_t) (space - identity)) && (strncmp (reply->name, identity, (size_t) (space - identity)) == 0);
		}

		if (!matches)
		{
			debug_fatal ("host model: [model-provider:entity_scan.key] the host acknowledged the key \"%.40s\" of a city event of phase %d, which names another entry", reply->name, phase);
		}
	}
	else if ((phase == CAMPAIGN_SCAN_PHASE_CHILD_REMOVE) && (reply->name [0] != 0))
	{
		// the host may name the city record this removal leaves kept for the box it built: the list the removal was asked about, never another
		space = strchr (identity, ' ');

		if ((strlen (reply->name) != (size_t) (space - identity)) || (strncmp (reply->name, identity, (size_t) (space - identity)) != 0))
		{
			debug_fatal ("host model: [model-provider:entity_scan.key] the host named the record \"%.40s\" for a city removal, which names another list", reply->name);
		}
	}
	else if ((phase == CAMPAIGN_SCAN_PHASE_CITY_RELEASE) && (reply->name [0] != 0))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the host issued a key for the release of a city");
	}

	if ((!scan_word_is_zero (&reply->candidate [0])) || (!scan_word_is_zero (&reply->candidate [1])))
	{
		debug_fatal ("host model: [model-provider:entity_scan.cursor] the host named a sector for a city event, which carries none");
	}
}

static void campaign_entity_city_exchange (int phase, int entity_type, const char *identity, char *key)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	city_identity_check (phase, identity);

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) phase;

	if (phase == CAMPAIGN_SCAN_PHASE_CHILD_INSERT)
	{
		request.pose [CAMPAIGN_ROSTER_TYPE] = (float) entity_type;
	}

	memcpy (request.host_id, identity, strlen (identity));

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_city_reply (&reply, phase, identity);

	memcpy (key, reply.name, strlen (reply.name) + 1);
}

// The FIXED_STATE report (the same question; phase CAMPAIGN_SCAN_PHASE_FIXED_STATE, Task49; see mb_hostmodel.h): the campaign states the collision state of the fixed-family placement `key` names. The key is validated
// HERE as a roster key before anything is sent, and the host's reply is checked completely: the state `placed`, the same key acknowledged (never another placement's), and no sector.
static void check_fixed_reply (const campaign_model_reply *reply, const char *key, const char *report)
{
	if (reply->candidate_state != CAMPAIGN_MODEL_SCAN_PLACED)
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a %s, which allows only state %d", reply->candidate_state, report, CAMPAIGN_MODEL_SCAN_PLACED);
	}

	check_reply_key (reply, "entity_scan");

	if (strcmp (reply->name, key) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the host acknowledged the key \"%.40s\" of a %s, which names another placement", reply->name, report);
	}

	if ((!scan_word_is_zero (&reply->candidate [0])) || (!scan_word_is_zero (&reply->candidate [1])))
	{
		debug_fatal ("host model: [model-provider:entity_scan.cursor] the host named a sector for a %s, which carries none", report);
	}
}

static void campaign_entity_fixed_exchange (int entity_type, const char *key, const float *state)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	if (!roster_key_is_valid (key))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] a fixed state report names \"%.40s\", which is not a roster key", key);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) CAMPAIGN_SCAN_PHASE_FIXED_STATE;
	memcpy (&request.pose [CAMPAIGN_FIXED_POSITION], &state [0], 3 * sizeof (float));
	request.pose [CAMPAIGN_FIXED_TYPE] = (float) entity_type;
	request.pose [CAMPAIGN_FIXED_HEADING] = state [3];
	request.pose [CAMPAIGN_FIXED_SHAPE] = state [4];
	request.pose [CAMPAIGN_FIXED_SUBTYPE] = state [5];

	memcpy (request.host_id, key, strlen (key));

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_fixed_reply (&reply, key, "fixed state report");
}

// The CITY_STATE report (the same question; phase CAMPAIGN_SCAN_PHASE_CITY_STATE, Task49; see mb_hostmodel.h): the campaign states the position and bounding box of the city placement `key` names, as the entity holds the
// words. The key is validated HERE as a roster key before anything is sent, and the host's reply is checked completely, as a fixed state report's is.
static void campaign_entity_city_state_exchange (const char *key, const float *state)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	if (!roster_key_is_valid (key))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] a city state report names \"%.40s\", which is not a roster key", key);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) CAMPAIGN_SCAN_PHASE_CITY_STATE;
	memcpy (&request.pose [CAMPAIGN_CITY_POSITION], &state [0], 3 * sizeof (float));
	memcpy (&request.pose [CAMPAIGN_CITY_BOX_MIN], &state [3], 3 * sizeof (float));
	memcpy (&request.pose [CAMPAIGN_CITY_BOX_MAX], &state [6], 3 * sizeof (float));
	request.pose [CAMPAIGN_CITY_SUBTYPE] = state [9];
	request.pose [CAMPAIGN_CITY_SHAPE] = state [10];

	memcpy (request.host_id, key, strlen (key));

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_fixed_reply (&reply, key, "city state report");
}

// The CITY_EXPAND question (the same question; phase CAMPAIGN_SCAN_PHASE_CITY_EXPAND, Task49; see mb_hostmodel.h): a building's link expanded its city's bounding box and the host builds it. The question states nothing
// but its phase and the identity place, which is the city's sector key, one space and the building's key (at most 2 * CAMPAIGN_ROSTER_KEY_MAX + 1 bytes, inside the host identity's domain); both are validated HERE as roster keys
// before anything is sent. The reply is checked COMPLETELY before one word of the caller's `box` is written: the state EXPANDED, the city's key echoed and nothing else named, exactly CAMPAIGN_CITY_EXPAND_WORDS words (and no
// word beyond), every word a finite number classified from its bytes, and then each axis of the box with its minimum at or below its maximum (an expanded box always is: the original unions both corners of the
// building's rotated bounds into both the minimum and the maximum; the inverted box of a city with no building is the INITIAL state the CITY_STATE report carries, not an expanded one). A reply that fails any of it stops
// the engine and `box` is as it was; the checks of the campaign's own mirror in ct_msgs.c are ASSERTs, which a shipping build only logs and which run after the words are written, so this is where the order is enforced.

// the order of a finite word among the finite numbers, from its bytes: the sign and magnitude become one signed key, so the two zeros are the same number and no floating value is loaded or compared
static int finite_word_order (const float *value)
{
	uint32_t
		word;

	int32_t
		magnitude;

	memcpy (&word, value, sizeof (word));

	magnitude = (int32_t) (word & 0x7fffffffu);

	return ((word & 0x80000000u) != 0 ? -magnitude : magnitude);
}

static void check_city_expand_reply (const campaign_model_reply *reply, const char *city)
{
	int
		index;

	if (reply->candidate_state != CAMPAIGN_MODEL_SCAN_EXPANDED)
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a city expansion, which allows only state %d", reply->candidate_state, CAMPAIGN_MODEL_SCAN_EXPANDED);
	}

	check_reply_key (reply, "entity_scan");

	if (strcmp (reply->name, city) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] the host acknowledged the key \"%.40s\" for a city expansion, which names another city", reply->name);
	}

	if (reply->candidate_count != CAMPAIGN_CITY_EXPAND_WORDS)
	{
		debug_fatal ("host model: [model-provider:entity_scan.box] the host answered %d words for a city expansion, which carries %d", reply->candidate_count, CAMPAIGN_CITY_EXPAND_WORDS);
	}

	for (index = 0; index < CAMPAIGN_CITY_EXPAND_WORDS; index++)
	{
		if (!candidate_word_is_finite (&reply->candidate [index]))
		{
			debug_fatal ("host model: [model-provider:entity_scan.box] the box word %d the host answered for a city expansion is not a finite number", index);
		}
	}

	for (index = CAMPAIGN_CITY_EXPAND_WORDS; index < CAMPAIGN_MODEL_CANDIDATE_POSE_MAX; index++)
	{
		if (!scan_word_is_zero (&reply->candidate [index]))
		{
			debug_fatal ("host model: [model-provider:entity_scan.box] the host answered more than %d words for a city expansion", CAMPAIGN_CITY_EXPAND_WORDS);
		}
	}

	for (index = 0; index < 3; index++)
	{
		if (finite_word_order (&reply->candidate [index]) > finite_word_order (&reply->candidate [index + 3]))
		{
			debug_fatal ("host model: [model-provider:entity_scan.box] the box axis %d the host answered for a city expansion has its minimum above its maximum", index);
		}
	}
}

static void campaign_entity_city_expand_exchange (const char *city, const char *building, float *box)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	if (!roster_key_is_valid (city))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] a city expansion names the city \"%.40s\", which is not a roster key", city);
	}

	if (!roster_key_is_valid (building))
	{
		debug_fatal ("host model: [model-provider:entity_scan.key] a city expansion names the building \"%.40s\", which is not a roster key", building);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) CAMPAIGN_SCAN_PHASE_CITY_EXPAND;

	memcpy (request.host_id, city, strlen (city));
	request.host_id [strlen (city)] = ' ';
	memcpy (request.host_id + strlen (city) + 1, building, strlen (building));

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_city_expand_reply (&reply, city);

	memcpy (box, reply.candidate, CAMPAIGN_CITY_EXPAND_WORDS * sizeof (float));
}

// The SECTOR HEIGHT read and restore (the same question; phases CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT and CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE, Task49; see mb_hostmodel.h). The sector is stated by its whole-number coordinates (a
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
		debug_fatal ("host model: [model-provider:entity_scan.key] the host named the placement \"%.40s\" for a sector height, which names none", reply->name);
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

// The PHYSICAL FLAG STORE (phases CAMPAIGN_SCAN_PHASE_FLAG_GROUP to _FLAG_RESET, Task49; see mb_hostmodel.h): the dialogue only. The host owns the three per-entity words; the campaign names a group by the opaque key the host issued and an entity by its
// lane. The reply of every phase is checked COMPLETELY before one word of the caller's is written: the state the phase allows, a key only for a group (and then a flag key, "f" and decimal digits, which is never a roster key), exactly
// CAMPAIGN_FLAG_REPLY_WORDS words and none beyond, the array and lane echoed bit for bit as asked, and the raw word of a set or a read being the entity's own mask (a read: 0 or the mask). The raw word is read as bits and never
// classified as a boolean, so lane 31's 0x80000000 is valid. Nothing here reads an entity or a heap slot.
// The CANONICAL flag key domain (ENV-PHYSICAL-FLAGS-KEY-CAPACITY-1): `f`, then one to CAMPAIGN_ROSTER_KEY_MAX - 1 decimal digits whose first is not zero (so f0 and f01 are refused, as the host's own grammar refuses them), that is at most
// CAMPAIGN_ROSTER_KEY_MAX bytes before the terminating NUL, which is the size of every identity slot the campaign holds (mb_hostmodel.c: char [CAMPAIGN_ROSTER_KEY_MAX + 1]). The whole domain is decided here, BEFORE any key is copied anywhere: a key of
// CAMPAIGN_ROSTER_KEY_MAX + 1 bytes is refused, and so is anything longer (the bridge may carry up to 127 bytes of name for every family; the flag key is the narrower one). The scan never reads past the byte that decides.
static int flag_key_is_valid (const char *key)
{
	int
		index;

	if ((key [0] != 'f') || (key [1] < '1') || (key [1] > '9'))
	{
		return FALSE;
	}

	for (index = 2; key [index] != 0; index++)
	{
		if ((key [index] < '0') || (key [index] > '9') || (index >= CAMPAIGN_ROSTER_KEY_MAX))
		{
			return FALSE;
		}
	}

	return TRUE;
}

static void check_flag_reply (const campaign_model_reply *reply, int phase, const float *asked, int lane)
{
	int
		index,
		wanted;

	uint32_t
		raw,
		mask;

	switch (phase)
	{
		case CAMPAIGN_SCAN_PHASE_FLAG_GROUP: wanted = CAMPAIGN_MODEL_SCAN_FLAG_GROUP; break;
		case CAMPAIGN_SCAN_PHASE_FLAG_SET: wanted = CAMPAIGN_MODEL_SCAN_FLAG_SET; break;
		case CAMPAIGN_SCAN_PHASE_FLAG_READ: wanted = CAMPAIGN_MODEL_SCAN_FLAG_READ; break;
		default: wanted = CAMPAIGN_MODEL_SCAN_FLAG_RESET; break;
	}

	if (reply->candidate_state != wanted)
	{
		debug_fatal ("host model: [model-provider:entity_scan.state] the host decided state %d for a flag question of phase %d, which allows only state %d", reply->candidate_state, phase, wanted);
	}

	if (memchr (reply->name, 0, sizeof (reply->name)) == NULL)
	{
		debug_fatal ("host model: [model-provider:flag.key] the key the host named does not end inside the reply");
	}

	if (phase == CAMPAIGN_SCAN_PHASE_FLAG_GROUP)
	{
		if (!flag_key_is_valid (reply->name))
		{
			debug_fatal ("host model: [model-provider:flag.key] the key \"%.40s\" the host issued for a flag group is not a flag key", reply->name);
		}
	}
	else if (reply->name [0] != 0)
	{
		debug_fatal ("host model: [model-provider:flag.key] the host named \"%.40s\" for a flag question that names no new key", reply->name);
	}

	if (reply->candidate_count != CAMPAIGN_FLAG_REPLY_WORDS)
	{
		debug_fatal ("host model: [model-provider:flag.answer] the host answered %d words for a flag question, which carries %d", reply->candidate_count, CAMPAIGN_FLAG_REPLY_WORDS);
	}

	if (phase == CAMPAIGN_SCAN_PHASE_FLAG_GROUP)
	{
		if (memcmp (&reply->candidate [0], &asked [0], sizeof (float)) != 0)
		{
			debug_fatal ("host model: [model-provider:flag.answer] the host answered a flag group with another eligibility than the one stated");
		}

		if (!scan_word_is_zero (&reply->candidate [1]) || !scan_word_is_zero (&reply->candidate [2]))
		{
			debug_fatal ("host model: [model-provider:flag.answer] the host answered more than the eligibility for a flag group");
		}
	}
	else
	{
		if (memcmp (&reply->candidate [0], &asked [0], sizeof (float)) != 0)
		{
			debug_fatal ("host model: [model-provider:flag.answer] the host answered for another flag word than the one asked about");
		}

		if (phase == CAMPAIGN_SCAN_PHASE_FLAG_RESET)
		{
			// the number of groups cleared is the host's own fact and is not judged; the third place carries nothing
			if (!scan_word_is_zero (&reply->candidate [2]))
			{
				debug_fatal ("host model: [model-provider:flag.answer] the host answered more than the array and a count for a flag reset");
			}
		}
		else
		{
			if (memcmp (&reply->candidate [1], &asked [1], sizeof (float)) != 0)
			{
				debug_fatal ("host model: [model-provider:flag.answer] the host answered for another flag lane than the one asked about");
			}

			memcpy (&raw, &reply->candidate [2], sizeof (raw));

			mask = ((uint32_t) 1) << lane;

			if ((raw != mask) && ((phase == CAMPAIGN_SCAN_PHASE_FLAG_SET) || (raw != 0)))
			{
				debug_fatal ("host model: [model-provider:flag.word] the host answered the raw word 0x%08x for lane %d of a flag %s, which is %s", (unsigned int) raw, lane, (phase == CAMPAIGN_SCAN_PHASE_FLAG_SET) ? "set" : "read", (phase == CAMPAIGN_SCAN_PHASE_FLAG_SET) ? "not the lane's mask" : "neither 0 nor the lane's mask");
			}
		}
	}

	for (index = CAMPAIGN_FLAG_REPLY_WORDS; index < CAMPAIGN_MODEL_CANDIDATE_POSE_MAX; index++)
	{
		if (!scan_word_is_zero (&reply->candidate [index]))
		{
			debug_fatal ("host model: [model-provider:flag.answer] the host answered more than %d words for a flag question", CAMPAIGN_FLAG_REPLY_WORDS);
		}
	}

}

// `key` names the group (a SET or a READ) and is NULL otherwise; `eligible` is a group's statement (FLAG_GROUP only); `lane` is 0 for a reset or a group. `issued` receives the key a FLAG_GROUP answered, `word` the raw word of a read, both
// only after the whole answer was checked.
static void campaign_entity_flag_exchange (int phase, int array, int lane, const char *key, int eligible, char *issued, uint32_t *word)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	float
		asked [2];

	if (phase != CAMPAIGN_SCAN_PHASE_FLAG_GROUP)
	{
		if ((array < 0) || (array >= CAMPAIGN_FLAG_ARRAYS))
		{
			debug_fatal ("host model: [model-provider:flag.array] a flag question names the array %d, which is not 0 to %d", array, CAMPAIGN_FLAG_ARRAYS - 1);
		}
	}

	if ((phase == CAMPAIGN_SCAN_PHASE_FLAG_SET) || (phase == CAMPAIGN_SCAN_PHASE_FLAG_READ))
	{
		if ((lane < 0) || (lane > 31))
		{
			debug_fatal ("host model: [model-provider:flag.lane] a flag question names the lane %d, which is not 0 to 31", lane);
		}

		if ((key == NULL) || !flag_key_is_valid (key))
		{
			debug_fatal ("host model: [model-provider:flag.key] a flag question names no group the host issued");
		}
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_SCAN;
	request.scene = 0;
	request.pose_count = CAMPAIGN_MODEL_SCAN_FACTS;
	request.pose [CAMPAIGN_SCAN_PHASE] = (float) phase;

	asked [0] = (phase == CAMPAIGN_SCAN_PHASE_FLAG_GROUP) ? (float) (eligible ? 1 : 0) : (float) array;
	asked [1] = (float) lane;

	if (phase == CAMPAIGN_SCAN_PHASE_FLAG_GROUP)
	{
		request.pose [CAMPAIGN_FLAG_ELIGIBLE] = asked [0];
	}
	else
	{
		request.pose [CAMPAIGN_FLAG_ARRAY] = asked [0];

		if (phase != CAMPAIGN_SCAN_PHASE_FLAG_RESET)
		{
			request.pose [CAMPAIGN_FLAG_LANE] = asked [1];

			memcpy (request.host_id, key, strlen (key));
		}
	}

	CAMPAIGN_ENTITY_SCAN_ASK (&request, &reply);

	check_flag_reply (&reply, phase, asked, lane);

	if (issued != NULL)
	{
		memcpy (issued, reply.name, strlen (reply.name) + 1);
	}

	if (word != NULL)
	{
		memcpy (word, &reply.candidate [2], sizeof (uint32_t));
	}
}

// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
// ENTITY_POSE (EEDCS-49, D3, fifth slice): an OBSERVATION of a mobile candidate's pose against the host's record of its identity. The answer chooses no pose and no result; its shape is checked and a mirror difference stops the engine (a conformance assertion, not a decision-neutral read).
// ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

// Everything about an observation's answer that can be malformed. `named` is whether the question carried a host identity and `stage` which words it carried. The answer is never used to choose a pose or a result (the mirror difference is asserted, not decided from), but a host
// that answers with a class, a kind or a difference mask outside the contract is a host that cannot be trusted to observe, and stops the engine by marker. The mask is checked against what the stage could
// have compared: STAGE_OFFER carries no attitude, so an attitude difference is impossible there, and a held motion vector that differs contradicts "no motion vector held".
static void check_pose_reply (const campaign_model_reply *reply, int named, int stage)
{
	if ((reply->candidate_state < CAMPAIGN_MODEL_POSE_RECORDED) || (reply->candidate_state > CAMPAIGN_MODEL_POSE_NO_ID))
	{
		debug_fatal ("host model: [model-provider:entity_pose.state] the host classified a candidate as %d, which is not one of the five", reply->candidate_state);
	}

	if ((reply->candidate_state == CAMPAIGN_MODEL_POSE_NO_ID) == named)
	{
		debug_fatal ("host model: [model-provider:entity_pose.state] the host classified a candidate as %d, which contradicts the question (identity named: %d)", reply->candidate_state, named);
	}

	if ((reply->candidate_count < 0) || (reply->candidate_count > 5) || (reply->write_mask < 0) || (reply->write_mask > 31))
	{
		debug_fatal ("host model: [model-provider:entity_pose.answer] the host's record kind %d or difference mask %d is outside 0 to 5 and 0 to 31", reply->candidate_count, reply->write_mask);
	}

	if ((reply->candidate_state != CAMPAIGN_MODEL_POSE_RECORDED) && ((reply->candidate_count != 0) || (reply->write_mask != 0)))
	{
		debug_fatal ("host model: [model-provider:entity_pose.answer] the host gave a record kind %d and differences %d for class %d, which has no record", reply->candidate_count, reply->write_mask, reply->candidate_state);
	}

	if ((reply->candidate_state == CAMPAIGN_MODEL_POSE_RECORDED) && (reply->candidate_count == 0))
	{
		debug_fatal ("host model: [model-provider:entity_pose.answer] the host reports a record of no kind");
	}

	if ((stage == CAMPAIGN_POSE_STAGE_OFFER) && ((reply->write_mask & CAMPAIGN_POSE_DIFFERS_ATTITUDE) != 0))
	{
		debug_fatal ("host model: [model-provider:entity_pose.answer] the host reports an attitude difference (mask %d) for the offer stage, which carries no attitude", reply->write_mask);
	}

	if (((reply->write_mask & CAMPAIGN_POSE_DIFFERS_MOTION) != 0) && ((reply->write_mask & CAMPAIGN_POSE_NO_HELD_MOTION) != 0))
	{
		debug_fatal ("host model: [model-provider:entity_pose.answer] the host reports a difference from a held motion vector and that none is held (mask %d)", reply->write_mask);
	}

	// The CONFORMANCE assertion: the host's physical record decides the candidate's pose, and the campaign's words (its mirror) are compared with it. A position, attitude or motion vector that is not the host's is a
	// mirror the host did not make (a write that bypassed the host): it stops the engine by name. The other bits only describe the record and decide nothing.
	if ((reply->write_mask & (CAMPAIGN_POSE_DIFFERS_POSITION | CAMPAIGN_POSE_DIFFERS_ATTITUDE | CAMPAIGN_POSE_DIFFERS_MOTION)) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_pose.mirror] the campaign's words for a mobile candidate differ from the host's physical record (difference mask %d: 1 position, 2 attitude, 4 motion vector)", reply->write_mask);
	}
}

// One observation: `words` are CAMPAIGN_MODEL_POSE_FACTS floats (see mb_hostmodel.h), sent as the bytes the entity held. Returns the class the host answered. The identity is carried whole: the roster's domain
// is 1 to CAMPAIGN_MODEL_HOST_ID_MAX bytes, so a longer one cannot exist and is refused by marker, never shortened or skipped.
static int campaign_entity_pose_exchange (int stage, int entity_type, int alive, const char *host_id, const float *words)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	size_t
		length;

	length = (host_id != NULL) ? strlen (host_id) : 0;

	if (length > CAMPAIGN_MODEL_HOST_ID_MAX)
	{
		debug_fatal ("host model: [model-provider:entity_pose.request] a host identity of %d bytes is outside the roster's domain of 1 to %d", (int) length, CAMPAIGN_MODEL_HOST_ID_MAX);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_POSE;
	request.scene = 0;
	request.depth = stage;
	request.kind = entity_type;
	request.definition = alive;
	request.pose_count = CAMPAIGN_MODEL_POSE_FACTS;
	memcpy (&request.pose [0], words, CAMPAIGN_MODEL_POSE_FACTS * sizeof (float));

	if (length > 0)
	{
		memcpy (request.host_id, host_id, length);
	}

	CAMPAIGN_ENTITY_POSE_ASK (&request, &reply);

	check_pose_reply (&reply, (length > 0), stage);

	return (reply.candidate_state);
}

#endif

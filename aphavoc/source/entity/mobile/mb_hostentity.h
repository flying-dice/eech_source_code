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

// whether a float is a finite number, read as bytes (an infinity or a NaN has every exponent bit set)
static int candidate_word_is_finite (float value)
{
	uint32_t
		word;

	memcpy (&word, &value, sizeof (word));

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

	if ((state < CAMPAIGN_MODEL_CANDIDATE_SKIP) || (state > CAMPAIGN_MODEL_CANDIDATE_TEST))
	{
		debug_fatal ("host model: [model-provider:entity_candidate.state] the host decided state %d for scene %d, which is not one of the five", state, scene);
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

	if (state == CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL)
	{
		if ((facts [CAMPAIGN_CANDIDATE_ALIGN] == 0.0f) || (facts [CAMPAIGN_CANDIDATE_NORMAL_GIVEN] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_MOBILE] != 0.0f) || (facts [CAMPAIGN_CANDIDATE_FIXED] == 0.0f))
		{
			debug_fatal ("host model: [model-provider:entity_candidate.state] the host asked for the terrain face normal of scene %d, which is not a fixed candidate aligned with the terrain whose normal was not given yet", scene);
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

		for (index = 0; index < count; index++)
		{
			if (!candidate_word_is_finite (reply->candidate [index]))
			{
				debug_fatal ("host model: [model-provider:entity_candidate.pose] float %d of the pose the host gave for scene %d is not a finite number", index, scene);
			}
		}
	}
}

// One question about one candidate: the facts in, the host's checked decision out. `scene` is the candidate's scene, and 0 for a city. Returns a campaign_model_candidate_state and, for TEST,
// the narrow phase's pose in `pose` (`*pose_count` floats).
static int campaign_entity_candidate_exchange (int scene, const float *facts, float *pose, int *pose_count)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	int
		index;

	if (facts [CAMPAIGN_CANDIDATE_CITY] != 0.0f)
	{
		if (scene != 0)
		{
			debug_fatal ("host model: [model-provider:entity_candidate.request] a city has no scene of its own, but scene %d was named", scene);
		}
	}
	else
	{
		CAMPAIGN_ENTITY_REQUIRE_SCENE (scene);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_ENTITY_CANDIDATE;
	request.scene = scene;
	request.pose_count = CAMPAIGN_MODEL_CANDIDATE_FACTS;
	memcpy (&request.pose [0], facts, CAMPAIGN_MODEL_CANDIDATE_FACTS * sizeof (float));

	CAMPAIGN_ENTITY_CANDIDATE_ASK (&request, &reply);

	if (facts [CAMPAIGN_CANDIDATE_CITY] != 0.0f)
	{
		if (reply.name [0] != 0)
		{
			debug_fatal ("host model: [model-provider:entity_candidate.identity] the host named a city \"%.60s\", which has no scene", reply.name);
		}
	}
	else
	{
		CAMPAIGN_ENTITY_IDENTITY (&reply, scene);
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

#endif

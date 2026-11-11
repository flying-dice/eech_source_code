/*
 * The dialogue of the host-owned tree segment query (EEDCS-49, D3): see mb_hostterrain.h, TREE_BEGIN and TREE_TERRAIN.  This is the CAMPAIGN'S half of the exchange and nothing else:
 * it sends the caller's words, answers each terrain elevation the host asks for with the campaign's own (host-backed) terrain call, checks every reply completely and only then writes
 * the caller's buffers.  It holds no tree data, scan or intersection.
 *
 * It is a header, included by mb_hostterrain.c (which binds it to the real host callback and the real terrain call) and by the isolated control csrc/eech_tree_dialogue_check.c
 * (which binds it to a scripted host), so that the control exercises THIS code and not a copy of it.  The includer provides: vec3d, campaign_terrain_request and _reply, FALSE,
 * TRUE, ASSERT, debug_fatal, isfinite, memcpy, memset, and
 *
 *   CAMPAIGN_TREE_ASK (request, reply, what)   one host exchange, which stops the engine on a missing provider or a refusal
 *   CAMPAIGN_TREE_ELEVATION (x, z)             the campaign's terrain elevation at (x, z), with its shared-record side effect
 */
#ifndef MB_HOSTTREE_H
#define MB_HOSTTREE_H

// the original reads the terrain elevation of the OLD end only when the old end is inside the map, and of the NEW end only when the new end is, in that order: a query asks for
// none, the old end only, the new end only (a segment that enters the map), or the old end and then the new end.  An end is asked at most once and the old end is never asked after
// the new one, so a third question is never possible; the bits below say which ends have been asked.
#define CAMPAIGN_TREE_ASKED_OLD 1
#define CAMPAIGN_TREE_ASKED_NEW 2

static int tree_query_open;

// the caller's floats are copied as bytes, never loaded as floating point values: a signed zero or a NaN in a buffer the caller never initialised reaches the host, and comes back, as it is
static void tree_words_of (uint32_t *words, const vec3d *position)
{
	memcpy (&words[0], &position->x, sizeof (uint32_t));
	memcpy (&words[1], &position->y, sizeof (uint32_t));
	memcpy (&words[2], &position->z, sizeof (uint32_t));
}

static void tree_words_into (vec3d *position, const uint32_t *words)
{
	memcpy (&position->x, &words[0], sizeof (uint32_t));
	memcpy (&position->y, &words[1], sizeof (uint32_t));
	memcpy (&position->z, &words[2], sizeof (uint32_t));
}

static float tree_float_of (uint32_t word)
{
	float
		value;

	memcpy (&value, &word, sizeof (value));

	return (value);
}

static void check_tree_reply (const campaign_terrain_reply *reply, int asked)
{
	switch (reply->tree_state)
	{
		case CAMPAIGN_TERRAIN_TREE_DONE:
		{
			if ((reply->tree_hit != FALSE) && (reply->tree_hit != TRUE))
			{
				debug_fatal ("host terrain: [terrain-provider:tree.answer] the host answered a tree query DONE without a boolean hit");
			}

			break;
		}
		case CAMPAIGN_TERRAIN_TREE_NEED_TERRAIN:
		{
			// the order is the original's: the old end before the new end, and neither twice
			if ((reply->tree_end != CAMPAIGN_TERRAIN_TREE_END_OLD) && (reply->tree_end != CAMPAIGN_TERRAIN_TREE_END_NEW))
			{
				debug_fatal ("host terrain: [terrain-provider:tree.lookups] the host asked for the elevation of end %d, which is neither the old end nor the new end", reply->tree_end);
			}

			if (asked & ((reply->tree_end == CAMPAIGN_TERRAIN_TREE_END_OLD) ? CAMPAIGN_TREE_ASKED_OLD : CAMPAIGN_TREE_ASKED_NEW))
			{
				debug_fatal ("host terrain: [terrain-provider:tree.lookups] the host asked for the elevation of the %s end twice in one tree query", (reply->tree_end == CAMPAIGN_TERRAIN_TREE_END_OLD) ? "old" : "new");
			}

			if ((reply->tree_end == CAMPAIGN_TERRAIN_TREE_END_OLD) && (asked & CAMPAIGN_TREE_ASKED_NEW))
			{
				debug_fatal ("host terrain: [terrain-provider:tree.lookups] the host asked for the elevation of the old end after the new end");
			}

			if (!isfinite (tree_float_of (reply->tree_x)) || !isfinite (tree_float_of (reply->tree_z)))
			{
				debug_fatal ("host terrain: [terrain-provider:tree.lookups] the host asked for the terrain elevation at a non-finite point");
			}

			break;
		}
		default:
		{
			debug_fatal ("host terrain: [terrain-provider:tree.answer] the host answered a tree query with the unknown state %d", reply->tree_state);
		}
	}
}

static int campaign_tree_dialogue
(
	const vec3d *old_position,
	const vec3d *new_position,
	vec3d *intercept_point,
	vec3d *face_normal,
	int ignore_forest_edge_trees,
	int skip_tree_top_test
)
{
	campaign_terrain_request
		request;

	campaign_terrain_reply
		reply;

	float
		elevation;

	int
		asked;

	ASSERT (old_position);

	ASSERT (new_position);

	ASSERT (intercept_point);

	ASSERT (face_normal);

	if (tree_query_open)
	{
		debug_fatal ("host terrain: [terrain-provider:tree.query] a tree query was started inside another");
	}

	tree_query_open = TRUE;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_TERRAIN_TREE_BEGIN;
	tree_words_of (request.tree_old, old_position);
	tree_words_of (request.tree_new, new_position);
	tree_words_of (request.tree_point, intercept_point);
	tree_words_of (request.tree_normal, face_normal);
	request.tree_ignore_forest_edge_trees = (ignore_forest_edge_trees != 0);
	request.tree_skip_tree_top_test = (skip_tree_top_test != 0);

	CAMPAIGN_TREE_ASK (&request, &reply, "tree begin");

	check_tree_reply (&reply, 0);

	asked = 0;

	while (reply.tree_state == CAMPAIGN_TERRAIN_TREE_NEED_TERRAIN)
	{
		// the campaign's half of the query: the same call, on the same host terrain and its shared record, that the original made at this point
		elevation = CAMPAIGN_TREE_ELEVATION (tree_float_of (reply.tree_x), tree_float_of (reply.tree_z));

		// the end the host asked for is now asked: the next question may be the other end, never this one again
		asked |= (reply.tree_end == CAMPAIGN_TERRAIN_TREE_END_OLD) ? CAMPAIGN_TREE_ASKED_OLD : CAMPAIGN_TREE_ASKED_NEW;

		memset (&request, 0, sizeof (request));

		request.op = CAMPAIGN_TERRAIN_TREE_TERRAIN;
		memcpy (&request.tree_elevation, &elevation, sizeof (uint32_t));

		CAMPAIGN_TREE_ASK (&request, &reply, "tree terrain");

		check_tree_reply (&reply, asked);
	}

	// the whole answer is checked: only now are the caller's buffers written
	tree_words_into (intercept_point, reply.tree_result_point);
	tree_words_into (face_normal, reply.tree_result_normal);

	tree_query_open = FALSE;

	return (reply.tree_hit);
}

#endif

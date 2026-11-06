/*
 * The campaign's half of host-owned collision-model data (EEDCS-50, D3): see mb_hostmodel.h. This file holds no model arithmetic and reads no model array: it
 * carries each of the four accessor questions to the host and returns what the host answered, in the order and with the buffer behaviour of the original accessors.
 * The bounds and radius are copied from the host's answers at boot into tables that nothing else writes (callers keep the bounds pointers across loops).
 *
 * Compiled into every build; the requests below exist only where EECH_CAMPAIGN_HOST_MODEL is defined (the shipping module and the host-model test variant). The
 * original-characterisation build (feature model-fixture) keeps the original accessors on purpose, to ask them what they answer, and has no host.
 */

#include "project.h"

#include <math.h>
#include <stdint.h>
#include <string.h>

#include "mb_hostmodel.h"

static campaign_model_host current_host;
static int host_installed;

int campaign_model_host_installed (void)
{
	return (host_installed);
}

void campaign_model_set_host (const campaign_model_host *host)
{
	// every entry that carries a callback refreshes it; an entry without one leaves it. The entry's `user` names that entry's own host context, which ends with the
	// entry, so it is never kept: the callback is always called with a NULL user (mb_hostmodel.h, "Lifecycle")
	if (host && host->model)
	{
		current_host.model = host->model;
		current_host.user = NULL;
		host_installed = TRUE;
	}
}

#ifdef EECH_CAMPAIGN_HOST_MODEL

// the host's scalars, one entry per scene index; written once at boot (campaign_model_install_scenes), never again
static struct OBJECT_3D_BOUNDS
	host_bounding_box [OBJECT_3D_LAST],
	host_bounding_box_without_lines [OBJECT_3D_LAST];

static float
	host_radius [OBJECT_3D_LAST];

// what the host declared about each scene (public, semantic): whether it has a model for it, and whether that model has a collider. A triangle answer is checked
// against these declarations, never against the campaign's own arrays.
static unsigned char
	host_loaded [OBJECT_3D_LAST],
	host_collider [OBJECT_3D_LAST];

static int tables_installed;

#ifdef EECH_MODEL_HOST_FIXTURE
// TEST-ONLY: how many questions of each kind were asked of the host, and what it answered
struct campaign_model_counters
{
	/* catalogue, scene, triangle: questions ASKED of the host. found, end, absent: triangle replies the native code ACCEPTED (a reply it refused stops the engine and is
	   not counted). bounds_reads, bounds_without_lines_reads, radius_reads: reads of the host-authored tables by the campaign. */
	long catalogue, scene, triangle, found, end, absent, bounds_reads, bounds_without_lines_reads, radius_reads;
	/* REFERENCE-ADAPTER DIAGNOSTIC, never an acceptance rule: scenes whose host-declared presence (loaded, collider) differs from what the campaign's own loaded arrays imply
	   (a scene has a main object; it has a collision object other than -1), and how many were compared. Taken at boot, before anything can poison the arrays. */
	long presence_checked, presence_mismatches;
	/* TARGET_POINT_OFFSET questions asked of the host (EEDCS-52); read by the target-offset diagnostic, not by the counters line */
	long target_offset;
	/* TARGET_POINT_CHAIN, _WORLD and _FIXED questions asked of the host (EEDCS-52); read by the target-point query diagnostic */
	long target_chain, target_world, target_fixed;
	/* ENTITY_COLLISION questions asked of the host (EEDCS-49) and the hits and misses the native code ACCEPTED */
	long entity_collision, entity_hit, entity_miss;
	/* ENTITY_CANDIDATE questions asked of the host (EEDCS-49) and the states the native code ACCEPTED: skip, UPDATED asked, terrain normal asked, buildings, narrow phase required */
	long entity_scan, scan_first, scan_next, scan_offer, scan_done;
	/* the roster's events of the sector lists (the same question, phases 4 to 6): entities placed, removed and the boots that cleared the roster. Not counted among the scan's own states above */
	long roster_placed, roster_removed, roster_cleared;
	/* the city events (the same question, phases 7 to 12): buildings listed and unlisted, the first and next entries the host served, the placements linked and the placements named */
	long city_placed, city_removed, city_first, city_next, city_linked, city_named;
	/* ENTITY_POSE observations asked of the host (EEDCS-49), and the classes it answered: recorded, unrecorded, retired, unknown, no identity */
	long entity_pose, pose_recorded, pose_unrecorded, pose_retired, pose_unknown, pose_no_id;
	long entity_candidate, candidate_skip, candidate_updated, candidate_normal, candidate_buildings, candidate_test, candidate_fixed, candidate_placement;
};

struct campaign_model_counters campaign_model_counts;
#endif

static void ask (const campaign_model_request *request, campaign_model_reply *reply, const char *what)
{
	char
		error[CAMPAIGN_MODEL_ERROR_MAX + 1];

	if (!host_installed)
	{
		debug_fatal ("host model: [model-provider:missing] no model provider is installed (%s)", what);
	}

	memset (reply, 0, sizeof (*reply));

	memset (error, 0, sizeof (error));

	if (current_host.model (request, reply, error, CAMPAIGN_MODEL_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host model: %s", error[0] ? error : "[model-provider:callback] the host model provider failed without a message");
	}
}

static int all_finite (const float *values, int count)
{
	int i;

	for (i = 0; i < count; i++)
	{
		if (!isfinite (values[i]))
		{
			return FALSE;
		}
	}

	return TRUE;
}

static void bounds_from (struct OBJECT_3D_BOUNDS *bounds, const float *values)
{
	bounds->xmin = values[0];
	bounds->xmax = values[1];
	bounds->ymin = values[2];
	bounds->ymax = values[3];
	bounds->zmin = values[4];
	bounds->zmax = values[5];
}

void campaign_model_install_scenes (void)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	int
		scene;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_MODEL_CATALOGUE;

	ask (&request, &reply, "catalogue");

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.catalogue++;
#endif

	if (reply.scene_count != OBJECT_3D_LAST)
	{
		debug_fatal ("host model: [model-provider:catalogue.mismatch] the host's models cover %d scenes but the campaign's scene database has %d", reply.scene_count, (int) OBJECT_3D_LAST);
	}

	for (scene = 0; scene < OBJECT_3D_LAST; scene++)
	{
		memset (&request, 0, sizeof (request));

		request.op = CAMPAIGN_MODEL_SCENE;
		request.scene = scene;

		ask (&request, &reply, "scene");

#ifdef EECH_MODEL_HOST_FIXTURE
		campaign_model_counts.scene++;
#endif

		if (!isfinite (reply.radius) || !all_finite (reply.bounding_box, 6) || !all_finite (reply.bounding_box_without_lines, 6) ||
			((reply.loaded != FALSE) && (reply.loaded != TRUE)) || ((reply.collider != FALSE) && (reply.collider != TRUE)) || (reply.collider && !reply.loaded) ||
			(memchr (reply.name, '\0', sizeof (reply.name)) == NULL))
		{
			debug_fatal ("host model: [model-provider:scene.answer] the host's answer for scene %d is malformed", scene);
		}

		// The public scene identity: the host and the campaign mean the same scene by the same index when the compiled scene name agrees (the invalid index 0 has none).
		// The legacy loader's own object numbering is private and is neither asked for nor compared; the physical values are the host's own and are never checked
		// against the campaign's arrays.
		{
			const char
				*expected = (object_3d_scene_names[scene]) ? object_3d_scene_names[scene] : "";

			if (strcmp (reply.name, expected) != 0)
			{
				debug_fatal ("host model: [model-provider:catalogue.identity] scene %d is named \"%.60s\" by the host but \"%.60s\" by the campaign", scene, reply.name, expected);
			}
		}

		// the null scene (the invalid index) is not a scene any host has: it is declared as the original's zeroed entry
		if ((scene == 0) && (reply.loaded || reply.collider))
		{
			debug_fatal ("host model: [model-provider:scene.answer] the host declared the invalid scene index 0 loaded");
		}

		host_radius[scene] = reply.radius;
		bounds_from (&host_bounding_box[scene], reply.bounding_box);
		bounds_from (&host_bounding_box_without_lines[scene], reply.bounding_box_without_lines);
		host_loaded[scene] = (unsigned char) reply.loaded;
		host_collider[scene] = (unsigned char) reply.collider;

#ifdef EECH_MODEL_HOST_FIXTURE
		// reference-adapter diagnostic only: does the host's declared presence agree with what the campaign's own loaded arrays imply? Counted, never enforced.
		campaign_model_counts.presence_checked++;

		if ((reply.loaded != (objects_3d_scene_database[scene].index != 0)) || (reply.collider != (reply.loaded && (objects_3d_scene_database[scene].collision_object_index != -1))))
		{
			campaign_model_counts.presence_mismatches++;
		}
#endif
	}

	tables_installed = TRUE;
}

static void require_scene (int scene, int allow_invalid_index, const char *what)
{
	if (!tables_installed)
	{
		debug_fatal ("host model: [model-provider:not.installed] %s was asked before the host's scenes were installed", what);
	}

	if ((scene < (allow_invalid_index ? 0 : 1)) || (scene >= OBJECT_3D_LAST))
	{
		debug_fatal ("host model: [model-provider:scene.index] %s was asked about scene %d, outside %d to %d", what, scene, allow_invalid_index ? 0 : 1, (int) OBJECT_3D_LAST - 1);
	}
}

// The host-authored LOCAL target-point offset of a scene (mb_hostmodel.h, TARGET_POINT_OFFSET). One question, one reply, no cache here: the caller copies the three coordinates
// into its own table. The reply must echo the scene's compiled name, be FOUND, and carry three finite floats; anything else stops the engine and nothing is invented.
void campaign_model_target_point_offset (int scene, vec3d *offset)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	const char
		*expected;

	require_scene (scene, FALSE, "a target point offset");

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_TARGET_POINT_OFFSET;
	request.scene = scene;

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.target_offset++;
#endif

	ask (&request, &reply, "target point offset");

	expected = object_3d_scene_names[scene] ? object_3d_scene_names[scene] : "";

	if ((memchr (reply.name, '\0', sizeof (reply.name)) == NULL) || (strcmp (reply.name, expected) != 0))
	{
		debug_fatal ("host model: [model-provider:target_point_offset.identity] the host named scene %d \"%.60s\" but the campaign names it \"%.60s\"", scene, reply.name, expected);
	}

	if (reply.result != CAMPAIGN_MODEL_TRIANGLE_FOUND)
	{
		debug_fatal ("host model: [model-provider:target_point_offset.result] the host gave no target point offset for scene %d (result %d)", scene, reply.result);
	}

	if (!all_finite (reply.points, 3))
	{
		debug_fatal ("host model: [model-provider:target_point_offset.value] the host's target point offset for scene %d is not finite", scene);
	}

	offset->x = reply.points[0];
	offset->y = reply.points[1];
	offset->z = reply.points[2];
}

// The three host questions behind get_local_entity_target_point (mb_hostmodel.h, TARGET_POINT_CHAIN / _WORLD / _FIXED; EEDCS-52, D3). They carry no policy: the caller (en_maths.c) decides
// which of its branches applies and what to do with the answer; these functions only package the current facts the host needs, ask, and refuse anything but a well-formed reply.
static void check_target_point_identity (const campaign_model_reply *reply, int scene, const char *what)
{
	const char
		*expected;

	expected = object_3d_scene_names[scene] ? object_3d_scene_names[scene] : "";

	if ((memchr (reply->name, '\0', sizeof (reply->name)) == NULL) || (strcmp (reply->name, expected) != 0))
	{
		debug_fatal ("host model: [model-provider:%s.identity] the host named scene %d \"%.60s\" but the campaign names it \"%.60s\"", what, scene, reply->name, expected);
	}
}

static void take_target_point_position (const campaign_model_reply *reply, int scene, const char *what, vec3d *world)
{
	check_target_point_identity (reply, scene, what);

	if (reply->result != CAMPAIGN_MODEL_TRIANGLE_FOUND)
	{
		debug_fatal ("host model: [model-provider:%s.result] the host gave no target point for scene %d (result %d)", what, scene, reply->result);
	}

	if (!all_finite (reply->points, 3))
	{
		debug_fatal ("host model: [model-provider:%s.value] the host's target point for scene %d is not finite", what, scene);
	}

	world->x = reply->points[0];
	world->y = reply->points[1];
	world->z = reply->points[2];
}

// The levels of the chain the host names for an instance's scene, 0 when the scene has no TARGET_POINT sub-object (the original's "not found" branch). The scene may be the
// invalid index 0 (an instance of it has no sub-objects; the host says so).
int campaign_model_target_point_chain (int scene, int *offsets)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	int
		level;

	require_scene (scene, TRUE, "a target point chain");

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_TARGET_POINT_CHAIN;
	request.scene = scene;

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.target_chain++;
#endif

	ask (&request, &reply, "target point chain");

	check_target_point_identity (&reply, scene, "target_point_chain");

	if (reply.result == CAMPAIGN_MODEL_TRIANGLE_ABSENT)
	{
		return 0;
	}

	if (reply.result != CAMPAIGN_MODEL_TRIANGLE_FOUND)
	{
		debug_fatal ("host model: [model-provider:target_point_chain.result] the host gave no target point chain for scene %d (result %d)", scene, reply.result);
	}

	if ((reply.chain_length < 1) || (reply.chain_length > CAMPAIGN_MODEL_CHAIN_MAX))
	{
		debug_fatal ("host model: [model-provider:target_point_chain.chain] the host's chain for scene %d has %d levels, outside 1 to %d", scene, reply.chain_length, CAMPAIGN_MODEL_CHAIN_MAX);
	}

	for (level = 0; level < reply.chain_length; level++)
	{
		if (reply.chain_offsets[level] < 0)
		{
			debug_fatal ("host model: [model-provider:target_point_chain.chain] the host's chain for scene %d has a negative position at level %d", scene, level);
		}

		offsets[level] = reply.chain_offsets[level];
	}

	return reply.chain_length;
}

// The world position of the target point the host named. The host needs the CURRENT placement, which the campaign alone holds: the instance's world position, attitude and relative
// scale, and the live relative position, heading, pitch, roll and scale of each sub-instance along the chain. They are read here, exactly as the original's world-viewpoint walk
// reads them, and sent as plain floats; the host does the arithmetic. The legacy tree is walked only along the positions the host named, and its own counts (in the scene database)
// are checked so that a chain the instance does not have stops the engine instead of reading outside it; nothing is searched.
void campaign_model_target_point_world (object_3d_instance *instance, const int *offsets, int levels, vec3d *world)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	object_3d_sub_instance
		*sub_instances,
		*node;

	object_3d_database_entry
		*scenes;

	int
		scene,
		level,
		count,
		at;

	float
		*pose;

	scene = instance->object_number;

	require_scene (scene, TRUE, "a target point world position");

	if ((levels < 1) || (levels > CAMPAIGN_MODEL_CHAIN_MAX))
	{
		debug_fatal ("host model: [model-provider:target_point_world.chain] a chain of %d levels was given for scene %d", levels, scene);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_TARGET_POINT_WORLD;
	request.scene = scene;
	request.pose_count = 15 + (9 * levels);

	pose = request.pose;
	pose[0] = instance->vp.position.x;
	pose[1] = instance->vp.position.y;
	pose[2] = instance->vp.position.z;
	memcpy (&pose[3], instance->vp.attitude, 9 * sizeof (float));
	pose[12] = instance->relative_scale.x;
	pose[13] = instance->relative_scale.y;
	pose[14] = instance->relative_scale.z;
	pose += 15;

	sub_instances = instance->sub_objects;
	scenes = objects_3d_scene_database[scene].sub_objects;
	count = objects_3d_scene_database[scene].number_of_sub_objects;

	for (level = 0; level < levels; level++)
	{
		at = offsets[level];

		if ((at < 0) || (at >= count) || (!sub_instances) || (!scenes))
		{
			debug_fatal ("host model: [model-provider:target_point_world.chain] the host's chain for scene %d names position %d at level %d, which the instance does not have", scene, at, level);
		}

		node = &sub_instances[at];

		pose[0] = node->relative_position.x;
		pose[1] = node->relative_position.y;
		pose[2] = node->relative_position.z;
		pose[3] = node->relative_heading;
		pose[4] = node->relative_pitch;
		pose[5] = node->relative_roll;
		pose[6] = node->relative_scale.x;
		pose[7] = node->relative_scale.y;
		pose[8] = node->relative_scale.z;
		pose += 9;

		count = scenes[at].number_of_sub_objects;
		scenes = scenes[at].sub_objects;
		sub_instances = node->sub_objects;
	}

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.target_world++;
#endif

	ask (&request, &reply, "target point world position");

	take_target_point_position (&reply, scene, "target_point_world", world);
}

// A fixed object's target point: the position the caller already copied, and the heading only when the caller read it (it does so when the shape's cached host offset has a
// horizontal part, as the original did). The shape is a valid (non-zero) scene; the caller keeps the invalid shape's bit-for-bit position copy.
void campaign_model_target_point_fixed (int shape, const vec3d *position, const float *heading, vec3d *world)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	require_scene (shape, FALSE, "a fixed target point");

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_TARGET_POINT_FIXED;
	request.scene = shape;
	request.pose[0] = position->x;
	request.pose[1] = position->y;
	request.pose[2] = position->z;
	request.pose_count = 3;

	if (heading)
	{
		request.pose[3] = *heading;
		request.pose_count = 4;
	}

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.target_fixed++;
#endif

	ask (&request, &reply, "fixed target point");

	take_target_point_position (&reply, shape, "target_point_fixed", world);
}

struct OBJECT_3D_BOUNDS *campaign_model_bounding_box (int scene)
{
	// the original indexed the zeroed entry of the invalid scene without complaint; the host authors that entry too (all zeros)
	require_scene (scene, TRUE, "get_object_3d_bounding_box");

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.bounds_reads++;
#endif

	return &host_bounding_box[scene];
}

struct OBJECT_3D_BOUNDS *campaign_model_bounding_box_without_lines (int scene)
{
	require_scene (scene, TRUE, "get_object_3d_bounding_box_without_lines");

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.bounds_without_lines_reads++;
#endif

	return &host_bounding_box_without_lines[scene];
}

float campaign_model_radius (int scene)
{
	require_scene (scene, TRUE, "get_object_3d_radius");

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.radius_reads++;
#endif

	return host_radius[scene];
}

int campaign_model_triangle (int scene, vec3d *points, int depth)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	require_scene (scene, FALSE, "get_object_3d_collision_object_geometry_triangle");

	if (!points)
	{
		debug_fatal ("host model: [model-provider:triangle.buffer] get_object_3d_collision_object_geometry_triangle was given no buffer");
	}

	if (depth < 0)
	{
		debug_fatal ("host model: [model-provider:triangle.depth] get_object_3d_collision_object_geometry_triangle was asked for depth %d", depth);
	}

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_MODEL_TRIANGLE;
	request.scene = scene;
	request.depth = depth;

	ask (&request, &reply, "triangle");

#ifdef EECH_MODEL_HOST_FIXTURE
	campaign_model_counts.triangle++;
#endif

	// The answer must agree with what the host itself declared about the scene at boot, BEFORE any word of the caller's buffer is written: a scene declared not loaded
	// has nothing to answer (the original would have dereferenced a NULL surface list), FOUND and END need a collider, ABSENT needs a loaded scene without one.
	if (!host_loaded[scene])
	{
		debug_fatal ("host model: [model-provider:triangle.answer] the host answered a triangle question for scene %d, which it declared not loaded", scene);
	}

	switch (reply.result)
	{
		case CAMPAIGN_MODEL_TRIANGLE_FOUND:
		{
			if (!host_collider[scene])
			{
				debug_fatal ("host model: [model-provider:triangle.answer] the host gave a triangle for scene %d, which it declared has no collider", scene);
			}

			if (!all_finite (reply.points, 9))
			{
				debug_fatal ("host model: [model-provider:triangle.answer] the host's triangle for scene %d depth %d is not finite", scene, depth);
			}

			// the caller's buffer is written only by a triangle, as the original wrote it only when it returned TRUE
			points[0].x = reply.points[0];
			points[0].y = reply.points[1];
			points[0].z = reply.points[2];
			points[1].x = reply.points[3];
			points[1].y = reply.points[4];
			points[1].z = reply.points[5];
			points[2].x = reply.points[6];
			points[2].y = reply.points[7];
			points[2].z = reply.points[8];

#ifdef EECH_MODEL_HOST_FIXTURE
			campaign_model_counts.found++;
#endif

			return TRUE;
		}
		case CAMPAIGN_MODEL_TRIANGLE_END:
		{
			// a collider is present and the list is over: the original's FALSE, the buffer untouched
			if (!host_collider[scene])
			{
				debug_fatal ("host model: [model-provider:triangle.answer] the host ended the triangles of scene %d, which it declared has no collider", scene);
			}

#ifdef EECH_MODEL_HOST_FIXTURE
			campaign_model_counts.end++;
#endif

			return FALSE;
		}
		case CAMPAIGN_MODEL_TRIANGLE_ABSENT:
		{
			// the scene has no collider: the original's FALSE without reading anything, the buffer untouched
			if (host_collider[scene])
			{
				debug_fatal ("host model: [model-provider:triangle.answer] the host answered that scene %d has no collider, but it declared one", scene);
			}

#ifdef EECH_MODEL_HOST_FIXTURE
			campaign_model_counts.absent++;
#endif

			return FALSE;
		}
		default:
		{
			debug_fatal ("host model: [model-provider:triangle.answer] the host's triangle result for scene %d depth %d is %d", scene, depth, reply.result);
		}
	}

	return FALSE;
}

// ---------------------------------------------------------------------------------------------------------------------------------
// ROUTE data (EEDCS-50, D3). The campaign's airfield route parser (aphavoc/source/ai/faction/routegen.c) used to decode its routes from the DLL's own model arrays: the stored
// short points, scaled by the object's bounds, and the line faces through the surface-point indirection. The host owns that decode now (mb_hostmodel.h, ROUTE_NODE and
// ROUTE_FACE). What stays here is the campaign's half: which route (scene and kind) its own hierarchy lookup selected, and the shape the unchanged route-planning code walks.
//
//   campaign_model_route_lookup   called by get_airfield_waypoint_route_object (popread.c, the staged patch) with the airfield scene, the sub-object kind and the route entry
//                                 that lookup found (NULL when it found none). The entry is a persistent entry of the campaign's own scene hierarchy, so the pair (entry, scene,
//                                 kind) is fixed: it is kept as a private key. The pointer is never sent anywhere and never leaves this file.
//   campaign_model_route_handle   called at every place the campaign took the model number from the route entry it holds (the staged patches): it returns that entry's key
//                                 number, which is what the parse call receives where it used to receive the legacy object number. An entry that no lookup returned stops the engine.
//   campaign_model_route_object   called by the parse: it asks the host for the route's ORDERED nodes and ORDERED line faces and returns them in the shape the parse's loops
//                                 walk (a node count, and surfaces of at most 128 faces each, so that a face-local reference stays a byte), so the route-planning statements of
//                                 routegen.c are the original's, unchanged. The surfaces here are a packaging of the host's flat faces (same order, colour and polygon bit per
//                                 run), not the model's: the indirection the host resolved is not reconstructed from anything.
//   campaign_model_route_position called by the parse for each node: the host's decoded position of the node last loaded.
//
// An answer the host cannot give (ABSENT for a route the campaign's hierarchy found, a refusal, a malformed or out-of-domain reply, more nodes or faces than the parser's own
// tables hold) stops the engine with a named marker; nothing is invented, and nothing falls back to the campaign's arrays. A route of valid nodes and faces whose SHAPE the parser
// cannot take (more starts or links than its tables, no start, nodes it can never place, slots it can never settle) stops the same way, from the guards after campaign_model_route_position.

#ifdef EECH_ROUTE_FIXTURE
#include "eech_route_hooks.h"
#endif

#define ROUTE_KEYS 4096
/* MAX_WAYPOINT_NODES of routegen.c: the parser's own node table */
#define ROUTE_NODES 768
#define ROUTE_FACES 8192
/* a face-local reference is a byte: 128 faces use 256 references */
#define ROUTE_SURFACE_FACES 128

struct route_key
{
	const void *entry;
	int scene, kind;
};

static struct route_key route_keys[ROUTE_KEYS];
static int route_key_count;

// the route being parsed and the state of the topology guards (campaign_model_route_room, _progress, _match_pass): reset by campaign_model_route_object at every parse
static const struct route_key *route_current;
static int route_progress_phase;
static int route_progress_last;
static int route_match_passes;

static vec3d route_positions[ROUTE_NODES];
static int route_loaded_nodes;
static struct OBJECT_3D route_object;
static struct FACE_SURFACE_DESCRIPTION route_surfaces[ROUTE_FACES];
static struct POINT_3D_PLAIN_REFERENCE route_surface_points[2 * ROUTE_FACES];
static struct POINT_3D_SHORT_REFERENCE route_face_references[2 * ROUTE_FACES];

void campaign_model_route_lookup (int scene, int kind, const void *entry)
{
	int
		i;

	if (entry)
	{
		for (i = 0; i < route_key_count; i++)
		{
			if (route_keys[i].entry == entry)
			{
				if ((route_keys[i].scene != scene) || (route_keys[i].kind != kind))
				{
					debug_fatal ("host model: [model-provider:route.context] one route entry was found for scene %d kind %d and for scene %d kind %d", route_keys[i].scene, route_keys[i].kind, scene, kind);
				}

				break;
			}
		}

		if (i == route_key_count)
		{
			if (route_key_count >= ROUTE_KEYS)
			{
				debug_fatal ("host model: [model-provider:route.limit] more than %d distinct route entries", ROUTE_KEYS);
			}

			route_keys[route_key_count].entry = entry;
			route_keys[route_key_count].scene = scene;
			route_keys[route_key_count].kind = kind;
			route_key_count++;
		}
	}
	else
	{
		i = -1;
	}

#ifdef EECH_ROUTE_FIXTURE
	eech_rf_lookup (scene, kind, entry != NULL, i, entry);
#endif
}

int campaign_model_route_handle (const void *entry)
{
	int
		i;

	for (i = 0; i < route_key_count; i++)
	{
		if (route_keys[i].entry == entry)
		{
#ifdef EECH_ROUTE_FIXTURE
			eech_rf_caller (entry, i);
#endif

			return i;
		}
	}

	debug_fatal ("host model: [model-provider:route.context] a route entry the campaign holds was not returned by its own hierarchy lookup");

	return -1;
}

// one step of a route cursor as the host answered it, with only the identity check: the kind's compiled name is echoed
static void ask_route_raw (int operation, int scene, int kind, int cursor, campaign_model_reply *reply, const char *what)
{
	campaign_model_request
		request;

	const char
		*expected;

	memset (&request, 0, sizeof (request));
	request.op = operation;
	request.scene = scene;
	request.depth = cursor;
	request.kind = kind;
	ask (&request, reply, what);

	expected = ((kind >= 1) && (kind < OBJECT_3D_SUB_OBJECT_LAST) && (object_3d_subobject_names[kind])) ? object_3d_subobject_names[kind] : "";

	if ((memchr (reply->name, '\0', sizeof (reply->name)) == NULL) || (strcmp (reply->name, expected) != 0))
	{
		debug_fatal ("host model: [model-provider:route.identity] the host named the route kind %d \"%.60s\" but the campaign names it \"%.60s\"", kind, reply->name, expected);
	}
}

#ifdef EECH_MODEL_HOST_FIXTURE
// TEST-ONLY diagnostic (the model-host-fixture's probe entry, called by csrc/eech_model_host_fixture.c): one route node and one route face, as the host answered them, for a
// scene and kind the campaign's key table does not hold. An absence is reported, not stopped on: the probe asks about any scene. Absent from every other build.
int campaign_model_route_probe (int scene, int kind, int cursor, int *node_result, float *node, int *face_result, int *face)
{
	campaign_model_reply
		reply;

	require_scene (scene, FALSE, "a route probe");

	ask_route_raw (CAMPAIGN_MODEL_ROUTE_NODE, scene, kind, cursor, &reply, "route node");
	*node_result = reply.result;
	node[0] = reply.points[0];
	node[1] = reply.points[1];
	node[2] = reply.points[2];

	ask_route_raw (CAMPAIGN_MODEL_ROUTE_FACE, scene, kind, cursor, &reply, "route face");
	*face_result = reply.result;
	face[0] = reply.route_point[0];
	face[1] = reply.route_point[1];
	face[2] = reply.route_colour[0];
	face[3] = reply.route_colour[1];
	face[4] = reply.route_colour[2];
	face[5] = reply.route_polygons;

	return TRUE;
}
#endif

// one step of a route cursor, with the checks every reply must pass: the kind's compiled name is echoed, and the host has not said the route is absent
static void ask_route (int operation, const struct route_key *key, int cursor, campaign_model_reply *reply, const char *what)
{
	ask_route_raw (operation, key->scene, key->kind, cursor, reply, what);

	if (reply->result == CAMPAIGN_MODEL_TRIANGLE_ABSENT)
	{
		debug_fatal ("host model: [model-provider:route.answer] the host has no route of kind %d for scene %d, which the campaign's hierarchy found", key->kind, key->scene);
	}

	if ((reply->result != CAMPAIGN_MODEL_TRIANGLE_FOUND) && (reply->result != CAMPAIGN_MODEL_TRIANGLE_END))
	{
		debug_fatal ("host model: [model-provider:route.answer] the host's result for the %s %d of scene %d kind %d is %d", what, cursor, key->scene, key->kind, reply->result);
	}
}

struct OBJECT_3D *campaign_model_route_object (int handle)
{
	campaign_model_reply
		reply;

	const struct route_key
		*key;

	int
		nodes,
		faces,
		surfaces,
		in_surface;

	if ((handle < 0) || (handle >= route_key_count))
	{
		debug_fatal ("host model: [model-provider:route.context] route key %d is not one of the %d the campaign recorded", handle, route_key_count);
	}

	key = &route_keys[handle];

	// a new parse: the topology guards below (campaign_model_route_room and the others) count from here
	route_current = key;
	route_progress_phase = 0;
	route_progress_last = 0;
	route_match_passes = 0;

	require_scene (key->scene, FALSE, "a route question");

	if (!host_loaded[key->scene])
	{
		debug_fatal ("host model: [model-provider:route.answer] the campaign asked for a route of scene %d, which the host declared not loaded", key->scene);
	}

	// the nodes, in order, until the host says the list is over
	route_loaded_nodes = 0;
	nodes = 0;

	for (;;)
	{
		ask_route (CAMPAIGN_MODEL_ROUTE_NODE, key, nodes, &reply, "route node");

		if (reply.result == CAMPAIGN_MODEL_TRIANGLE_END)
		{
			break;
		}

		if (nodes >= ROUTE_NODES)
		{
			debug_fatal ("host model: [model-provider:route.limit] the route of scene %d kind %d has more than the %d nodes the route parser holds", key->scene, key->kind, ROUTE_NODES);
		}

		if (!all_finite (reply.points, 3))
		{
			debug_fatal ("host model: [model-provider:route.node] the host's node %d of scene %d kind %d is not finite", nodes, key->scene, key->kind);
		}

		route_positions[nodes].x = reply.points[0];
		route_positions[nodes].y = reply.points[1];
		route_positions[nodes].z = reply.points[2];

#ifdef EECH_ROUTE_FIXTURE
		eech_rf_host_node (nodes, &route_positions[nodes]);
#endif

		nodes++;
	}

	// the line faces, in order, packed into surfaces of one colour and polygon bit
	faces = 0;
	surfaces = 0;
	in_surface = 0;

	for (;;)
	{
		ask_route (CAMPAIGN_MODEL_ROUTE_FACE, key, faces, &reply, "route face");

		if (reply.result == CAMPAIGN_MODEL_TRIANGLE_END)
		{
			break;
		}

		if (faces >= ROUTE_FACES)
		{
			debug_fatal ("host model: [model-provider:route.limit] the route of scene %d kind %d has more than %d line faces", key->scene, key->kind, ROUTE_FACES);
		}

		if ((reply.route_point[0] < 0) || (reply.route_point[0] >= nodes) || (reply.route_point[1] < 0) || (reply.route_point[1] >= nodes) ||
			(reply.route_colour[0] < 0) || (reply.route_colour[0] > 255) || (reply.route_colour[1] < 0) || (reply.route_colour[1] > 255) ||
			(reply.route_colour[2] < 0) || (reply.route_colour[2] > 255) || ((reply.route_polygons != FALSE) && (reply.route_polygons != TRUE)))
		{
			debug_fatal ("host model: [model-provider:route.face] the host's line face %d of scene %d kind %d is outside its route's %d nodes or the colour / polygon domain", faces, key->scene, key->kind, nodes);
		}

		if ((surfaces == 0) || (in_surface == ROUTE_SURFACE_FACES) || (route_surfaces[surfaces - 1].red != reply.route_colour[0]) ||
			(route_surfaces[surfaces - 1].green != reply.route_colour[1]) || (route_surfaces[surfaces - 1].blue != reply.route_colour[2]) ||
			(route_surfaces[surfaces - 1].polygons != reply.route_polygons))
		{
			memset (&route_surfaces[surfaces], 0, sizeof (route_surfaces[surfaces]));

			route_surfaces[surfaces].polygons = reply.route_polygons;
			route_surfaces[surfaces].red = (unsigned char) reply.route_colour[0];
			route_surfaces[surfaces].green = (unsigned char) reply.route_colour[1];
			route_surfaces[surfaces].blue = (unsigned char) reply.route_colour[2];

			surfaces++;
			in_surface = 0;
		}

		// the face-local references of a surface count from its first face; the references of every surface are stored back to back, so the surface's run of
		// surface points begins where the previous surface's ended (the parse advances by the surface's point count, 0 meaning 256)
		route_face_references[2 * faces].point = (unsigned char) (2 * in_surface);
		route_face_references[2 * faces + 1].point = (unsigned char) (2 * in_surface + 1);
		route_surface_points[2 * faces].point = (unsigned short) reply.route_point[0];
		route_surface_points[2 * faces + 1].point = (unsigned short) reply.route_point[1];

		in_surface++;
		route_surfaces[surfaces - 1].number_of_faces = (unsigned short) in_surface;
		route_surfaces[surfaces - 1].number_of_points = (unsigned char) ((2 * in_surface == 256) ? 0 : 2 * in_surface);

#ifdef EECH_ROUTE_FIXTURE
		eech_rf_host_face (faces, reply.route_point[0], reply.route_point[1], reply.route_colour[0], reply.route_colour[1], reply.route_colour[2], reply.route_polygons);
#endif

		faces++;
	}

	memset (&route_object, 0, sizeof (route_object));

	route_object.number_of_points = (unsigned short) nodes;
	route_object.number_of_faces = (unsigned short) faces;
	route_object.number_of_surfaces = (unsigned short) surfaces;
	route_object.surfaces = route_surfaces;
	route_object.surface_points = route_surface_points;
	route_object.object_faces_point_plain_list = route_face_references;

	route_loaded_nodes = nodes;

#ifdef EECH_ROUTE_FIXTURE
	eech_rf_host_counts (nodes, surfaces, faces);
#endif

	return &route_object;
}

void campaign_model_route_position (int node, vec3d *position)
{
	if ((node < 0) || (node >= route_loaded_nodes) || (!position))
	{
		debug_fatal ("host model: [model-provider:route.node] the route parser read node %d of the %d the host gave", node, route_loaded_nodes);
	}

	*position = route_positions[node];
}

// ---------------------------------------------------------------------------------------------------------------------------------
// The bounded-input contract of the unchanged route-planning code. routegen.c was written for the routes the original models hold: it indexes a 24-entry start table and 24-entry
// child and parent tables by whatever shape the route happens to have, it reads the first start without asking whether there is one, and its tree loops end only when every node has
// been placed. A host can answer a route that is well formed (finite nodes, endpoints in range, colours in domain) and still not one of those shapes. The staged patches
// H6..H15 (build/patches.rs) call the functions below at the places where the parser would write past a table, read a start that is not there, or go round again with nothing
// changed. They only test the parser's OWN counters on its OWN statements, at the point the statement is about to be taken: they place no node, solve nothing and change no
// value of a route the parser could take. A shape it cannot take stops the engine with a named marker before the write or the next pass, so no table is overrun, no loop
// spins, and nothing is truncated, skipped or invented.

// A WORK LIMIT of this host boundary, not a bound derived from the original parser: the original has no limit on the passes of its end-slot matching, and nothing here proves that a
// matching that reaches this many passes would never have settled, or that it was undoing its own swaps. 4096 is an explicitly chosen, newly supported finite contract: a host route
// whose slot matching needs more passes is UNSUPPORTED and UNMEASURED here and stops with the named marker instead of running without bound. It excuses no baseline case: a route of
// the original models that reached it would be a regression to investigate, not an accepted refusal. (No smaller invariant of the matching is claimed.)
#define ROUTE_MATCH_PASSES 4096

// the next write to a table of `capacity` entries would be entry `used`: 0 the start table, 1 the children of a node, 2 the parents of a node
void campaign_model_route_room (int what, int used, int capacity)
{
	if ((used < 0) || (used >= capacity))
	{
		debug_fatal ("host model: [model-provider:route.%s] the route of scene %d kind %d needs entry %d of the parser's %d-entry %s table", (what == 0) ? "starts" : "links",
			route_current->scene, route_current->kind, used + 1, capacity, (what == 0) ? "start" : ((what == 1) ? "child" : "parent"));
	}
}

// A loop of the parser is about to take a pass with `left` nodes still to place: the pass before it walked the same faces over the same nodes, so unless the count fell that pass
// placed nothing and this one would place nothing either. The state (`phase`, last count) belongs to ONE invocation of ONE loop: the tree and position loops each run once per
// parse (campaign_model_route_object resets it), and set_node_indices, which the parser calls again from every end-slot swap, starts each invocation with
// campaign_model_route_progress_begin, so the count of one invocation is never compared with the last count of another.
void campaign_model_route_progress_begin (void)
{
	route_progress_phase = 0;
	route_progress_last = 0;
}

int campaign_model_route_progress (int phase, int left)
{
	if (phase != route_progress_phase)
	{
		route_progress_phase = phase;
		route_progress_last = left;

		return TRUE;
	}

	if (left >= route_progress_last)
	{
		debug_fatal ("host model: [model-provider:route.progress] a pass over the line faces of scene %d kind %d placed nothing (%s, %d left)", route_current->scene, route_current->kind,
			(phase == 1) ? "primary nodes" : ((phase == 2) ? "remaining nodes" : ((phase == 3) ? "node indices" : "positions of a depth")), left);
	}

	route_progress_last = left;

	return TRUE;
}

// the slot matching is about to take another pass
int campaign_model_route_match_pass (void)
{
	route_match_passes++;

	if (route_match_passes > ROUTE_MATCH_PASSES)
	{
		debug_fatal ("host model: [model-provider:route.match] the end-slot matching of scene %d kind %d did not settle in %d passes", route_current->scene, route_current->kind, ROUTE_MATCH_PASSES);
	}

	return TRUE;
}

// The end-slot matching is about to swap two nodes (swap_waypoint_nodes) and `index` is the table index the swap's own search resolved for one of them. what 0: a TREE swap, whose
// search of the start table begins at entry 1 (the primary tree at entry 0 is never looked for), so a tree it did not find comes back as `count`; the valid entries are 1 .. count - 1.
// what 1: a swap of two children of one parent, whose search of the parent's children comes back as -1 when it found nothing (valid 0 .. count - 1). `count` is the table's LOGICAL
// count, `capacity` its physical size; an index outside either is a swap the unchanged algorithm cannot take. The original has only an ASSERT before the start-table write (none
// for the children), and nothing else bounds what the matching may select from host positions, so the refusal is made here, before the assertion and before either write. The swap
// is not redirected, skipped or repaired: the index 0 tree is not searched for, partners are not chosen differently, and every swap the original takes runs as before.
void campaign_model_route_swap_index (int what, int index, int count, int capacity)
{
	if ((index < ((what == 0) ? 1 : 0)) || (index >= count) || (index >= capacity))
	{
		debug_fatal ("host model: [model-provider:route.swap] the end-slot matching of scene %d kind %d swaps %s at index %d, which the parser's search did not resolve inside its %d of %d entries",
			route_current->scene, route_current->kind, (what == 0) ? "a tree through the start table" : "two children of one parent", index, count, capacity);
	}
}

// A legacy VISUAL / DISPLAY reader that would read a physical geometry buffer the loader no longer holds (EEDCS-50, D3 loader detachment). The common and custom loaders keep
// no point, face, face-point-reference or surface-point buffer of any model (the campaign's collision and route data are the host's), so every renderer, shadow, explosion,
// cloud and horizon-image entry that dereferences one, the cockpit display poly-lines and the Mi-24 HUD stop here, by name, at their head (the LR-reader patches of
// LOADER_HOST_PATCHES) and before any buffer, image or device is touched: nothing is drawn from nothing, no buffer or colour is invented, nothing falls back to the campaign's
// own arrays and nothing is ported. No renderer, cockpit display or HUD is supported by the hosted engine, and the campaign's state does not depend on them. Whether one of
// them is reachable in a hosted engine is NOT established (compiled presence is not dormancy): a run that reaches one reports it by this marker and the operation's name.
// The one horizon-image path the boot takes (a pre-processed `.bin` that matches the object's point count: point colours and fog only) never reaches it.
void campaign_model_geometry_reader (const char *reader)
{
	debug_fatal ("host model: [model-provider:object.geometry] %s is a legacy visual / display reader of a model's point, face, face-point-reference or surface-point buffer, which the hosted campaign no longer holds and does not serve (no renderer, cockpit display or HUD is supported)", reader);
}

// The loaders no longer keep the point, face, face-point-reference and surface-point regions of a model (the host owns them, from its own load of the same files); they READ PAST
// them. A region is `count` records of `element` bytes, taken from `file` through a small buffer: every byte must be there, so a file that ends inside one is refused by name here
// instead of leaving the stream mis-aligned or an unread region counted as valid. Nothing is seeked (a seek past the end of a file succeeds and proves nothing, and a Windows long is
// 32 bits); the byte count is formed in 64 bits after `count` and `element` are checked, so it cannot overflow. This is the loader's own error handling for the regions it newly
// skips, not a validation of the model: the legacy header reads are as they were.
void campaign_model_loader_skip (FILE *file, long long count, unsigned long long element, const char *what)
{
	unsigned char
		buffer[4096];

	unsigned long long
		remaining;

	if ((!file) || (count < 0) || (count > 0x7fffffffLL) || (element < 1) || (element > 16))
	{
		debug_fatal ("host model: [model-provider:object.loader] %s: a region of %d records of %d bytes is not one the loader can read past", what, (int) count, (int) element);
	}

	remaining = (unsigned long long) count * element;

	while (remaining)
	{
		size_t
			chunk = (remaining < sizeof (buffer)) ? (size_t) remaining : sizeof (buffer);

		if (fread (buffer, 1, chunk, file) != chunk)
		{
			debug_fatal ("host model: [model-provider:object.loader] %s: the file ends inside a region of %d records of %d bytes (%d KiB of it still to read)", what, (int) count, (int) element, (int) (remaining >> 10));
		}

		remaining -= chunk;
	}
}

// The custom loader streams its polygon faces through a local buffer for their one validation, and the stream loop is driven by the header's signed count. A NEGATIVE count is no
// number of records: the loop would not run, the validation would be skipped and the file would be accepted with its polygon region unread, which the retained buffer never allowed.
// It is refused here, by name and before the stream, as a bad count (not as a short read). An int count needs no upper bound: a large count meets the end of the file and is the
// short-read refusal below.
void campaign_model_loader_count (const char *what, int count)
{
	if (count < 0)
	{
		debug_fatal ("host model: [model-provider:object.loader] %s: a count of %d is not a number of records", what, count);
	}
}

// the custom loader's polygon faces are streamed through a local buffer for their one validation; a chunk that was not read in full is the same refusal
void campaign_model_loader_short (const char *what)
{
	debug_fatal ("host model: [model-provider:object.loader] %s: the file ends inside the region, a short read is not read as valid data", what);
}

// ---------------------------------------------------------------------------------------------------------------------------------
// MODEL-FILE METADATA (EEDCS-50, D3, L2). The common loader (3dobjs.bin, .pts, .sp, .spn, .spt, the binary-hex .EEO overrides) and the custom loader (a named .EEO) no longer parse a
// model file: the HOST owns what the files say. The campaign keeps its own private object table (objects_3d_data), its numbering of objects (the ids of 3dobjdb.bin, and for named
// objects the history and the custom allocation that get_object keeps), and the scene / hierarchy loader (3dobjdb.bin and .EES: L3, still the campaign's). What this half does:
//
//   campaign_model_object_table   asked at the start of the object loader: how many objects the host's common database holds. The room the campaign keeps for custom objects is
//                                 the campaign's own policy (custom_number_of_objects) and is neither asked nor answered; the arithmetic of the table's size is checked here before
//                                 the campaign allocates it.
//   campaign_model_scene_objects  called by read_scene after the scene's sub-objects are read, so before anything reads a bounding box: for each object the scene names (main, each
//                                 approximation, shadow polygon object, collision object, each sub-object) the host is asked by SEMANTIC address (scene, role, cursor), never by
//                                 an object number, and the campaign's private row for the object the campaign's own file numbered is filled with the host's TRUE counts, radius and
//                                 bounds. A row two references fill must agree with itself (a named conflict otherwise); a reference the host has no object for, an object the host has
//                                 where the campaign names none, and an id outside the table stop the engine by name. A row nobody filled stays unfilled and is tracked as such:
//                                 campaign_model_object_row refuses a required read of one before it touches the row.
//   campaign_model_animation_reference   the ordered animated-surface references of a scene a custom scene file defines, as LOGICAL names the campaign resolves in its own animation
//                                 table (a name that table does not hold stops the engine by name); the campaign's own add_animation de-duplicates and orders as it did.
//
// No point, face, surface, normal or texture-point buffer is read or kept: the rows keep NULL for all of them, and every legacy visual / display reader of one stops by name at its
// head (the LR patches). Nothing falls back to a file of the campaign's own.

#define OBJECT_COMMON_MAX 0x00ffffff
#define ANIMATION_REFERENCES_MAX 1048576

static unsigned char *object_row_filled;
// a row that get_object REPLACED while the host had described it: read_object never assigns culling_normals_offset, so the row keeps its value, and the host's description of the new
// object must carry the same one (checked when the new object is described)
static unsigned char *object_row_carry;
static int object_row_count;

// The custom scene files in the order the campaign's own walk visits them (announced to the host before any is read; see mb_hostmodel.h, CUSTOM_ORDER): the scene and the name of
// the directory that holds each, how many, whether the announcement is complete, the next one to be read and the one being read. A custom definition is named to the host by its
// POSITION in this walk, verified against the announcement as each file is read.
#define CUSTOM_WALK_MAX 4096
static int custom_walk_scene[CUSTOM_WALK_MAX];
static char custom_walk_key[CUSTOM_WALK_MAX][CAMPAIGN_MODEL_NAME_MAX + 1];
static int custom_walk_count, custom_walk_announced, custom_walk_cursor, custom_walk_current, custom_walk_read, custom_walk_skipped;
static int scene_database_reads[OBJECT_3D_LAST];
// How many times the object loader has read each scene (the scene database's record, then every custom scene file that defines it again), and which of those reads is the current one:
// the DEFINITION of the scene the host is asked about. Each read is answered with the facts the objects had when that definition was read, never a later one's.
static int scene_reads[OBJECT_3D_LAST];
static int scene_definition[OBJECT_3D_LAST];

#ifdef EECH_MODEL_HOST_FIXTURE
// TEST-ONLY: what the metadata exchange did. objects_*: questions asked and how they were answered; rows_filled / rows_confirmed: first fills and agreeing refills of a row;
// rows_replaced: filled rows that a new object number took over (so the rows holding a description now are rows_filled - rows_replaced);
// animation_*: reference questions asked and the names they carried; animation_ends: lists that ended.
static long meta_objects_asked, meta_objects_found, meta_objects_absent, meta_rows_filled, meta_rows_confirmed, meta_rows_replaced, meta_animation_asked, meta_animation_names,
	meta_animation_ends;
#endif

static const char *role_name (int role)
{
	switch (role)
	{
		case CAMPAIGN_MODEL_ROLE_MAIN: return "main object";
		case CAMPAIGN_MODEL_ROLE_APPROXIMATION: return "approximation";
		case CAMPAIGN_MODEL_ROLE_SHADOW: return "shadow polygon object";
		case CAMPAIGN_MODEL_ROLE_COLLISION: return "collision object";
		case CAMPAIGN_MODEL_ROLE_SUB_OBJECT: return "sub-object";
	}

	return "unknown role";
}

// one metadata question about a scene, with the identity check every reply must pass: the scene's compiled name is echoed
static void ask_metadata (int operation, int scene, int role, int cursor, int definition, campaign_model_reply *reply, const char *what)
{
	campaign_model_request
		request;

	const char
		*expected;

	memset (&request, 0, sizeof (request));
	request.op = operation;
	request.scene = scene;
	request.depth = cursor;
	request.role = role;
	request.definition = definition;

	ask (&request, reply, what);

	expected = ((scene > 0) && (scene < OBJECT_3D_LAST) && (object_3d_scene_names[scene])) ? object_3d_scene_names[scene] : "";

	if ((memchr (reply->name, '\0', sizeof (reply->name)) == NULL) || (strcmp (reply->name, expected) != 0))
	{
		debug_fatal ("host model: [model-provider:%s.identity] the host named scene %d \"%.60s\" but the campaign names it \"%.60s\"", what, scene, reply->name, expected);
	}
}

int campaign_model_object_table (int custom_room)
{
	campaign_model_request
		request;

	campaign_model_reply
		reply;

	unsigned long long
		rows;

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_OBJECT_TABLE;

	ask (&request, &reply, "object table");

	if ((reply.object_count < 0) || (reply.object_count > OBJECT_COMMON_MAX) || (custom_room < 0) || (custom_room > OBJECT_COMMON_MAX))
	{
		debug_fatal ("host model: [model-provider:object.table] the host's common database holds %d objects, which is not a count of objects (0 to %d), beside the campaign's room for %d custom ones", reply.object_count, OBJECT_COMMON_MAX, custom_room);
	}

	// the campaign's own table: the common objects, its custom room and the null object 0 (a row of the original's size each), all in 64 bits before anything is allocated
	rows = (unsigned long long) reply.object_count + (unsigned long long) custom_room + 1ULL;

	if (rows * (unsigned long long) sizeof (struct OBJECT_3D) > 0x7fffffffULL)
	{
		debug_fatal ("host model: [model-provider:object.table] %d common objects and room for %d custom ones are more rows than the campaign's object table can hold", reply.object_count, custom_room);
	}

	if (object_row_filled)
	{
		safe_free (object_row_filled);
	}

	if (object_row_carry)
	{
		safe_free (object_row_carry);
	}

	object_row_carry = (unsigned char *) safe_malloc ((size_t) rows);

	memset (object_row_carry, 0, (size_t) rows);

	object_row_filled = (unsigned char *) safe_malloc ((size_t) rows);

	memset (object_row_filled, 0, (size_t) rows);

	object_row_count = (int) (rows - 1ULL);

	memset (scene_reads, 0, sizeof (scene_reads));
	memset (scene_definition, 0, sizeof (scene_definition));
	memset (scene_database_reads, 0, sizeof (scene_database_reads));

	custom_walk_count = 0;
	custom_walk_announced = FALSE;
	custom_walk_cursor = 0;
	custom_walk_current = -1;
	custom_walk_read = 0;
	custom_walk_skipped = 0;

	return reply.object_count;
}

static int bounds_equal (const struct OBJECT_3D_BOUNDS *bounds, const float *values)
{
	return (memcmp (&bounds->xmin, &values[0], sizeof (float)) == 0) && (memcmp (&bounds->xmax, &values[1], sizeof (float)) == 0) &&
		(memcmp (&bounds->ymin, &values[2], sizeof (float)) == 0) && (memcmp (&bounds->ymax, &values[3], sizeof (float)) == 0) &&
		(memcmp (&bounds->zmin, &values[4], sizeof (float)) == 0) && (memcmp (&bounds->zmax, &values[5], sizeof (float)) == 0);
}

// the object the campaign's own scene data numbers 'id' in 'role' of 'scene' ('custom': a custom scene file's definition), against the host's object in that place
static void fill_object (int scene, int custom, int definition, int role, int cursor, int id)
{
	campaign_model_reply
		reply;

	struct OBJECT_3D
		*row;

	int
		i;

	ask_metadata (custom ? CAMPAIGN_MODEL_CUSTOM_OBJECT : CAMPAIGN_MODEL_OBJECT, scene, role, cursor, definition, &reply, "object");

#ifdef EECH_MODEL_HOST_FIXTURE
	meta_objects_asked++;
#endif

	if ((reply.result != CAMPAIGN_MODEL_TRIANGLE_FOUND) && (reply.result != CAMPAIGN_MODEL_TRIANGLE_ABSENT))
	{
		debug_fatal ("host model: [model-provider:object.answer] the host's result for the %s (cursor %d) of scene %d is %d", role_name (role), cursor, scene, reply.result);
	}

	// the campaign's scene file names no object in this place (0, or the collision object's -1): the host must name none either
	if (id <= 0)
	{
		if (reply.result == CAMPAIGN_MODEL_TRIANGLE_FOUND)
		{
			debug_fatal ("host model: [model-provider:object.disagree] the host has an object as the %s (cursor %d) of scene %d, where the campaign's scene file names none", role_name (role), cursor, scene);
		}

#ifdef EECH_MODEL_HOST_FIXTURE
		meta_objects_absent++;
#endif

		return;
	}

	if ((!object_row_filled) || (id > object_row_count))
	{
		debug_fatal ("host model: [model-provider:object.range] the campaign's scene %d names object %d as its %s (cursor %d), outside its table of %d objects", scene, id, role_name (role), cursor, object_row_count);
	}

	if (reply.result == CAMPAIGN_MODEL_TRIANGLE_ABSENT)
	{
		debug_fatal ("host model: [model-provider:object.absent] the host has no object as the %s (cursor %d) of scene %d, which the campaign's scene file names", role_name (role), cursor, scene);
	}

	// every number is checked in its domain before it is stored or compared: the counts are the unsigned shorts of the campaign's rows
	for (i = 0; i < 6; i++)
	{
		if ((reply.object_counts[i] < 0) || (reply.object_counts[i] > 65535))
		{
			debug_fatal ("host model: [model-provider:object.answer] the host's count %d for the %s (cursor %d) of scene %d is %d, outside 0 to 65535", i, role_name (role), cursor, scene, reply.object_counts[i]);
		}
	}

	if (!isfinite (reply.object_radius) || !all_finite (reply.object_bounds, 12))
	{
		debug_fatal ("host model: [model-provider:object.answer] the host's radius or bounds for the %s (cursor %d) of scene %d are not finite", role_name (role), cursor, scene);
	}

#ifdef EECH_MODEL_HOST_FIXTURE
	meta_objects_found++;
#endif

	row = &objects_3d_data[id];

	if (object_row_filled[id])
	{
		// a second reference to the same object of the campaign: the host must describe it the same way
		if ((row->number_of_points != (unsigned short) reply.object_counts[0]) || (row->number_of_faces != (unsigned short) reply.object_counts[1]) ||
			(row->number_of_surfaces != (unsigned short) reply.object_counts[2]) || (row->number_of_point_normals != (unsigned short) reply.object_counts[3]) ||
			(row->number_of_lighting_normals != (unsigned short) reply.object_counts[4]) || (row->culling_normals_offset != (unsigned short) reply.object_counts[5]) ||
			(memcmp (&row->radius, &reply.object_radius, sizeof (float)) != 0) || !bounds_equal (&row->bounding_box, &reply.object_bounds[0]) ||
			!bounds_equal (&row->bounding_box2, &reply.object_bounds[6]))
		{
			debug_fatal ("host model: [model-provider:object.conflict] object %d is named by more than one reference of the campaign (this is the %s, cursor %d, of scene %d) and the host describes it differently each time", id, role_name (role), cursor, scene);
		}

#ifdef EECH_MODEL_HOST_FIXTURE
		meta_rows_confirmed++;
#endif

		return;
	}

	if (object_row_carry[id])
	{
		// the row was replaced by a new number and still holds the one field read_object does not assign: the host's description of the new object must carry the same value
		if (row->culling_normals_offset != (unsigned short) reply.object_counts[5])
		{
			debug_fatal ("host model: [model-provider:object.carry] object %d replaced a described row and kept its culling normals offset %d, but the host describes the %s (cursor %d) of scene %d with %d", id, (int) row->culling_normals_offset, role_name (role), cursor, scene, reply.object_counts[5]);
		}

		object_row_carry[id] = FALSE;
	}

	row->number_of_points = (unsigned short) reply.object_counts[0];
	row->number_of_faces = (unsigned short) reply.object_counts[1];
	row->number_of_surfaces = (unsigned short) reply.object_counts[2];
	row->number_of_point_normals = (unsigned short) reply.object_counts[3];
	row->number_of_lighting_normals = (unsigned short) reply.object_counts[4];
	row->culling_normals_offset = (unsigned short) reply.object_counts[5];
	row->radius = reply.object_radius;
	bounds_from (&row->bounding_box, &reply.object_bounds[0]);
	bounds_from (&row->bounding_box2, &reply.object_bounds[6]);

	object_row_filled[id] = TRUE;

#ifdef EECH_MODEL_HOST_FIXTURE
	meta_rows_filled++;
#endif
}

void campaign_model_scene_objects (int scene_index, int custom)
{
	const struct OBJECT_3D_SCENE_DATABASE_ENTRY
		*scene;

	int
		definition,
		i;

	if ((!object_row_filled) || (!objects_3d_data) || (!objects_3d_scene_database))
	{
		debug_fatal ("host model: [model-provider:object.table] scene %d was read before the host's object table was asked for", scene_index);
	}

	if ((scene_index <= OBJECT_3D_INVALID_OBJECT_INDEX) || (scene_index >= OBJECT_3D_LAST))
	{
		debug_fatal ("host model: [model-provider:scene.index] the object loader read scene %d, outside %d to %d", scene_index, OBJECT_3D_INVALID_OBJECT_INDEX + 1, (int) OBJECT_3D_LAST - 1);
	}

	scene = &objects_3d_scene_database[scene_index];

	if ((scene->number_of_approximations < 0) || ((scene->number_of_approximations > 0) && (!scene->approximations)) || (scene->total_number_of_sub_objects < 0) ||
		((scene->total_number_of_sub_objects > 0) && (!scene->scene_sub_object_array)))
	{
		debug_fatal ("host model: [model-provider:object.scene] scene %d has %d approximations and %d sub-objects, which its own arrays do not hold", scene_index, scene->number_of_approximations, scene->total_number_of_sub_objects);
	}

	custom = (custom != 0);

	// The definition this read is, in terms the host can check: a custom scene file is the one at this POSITION of the campaign's announced walk (verified when the file was
	// opened); a record of the scene database is the scene's n-th such record. The animation references of this read are of the same definition.
	if (custom)
	{
		if ((custom_walk_current < 0) || (custom_walk_scene[custom_walk_current] != scene_index))
		{
			debug_fatal ("host model: [model-provider:order.walk] the custom scene %d was read without the announced walk reaching it (position %d)", scene_index, custom_walk_current);
		}

		definition = custom_walk_current;

		custom_walk_current = -1;
		custom_walk_read++;
	}
	else
	{
		definition = scene_database_reads[scene_index]++;
	}

	scene_reads[scene_index]++;
	scene_definition[scene_index] = definition;
	fill_object (scene_index, custom, definition, CAMPAIGN_MODEL_ROLE_MAIN, 0, scene->index);

	for (i = 0; i < scene->number_of_approximations; i++)
	{
		fill_object (scene_index, custom, definition, CAMPAIGN_MODEL_ROLE_APPROXIMATION, i, scene->approximations[i].object_number);
	}

	fill_object (scene_index, custom, definition, CAMPAIGN_MODEL_ROLE_SHADOW, 0, scene->shadow_polygon_object_index);

	fill_object (scene_index, custom, definition, CAMPAIGN_MODEL_ROLE_COLLISION, 0, scene->collision_object_index);

	for (i = 0; i < scene->total_number_of_sub_objects; i++)
	{
		fill_object (scene_index, custom, definition, CAMPAIGN_MODEL_ROLE_SUB_OBJECT, i, scene->scene_sub_object_array[i].index);
	}

}

// ---- the campaign's walk of the custom scene files (EEDCS-50, D3, L2). The original visits OBJECTS/<directory>/<SCENE NAME>.EES with _findfirst / _findnext, which is the file
// system's own order and not a sorted one, and what a custom definition sees depends on it. The campaign therefore ANNOUNCES its walk to the host, file by file, before it reads
// any (campaign_model_custom_order, once per file, then campaign_model_custom_order_end), and then names each definition it reads by its position in that walk
// (campaign_model_custom_reading, called as each file is opened, checks that the file read is the one announced at that position).

static void ask_order (int position, int scene, int end, const char *directory, campaign_model_reply *reply)
{
	campaign_model_request
		request;

	size_t
		length = directory ? strlen (directory) : 0;

	if ((!end) && ((length == 0) || (length > CAMPAIGN_MODEL_NAME_MAX)))
	{
		debug_fatal ("host model: [model-provider:order.walk] the custom scene directory name \"%.60s\" is not 1 to %d bytes", directory ? directory : "", (int) CAMPAIGN_MODEL_NAME_MAX);
	}

	memset (&request, 0, sizeof (request));
	request.op = CAMPAIGN_MODEL_CUSTOM_ORDER;
	request.scene = scene;
	request.depth = position;
	request.role = end ? 1 : 0;

	if (!end)
	{
		memcpy (request.key, directory, length);
	}

	ask (&request, reply, "custom walk");
}

void campaign_model_custom_order (int scene, int position, const char *directory)
{
	campaign_model_reply
		reply;

	if ((!object_row_filled) || (position != custom_walk_count) || (position >= CUSTOM_WALK_MAX) || (scene <= OBJECT_3D_INVALID_OBJECT_INDEX) || (scene >= OBJECT_3D_LAST))
	{
		debug_fatal ("host model: [model-provider:order.walk] the walk announcement step %d (scene %d) is not the next of %d, or is outside the walk's %d files", position, scene, custom_walk_count, CUSTOM_WALK_MAX);
	}

	ask_order (position, scene, FALSE, directory, &reply);

	if (reply.result == CAMPAIGN_MODEL_TRIANGLE_ABSENT)
	{
		debug_fatal ("host model: [model-provider:order.absent] the host has no custom scene file of scene %d in the directory \"%.60s\" that the campaign's walk visits", scene, directory);
	}

	if (reply.result != CAMPAIGN_MODEL_TRIANGLE_FOUND)
	{
		debug_fatal ("host model: [model-provider:order.answer] the host's result for walk step %d is %d", position, reply.result);
	}

	custom_walk_scene[position] = scene;

	strcpy (custom_walk_key[position], directory);

	custom_walk_count++;
}

void campaign_model_custom_order_end (int count)
{
	campaign_model_reply
		reply;

	if ((!object_row_filled) || (count != custom_walk_count))
	{
		debug_fatal ("host model: [model-provider:order.walk] the walk announcement ends with %d files but %d were announced", count, custom_walk_count);
	}

	ask_order (count, 0, TRUE, NULL, &reply);

	if (reply.result != CAMPAIGN_MODEL_TRIANGLE_END)
	{
		debug_fatal ("host model: [model-provider:order.answer] the host's result for the end of the walk is %d", reply.result);
	}

	custom_walk_announced = TRUE;
	custom_walk_cursor = 0;
	custom_walk_current = -1;
}

void campaign_model_custom_reading (int scene, const char *directory)
{
	if ((!custom_walk_announced) || (custom_walk_cursor >= custom_walk_count) || (scene != custom_walk_scene[custom_walk_cursor]) || (strcmp (directory, custom_walk_key[custom_walk_cursor]) != 0))
	{
		debug_fatal ("host model: [model-provider:order.walk] the campaign opened the custom scene file of scene %d in the directory \"%.60s\", which is not the next of its announced walk (position %d of %d)", scene, directory, custom_walk_cursor, custom_walk_count);
	}

	// a file opened before this one was neither read nor skipped: a read is outstanding
	if (custom_walk_current >= 0)
	{
		debug_fatal ("host model: [model-provider:order.walk] the custom scene file at position %d of the announced walk was opened and neither read nor skipped before the next was opened", custom_walk_current);
	}

	custom_walk_current = custom_walk_cursor++;
}

// The file just opened has a scene version the original does not read: it logs that and returns without read_scene. The position it held is consumed as SKIPPED, and no object row
// is claimed read for it (the host holds no definition at that position either).
void campaign_model_custom_skipped (void)
{
	if (custom_walk_current < 0)
	{
		debug_fatal ("host model: [model-provider:order.walk] a custom scene file was skipped that the announced walk had not reached");
	}

	custom_walk_current = -1;
	custom_walk_skipped++;
}

// The original traversal of the custom scene files is over. It must have opened every announced file (a traversal that ends early has an announced tail it never reached), each
// either read by read_scene or skipped for its version, and none may be left outstanding.
void campaign_model_custom_walk_complete (void)
{
	if ((!custom_walk_announced) || (custom_walk_current >= 0) || (custom_walk_cursor != custom_walk_count) || (custom_walk_read + custom_walk_skipped != custom_walk_count))
	{
		debug_fatal ("host model: [model-provider:order.walk] the traversal of the custom scene files ended with %d of the %d announced files opened (%d read, %d skipped for an unsupported version) and %s read outstanding", custom_walk_cursor, custom_walk_count, custom_walk_read, custom_walk_skipped, (custom_walk_current >= 0) ? "a" : "no");
	}
}

// A NEW object number is being allocated by get_object (the campaign's own allocation, history and sharing are unchanged). The original opened the object's file into this row at
// this moment, REPLACING whatever the row held: for the first named object, the last common object, which earlier scene reads had already taken their bounds from. The same
// replacement is made here: the row is cleared and unfilled, and the host describes it when the scene that names the object has been read.
void campaign_model_object_reset (int object_number)
{
	struct OBJECT_3D
		*row;

	if ((!object_row_filled) || (!objects_3d_data) || (object_number < 1) || (object_number > object_row_count))
	{
		debug_fatal ("host model: [model-provider:object.range] the campaign allocated object %d, outside its table of %d objects", object_number, object_row_count);
	}

	// what read_object assigns is cleared (and described again by the host); what it never assigns, culling_normals_offset, stays as the row had it, and a row the host had described
	// is remembered so that the new description can be checked against it. The buffers are NULL in every row.
	row = &objects_3d_data[object_number];

	if (object_row_filled[object_number])
	{
		object_row_carry[object_number] = TRUE;

#ifdef EECH_MODEL_HOST_FIXTURE
		meta_rows_replaced++;
#endif
	}

	row->number_of_points = 0;
	row->number_of_faces = 0;
	row->number_of_surfaces = 0;
	row->number_of_point_normals = 0;
	row->number_of_lighting_normals = 0;
	row->radius = 0;
	memset (&row->bounding_box, 0, sizeof (row->bounding_box));
	memset (&row->bounding_box2, 0, sizeof (row->bounding_box2));

	object_row_filled[object_number] = FALSE;
}

// A required read of an object's compatibility row (the sub-object bounds, the horizon environment): the row the host filled, or the null object 0 (which the campaign always
// kept zero), or a named stop before the row is touched. An unfilled row is not zero metadata and is never read as if it were.
struct OBJECT_3D *campaign_model_object_row (int object_number, const char *reader)
{
	if ((!object_row_filled) || (!objects_3d_data))
	{
		debug_fatal ("host model: [model-provider:object.unfilled] %s read object %d before the host's metadata was asked for", reader, object_number);
	}

	if ((object_number < 0) || (object_number > object_row_count))
	{
		debug_fatal ("host model: [model-provider:object.range] %s read object %d, outside the campaign's table of %d objects", reader, object_number, object_row_count);
	}

	if ((object_number != 0) && (!object_row_filled[object_number]))
	{
		debug_fatal ("host model: [model-provider:object.unfilled] %s read object %d, which no scene read has had the host describe", reader, object_number);
	}

	return &objects_3d_data[object_number];
}

// The ordered animated-surface references of the scene a custom scene file defines: reference number 'index', or -1 where the list ends. A reference is a LOGICAL name; the
// campaign maps it to its own animation table here, and the campaign's own add_animation orders and de-duplicates the result.
int campaign_model_animation_reference (int scene_index, int index)
{
	campaign_model_reply
		reply;

	int
		animation;

	if ((scene_index <= OBJECT_3D_INVALID_OBJECT_INDEX) || (scene_index >= OBJECT_3D_LAST) || (index < 0))
	{
		debug_fatal ("host model: [model-provider:animation.index] animation reference %d of scene %d is outside the campaign's scenes or cursor", index, scene_index);
	}

	if (index >= ANIMATION_REFERENCES_MAX)
	{
		debug_fatal ("host model: [model-provider:animation.limit] the host gave scene %d more than %d animation references", scene_index, ANIMATION_REFERENCES_MAX);
	}

	if (scene_reads[scene_index] <= 0)
	{
		debug_fatal ("host model: [model-provider:animation.order] animation reference %d of scene %d was asked before the object loader read the scene", index, scene_index);
	}

	ask_metadata (CAMPAIGN_MODEL_ANIMATION, scene_index, 0, index, scene_definition[scene_index], &reply, "animation");

#ifdef EECH_MODEL_HOST_FIXTURE
	meta_animation_asked++;
#endif

	switch (reply.result)
	{
		case CAMPAIGN_MODEL_TRIANGLE_END:
		{
#ifdef EECH_MODEL_HOST_FIXTURE
			meta_animation_ends++;
#endif

			return -1;
		}
		case CAMPAIGN_MODEL_TRIANGLE_ABSENT:
		{
			debug_fatal ("host model: [model-provider:animation.absent] the host has no animation references for scene %d, which the campaign loaded from a custom scene file", scene_index);
		}
		case CAMPAIGN_MODEL_TRIANGLE_FOUND:
		{
			extern int add_new_animation ( const char *animation_name );

			if ((memchr (reply.animation_name, '\0', sizeof (reply.animation_name)) == NULL) || (reply.animation_name[0] == '\0'))
			{
				debug_fatal ("host model: [model-provider:animation.answer] the host's animation reference %d of scene %d is not a name", index, scene_index);
			}

			// the campaign's own decision, taken where the original took it (read_object): the original add_new_animation, with whatever the accepted build does for a name its table
			// lacks (the T2 patch maps it to animation 0). The host's text is the truthful name; no number, and no invented name, crosses.
			animation = add_new_animation (reply.animation_name);

#ifdef EECH_MODEL_HOST_FIXTURE
			meta_animation_names++;
#endif

			return animation;
		}
		default:
		{
			debug_fatal ("host model: [model-provider:animation.answer] the host's result for animation reference %d of scene %d is %d", index, scene_index, reply.result);
		}
	}

	return -1;
}

#ifdef EECH_MODEL_HOST_FIXTURE
// TEST-ONLY (the model-host-fixture counters entry): the metadata exchange as integer counts. values: objects asked, found, absent; rows filled, rows confirmed, rows replaced; animation
// questions, names carried, lists ended.
void campaign_model_core_metadata (long *values)
{
	values[0] = meta_objects_asked;
	values[1] = meta_objects_found;
	values[2] = meta_objects_absent;
	values[3] = meta_rows_filled;
	values[4] = meta_rows_confirmed;
	values[5] = meta_rows_replaced;
	values[6] = meta_animation_asked;
	values[7] = meta_animation_names;
	values[8] = meta_animation_ends;
}
#endif

#ifdef EECH_MODEL_HOST_FIXTURE
// TEST-ONLY: how often add_3d_horizon_image took the pre-processed `.bin` branch (point colours and fog only; no point read) and how often it did not. Counted at the branch the
// real reader takes (staged patch LU1), before any refusal; an uncached call stops at the named guard right after.
static int horizon_cached_calls, horizon_uncached_calls;

void campaign_model_horizon_branch (int binary_loaded)
{
	if (binary_loaded)
	{
		horizon_cached_calls++;
	}
	else
	{
		horizon_uncached_calls++;
	}
}

void campaign_model_core_horizon (int *cached, int *uncached)
{
	*cached = horizon_cached_calls;
	*uncached = horizon_uncached_calls;
}

// TEST-ONLY (the model-host-fixture's counters entry): how many rows of the campaign's own object table still hold each physical buffer, as integer counts only (no
// address). `rows` are the table's objects 1 .. total_number_of_raw_3d_objects and `metadata_rows` those with a true point count; `points`, `faces`, `point_refs` and
// `surface_points` count the rows whose buffer pointer is not NULL. The metadata is the file's true header values and stays populated; the loader detachment is shown by
// the four buffer counts being zero while the metadata count is not.
void campaign_model_core_residency (int *rows, int *metadata_rows, int *points, int *faces, int *point_refs, int *surface_points)
{
	int
		i;

	*rows = 0;
	*metadata_rows = 0;
	*points = 0;
	*faces = 0;
	*point_refs = 0;
	*surface_points = 0;

	if (!objects_3d_data)
	{
		return;
	}

	for (i = 1; i <= total_number_of_raw_3d_objects; i++)
	{
		(*rows)++;

		if (objects_3d_data[i].number_of_points)
		{
			(*metadata_rows)++;
		}

		if (objects_3d_data[i].points)
		{
			(*points)++;
		}

		if (objects_3d_data[i].faces)
		{
			(*faces)++;
		}

		if (objects_3d_data[i].object_faces_point_plain_list)
		{
			(*point_refs)++;
		}

		if (objects_3d_data[i].surface_points)
		{
			(*surface_points)++;
		}
	}
}
#endif

// A CONSUMER of the parser's per-depth position table (popread.c: the airfield insertion functions and their WAYPOINT_ALTITUDE) is about to index `table` (the parser's
// route_waypoint_positions, `count` entries, number_of_route_waypoint_positions) with `index`. The consumers step the index by fixed amounts from the deepest depth (minus 1, minus 2,
// to a fixed floor) and read it, which holds for the routes the original models have and says nothing about a route a host answers: a three-node black/green chain has three depths,
// so fixed-wing insertion reaches -1 at its approach read. The staged patches H19/H20 wrap each such index in this function, once per indexed expression, so it is checked before anything is read, and
// it returns the same index; an index outside the table, or no table, stops the engine with a named marker. Nothing is clamped, defaulted or invented, and no minimum depth per route
// kind is assumed: only the table the parser actually built is the domain. (The parser's own reads and writes of the table are not wrapped, only popread's consumers.)
int campaign_model_route_position_index (int index, int count, const void *table)
{
	if ((!table) || (index < 0) || (index >= count))
	{
		if (route_current)
		{
			debug_fatal ("host model: [model-provider:route.position] the campaign reads depth %d of a route position table of %d (the route last parsed was scene %d kind %d)", index, table ? count : 0,
				route_current->scene, route_current->kind);
		}

		debug_fatal ("host model: [model-provider:route.position] the campaign reads depth %d of a route position table of %d", index, table ? count : 0);
	}

	return index;
}

// the parser is about to read its first start node
void campaign_model_route_starts (int starts)
{
	if (starts <= 0)
	{
		debug_fatal ("host model: [model-provider:route.start] the route of scene %d kind %d has no start node (a node on exactly one black line face)", route_current->scene, route_current->kind);
	}
}

// the parser is about to swap the primary start to the front: `index` is the later start it found primary, 0 when it found none
void campaign_model_route_primary_start (int index, int starts)
{
	if ((index < 1) || (index >= starts))
	{
		debug_fatal ("host model: [model-provider:route.start] no start node of the route of scene %d kind %d is on a green line face", route_current->scene, route_current->kind);
	}
}

// One collision candidate against the segment (mb_hostmodel.h, ENTITY_COLLISION; EEDCS-49, D3). The campaign has already decided that the candidate is collidable, moving or
// stationary, within the broad-phase radius test, and has derived its live pose; the host does the physical work. The dialogue is mb_hostentity.h, bound here to the real host
// callback and the real scene tables: the caller's intercept point and normal are outputs, never read and never sent, and only the components the original assigned are written back.
#define CAMPAIGN_ENTITY_REQUIRE_SCENE(SCENE) require_scene ((SCENE), FALSE, "an entity collision test")

#ifdef EECH_MODEL_HOST_FIXTURE
#define CAMPAIGN_ENTITY_ASK(REQUEST,REPLY) do { campaign_model_counts.entity_collision++; ask ((REQUEST), (REPLY), "entity collision"); } while (0)
#else
#define CAMPAIGN_ENTITY_ASK(REQUEST,REPLY) ask ((REQUEST), (REPLY), "entity collision")
#endif

#define CAMPAIGN_ENTITY_IDENTITY(REPLY,SCENE) check_target_point_identity ((REPLY), (SCENE), "entity_collision")

#ifdef EECH_MODEL_HOST_FIXTURE
#define CAMPAIGN_ENTITY_CANDIDATE_ASK(REQUEST,REPLY) do { campaign_model_counts.entity_candidate++; ask ((REQUEST), (REPLY), "entity candidate"); } while (0)
#else
#define CAMPAIGN_ENTITY_CANDIDATE_ASK(REQUEST,REPLY) ask ((REQUEST), (REPLY), "entity candidate")
#endif

#ifdef EECH_MODEL_HOST_FIXTURE
#define CAMPAIGN_ENTITY_SCAN_ASK(REQUEST,REPLY) do { campaign_model_counts.entity_scan++; ask ((REQUEST), (REPLY), "entity scan"); } while (0)
#else
#define CAMPAIGN_ENTITY_SCAN_ASK(REQUEST,REPLY) ask ((REQUEST), (REPLY), "entity scan")
#endif

#ifdef EECH_MODEL_HOST_FIXTURE
#define CAMPAIGN_ENTITY_POSE_ASK(REQUEST,REPLY) do { campaign_model_counts.entity_pose++; ask ((REQUEST), (REPLY), "entity pose"); } while (0)
#else
#define CAMPAIGN_ENTITY_POSE_ASK(REQUEST,REPLY) ask ((REQUEST), (REPLY), "entity pose")
#endif

// the candidate's own identity label: a bad scene-name echo of an ENTITY_CANDIDATE reply is not an entity_collision refusal
#define CAMPAIGN_ENTITY_CANDIDATE_IDENTITY(REPLY,SCENE) check_target_point_identity ((REPLY), (SCENE), "entity_candidate")

#include "mb_hostentity.h"

int campaign_model_entity_collision (int scene, const float *segment_old, const float *segment_new, const float *candidate, int candidate_count, float *point, float *normal)
{
	int
		hit;

	hit = campaign_entity_collision_exchange (scene, segment_old, segment_new, candidate, candidate_count, point, normal);

#ifdef EECH_MODEL_HOST_FIXTURE
	if (hit)
	{
		campaign_model_counts.entity_hit++;
	}
	else
	{
		campaign_model_counts.entity_miss++;
	}
#endif

	return (hit);
}

// ENTITY_POSE (mb_hostmodel.h; EEDCS-49, D3, fifth slice): an observation of a mobile candidate's pose against the host's record of its identity. The dialogue is mb_hostentity.h; nothing decides from it.
int campaign_model_entity_pose (int stage, int entity_type, int alive, const char *host_id, const float *words)
{
	int
		state;

	state = campaign_entity_pose_exchange (stage, entity_type, alive, host_id, words);

#ifdef EECH_MODEL_HOST_FIXTURE
	switch (state)
	{
		case CAMPAIGN_MODEL_POSE_RECORDED:
		{
			campaign_model_counts.pose_recorded++;

			break;
		}
		case CAMPAIGN_MODEL_POSE_UNRECORDED:
		{
			campaign_model_counts.pose_unrecorded++;

			break;
		}
		case CAMPAIGN_MODEL_POSE_RETIRED:
		{
			campaign_model_counts.pose_retired++;

			break;
		}
		case CAMPAIGN_MODEL_POSE_UNKNOWN:
		{
			campaign_model_counts.pose_unknown++;

			break;
		}
		case CAMPAIGN_MODEL_POSE_NO_ID:
		{
			campaign_model_counts.pose_no_id++;

			break;
		}
		default:
		{
			break;
		}
	}
#endif

	return (state);
}

// ENTITY_SCAN (mb_hostmodel.h; EEDCS-49, D3, fourth slice): the host's cursor over the sector lists of ITS roster. The dialogue is mb_hostentity.h.
int campaign_model_entity_scan (const float *facts, const char *entry_key, float *cursor, char *key)
{
	int
		state;

	state = campaign_entity_scan_exchange (facts, entry_key, cursor, key);

#ifdef EECH_MODEL_HOST_FIXTURE
	switch (state)
	{
		case CAMPAIGN_MODEL_SCAN_FIRST:
		{
			campaign_model_counts.scan_first++;

			break;
		}
		case CAMPAIGN_MODEL_SCAN_NEXT:
		{
			campaign_model_counts.scan_next++;

			break;
		}
		case CAMPAIGN_MODEL_SCAN_OFFER:
		{
			campaign_model_counts.scan_offer++;

			break;
		}
		default:
		{
			campaign_model_counts.scan_done++;

			break;
		}
	}
#endif

	return (state);
}

// ENTITY_SCAN, the roster's events (mb_hostmodel.h; EEDCS-49, D3, sixth slice): the campaign tells the host that an entity was inserted into a sector's list, is leaving it, or that a boot starts. The dialogue is
// mb_hostentity.h; the host decides the sector and the order and answers the key it issued.
void campaign_model_entity_roster (int phase, int entity_type, float side, const float *position, const char *identity, char *key)
{
	campaign_entity_roster_exchange (phase, entity_type, side, position, identity, key);

#ifdef EECH_MODEL_HOST_FIXTURE
	switch (phase)
	{
		case CAMPAIGN_SCAN_PHASE_INSERT:
		{
			campaign_model_counts.roster_placed++;

			break;
		}
		case CAMPAIGN_SCAN_PHASE_REMOVE:
		{
			campaign_model_counts.roster_removed++;

			break;
		}
		default:
		{
			campaign_model_counts.roster_cleared++;

			break;
		}
	}
#endif
}

// ENTITY_SCAN, the city events (mb_hostmodel.h; EEDCS-49, D3, seventh slice). The dialogue is mb_hostentity.h; the host holds the city lists, the records and their object identities.
void campaign_model_entity_city (int phase, int entity_type, const char *identity, char *key)
{
	campaign_entity_city_exchange (phase, entity_type, identity, key);

#ifdef EECH_MODEL_HOST_FIXTURE
	switch (phase)
	{
		case CAMPAIGN_SCAN_PHASE_CHILD_INSERT: { campaign_model_counts.city_placed++; break; }
		case CAMPAIGN_SCAN_PHASE_CHILD_REMOVE: { campaign_model_counts.city_removed++; break; }
		case CAMPAIGN_SCAN_PHASE_CHILD_FIRST: { campaign_model_counts.city_first++; break; }
		case CAMPAIGN_SCAN_PHASE_CHILD_NEXT: { campaign_model_counts.city_next++; break; }
		case CAMPAIGN_SCAN_PHASE_LINK: { campaign_model_counts.city_linked++; break; }
		default: { campaign_model_counts.city_named++; break; }
	}
#endif
}

// ENTITY_CANDIDATE (mb_hostmodel.h; EEDCS-49, D3, third slice): what the host decides about one candidate of the scan. The dialogue is mb_hostentity.h; the facts are read by the caller.
int campaign_model_entity_candidate (int scene, const float *facts, const char *host_id, float *pose, int *pose_count)
{
	int
		state;

	state = campaign_entity_candidate_exchange (scene, facts, host_id, pose, pose_count);

#ifdef EECH_MODEL_HOST_FIXTURE
	switch (state)
	{
		case CAMPAIGN_MODEL_CANDIDATE_SKIP:
		{
			campaign_model_counts.candidate_skip++;

			break;
		}
		case CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED:
		{
			campaign_model_counts.candidate_updated++;

			break;
		}
		case CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL:
		{
			campaign_model_counts.candidate_normal++;

			break;
		}
		case CAMPAIGN_MODEL_CANDIDATE_BUILDINGS:
		{
			campaign_model_counts.candidate_buildings++;

			break;
		}
		case CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED:
		{
			campaign_model_counts.candidate_fixed++;

			break;
		}
		case CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT:
		{
			campaign_model_counts.candidate_placement++;

			break;
		}
		default:
		{
			campaign_model_counts.candidate_test++;

			break;
		}
	}
#endif

	return (state);
}

#ifdef EECH_MODEL_HOST_FIXTURE
// TEST-ONLY (feature model-host-fixture): overwrites every campaign-side array the four ORIGINAL accessors read, so that a read of any of them gives a visibly wrong
// answer (NaN bounds and radius; a collision object index that is outside the object table). The campaign then runs on host answers alone: if an accessor, or code
// that bypasses them, still read the arrays, its result would change. The main object index, the scene links and the sub-object trees are left alone; they are
// hierarchy data, not an answer of the four accessors.
void campaign_model_poison_arrays (void)
{
	int
		scene;

	for (scene = 0; scene < OBJECT_3D_LAST; scene++)
	{
		struct OBJECT_3D_SCENE_DATABASE_ENTRY
			*entry = &objects_3d_scene_database[scene];

		entry->radius = NAN;
		entry->bounding_box.xmin = entry->bounding_box.xmax = entry->bounding_box.ymin = entry->bounding_box.ymax = entry->bounding_box.zmin = entry->bounding_box.zmax = NAN;
		entry->bounding_box2.xmin = entry->bounding_box2.xmax = entry->bounding_box2.ymin = entry->bounding_box2.ymax = entry->bounding_box2.zmin = entry->bounding_box2.zmax = NAN;
		entry->collision_object_index = 0x3fffffff;
	}
}
#endif

#endif

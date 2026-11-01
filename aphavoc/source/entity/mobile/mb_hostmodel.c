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
#include <string.h>

#include "mb_hostmodel.h"

static campaign_model_host current_host;
static int host_installed;

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

// the custom loader's polygon faces are streamed through a local buffer for their one validation; a chunk that was not read in full is the same refusal
void campaign_model_loader_short (const char *what)
{
	debug_fatal ("host model: [model-provider:object.loader] %s: the file ends inside the region, a short read is not read as valid data", what);
}

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

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

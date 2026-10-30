/*
 * Host-owned collision-model data (EEDCS-50, D3). The host owns the physical models: the collision triangles of a scene, the bounds the collision tests use and the
 * radius. Campaign code keeps what it does with them: which shape an entity has, which entities a query visits, the transform of a ray into a model's frame, the hit
 * decision and every consequence (until the physical solvers move to the host too, D3 / G3; this exchange is the intermediate boundary of that move and is not a
 * final one). The four public accessors of the 3d module that used to read the campaign DLL's own model arrays,
 *
 *     get_object_3d_collision_object_geometry_triangle    modules/3d/3dobjdb.c
 *     get_object_3d_bounding_box
 *     get_object_3d_bounding_box_without_lines
 *     get_object_3d_radius                                modules/3d/3dobjvis.c
 *
 * are replaced, by strict count-checked build patches (eech-engine-sys/build/patches.rs, MODEL_HOST_PATCHES), by the requests below. The original array-reading bodies
 * are not compiled into the headless campaign module; there is no fallback to them, and no answer is invented when the host has none.
 *
 * THE PUBLIC CONTRACT is the scene: the EECH scene index (object_3d_index_numbers), whose name is compiled into every build from the same header (3dmodels.h). A host
 * chooses its own storage for its models. Nothing of the campaign's private loader crosses the boundary: no object-table index (the numbers the legacy loader gives a
 * scene's main and collision objects are private numbering and are neither sent nor asked for), no pointer, offset or layout, no entity index, and no hash of physical
 * values. What a host declares about a scene is semantic: whether it has a model at all (`loaded`: a scene the host has no model for is not a scene it can answer for),
 * whether the model has a collider (`collider`), and the physical numbers.
 *
 * The exchange is one synchronous host callback with three operations, each a single request and a single reply:
 *
 *   CATALOGUE  the number of scene indices the host's models cover (OBJECT_3D_LAST). Asked once, at boot, straight after the campaign loaded its scene database.
 *   SCENE      one scene's public facts: its compiled name (an echo that the host and the campaign mean the same scene by the same index; empty for the invalid index 0),
 *              whether the host has a model for it (`loaded`), whether that model has a collision model (`collider`; only a loaded scene has one), the radius, and the
 *              two bounds (bounding_box_without_lines is the one the collision tests use). Asked for every scene index 0 to OBJECT_3D_LAST - 1 at boot. Scene 0, the
 *              invalid index, is the null scene: not loaded, no collider, all zeros, as the original's zeroed entry was.
 *   TRIANGLE   one step of the collision-triangle cursor: the scene and the depth (the caller's running index, never negative). The host answers FOUND with the nine
 *              floats (point 0, 1, 2, each x y z), END (the scene has a collider and the list is over at this depth), or ABSENT (the scene is loaded and has no collider).
 *              END and ABSENT are the original accessor's FALSE: the caller's buffer is left untouched. Asked live for every call, never cached. FOUND and END for a scene
 *              declared without a collider, ABSENT for one declared with, and any of the three for a scene declared not loaded, contradict the host's own declaration and
 *              stop the engine before any word of the caller's buffer is written.
 *
 * The bounds and the radius are copied once, at boot, into immutable tables owned by this module (host-authored data, valid for the life of the process and never
 * written again). Callers hold the bounds pointer across loop iterations, so it must be stable; a rotating cache would not be. The tables are a snapshot of the host's
 * answers, not an authority of the campaign's own.
 *
 * A request the host cannot answer, or answers with something malformed (a non-finite float, a result that is not one of the three, a name that is not the scene's, a
 * collider on a scene that is not loaded, an operation answered with another's reply), stops the engine with the host's reason; a hosted engine with no model callback
 * stops at boot. An unsupported or missing answer is therefore always distinct from the legitimate FALSE of a scene that has no collider.
 *
 * Lifecycle of the installed callback: the engine entry installs (refreshes) the callback on every entry that carries one, and an entry without one leaves the installed
 * one in place (the model is asked about from every entry, and a provider is fixed at boot). The `user` pointer an entry carries is per-entry (it names that entry's host
 * context, which ends with the entry) and is therefore NEVER retained: the callback is always called with a NULL `user`. The Rust engine's provider is a process-lifetime
 * closure of its own, not reached through `user`; another host must likewise not depend on `user` for the model callback.
 */
#ifndef MB_HOSTMODEL_H
#define MB_HOSTMODEL_H

#include <stddef.h>

#define CAMPAIGN_MODEL_ERROR_MAX 512
/* the longest scene name (without the terminator) a reply may carry */
#define CAMPAIGN_MODEL_NAME_MAX 127

enum campaign_model_op
{
	CAMPAIGN_MODEL_CATALOGUE = 1,
	CAMPAIGN_MODEL_SCENE,
	CAMPAIGN_MODEL_TRIANGLE
};

enum campaign_model_triangle_result
{
	CAMPAIGN_MODEL_TRIANGLE_FOUND = 1,
	CAMPAIGN_MODEL_TRIANGLE_END,
	CAMPAIGN_MODEL_TRIANGLE_ABSENT
};

typedef struct campaign_model_request
{
	int op;
	/* SCENE and TRIANGLE */
	int scene;
	/* TRIANGLE */
	int depth;
} campaign_model_request;

typedef struct campaign_model_reply
{
	/* CATALOGUE */
	int scene_count;
	/* SCENE: 0 or 1 each */
	int loaded;
	int collider;
	float radius;
	/* xmin, xmax, ymin, ymax, zmin, zmax */
	float bounding_box[6];
	float bounding_box_without_lines[6];
	char name[CAMPAIGN_MODEL_NAME_MAX + 1];
	/* TRIANGLE */
	int result;
	/* point 0 x y z, point 1, point 2 */
	float points[9];
} campaign_model_reply;

typedef int (*campaign_model_fn)
(
	const campaign_model_request *request,
	campaign_model_reply *reply,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_model_host
{
	campaign_model_fn model;
	void *user;
} campaign_model_host;

/* Installs the callback for the rest of the process (see the lifecycle above): a NULL host or one with no callback leaves the installed one; the host's `user` is not kept. */
void campaign_model_set_host (const campaign_model_host *host);

/* Asks the host for the catalogue and every scene, cross-checks the scene names, checks each scene's declaration is consistent, and fills the immutable tables. Called once
   per scene-database load, at the end of initialise_3d_objects (the staged build patch), so a missing provider, a catalogue that disagrees or a malformed scene stops the
   boot. The campaign's own scene arrays are not read for acceptance. */
void campaign_model_install_scenes (void);

#endif

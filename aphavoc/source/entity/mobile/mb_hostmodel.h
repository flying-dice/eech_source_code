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
 *   OBJECT_TABLE / OBJECT / ANIMATION   the metadata of the physical models the campaign's compatibility rows mirror (EEDCS-50, D3, L2: the common and custom model-file
 *              loaders no longer parse 3dobjs.bin, its point / surface-point / normal / texture-point files or a custom .EEO). The campaign keeps its own private object numbering,
 *              its history of named objects and their sharing, and the scene / hierarchy loader; the host owns what the model files say.
 *              OBJECT_TABLE (asked once, before any scene is read): the number of objects the common database holds (a count of what the host's files contain; `object_count`).
 *              The room the campaign keeps for custom objects, the ids it gives them and its history of named objects are the campaign's own allocation policy: they are neither
 *              asked for nor answered, and a provider need not know them.
 *              OBJECT: asked after a scene's records are read, for each object the scene names. The request is SEMANTIC: the scene (the EECH scene index), the `role` the
 *              object has in it (0 main, 1 approximation, 2 shadow polygon object, 3 collision object, 4 sub-object) and the `depth` cursor (the approximation or the
 *              sub-object, counted as the scene's files lay them out). No object number, pointer, offset or layout is sent or asked for. The reply echoes the scene's compiled
 *              name and is FOUND (the six header counts the compatibility rows hold: points, faces, surfaces, point normals, lighting normals, culling normals offset;
 *              the radius; both bounding boxes) or ABSENT (the host's files name no object in that role). A refusal is an error of the provider. The campaign fills its private
 *              CUSTOM_OBJECT is the same question about the scene a custom scene file defines (the campaign reads the database's record of a scene, and later the custom file that
 *              replaces it, and each definition names its own objects); OBJECT is always about the database's record. The campaign fills its private
 *              CUSTOM_ORDER: the campaign walks the custom scene files (OBJECTS/<directory>/<SCENE NAME>.EES) in the file system's own order, which is not sorted, and what a custom
 *              definition sees depends on that order (the number the first named object takes and the table object it replaces, the history of named objects of its directory, which
 *              definition of a scene is read last). Before it reads any of them it ANNOUNCES the walk: for each file in order, `scene` (the EECH scene index), `depth` (the
 *              position, from 0, one after another) and `key` (the directory name); the host answers FOUND if it has that file and ABSENT if not (a refusal that stops the engine).
 *              A final request with `role` 1 and `depth` the number of files ends the announcement; the host answers END and, if its own order differs, replays its definitions
 *              in the campaign's. CUSTOM_OBJECT and ANIMATION then name a custom definition by its POSITION in that walk (`definition`), which the campaign verifies against the
 *              announcement as it reads each file. Positions are the campaign's own ordinals of its walk; no object number or layout crosses.
 *              The announcement is ONE transaction of at most 4096 files, each (scene, directory) once, steps one after another from 0; a closed announcement is final (a late step
 *              is refused and a new provider is a new transaction). The campaign's actual traversal must then open every announced file, each read or skipped (a file whose scene
 *              version the original does not read is skipped without a read, and the walk position it held is consumed as skipped), and the boot stops by name if it does not.
 *              The host needs only the database and an INVENTORY of the custom files to take the announcement; it replays its custom definitions in the announced order after it,
 *              and gives no answer about a final scene before that.
 *              The campaign fills its private
 *              rows from FOUND answers before anything reads them; an object two references name differently, or a name the host has no object for, stops the engine.
 *              ANIMATION: for an .EES scene, the ordered animated-surface references of its objects in the order the original's add_animation_object visits them. FOUND
 *              carries `animation_name`, the LOGICAL name of the texture animation (never a file's animation index: the host resolves the file's own numbering to the name); END
 *              ends the list, ABSENT says the scene is not an .EES scene (it keeps its list in its own record). A reference the host cannot name (an index with no compiled name,
 *              bytes that are not text) is a refusal of the provider. The campaign maps the name to its own private animation table with its original add_new_animation, so the
 *              decision for a valid name its table lacks stays the campaign's (the accepted build's T2 tolerance), and de-duplicates in its own add_animation, as the original did.
 *
 *   ROUTE_NODE / ROUTE_FACE   the route data of an airfield scene (EEDCS-50, D3). A route is the line object a scene files under a sub-object kind (the landing, take-off
 *              and holding routes of airfields); the campaign's own hierarchy lookup still says whether the scene has one, and the campaign keeps the route
 *              planning (the tree, the primary and start nodes, the end-slot matching). What the host owns is the model data behind it: the request carries the scene, the
 *              sub-object kind and a cursor, and the host answers one step of the ORDERED nodes (ROUTE_NODE: FOUND with the node's decoded x y z, or END) or of the ORDERED
 *              line faces (ROUTE_FACE: FOUND with the two node indices already resolved, the surface's colour and its polygon bit, or END), in the order the original
 *              walks them. The reply echoes the kind's compiled name, as the scene reply echoes the scene's. The intrinsic decode (the stored short, the bounds, the surface-point
 *              indirection) is the host's; the campaign receives decoded floats and node numbers only. ABSENT means the host has no route of that kind for the scene; the
 *              campaign asks only for a route its own hierarchy lookup found, so an ABSENT is a disagreement and stops the engine, as does a malformed or refused answer.
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
	CAMPAIGN_MODEL_TRIANGLE,
	CAMPAIGN_MODEL_ROUTE_NODE,
	CAMPAIGN_MODEL_ROUTE_FACE,
	CAMPAIGN_MODEL_OBJECT_TABLE,
	CAMPAIGN_MODEL_OBJECT,
	CAMPAIGN_MODEL_ANIMATION,
	CAMPAIGN_MODEL_CUSTOM_OBJECT,
	CAMPAIGN_MODEL_CUSTOM_ORDER
};

enum campaign_model_object_role
{
	CAMPAIGN_MODEL_ROLE_MAIN = 0,
	CAMPAIGN_MODEL_ROLE_APPROXIMATION,
	CAMPAIGN_MODEL_ROLE_SHADOW,
	CAMPAIGN_MODEL_ROLE_COLLISION,
	CAMPAIGN_MODEL_ROLE_SUB_OBJECT
};

/* the longest animation name (without the terminator) a reply may carry */
#define CAMPAIGN_MODEL_ANIMATION_NAME_MAX 127

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
	/* TRIANGLE; ROUTE_NODE and ROUTE_FACE: the cursor (the node or line-face index) */
	int depth;
	/* ROUTE_NODE and ROUTE_FACE: the route's sub-object kind (object_3d_sub_object_index_numbers); `scene` is the airfield scene */
	int kind;
	/* OBJECT: the role the object has in `scene` (campaign_model_object_role); `depth` is the approximation or sub-object cursor. ANIMATION: `depth` is the reference cursor */
	int role;
	/* Which DEFINITION of the scene is meant. OBJECT: the scene's n-th record of the scene database (n from 0). CUSTOM_OBJECT and ANIMATION: the POSITION, from 0, of the custom
	   scene file in the walk the campaign announced with CUSTOM_ORDER, which it verifies against the file it opens */
	int definition;
	/* CUSTOM_ORDER: the name of the directory under OBJECTS that holds the custom scene file (NUL-terminated) */
	char key[CAMPAIGN_MODEL_NAME_MAX + 1];
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
	/* point 0 x y z, point 1, point 2; ROUTE_NODE: the node's x y z in the first three */
	float points[9];
	/* ROUTE_FACE: the two node indices the face joins, the surface's colour (red, green, blue) and its polygon bit */
	int route_point[2];
	int route_colour[3];
	int route_polygons;
	/* OBJECT_TABLE: the number of objects the common database holds */
	int object_count;
	/* OBJECT (FOUND): points, faces, surfaces, point normals, lighting normals, culling normals offset */
	int object_counts[6];
	float object_radius;
	/* xmin, xmax, ymin, ymax, zmin, zmax of the bounding box, then of the second bounding box */
	float object_bounds[12];
	/* ANIMATION (FOUND): the logical name of the texture animation */
	char animation_name[CAMPAIGN_MODEL_ANIMATION_NAME_MAX + 1];
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

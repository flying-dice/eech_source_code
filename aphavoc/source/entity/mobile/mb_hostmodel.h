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
 * The exchange is one synchronous host callback with three operations, each a single request and a single reply. A scene is the EECH scene index
 * (object_3d_index_numbers), a plain integer that the host and the campaign both derive from the same data; no EECH pointer, offset, object layout or entity index
 * crosses the boundary, and neither does a hash of the physical values: the host answers with its own numbers.
 *
 *   CATALOGUE  the number of scene indices the host's models cover (OBJECT_3D_LAST). Asked once, at boot, straight after the campaign loaded its scene database.
 *   SCENE      one scene's identity and physical scalars: the main object index and the collision object index (both only to cross-check the shape identity the
 *              campaign and the host share; the host's values are never compared with the campaign's physical values), the radius, and the two bounds
 *              (bounding_box_without_lines is the one the collision tests use). Asked for every scene index 0 to OBJECT_3D_LAST - 1 at boot.
 *   TRIANGLE   one step of the collision-triangle cursor: the scene and the depth (the caller's running index, never negative). The host answers FOUND with the nine
 *              floats (point 0, 1, 2, each x y z), END (a collision object is present and the list is over at this depth), or ABSENT (the scene has no collision object).
 *              END and ABSENT are the original accessor's FALSE: the caller's buffer is left untouched. Asked live for every call, never cached.
 *
 * The bounds and the radius are copied once, at boot, into immutable tables owned by this module (host-authored data, valid for the life of the process and never
 * written again). Callers hold the bounds pointer across loop iterations, so it must be stable; a rotating cache would not be. The tables are a snapshot of the host's
 * answers, not an authority of the campaign's own.
 *
 * A request the host cannot answer, or answers with something malformed (a non-finite float, a result that is not one of the three, ABSENT for a scene the catalogue
 * says has a collision object, an operation answered with another's reply), stops the engine with the host's reason; a hosted engine with no model callback stops at
 * boot. An unsupported or missing answer is therefore always distinct from the legitimate FALSE of a scene that has no collider.
 */
#ifndef MB_HOSTMODEL_H
#define MB_HOSTMODEL_H

#include <stddef.h>

#define CAMPAIGN_MODEL_ERROR_MAX 512

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
	/* SCENE */
	int main_object;
	int collision_object;
	float radius;
	/* xmin, xmax, ymin, ymax, zmin, zmax */
	float bounding_box[6];
	float bounding_box_without_lines[6];
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

/* Installs the host for the rest of the process: like the terrain host, and unlike the dialogue hosts, it is kept across engine entries, since the campaign asks
   about the models from every entry and the provider is fixed at boot. The first entry that carries a callback installs it; a NULL host or one with no callback
   leaves the installed one. */
void campaign_model_set_host (const campaign_model_host *host);

/* Asks the host for the catalogue and every scene, cross-checks the shape identity with the scene database the campaign has just loaded and fills the immutable
   tables. Called once, at the end of initialise_3d_objects (the staged build patch), so a missing provider, a catalogue that disagrees or a malformed scene stops the
   boot. */
void campaign_model_install_scenes (void);

#endif

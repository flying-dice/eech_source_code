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
 *   TARGET_POINT_OFFSET   a scene's LOCAL target-point offset (EEDCS-52, D3): the three floats initialise_target_point_offsets (wn_dbase.c) used to compute by constructing an
 *              instance of the scene, searching it for a TARGET_POINT sub-object, taking that sub-object's world position under an identity pose or, failing that, the centre of
 *              the scene's bounding box with y clamped at zero. The host owns that physical work now (the same model data and the same arithmetic as the original, on the host's
 *              own store); the campaign asks once per scene per initialiser invocation (the real boot invokes it twice, and each invocation asks again), receives the three
 *              coordinates in the first three words of `points`, and copies them into the private `target_point_offsets` table that the direct guidance consumer
 *              (gd_attck.c calculate_attack_guide_intercept_point) reads. The reply echoes the scene's compiled name and must be FOUND with three finite floats; an END or
 *              ABSENT, a non-finite coordinate, a refusal or a provider that is missing stops the engine with the host's reason, and there is no fallback to the original
 *              construction. A scene no file defines is a valid catalogue slot with the zero bounding box and answers a zero offset, which is not a refusal; the invalid index 0 is
 *              not asked (its entry is the zero the original assigned).
 *
 *   TARGET_POINT_CHAIN / TARGET_POINT_WORLD / TARGET_POINT_FIXED   the three questions behind get_local_entity_target_point (en_maths.c; EEDCS-52, D3). The host owns the
 *              physical work: WHICH sub-object is the target point and WHERE it is. The campaign keeps what is its own: whether the entity has a live instance or is fixed, the
 *              exact fallbacks (no target point, no instance, not fixed, the invalid shape: the entity position, bit for bit), the write of the instance viewpoint only when a target
 *              point exists, the order in which it reads the entity, and the use of the answer.
 *              CHAIN: `scene` is the instance's scene. ABSENT means the scene has no TARGET_POINT sub-object (the original's second branch); FOUND carries `chain_length` (1 to
 *              CAMPAIGN_MODEL_CHAIN_MAX) and, in `chain_offsets`, the position of the first TARGET_POINT sub-object among its siblings at each level, topmost level first
 *              (the original's depth-0 search result). The host names the chain; the campaign does not search for it.
 *              WORLD: the CURRENT placement the host needs and cannot hold, because the campaign mutates it (device angles, animation, scale): `pose` carries, in this order, the
 *              instance's world position (3), attitude (9, row-major) and relative scale (3), then for each level of the chain just named the live sub-instance's relative
 *              position (3), heading, pitch, roll and relative scale (3) (`pose_count` = 15 + 9 * chain_length). The host applies them along the chain it named, with the original's
 *              arithmetic, and answers the world position in the first three words of `points`; a chain length that is not the one it named is a refusal, never a default pose.
 *              FIXED: the position the original had copied (3) and, only when the cached host offset of the shape has a horizontal part, the entity heading (1); the host adds the
 *              shape's offset as the original did (including the zero-heading quirk: no horizontal addition) and answers the world position. A heading the host needs and was not
 *              given is a refusal.
 *              Every reply echoes the scene's compiled name and must be FOUND (CHAIN also ABSENT) with finite floats; a refusal, a malformed or non-finite answer, an END or a missing
 *              provider stops the engine with the host's reason. There is NO fallback to the original sub-object search, world-viewpoint solver or fixed rotation.
 *
 *   ENTITY_COLLISION   whether a line segment meets ONE collision candidate (EEDCS-49, D3): the narrow phase of get_line_of_sight_collision_entity (en_cllsn.c). The host owns the
 *              physical work: the transform of the scene's collision triangles (or, for a scene with none, of its bounding box without lines) into the world under the pose, the
 *              choice of the sweep positions of a moving candidate to test (its speed against its body length), the order of the faces and the intersection arithmetic. The
 *              campaign keeps what is its own and sends it: WHICH candidates and in what order (the sector scan, the exclusions, the broad-phase radius test and the tests that
 *              decide whether a candidate is collidable, moving or stationary), the candidate's live pose derived from its entity state in the original's order (a fixed object's
 *              terrain-aligned placement included, so the terrain questions it asks keep their order), the hit decision's consequences and the returned entity.
 *              `scene` is the candidate's scene (never 0). `pose_count` is 18 (a STATIONARY candidate) or 22 (a MOVING one) and the floats of `pose` are, in order: the segment's old
 *              end (3) and new end (3), then
 *                stationary: the position (3) and the attitude (9, row-major);
 *                moving:     the old position (3), the new position (3), the attitude (9) and the speed (1, the magnitude of the candidate's motion vector).
 *              THE OUTPUTS ARE OUTPUTS. The intercept point and face normal are never sent and never read: the original's intersection helpers assign the whole normal first, whatever
 *              the segment does, and the whole point only after the segment has crossed the face's plane, and a real caller (answer_weapon_step_lookup, wn_move.c) passes a normal
 *              it never initialised. So a miss can still have assigned the normal, or both. The reply says which components the original's arithmetic assigned and what it computed:
 *              `write_mask` (bits 0 to 2 the point, bits 3 to 5 the normal; each vector is written whole, a point implies a normal, and a hit is 0x3f) and, in the first six words of
 *              `points`, the point then the normal, each unassigned word zero. The campaign checks all of it first (mb_hostentity.h) and then writes ONLY the assigned components, the
 *              normal first, leaving every other word of the caller's buffers exactly as it was and unread. `result` is FOUND for a hit and END for a miss; the reply echoes the scene's
 *              compiled name. A refusal, a malformed answer (a mask or a word outside the domain, a hit that has not assigned both vectors), any other result or a missing provider stops
 *              the engine with the host's reason before anything is written; a miss is never invented for a question the host could not answer. There is NO fallback to the original
 *              arithmetic in the hosted module; the explicit original-characterisation profile (feature collision-fixture) keeps it. The model's data is the host's final announced-order
 *              store, the same one the other operations read; no second store is loaded for the question.
 *
 *   ENTITY_CANDIDATE   what the host decides about ONE candidate of the scan, BEFORE its narrow phase (EEDCS-49, D3, third slice): the per-candidate decisions of
 *              get_line_of_sight_collision_entity (en_cllsn.c) are the host's. The campaign keeps WHICH candidates are offered and in what order (the sector lists and the exclusions of the
 *              source and the target), the reads of the entity's own state, the two services that have effects of their own and the consequences of a hit. The host decides: whether a
 *              non-mobile candidate is skipped because the sector's tallest structure is below the segment, which branch applies (mobile first, then fixed: a city, a building listed directly
 *              in a sector, any other), whether a mobile candidate is MOVING (the square of its motion vector above one), its sweep ends from the UPDATED answer and the frame time, the
 *              broad-phase box of the scene's radius + 10 m, the city's height-then-xz box, and a stationary fixed candidate's live pose from its heading or its terrain-aligned face normal.
 *              The host's own model data supplies the radius, looked up ONLY when a branch reaches its broad phase: a candidate the original skipped needs no scene and no model.
 *              THE FACTS ARE STAGED, in the order the original needed them, so that the campaign reads nothing the original did not read on that branch. The campaign first sends what the
 *              original read for every candidate: the segment, the mobile flag, the sector's tallest structure height and, for a mobile candidate, its shape (`scene`), position and motion
 *              vector (a building listed under a city whose box was met sends its shape and position). A candidate that is not mobile in a sector below the segment is SKIPPED at this stage,
 *              before the original read its fixed flag or any shape, with `scene` 0. Otherwise the host asks for the facts of the next stage (NEED_FIXED, NEED_PLACEMENT) and the campaign
 *              asks again with them filled in.
 *              `pose_count` is 44 and the floats of `pose` are the FACTS, in order: the segment's old end (3) and new end (3); the collision-test-mobile flag, the collision-test-fixed flag,
 *              the entity is a city, the entity is a city building, the candidate is a building listed under a city whose box was met, the city has a first building (each exactly 0 or 1);
 *              the sector's tallest structure height; the position (3), the motion vector (3), the frame time; the UPDATED answer (0 not asked, 1 false, 2 true); the heading, the
 *              align-with-terrain flag, the terrain face normal given (0 or 1) and that normal (3); the attitude (9, row-major); the city's bounding box min (3) and max (3); the fixed facts
 *              were read (0 or 1); the placement facts were read (0 or 1). Facts that do not apply, or were not read yet, are zero. `scene` is 0 until the shape has been read.
 *              The host answers a `candidate_state`: SKIP (nothing more to do for the candidate); NEED_UPDATED (the campaign performs its UPDATED getter, which clears the entity's flag, exactly
 *              once and only when the host names it, and asks again with the fact filled in); NEED_FIXED (a non-mobile candidate the sector skip did not remove: the campaign reads the fixed
 *              flag and, if it is fixed, whether the entity is a city or a city building and what that branch reads: a city's first building, position and box, any other candidate's shape and
 *              position; facts 7 to 9, 11, 13 to 15, 36 to 41 and the scene, and fact 42); NEED_PLACEMENT (the broad phase has passed: the campaign reads the attitude of a mobile candidate, or
 *              the heading and align flag of a fixed one, and for a building listed under a city the collision flags as well; fact 43); NEED_NORMAL (the campaign asks the terrain for the face
 *              normal at the candidate's position, which has the effects of the terrain question, and asks again with the normal filled in); BUILDINGS (a city's box is met: the campaign offers
 *              its buildings one by one in its list order, each as a building listed under the city, which is not skipped by the sector's tallest structure again); or TEST with the narrow
 *              phase's candidate pose in `candidate` (`candidate_count` floats: 12 for a stationary candidate, 16 for a moving one), which the campaign then sends with ENTITY_COLLISION.
 *              The question is a pure function of its facts: no state is kept between questions, so the decision is the one the original made in one pass. The reply echoes the scene's compiled
 *              name (empty for scene 0). A refusal (a scene the branch needs and the model data cannot give is refused, never defaulted), a state the facts do not allow (NEED_UPDATED for a
 *              candidate that is not mobile or has the answer already, NEED_FIXED for a mobile candidate or one whose fixed facts were read, NEED_PLACEMENT when it was read, for a city, a building listed directly in a sector, an entity that is neither mobile nor fixed or a candidate that has no scene (refused before the campaign reads a heading, an align flag or an attitude), NEED_NORMAL
 *              for one that does not align with the terrain, has no placement yet or has the normal, BUILDINGS for a non-city, a TEST with scene 0), a pose of another size or of non-finite
 *              floats, a name that is not the scene's, or a missing provider stops the engine with the host's reason; nothing is invented. No entity identity crosses: a candidate is named only
 *              by the order the campaign offers it.
 *
 *   ENTITY_SCAN        the ENUMERATION of the scan, which precedes every ENTITY_CANDIDATE question (EEDCS-49, D3, fourth slice): which candidates are visited, in what order, which are skipped and
 *              when the scan ends are the host's. The host decides, from the segment and the campaign's world map: the segment's min and max, the sector window (the box widened by 150 m and clamped to
 *              the map, as sector indices), the traversal (x outer, z inner), that each sector's list is read from its TAIL backwards, that the source and the target are skipped, that an entry is
 *              OFFERED for its decisions, and that the FIRST candidate that hits ends the scan: a farther candidate in an earlier sector or list position wins over a nearer one, so the order IS the
 *              result. `scene` is 0 and `pose_count` is 16: the segment's old end (3) and new end (3); the sector side length, the world map's min x, min z, max x and max z; the PHASE (0 BEGIN, 1 ENTRY
 *              an entry is served, 2 END the cursor's list has no more entries, 3 RESOLVED the offered entry's decisions are finished); the cursor sector x and z the host last gave; whether the served
 *              entry is the source or the target (0 or 1); whether the offered entry hit (0 or 1). The host answers a `candidate_state`: FIRST (serve the TAIL of the list of the sector named in
 *              `candidate[0]` and `candidate[1]`, whole numbers), NEXT (serve the predecessor of the current entry), OFFER (offer the current entry for its decisions, ENTITY_CANDIDATE, and report whether it
 *              hit), DONE_NONE (the scan ends, no hit) or DONE_HIT (it ends with the entry offered last). The question is a pure function of its facts: the host keeps nothing between questions and the
 *              cursor travels in the facts and the answer, so there is no host state to lose and no second store. THE CAMPAIGN SERVES THE HOST'S CURSOR over its own sector lists: the entity heap and
 *              its list order are the campaign's TRANSITIONAL BACKING dependency (REQUIRED to move: they are the live state a roster must follow), and which entry is the source or the target is a
 *              comparison only the heap can make, reported as a fact. Nothing is copied, a list cannot change inside one scan (nothing in it links, unlinks or releases an entity), and no raw pointer or
 *              index crosses: an entry is named by the host's cursor only. A state the phase does not allow (an excluded entry offered, a resolved hit continued, a served entry that is not offered or
 *              skipped, a cursor that is not whole numbers inside the map), a refusal, a missing provider or a scan that does not end in a bounded number of questions stops the engine by marker.
 *
 *   ENTITY_POSE        an OBSERVATION of the live pose of a mobile candidate (EEDCS-49, D3, fifth slice; see campaign_model_pose_class above): the campaign names the candidate by its semantic host
 *              identity and offers the words it read, and the host reports whether it keeps a physical record of that identity (the one its movement steps use) and how the words compare with it.
 *              It exists to characterise the collaborator contract (which mobile offers have an identity and a record, and whether the campaign's mirror, its vehicle motion vector and its writes
 *              outside the host's steps agree with the record) BEFORE a candidate's pose is taken from the host. Its answer is a conformance assertion (a mirror difference stops the engine), so it is not decision-neutral, but it changes no physical read the original made and no order: it is asked
 *              beside the original's reads, nothing is skipped or reordered for it, and no entity pointer, index or heap layout crosses. It adds three read-only metadata reads (the entity's type, its alive flag and its
 *              roster identity; see the contract above) and only mobile candidates of a sector list are observed.
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
	CAMPAIGN_MODEL_CUSTOM_ORDER,
	CAMPAIGN_MODEL_TARGET_POINT_OFFSET,
	CAMPAIGN_MODEL_TARGET_POINT_CHAIN,
	CAMPAIGN_MODEL_TARGET_POINT_WORLD,
	CAMPAIGN_MODEL_TARGET_POINT_FIXED,
	CAMPAIGN_MODEL_ENTITY_COLLISION,
	CAMPAIGN_MODEL_ENTITY_CANDIDATE,
	CAMPAIGN_MODEL_ENTITY_SCAN,
	CAMPAIGN_MODEL_ENTITY_POSE
};

/* ENTITY_POSE (EEDCS-49, D3, fifth slice; an OBSERVATION): the words the campaign read from a mobile candidate, offered to the host beside the candidate's semantic host identity, and what the host makes of them.
   `pose_count` is 15: the position (3) and the motion vector (3) (STAGE_OFFER: the words the original read for every mobile candidate), or those and the attitude (9) (STAGE_PLACEMENT: read when the broad phase
   passed); the words of a stage that was not read are zero. `depth` is the stage, `kind` the entity's type, `definition` whether it is alive (0 or 1) and `host_id` its semantic host identity (empty when the roster has none;
   the roster's whole identity domain of 1 to 128 bytes of text is carried, never shortened and never skipped). The reply: `candidate_state` the class (campaign_model_pose_class), `candidate_count` the kind of the host's
   record (0 none, 1 routed vehicle, 2 ship, 3 person, 4 fixed-wing aircraft, 5 helicopter) and `write_mask` the differences (campaign_model_pose_difference bits). THE ANSWER CHOOSES NO POSE AND NO RESULT (a candidate that agrees is offered, tested and skipped exactly as without it), but it is NOT decision-neutral: it is a CONFORMANCE assertion. The host's physical record decides a mobile
   candidate's pose (ENTITY_CANDIDATE), and a position, attitude or motion-vector difference between the campaign's mirror and that record stops the engine by marker entity_pose.mirror; the other bits only describe the record.

   WHAT THE OBSERVATION READS. The words are the ones the original's scan read (position and motion vector for every mobile candidate, the attitude in its narrow phase): they are copied from the facts already read and are no
   new read. The observation adds exactly THREE read-only metadata reads per question, which the original did not make: the entity's TYPE (a direct field), its ALIVE flag (the mobile's own flag) and its roster HOST IDENTITY
   (a lookup in the roster's mapping, which stops the engine with its own named assertion only if that mapping is already inconsistent, as every movement setter's lookup would). A transcript of the original's physical getters
   does not include them and is not a proof of all getters. ELIGIBLE: a candidate of a sector list whose collision-test-mobile flag is set (an aircraft or a vehicle). NOT observed: every non-mobile candidate, a city and the
   buildings listed under it, including a building whose collision flags read as mobile during its placement. */
#define CAMPAIGN_MODEL_POSE_FACTS 15
/* the longest host identity (without the terminator) the roster issues (CAMPAIGN_HOST_ID_MAX, gp_roster.h; en_cllsn.c asserts the two are equal at compile time) */
#define CAMPAIGN_MODEL_HOST_ID_MAX 128

enum campaign_model_pose_stage
{
	CAMPAIGN_POSE_STAGE_OFFER = 0,
	CAMPAIGN_POSE_STAGE_PLACEMENT
};

enum campaign_model_pose_class
{
	/* the host holds a record of the identity: the differences say how it compares with the campaign's words */
	CAMPAIGN_MODEL_POSE_RECORDED = 1,
	/* the host issued the identity to a member it keeps no movement record for */
	CAMPAIGN_MODEL_POSE_UNRECORDED,
	/* the host retired the identity */
	CAMPAIGN_MODEL_POSE_RETIRED,
	/* the host never issued the identity (or does not keep movers at all) */
	CAMPAIGN_MODEL_POSE_UNKNOWN,
	/* the campaign has no identity for the candidate */
	CAMPAIGN_MODEL_POSE_NO_ID
};

enum campaign_model_pose_difference
{
	/* the campaign's position is not bit for bit the host record's */
	CAMPAIGN_POSE_DIFFERS_POSITION = 1,
	/* the campaign's attitude is not bit for bit the host record's: only STAGE_PLACEMENT reads an attitude, so a reply to STAGE_OFFER that carries this bit is malformed */
	CAMPAIGN_POSE_DIFFERS_ATTITUDE = 2,
	/* the host holds a motion vector for the record and the campaign's is not bit for bit it. Contradicts NO_HELD_MOTION: a reply with both bits is malformed */
	CAMPAIGN_POSE_DIFFERS_MOTION = 4,
	/* the host holds NO motion vector for the record (every kind but an aircraft, and a vehicle that is not a wreck) */
	CAMPAIGN_POSE_NO_HELD_MOTION = 8,
	/* the campaign's motion vector is not bit for bit the product, rounded toward zero, of the record's attitude row 2 and the record's speed (the original's vehicle formula, from the HOST's record, computed in
	   integers: no floating-point flag or rounding mode is involved). That product is the REFERENCE product: IEEE gradual-underflow round toward zero (denormals-are-zero and flush-to-zero off), not a claim about any
	   other caller's denormal mode. It is also set when an operand of that product is not finite, where the product is not defined */
	CAMPAIGN_POSE_DIFFERS_FORMULA = 16
};

/* ENTITY_SCAN: the floats of the question, where each is, its phases and the states of the answer (the cursor is `candidate[0..2]` of the reply) */
#define CAMPAIGN_MODEL_SCAN_FACTS 16
#define CAMPAIGN_SCAN_SEGMENT_OLD 0
#define CAMPAIGN_SCAN_SEGMENT_NEW 3
#define CAMPAIGN_SCAN_SIDE 6
#define CAMPAIGN_SCAN_MIN_X 7
#define CAMPAIGN_SCAN_MIN_Z 8
#define CAMPAIGN_SCAN_MAX_X 9
#define CAMPAIGN_SCAN_MAX_Z 10
#define CAMPAIGN_SCAN_PHASE 11
#define CAMPAIGN_SCAN_CURSOR 12
#define CAMPAIGN_SCAN_EXCLUDED 14
#define CAMPAIGN_SCAN_HIT 15

enum campaign_model_scan_phase
{
	CAMPAIGN_SCAN_PHASE_BEGIN = 0,
	CAMPAIGN_SCAN_PHASE_ENTRY,
	CAMPAIGN_SCAN_PHASE_END,
	CAMPAIGN_SCAN_PHASE_RESOLVED
};

enum campaign_model_scan_state
{
	CAMPAIGN_MODEL_SCAN_FIRST = 1,
	CAMPAIGN_MODEL_SCAN_NEXT,
	CAMPAIGN_MODEL_SCAN_OFFER,
	CAMPAIGN_MODEL_SCAN_DONE_NONE,
	CAMPAIGN_MODEL_SCAN_DONE_HIT
};

/* ENTITY_CANDIDATE: the floats of the question (the facts) and the states of the answer */
#define CAMPAIGN_MODEL_CANDIDATE_FACTS 44
#define CAMPAIGN_MODEL_CANDIDATE_POSE_MAX 16

enum campaign_model_candidate_state
{
	CAMPAIGN_MODEL_CANDIDATE_SKIP = 1,
	CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED,
	CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL,
	CAMPAIGN_MODEL_CANDIDATE_BUILDINGS,
	CAMPAIGN_MODEL_CANDIDATE_TEST,
	CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED,
	CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT
};

/* ENTITY_CANDIDATE: where each fact is in the question (the layout described with the operation) */
#define CAMPAIGN_CANDIDATE_SEGMENT_OLD 0
#define CAMPAIGN_CANDIDATE_SEGMENT_NEW 3
#define CAMPAIGN_CANDIDATE_MOBILE 6
#define CAMPAIGN_CANDIDATE_FIXED 7
#define CAMPAIGN_CANDIDATE_CITY 8
#define CAMPAIGN_CANDIDATE_CITY_BUILDING 9
#define CAMPAIGN_CANDIDATE_OF_CITY 10
#define CAMPAIGN_CANDIDATE_HAS_BUILDINGS 11
#define CAMPAIGN_CANDIDATE_TALLEST 12
#define CAMPAIGN_CANDIDATE_POSITION 13
#define CAMPAIGN_CANDIDATE_MOTION 16
#define CAMPAIGN_CANDIDATE_DELTA_TIME 19
#define CAMPAIGN_CANDIDATE_UPDATED 20
#define CAMPAIGN_CANDIDATE_HEADING 21
#define CAMPAIGN_CANDIDATE_ALIGN 22
#define CAMPAIGN_CANDIDATE_NORMAL_GIVEN 23
#define CAMPAIGN_CANDIDATE_NORMAL 24
#define CAMPAIGN_CANDIDATE_ATTITUDE 27
#define CAMPAIGN_CANDIDATE_BOX_MIN 36
#define CAMPAIGN_CANDIDATE_BOX_MAX 39
#define CAMPAIGN_CANDIDATE_FIXED_GIVEN 42
#define CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN 43

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

/* ENTITY_COLLISION: the floats of a stationary and of a moving candidate's pose (see the operations above) */
#define CAMPAIGN_MODEL_ENTITY_STATIONARY_POSE 18
#define CAMPAIGN_MODEL_ENTITY_MOVING_POSE 22

/* TARGET_POINT_*: the deepest sub-object chain the transport carries, and the floats of the largest pose (15 + 9 * CAMPAIGN_MODEL_CHAIN_MAX, rounded up) */
#define CAMPAIGN_MODEL_CHAIN_MAX 8
#define CAMPAIGN_MODEL_POSE_MAX 96

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
	/* TARGET_POINT_WORLD, TARGET_POINT_FIXED and ENTITY_COLLISION: the number of floats in `pose` (see the operations above) */
	int pose_count;
	float pose[CAMPAIGN_MODEL_POSE_MAX];
	/* ENTITY_POSE: the candidate's semantic host identity (NUL-terminated; empty when the roster has none) */
	char host_id[CAMPAIGN_MODEL_HOST_ID_MAX + 1];
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
	/* TARGET_POINT_CHAIN (FOUND): the levels of the chain and the sibling position of the first TARGET_POINT sub-object at each, topmost first */
	int chain_length;
	int chain_offsets[CAMPAIGN_MODEL_CHAIN_MAX];
	/* ENTITY_COLLISION: which components of the intercept point (bits 0 to 2) and face normal (bits 3 to 5) the original's arithmetic assigned; `points` holds their words */
	int write_mask;
	/* ENTITY_CANDIDATE: the state the host decided (campaign_model_candidate_state) and, for TEST, the narrow phase's pose it derived: `candidate_count` floats of `candidate` */
	int candidate_state;
	int candidate_count;
	float candidate[CAMPAIGN_MODEL_CANDIDATE_POSE_MAX];
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

/* ENTITY_COLLISION: asks the host whether the segment meets one candidate. `candidate` has `candidate_count` floats, 12 (stationary: position, attitude) or 16 (moving: old position, new
   position, attitude, speed); the request's pose is the segment's two ends then these (see above). `point` and `normal` are OUTPUTS: they are not read and not sent, and only the components
   the host reports the original's arithmetic assigned are written (normal first), after the whole reply has been checked. Returns TRUE for a hit. */
int campaign_model_entity_collision (int scene, const float *segment_old, const float *segment_new, const float *candidate, int candidate_count, float *point, float *normal);

/* ENTITY_CANDIDATE: asks the host what it decides about one candidate from `facts` (CAMPAIGN_MODEL_CANDIDATE_FACTS floats, see above; `scene` is the candidate's scene, 0 until its shape has been read). `host_id` is a MOBILE
   candidate's semantic host identity (NULL for any other candidate): the host takes the candidate's position, motion vector and attitude from ITS OWN physical record of that identity, never from the facts' words for those
   (which the campaign still reads and sends, and compares with the record in ENTITY_POSE); a mobile candidate with no record, a retired or unknown identity is refused by name. The reply is
   checked completely; the returned state is a campaign_model_candidate_state and, for TEST, `pose` receives the `*pose_count` floats of the narrow phase's candidate pose (room for
   CAMPAIGN_MODEL_CANDIDATE_POSE_MAX). */
int campaign_model_entity_candidate (int scene, const float *facts, const char *host_id, float *pose, int *pose_count);

/* ENTITY_SCAN: asks the host what to do next in the enumeration of the scan from `facts` (CAMPAIGN_MODEL_SCAN_FACTS floats, see above). The reply is checked completely; the returned state is a
   campaign_model_scan_state and, for FIRST, `cursor` receives the two sector indices (whole numbers inside the map). */
int campaign_model_entity_scan (const float *facts, float *cursor);

/* ENTITY_POSE: offers the host the words of a mobile candidate (`words`, CAMPAIGN_MODEL_POSE_FACTS floats, see above) beside its `host_id` (NULL or empty when the campaign has none) and returns the class
   of the host's answer (campaign_model_pose_class). The reply's shape is checked completely (including the stage's own difference bits) and the answer is checked and asserted (a mirror difference stops the engine by marker), never used to choose a pose. Every identity of the roster's
   domain is carried whole; one longer than CAMPAIGN_MODEL_HOST_ID_MAX cannot exist and stops the engine by marker rather than being shortened or skipped. */
int campaign_model_entity_pose (int stage, int entity_type, int alive, const char *host_id, const float *words);

#endif

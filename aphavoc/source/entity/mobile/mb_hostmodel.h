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
 *              hit), DONE_NONE (the scan ends, no hit) or DONE_HIT (it ends with the entry offered last).
 *              THE HOST'S ROSTER (EEDCS-49, sixth slice). The host owns WHICH entities stand in each sector and in what order: the campaign reports every insertion into a sector's list (at the head, so the
 *              tail is the oldest insertion) and every removal from it, through the one funnel all of them go through (en_list.c; mb_hostplace.c), as events of THIS question, and the host issues a KEY at each
 *              insertion (a decimal number, never issued twice; a relink is a removal and a new insertion), computes the sector from ITS state (a hosted mobile's from its own physical record, every other
 *              entity's from the position the campaign states; no coordinate is kept) and serves the scan from its own order. The events use `pose_count` 16 with the PHASE 4 (INSERT: the position's x y z in
 *              facts 0 to 2, the entity's type in fact 3, the sector side length in fact 6; the identity place carries a hosted mobile's semantic host identity, empty for any other entity; answered PLACED with
 *              the key as the reply's `name`), 5 (REMOVE: the identity place carries the key; answered REMOVED) or 6 (RESET at a boot; answered CLEARED). A scan question names the key of the entry the
 *              campaign is standing at in the identity place (the ENTRY and RESOLVED phases only) and the host answers FIRST and NEXT with the key of the entry to serve in the reply's `name`, empty when
 *              the sector's list has no more entries: the campaign resolves the key to its entity (mb_hostplace.c), still states whether it is the source or the target, offers it and reports the result.
 *              The question is a pure function of its facts and the host's roster. THE CAMPAIGN'S LISTS ARE A CONFORMANCE MIRROR: the campaign still reads them in the original's order and an entry the host
 *              serves that its list is not at, a key that names no entity, an ended, unknown or misplaced key, or a roster the host does not keep is a named refusal (entity_scan.roster_mirror,
 *              entity_scan.roster_key, entity_scan.key) and never an end, a miss or a fallback to the list. THE TALLEST STRUCTURE OF A SECTOR is the HOST's word since the host-owned sector height (Task49): the host
 *              raises it at the ARRIVAL of a fixed structure (the first state statement under a sector placement's key), holds it in its roster, and the campaign asks it by the sector's coordinates (phase 17) and states a
 *              session's saved word to it (phase 18); the campaign holds no word of its own in a hosted build. STILL THE CAMPAIGN'S, declared and required to move: a city's
 *              buildings, every entity's shape, flags and static pose, and the semantic identity of a structure (issued lazily, at the first time the campaign names it, as before). No raw pointer or
 *              index crosses: an entry is named by the host's key only. A state the phase does not allow (an excluded entry offered, a resolved hit continued, a served entry that is not offered or
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
	CAMPAIGN_SCAN_PHASE_RESOLVED,
	/* the roster's own events of the sector lists (EEDCS-49, sixth slice; see ENTITY_SCAN above): an entity was inserted into a sector's list, is about to leave it, or a new boot starts */
	CAMPAIGN_SCAN_PHASE_INSERT,
	CAMPAIGN_SCAN_PHASE_REMOVE,
	CAMPAIGN_SCAN_PHASE_RESET,
	/* the CITY events (seventh slice): a building's place in its city's list (LIST_TYPE_CITY_BUILDING) is inserted, removed, asked for its first or its next entry; a sector placement and a city child are
	   declared the same entity; a placement is named by the legacy object-report identity the host issued for its entity */
	CAMPAIGN_SCAN_PHASE_CHILD_INSERT,
	CAMPAIGN_SCAN_PHASE_CHILD_REMOVE,
	CAMPAIGN_SCAN_PHASE_CHILD_FIRST,
	CAMPAIGN_SCAN_PHASE_CHILD_NEXT,
	CAMPAIGN_SCAN_PHASE_LINK,
	CAMPAIGN_SCAN_PHASE_NAME,
	/* the FIXED collision state of a fixed-family placement (Task49, fixed/static collision backing): the campaign states it each time it writes one (see campaign_model_entity_fixed) */
	CAMPAIGN_SCAN_PHASE_FIXED_STATE,
	/* the CITY's own collision state (Task49): the position and bounding box of a city placement, stated to the host's record of the placement (see campaign_model_entity_city_state) */
	CAMPAIGN_SCAN_PHASE_CITY_STATE,
	/* the CITY expansion (Task49): a building's link has expanded its city's bounding box; the host builds the box from the records it holds and answers it (see campaign_model_entity_city_expand) */
	CAMPAIGN_SCAN_PHASE_CITY_EXPAND,
	/* the CITY's release (Task49, REQUIRED_CITY_BOX_LIFECYCLE_RETENTION): the campaign's city entity ends (its heap slot is freed); the host ends the record it kept for the box it built (see campaign_model_entity_city) */
	CAMPAIGN_SCAN_PHASE_CITY_RELEASE,
	/* the sector's HIGH-WATER HEIGHT (Task49, the host-owned sector height): the campaign asks the host for the word of a sector by its coordinates (every reader of the original's tallest structure height), and states a
	   session's restored word to it (see campaign_model_sector_height and campaign_model_sector_height_restore) */
	CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT,
	CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE,
	/* the PHYSICAL per-entity UPDATED / MOVED / ROTATED flag store (Task49, the host-owned flag word): the campaign opens the host's store of one word group (the entities that share one 32-bit word of the original's arrays; the host issues
	   its key), sets one lane of one of the three words, reads one lane destructively (the original's `&=` getter) and resets one of the three words over the extent the original's quarter-memset reaches (see campaign_model_flag_set, _read and _reset) */
	CAMPAIGN_SCAN_PHASE_FLAG_GROUP,
	CAMPAIGN_SCAN_PHASE_FLAG_SET,
	CAMPAIGN_SCAN_PHASE_FLAG_READ,
	CAMPAIGN_SCAN_PHASE_FLAG_RESET
};

/* the facts of a FIXED_STATE report (the same question, phase CAMPAIGN_SCAN_PHASE_FIXED_STATE): the position (3 floats), the entity type, the heading, the shape (a whole scene index), the entity's SUB TYPE (a whole
   number: the fixed record's ENTITY_SUB_TYPE_FIXED_*, which the original's fixed_database (fx_dbase.c) maps to the collision-fixed and align-with-terrain flags), and the withdrawal word, which is the HOST's own
   (the campaign always sends 0); every other fact is zero, so a report that still states the two flags is refused. The campaign states the sub type it holds and never a flag derived from it: the host owns that
   derivation (roster.rs, FIXED_RULES). The identity place is the key of the placement the state was written under. */
#define CAMPAIGN_FIXED_POSITION 0
#define CAMPAIGN_FIXED_TYPE 3
#define CAMPAIGN_FIXED_HEADING 4
#define CAMPAIGN_FIXED_SHAPE 5
#define CAMPAIGN_FIXED_WITHDRAW 9
#define CAMPAIGN_FIXED_SUBTYPE 10
/* the number of declared fixed sub types (ENTITY_SUB_TYPE_FIXED_UNKNOWN to ENTITY_SUB_TYPE_FIXED_TERRAIN_OBJECT, en_sbtyp.h): the domain of the sub type fact, which the host's rule table (roster.rs, FIXED_RULES) has
   one row for each of, in the enumeration's order (mb_hostplace.c ties this number to NUM_ENTITY_SUB_TYPE_FIXED at compile time) */
#define CAMPAIGN_FIXED_SUB_TYPES 15
/* the floats of the state the campaign reads from a fixed entity (campaign_model_entity_fixed): position (3), heading, shape, sub type */
#define CAMPAIGN_FIXED_STATE_WORDS 6

/* the facts of a CITY_STATE report (the same question, phase CAMPAIGN_SCAN_PHASE_CITY_STATE): the city's position (3 floats), its bounding box minimum (3), its bounding box maximum (3), exactly as the entity holds
   the words, and the city's SUB TYPE (the same whole number, the same fact, as a FIXED_STATE report's: a city carries the fixed record, and the host derives its collision-fixed flag from it); every other fact is zero
   (the host's withdrawal word is not part of it: the host retires a state with the FIXED_STATE report's withdrawal word). The identity place is the key of the city's sector placement. The box of a city that has
   no building yet is the original's INVERTED initial box (minimum 1.0e6, maximum -1.0e6): a valid statement. */
#define CAMPAIGN_CITY_POSITION 0
#define CAMPAIGN_CITY_BOX_MIN 3
#define CAMPAIGN_CITY_BOX_MAX 6
#define CAMPAIGN_CITY_SUBTYPE 10
/* the city's SHAPE (a whole scene index; 0, OBJECT_3D_INVALID_OBJECT_INDEX, is what a city is created with): the one fact the host needs of a city to decide the sector's height raise the original makes at the city's arrival
   when the shape is valid. Fact 9 is the fixed state report's withdrawal word and was stated by no city report, so it carries the shape there */
#define CAMPAIGN_CITY_SHAPE 9
/* the floats of the state the campaign reads from a city (campaign_model_entity_city_state): position (3), box minimum (3), box maximum (3), sub type, shape */
#define CAMPAIGN_CITY_STATE_WORDS 11
/* the floats of the box a city expansion answers (campaign_model_entity_city_expand): the bounding box minimum (3), then the maximum (3), as the host holds the words */
#define CAMPAIGN_CITY_EXPAND_WORDS 6

/* the SECTOR HEIGHT read and restore (the same question, phases CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT and _SECTOR_RESTORE; Task49): the sector's x and z in the cursor facts (whole numbers), every other fact zero but the phase and,
   for a restore, the saved word RAW in fact 0; the identity place is empty. The reply (state HEIGHT or RESTORED) holds exactly CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS words: the x and z echoed, then the word the host holds. The word is
   NEVER validated as finite, non-negative or non-NaN: a negative, infinite or quiet-NaN word the original saved is valid history and crosses exactly */
#define CAMPAIGN_SECTOR_HEIGHT_WORD 0
#define CAMPAIGN_SECTOR_HEIGHT_REPLY_WORDS 3

/* the PHYSICAL FLAG STORE (the same question, phases CAMPAIGN_SCAN_PHASE_FLAG_GROUP to _FLAG_RESET; Task49). The host holds the three per-entity words (UPDATED, MOVED, ROTATED) and the operations on them as ONE responsibility; the
   campaign holds no flag bit of its own in a hosted build. A word is shared by the entities of one GROUP (the original's 32-bit word: entities whose heap slots agree above their five low bits), so the host cannot treat the entities
   one by one: a destructive read of one entity clears every OTHER entity's bit of the group's word. The campaign never states a heap slot. It states, for each group it uses, an opaque GROUP KEY the HOST issued (a FLAG_GROUP
   question: fact 0 is whether the group lies inside the extent the original's reset reaches, 0.0 or 1.0; the answer's name is the key, "f" and decimal digits, which names nothing else and is never a roster key), and then
   names the group by that key and the entity by its LANE, a whole number 0 to 31, the bit the entity has in its group's word. The array is a whole number too: 0 UPDATED, 1 MOVED, 2 ROTATED.
     FLAG_GROUP  fact 0 the eligibility; the identity place is empty; the reply (state FLAG_GROUP) names the new key and holds [eligibility, 0, 0].
     FLAG_SET    facts 12 (array) and 13 (lane); the identity place is the group key; the reply holds [array, lane, the mask 1 << lane]. The word's bit is only ever set (the original's setter is TRUE-only).
     FLAG_READ   the same facts; the destructive read (`word &= mask`); the reply holds [array, lane, the RAW word the original's getter returns]: 0, or the entity's own mask (INT_MIN for lane 31, the bit pattern 0x80000000).
     FLAG_RESET  fact 12 (array); the identity place is empty; clears that word of every group whose eligibility is 1.0; the reply holds [array, the number of groups cleared, 0].
   Every other fact of a question is zero. The raw word is read as bits and NEVER normalised to a boolean. */
#define CAMPAIGN_FLAG_ELIGIBLE 0
#define CAMPAIGN_FLAG_ARRAY 12
#define CAMPAIGN_FLAG_LANE 13
#define CAMPAIGN_FLAG_REPLY_WORDS 3
#define CAMPAIGN_FLAG_ARRAYS 3
#define CAMPAIGN_FLAG_ARRAY_UPDATED 0
#define CAMPAIGN_FLAG_ARRAY_MOVED 1
#define CAMPAIGN_FLAG_ARRAY_ROTATED 2

enum campaign_model_scan_state
{
	CAMPAIGN_MODEL_SCAN_FIRST = 1,
	CAMPAIGN_MODEL_SCAN_NEXT,
	CAMPAIGN_MODEL_SCAN_OFFER,
	CAMPAIGN_MODEL_SCAN_DONE_NONE,
	CAMPAIGN_MODEL_SCAN_DONE_HIT,
	/* the answers to the roster's events: the entity was placed (the reply's name is the key the host issued), removed, or the roster was cleared */
	CAMPAIGN_MODEL_SCAN_PLACED,
	CAMPAIGN_MODEL_SCAN_REMOVED,
	CAMPAIGN_MODEL_SCAN_CLEARED,
	/* the answer to a city expansion: the host built the box (the reply's name is the city's key; its candidate_count is CAMPAIGN_CITY_EXPAND_WORDS and the words of candidate are the box minimum then maximum) */
	CAMPAIGN_MODEL_SCAN_EXPANDED,
	/* the answers to the sector height read and restore: the reply's candidate holds the sector's x and z as asked, then the word the host holds (the restored word as stored) */
	CAMPAIGN_MODEL_SCAN_HEIGHT,
	CAMPAIGN_MODEL_SCAN_RESTORED,
	/* the answers to the physical flag store's questions: the reply's candidate holds CAMPAIGN_FLAG_REPLY_WORDS words (see above); FLAG_GROUP also names the key the host issued */
	CAMPAIGN_MODEL_SCAN_FLAG_GROUP,
	CAMPAIGN_MODEL_SCAN_FLAG_SET,
	CAMPAIGN_MODEL_SCAN_FLAG_READ,
	CAMPAIGN_MODEL_SCAN_FLAG_RESET
};

/* the roster's insertion event: the position the campaign holds (the x, y and z words; the host decides the sector and compares a hosted mobile's with its own record), the entity's type, and the
   sector side length in the same place the scan keeps it. The identity place carries a hosted mobile's semantic identity (empty for every other entity); a removal names the key instead */
#define CAMPAIGN_ROSTER_POSITION 0
#define CAMPAIGN_ROSTER_TYPE 3

/* the longest key the host's roster issues (a decimal number of at most 20 digits, without the terminator): the campaign carries it whole and never reads it as anything else */
#define CAMPAIGN_ROSTER_KEY_MAX 20

/* the longest legacy object-report identity (without the terminator; CAMPAIGN_OBJECT_ID_MAX, mb_hostobj.h; mb_hostplace.c asserts the two are equal at compile time): its own domain, independent of the host identity's */
#define CAMPAIGN_MODEL_OBJECT_ID_MAX 128

/* the longest identity place of an ENTITY_SCAN question (without the terminator): a city NAME carries a placement key, one space and an object identity, EACH at its own maximum, so the request's identity field is
   sized for the whole join and not for either part (a host identity or an object identity alone is never longer than its own domain) */
#define CAMPAIGN_MODEL_SCAN_IDENTITY_MAX (CAMPAIGN_ROSTER_KEY_MAX + 1 + CAMPAIGN_MODEL_OBJECT_ID_MAX)

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
	/* ENTITY_POSE and ENTITY_CANDIDATE: the candidate's semantic host identity (NUL-terminated; empty when the roster has none; at most CAMPAIGN_MODEL_HOST_ID_MAX bytes). ENTITY_SCAN: the identity place of the question
	   (NUL-terminated; at most CAMPAIGN_MODEL_SCAN_IDENTITY_MAX bytes, a key, a hosted mobile's identity or a city NAME's key, space and object identity) */
	char host_id[CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1];
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
   CAMPAIGN_MODEL_CANDIDATE_POSE_MAX).
   For every candidate that is NOT a mobile of a sector list (a fixed object, a city, a building listed in a sector or under a city) `host_id` is instead the key of the PLACEMENT the host served the candidate by (its
   sector placement's, or a city building's place in the city's list), on every staged question: the host joins it to the record of the placement and, once the candidate's fixed facts were read, decides from the
   fixed collision state the record holds (campaign_model_entity_fixed), comparing the campaign's words for the same facts (the collision-fixed flag, position, shape, heading, align flag) with it, bit for bit. The
   two flags are the HOST's: it derives them from the sub type the record holds, and the campaign's own flags (its fixed_database lookups) are only a conformance mirror compared with them. A key
   that names no live placement of the kind the branch has, and a record that holds no state (or whose state the host withdrew) where the branch needs one, are refused by name.
   A CITY is decided from its own state the same way (campaign_model_entity_city_state): once its fixed facts were read, the host compares the campaign's fixed flag with the one its record's sub type derives (a city
   the campaign calls not fixed that the host derives fixed is refused by name, never skipped), and at the stage that reads its position and bounding box (fixed, and its first building served: a city with none is
   skipped before either is read) it compares the campaign's words for the position and the box with the record's, bit for bit, and decides over the record's words; a record that holds none, or whose state the host
   withdrew, is refused by name at the stage that reads its fixed facts and never completed from the campaign's words. */
int campaign_model_entity_candidate (int scene, const float *facts, const char *host_id, float *pose, int *pose_count);

/* ENTITY_SCAN: asks the host what to do next in the enumeration of the scan from `facts` (CAMPAIGN_MODEL_SCAN_FACTS floats, see above). `entry_key` is the host's key of the entry the campaign is standing at
   (NULL or empty before the first entry of a sector). The reply is checked completely; the returned state is a campaign_model_scan_state, for FIRST `cursor` receives the two sector indices (whole numbers inside
   the map) and for FIRST and NEXT `key` receives the key (CAMPAIGN_ROSTER_KEY_MAX + 1 bytes) of the entry to serve, empty when the sector's list has no more. */
int campaign_model_entity_scan (const float *facts, const char *entry_key, float *cursor, char *key);

/* ENTITY_SCAN, the roster's events: reports that an entity of `entity_type` stands at `position` (3 floats) and was inserted at the head of the list of the sector of that position (`phase`
   CAMPAIGN_SCAN_PHASE_INSERT; `identity` is a hosted mobile's semantic identity, NULL for every other entity; the host's key for it is written to `key`), that the entity with the host's `identity` key
   is leaving its list (CAMPAIGN_SCAN_PHASE_REMOVE), or that a new boot starts (CAMPAIGN_SCAN_PHASE_RESET). The reply is checked completely; a refusal stops the engine with the host's reason. The host's answer to
   a removal of a CITY's placement may name (in `key`) the record it keeps for the bounding box it built (the city has no building left to hold it): the campaign keeps that name, declares the city's next sector
   placement to continue it (a LINK) and states its release when the city entity ends. It is empty for every other removal, and for any entity that is no city. */
void campaign_model_entity_roster (int phase, int entity_type, float side, const float *position, const char *identity, char *key);

/* ENTITY_SCAN, the city events (phases CAMPAIGN_SCAN_PHASE_CHILD_INSERT to CAMPAIGN_SCAN_PHASE_NAME): `identity` is the identity place text the caller built from the host's keys (see mb_hostentity.h,
   campaign_entity_city_exchange: it is validated against the phase's grammar before anything is sent); for an insertion `entity_type` is the building's type; the host's answer is written to `key`: the key issued (an
   insertion), the entry served (CHILD_FIRST and CHILD_NEXT: empty at the end of the list) or the key the host acknowledged (LINK and NAME). The reply is checked completely. A city's list is named by the key the
   city's sector placement had at its first building, and keeps that name when the city's placement is relinked (a LINK of the new placement and that name continues the list); a NAME's identity place is a key, one
   space and the object identity, each at its own domain (CAMPAIGN_MODEL_SCAN_IDENTITY_MAX in all). A CHILD_REMOVE that leaves a placement-less city holding only the box the host built is answered with the key of that
   record (the list's own name), which the campaign keeps as above. The release of a city (CAMPAIGN_SCAN_PHASE_CITY_RELEASE; `identity` is the key of the city's record alone, the answer the state REMOVED and no key)
   ends the record the host kept: the real end of the entity's lifetime, never the host's own diagnostic withdrawal of a state, which keeps every record. */
void campaign_model_entity_city (int phase, int entity_type, const char *identity, char *key);

/* ENTITY_SCAN, the FIXED_STATE report (phase CAMPAIGN_SCAN_PHASE_FIXED_STATE; Task49): the collision state of the fixed-family placement the key `key` names (a sector placement's, or a building's place in its city's list)
   is `state` (CAMPAIGN_FIXED_STATE_WORDS floats: position, heading, shape, sub type) for an entity of `entity_type`. The host's RECORD of the placement holds it, derives the collision-fixed and align flags from the sub type
   (the original's fixed_database, ENTITY_SUB_TYPE_FIXED_UNKNOWN included: it is one of the fifteen declared values, a sub type outside them is refused by name), and a candidate question about that
   placement is decided from it. The campaign states it every time it writes one of the fields: the insertion into a list, a setter (the sub type's own included), a kill's destroyed shape, a restore, an unpack and a bridge's damage or fall. The reply
   is checked completely (the state `placed` and the same key acknowledged); a refusal stops the engine with the host's reason. */
void campaign_model_entity_fixed (int entity_type, const char *key, const float *state);

/* ENTITY_SCAN, the CITY_STATE report (phase CAMPAIGN_SCAN_PHASE_CITY_STATE; Task49): the position and bounding box of the city placement the key `key` names (its sector placement's) are `state` (CAMPAIGN_CITY_STATE_WORDS floats:
   position, box minimum, box maximum, as the entity holds the words, then the sub type, from which the host derives the city's collision-fixed flag). The host's RECORD of the placement holds them, and a candidate question about the city (its fixed facts read) and about the position and box it reads (fixed, a city with a first building
   served) is decided from it. The campaign states it every time it writes one of the fields: the insertion into a sector list (which a relink repeats), the shared fixed setter, and the expansion a building's link makes. The reply
   is checked completely (the state `placed` and the same key acknowledged); a refusal stops the engine with the host's reason. */
void campaign_model_entity_city_state (const char *key, const float *state);
/* ENTITY_SCAN, the sector's HIGH-WATER HEIGHT (phase CAMPAIGN_SCAN_PHASE_SECTOR_HEIGHT; Task49): the word the HOST holds for the sector (`x_sector`, `z_sector`), copied into `word` as the 32 bits it holds (never computed with
   here). The campaign holds no word of its own in a hosted build: this is the one read behind the sector's float getter, so the scan, the fixed-wing clearance and the session pack all read the host's. The reply is checked
   completely before the word is written: the state HEIGHT, no placement named, exactly three words, the sector echoed as asked and nothing beyond; a refusal, a missing provider or an answer that does not hold stops the engine by marker, and there is no fallback. */
void campaign_model_sector_height (int x_sector, int z_sector, float *word);

/* ENTITY_SCAN, the session RESTORE of a sector's height (phase CAMPAIGN_SCAN_PHASE_SECTOR_RESTORE; Task49): the saved word (`word`, the 32 raw bits) is stated to the host, which holds exactly these bits afterwards. The reply is
   checked completely: the state RESTORED, the sector echoed as asked and the word stored equal, bit for bit, to the word stated. */
void campaign_model_sector_height_restore (int x_sector, int z_sector, const float *word);

/* ENTITY_SCAN, the PHYSICAL FLAG STORE (phases CAMPAIGN_SCAN_PHASE_FLAG_GROUP to _FLAG_RESET; Task49): the host owns the three per-entity UPDATED / MOVED / ROTATED words. `slot` is the entity's heap slot, which the campaign alone
   knows and which is never sent: the campaign derives from it the group (slot >> 5) and the lane (slot & 31) and keeps, per group, only the opaque key the host issued (the key table is identity, never a flag). `array` is
   CAMPAIGN_FLAG_ARRAY_UPDATED, _MOVED or _ROTATED. A set only sets the lane's bit (the original's setter is TRUE-only). A read returns the RAW word of the original's destructive getter (0, or the entity's own mask, INT_MIN for
   lane 31) after clearing every other lane of the group's word in the host. A reset clears the array's word in every group inside the original's reset extent (the quarter-memset: slots 0 to 131071). Each reply is checked completely (the state
   the phase allows, no placement named, exactly three words, the array and lane echoed as asked, a mask that is 0 or the entity's own, nothing beyond); a refusal, a missing provider or an answer that does not hold stops the engine by
   marker, and there is no fallback to a campaign copy of the word. */
void campaign_model_flag_set (int array, int slot);
int campaign_model_flag_read (int array, int slot);
void campaign_model_flag_reset (int array);

/* ENTITY_SCAN, the CITY_EXPAND question (phase CAMPAIGN_SCAN_PHASE_CITY_EXPAND; Task49): the building the key `building` names (its place in the city's list) was linked to the city the key `city` names (its sector
   placement), and the city's bounding box is to be expanded for it. The campaign states no word of it: the host holds the city's own state (its position and the box it holds) and the building's fixed state (its
   position, heading and shape), takes the shape's bounds from the model data it owns, builds the new box with the original's arithmetic, holds it in its record of the city and answers it. `box` receives the
   CAMPAIGN_CITY_EXPAND_WORDS floats (the minimum, then the maximum) the host holds now, and is written ONLY after the whole answer was checked (the state EXPANDED, the city's key echoed, the count of words, every word finite,
   no sector named, and each axis of the box with its minimum at or below its maximum): a refusal, a missing provider or an answer that does not hold stops the engine by marker and leaves `box` as it was. The campaign applies the words to its mirror of the box, as they are. */
void campaign_model_entity_city_expand (const char *city, const char *building, float *box);

/* whether a model provider is installed (the roster reports only to one: a build with no host has no roster and no scan) */
int campaign_model_host_installed (void);

/* ENTITY_POSE: offers the host the words of a mobile candidate (`words`, CAMPAIGN_MODEL_POSE_FACTS floats, see above) beside its `host_id` (NULL or empty when the campaign has none) and returns the class
   of the host's answer (campaign_model_pose_class). The reply's shape is checked completely (including the stage's own difference bits) and the answer is checked and asserted (a mirror difference stops the engine by marker), never used to choose a pose. Every identity of the roster's
   domain is carried whole; one longer than CAMPAIGN_MODEL_HOST_ID_MAX cannot exist and stops the engine by marker rather than being shortened or skipped. */
int campaign_model_entity_pose (int stage, int entity_type, int alive, const char *host_id, const float *words);

#endif

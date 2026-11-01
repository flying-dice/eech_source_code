/*
 * TEST-ONLY (EEDCS-50, D3; feature `route-fixture`): the observer hooks the route-fixture build's patched aphavoc/source/ai/faction/routegen.c and popread.c call
 * (build/patches.rs, ROUTE_FIXTURE_PATCHES). This header is included by the staged routegen.c ONLY when the feature is on; with it off none of the hooks, the patches or
 * the definitions (eech_route_fixture.c) exist in the build.
 *
 * Every hook only RECORDS a value the original route parser has just read or produced, in bounded private tables (eech_route_fixture.c). None allocates, frees, writes a
 * model or campaign byte, draws a random number, reads the clock, changes an entity or does floating-point arithmetic: floats cross as pointers and are copied as bit
 * patterns. Row indices are plain integers (the parser's own node numbers, which are positions in its node array); no pointer is ever emitted (the route entries of the lookups and of the callers are kept only as private tokens that are compared with each other).
 */
#ifndef EECH_ROUTE_HOOKS_H
#define EECH_ROUTE_HOOKS_H

/* popread.c get_airfield_waypoint_route_object has just searched the airfield instance `scene` (its scene index) for sub-object `kind`: found (1) with the raw object
   `legacy_object` it returned and the route entry `entry` (an opaque token the fixture only compares and never reports), or not found (0, legacy_object -1, entry NULL) */
extern void eech_rf_lookup (int scene, int kind, int found, int legacy_object, const void *entry);

/* popread.c, immediately before an original parse call: the route entry the caller holds (the result of its own lookup) and the raw object number it has just taken from that entry,
   which is the argument the parse will receive. The next eech_rf_begin consumes it. */
extern void eech_rf_caller (const void *entry, int raw_object_index);

/* the same, for the two troop-route calls that pass the entry's number inline: the call site evaluates `route->index` ONCE, as this function's argument, and uses the value returned here
   (exactly that argument, unchanged) as the parse's argument */
extern int eech_rf_caller_index (const void *entry, int raw_object_index);

/* parse_waypoint_routes_from_object begins: its arguments (`slots` is number_of_slots vec3d, or NULL) and the raw object it is about to read (struct OBJECT_3D) */
extern void eech_rf_begin (int object_index, int number_of_slots, const void *slots, const void *object);

/* one waypoint node, right after the parser decoded its position: its own index, the raw stored point (object_short_3d_point) and the decoded position (vec3d) */
extern void eech_rf_node (int index, const void *point, const void *position);

/* one line face as the parser's first pass reads it: surface and face index within it, the two face-local point references, the two resolved node indices, the surface
   record (struct FACE_SURFACE_DESCRIPTION, for its colour and flags) and the surface's point count as the parser resolved it (0 means 256 there) */
extern void eech_rf_face (int surface, int face, int local0, int local1, int point0, int point1, const void *surface_description, int surface_points);

/* the parse is over: the totals, then every final node, link, start and depth row the parser's tables hold, then the close of this call */
extern void eech_rf_final_totals (int nodes, int starts, int depths, int maximum_depth);
extern void eech_rf_final_node (int index, int depth, int tree, int node_index, int primary, int start, int possible_start, int references, int in_route, int processed,
	int children, int parents);
extern void eech_rf_final_link (int node, int parent_link, int slot, int target);
extern void eech_rf_final_start (int slot, int node);
extern void eech_rf_final_position (int depth, int count, const void *position, const void *offsets);
extern void eech_rf_end (void);

#endif

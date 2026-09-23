//
// Port: terrain elevation.
//
// C provenance: modules/3d/terrain/terrelev.h :: #define get_3d_terrain_elevation(X,Z)
//               (get_3d_terrain_point_data ((X), (Z), NULL)), called by
//               ai/taskgen/croute.c :: get_best_point (route search samples) and
//               create_generic_waypoint_route (once per waypoint; the value is discarded)
//
// The terrain is map data EECH loads with the theatre. The route search is
// campaign code and stays in the core; it asks this port for the elevation of
// the terrain at a point on the map, as the C does. Arguments are C floats
// (the callers keep points inside the map area); the result is narrowed to
// float by the core, not by the adapter.
//
export interface TerrainElevation {
	// C: get_3d_terrain_elevation (x, z)
	getTerrainElevation(x: number, z: number): number;
}

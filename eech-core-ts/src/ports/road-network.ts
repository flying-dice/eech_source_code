//
// Port: the road network's node table.
//
// C provenance: aphavoc/source/ai/ai_misc/ai_route.h (node_data *road_nodes,
//               vec3d *road_node_positions, int total_number_of_road_nodes),
//               read by ai/ai_misc/ai_misc.c :: get_closest_road_node, which
//               croute.c :: create_generic_waypoint_route calls for every waypoint
//
// The road network is map data EECH loads with the theatre. The closest-node
// search stays in the core; it reads the three things the C reads. A theatre
// without a road table (road_node_positions NULL) is not a valid EECH map:
// get_closest_road_node ASSERTs it, and so does the port.
//
import type { Vec3d } from "../core/maths/vec3d";

export interface RoadNetwork {
	// C: road_node_positions != NULL
	hasRoadNodeTable(): boolean;

	// C: total_number_of_road_nodes
	getTotalNumberOfRoadNodes(): number;

	// C: road_node_positions [node] (floats)
	getRoadNodePosition(node: number): Vec3d;

	// C: road_nodes [node].number_of_links (unsigned int : 7)
	getRoadNodeNumberOfLinks(node: number): number;
}

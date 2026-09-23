//
// Road network queries.
//
// C provenance: ai/ai_misc/ai_misc.c :: get_closest_road_node
//
// The road node table is map data (the RoadNetwork port); the search over it
// is campaign code. Only nodes with links are candidates; the first node
// within `error` of the position is taken at once, otherwise the strictly
// closest (2D approximate range), and node 0 when no node has links.
//

import { ASSERT } from "../../core/assert";
import { toFloat32 } from "../../core/float32";
import type { Vec3d } from "../../core/maths/vec3d";
import { getApprox2dRange } from "../../core/maths/range";
import { getCampaignPorts } from "../../entity/system/entity";

// C provenance: ai_misc.c :: get_closest_road_node (vec3d *pos, float error)
export function getClosestRoadNode(pos: Vec3d | undefined, error: number): number {
	const roads = getCampaignPorts().roadNetwork;

	let best_node = 0;

	ASSERT(pos !== undefined, "pos");

	// ASSERT (road_node_positions): a map without a road table (slice 6b decision D3)
	ASSERT(roads.hasRoadNodeTable(), "road_node_positions");

	// best_range = 99999999: the int constant converted to float (to nearest, at compile time)
	let best_range = toFloat32(99999999);

	const total_number_of_road_nodes = roads.getTotalNumberOfRoadNodes();

	for (let loop = 0; loop < total_number_of_road_nodes; loop++) {
		if (roads.getRoadNodeNumberOfLinks(loop) > 0) {
			const range = getApprox2dRange(roads.getRoadNodePosition(loop), pos);

			if (range < best_range) {
				best_range = range;

				best_node = loop;

				if (range <= error) {
					return best_node;
				}
			}
		}
	}

	return best_node;
}

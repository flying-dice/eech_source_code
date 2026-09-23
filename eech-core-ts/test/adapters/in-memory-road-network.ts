//
// Deterministic RoadNetwork: the road nodes a scenario declares, in order, as
// the C reference harness fills ai_route.h's tables ("road-node" line). With no
// node declared there is no road table (road_node_positions NULL).
//

import type { Vec3d } from "../../src/core/maths/vec3d";
import type { RoadNetwork } from "../../src/ports";

export class InMemoryRoadNetwork implements RoadNetwork {
	private readonly positions: Vec3d[] = [];

	private readonly links: number[] = [];

	public addRoadNode(position: Vec3d, numberOfLinks: number): void {
		this.positions.push({ x: position.x, y: position.y, z: position.z });
		this.links.push(numberOfLinks);
	}

	public hasRoadNodeTable(): boolean {
		return this.positions.length > 0;
	}

	public getTotalNumberOfRoadNodes(): number {
		return this.positions.length;
	}

	public getRoadNodePosition(node: number): Vec3d {
		const p = this.positions[node];
		return { x: p.x, y: p.y, z: p.z };
	}

	public getRoadNodeNumberOfLinks(node: number): number {
		return this.links[node];
	}
}

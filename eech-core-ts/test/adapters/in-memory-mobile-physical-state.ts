//
// Deterministic MobilePhysicalState: positions are whatever the scenario put
// there. The adapter never invents a position; asking for an unknown entity is
// a scenario error.
//

import type { Vec3d } from "../../src/core/maths/vec3d";
import type { MobilePhysicalState } from "../../src/ports";

export class InMemoryMobilePhysicalState implements MobilePhysicalState {
	private readonly positions: Record<number, Vec3d> = {};

	public setMobilePosition(entityIndex: number, position: Vec3d): void {
		this.positions[entityIndex] = { x: position.x, y: position.y, z: position.z };
	}

	public getMobilePosition(entityIndex: number): Vec3d {
		const position = this.positions[entityIndex];

		if (position === undefined) {
			throw new Error(`scenario has no physical position for mobile entity ${entityIndex}`);
		}

		return { x: position.x, y: position.y, z: position.z };
	}
}

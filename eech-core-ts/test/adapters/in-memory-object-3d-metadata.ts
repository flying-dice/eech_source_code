//
// Deterministic Object3DMetadata: the scenario declares the bounds of each
// object it uses, as the C reference harness does ("bounds" line). The adapter
// never invents geometry; asking for an undeclared object is a scenario error.
//

import type { Object3DBounds, Object3DMetadata } from "../../src/ports";

export class InMemoryObject3DMetadata implements Object3DMetadata {
	private readonly bounds: Record<number, Object3DBounds> = {};

	public setBoundingBox(objectIndex: number, bounds: Object3DBounds): void {
		this.bounds[objectIndex] = { xmin: bounds.xmin, xmax: bounds.xmax, ymin: bounds.ymin, ymax: bounds.ymax, zmin: bounds.zmin, zmax: bounds.zmax };
	}

	public getBoundingBox(objectIndex: number): Object3DBounds {
		const b = this.bounds[objectIndex];

		if (b === undefined) {
			throw new Error(`scenario declares no bounds for 3D object ${objectIndex}`);
		}

		return { xmin: b.xmin, xmax: b.xmax, ymin: b.ymin, ymax: b.ymax, zmin: b.zmin, zmax: b.zmax };
	}
}

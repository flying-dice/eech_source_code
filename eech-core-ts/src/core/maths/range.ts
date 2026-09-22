//
// C provenance: modules/maths/range.c
//
// Only the functions required by adopted campaign slices are ported.
//

import { ASSERT } from "../assert";
import { toFloat32 } from "../float32";
import type { Vec3d } from "./vec3d";

// C provenance: modules/maths/range.c :: get_2d_range
export function get2dRange(v1: Vec3d | undefined, v2: Vec3d | undefined): number {
	ASSERT(v1 !== undefined, "v1");
	ASSERT(v2 !== undefined, "v2");

	const dx = toFloat32(v1.x - v2.x);
	const dz = toFloat32(v1.z - v2.z);

	// float * float and float + float are single precision operations in C;
	// sqrt () takes and returns double, narrowed on assignment to range
	return toFloat32(Math.sqrt(toFloat32(toFloat32(dx * dx) + toFloat32(dz * dz))));
}

// C provenance: modules/maths/range.c :: get_approx_2d_range
export function getApprox2dRange(v1: Vec3d | undefined, v2: Vec3d | undefined): number {
	ASSERT(v1 !== undefined, "v1");
	ASSERT(v2 !== undefined, "v2");

	const dx = Math.abs(toFloat32(v1.x - v2.x));
	const dz = Math.abs(toFloat32(v1.z - v2.z));

	let range: number;

	if (dx > dz) {
		range = toFloat32((dx * 4.0 + dz) * (1.0 / 4.0));
	} else {
		range = toFloat32((dz * 4.0 + dx) * (1.0 / 4.0));
	}

	return range;
}

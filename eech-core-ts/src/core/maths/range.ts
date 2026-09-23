//
// C provenance: modules/maths/range.c
//
// Only the functions required by adopted campaign slices are ported.
//

import { ASSERT } from "../assert";
import { f32Add, f32Mul, f32Sqrt, f32Sub } from "../float32";
import type { Vec3d } from "./vec3d";

// C provenance: modules/maths/range.c :: get_2d_range
export function get2dRange(v1: Vec3d | undefined, v2: Vec3d | undefined): number {
	ASSERT(v1 !== undefined, "v1");
	ASSERT(v2 !== undefined, "v2");

	const dx = f32Sub(v1.x, v2.x);
	const dz = f32Sub(v1.z, v2.z);

	// float * float and float + float are single precision operations in C;
	// sqrt () takes and returns double, narrowed on assignment to range.
	// Intermediates at declared type: x87 intermediate precision is unresolved,
	// and this sqrt argument is its canary (docs/fidelity/fpu-semantics.md).
	return f32Sqrt(f32Add(f32Mul(dx, dx), f32Mul(dz, dz)));
}

// C provenance: modules/maths/range.c :: get_approx_2d_range
export function getApprox2dRange(v1: Vec3d | undefined, v2: Vec3d | undefined): number {
	ASSERT(v1 !== undefined, "v1");
	ASSERT(v2 !== undefined, "v2");

	const dx = Math.abs(f32Sub(v1.x, v2.x));
	const dz = Math.abs(f32Sub(v1.z, v2.z));

	let range: number;

	// ((dx * 4.0) + dz) * (1.0 / 4.0) in double, stored as float: the scalings
	// are exact, so this is the exact dx + dz / 4 truncated to float
	if (dx > dz) {
		range = f32Add(dx, dz * 0.25);
	} else {
		range = f32Add(dz, dx * 0.25);
	}

	return range;
}

//
// C provenance: modules/maths/vector.c
//
// Only the functions required by adopted campaign slices are ported.
//

import { ASSERT } from "../assert";
import { f32Add, f32Div, f32Mul, f32Sqrt } from "../float32";
import type { Vec3d } from "./vec3d";

//
// C provenance: vector.c :: normalise_any_3d_vector
//
// Float throughout under the RTZ contract (src/core/float32.ts): the squares
// and their sum are float operations, sqrt () computes in double and is
// stored to the float length, 1.0 / length is a double quotient stored to a
// float, and each component is scaled by that float.
//
export function normaliseAny3dVector(vector: Vec3d): number {
	ASSERT(vector !== undefined, "vector");

	const x = f32Mul(vector.x, vector.x);
	const y = f32Mul(vector.y, vector.y);
	const z = f32Mul(vector.z, vector.z);

	const length = f32Sqrt(f32Add(f32Add(x, y), z));

	if (length > 0) {
		const one_over_length = f32Div(1.0, length);

		vector.x = f32Mul(vector.x, one_over_length);
		vector.y = f32Mul(vector.y, one_over_length);
		vector.z = f32Mul(vector.z, one_over_length);
	} else {
		vector.x = 0;
		vector.y = 0;
		vector.z = 0;
	}

	return length;
}

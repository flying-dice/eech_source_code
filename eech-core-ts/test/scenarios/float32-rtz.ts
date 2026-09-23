//
// The round-toward-zero float operations of src/core/float32.ts as one
// dispatch, shared by the vitest suites, the C reference differential and the
// Lua 5.1 runner. Each operation names the C expression the harness `f32`
// command evaluates under the canonical environment (c-reference/harness.c).
//
// TSTL-compatible: no Node APIs.
//

import { f32Add, f32Div, f32Mul, f32Sqrt, f64AddRTZ, toFloat32RTZ } from "../../src/core/float32";

export type Float32RtzOp = "narrow" | "sum" | "mul" | "div" | "sqrt" | "dsum";

// [operation, a, b, expected]: narrow and sqrt ignore b
export type Float32RtzCase = [Float32RtzOp, number, number, number];

export function applyFloat32RtzOp(op: Float32RtzOp, a: number, b: number): number {
	if (op === "narrow") {
		// (float) d
		return toFloat32RTZ(a);
	}
	if (op === "sum") {
		// (float) (d1 + d2)
		return f32Add(a, b);
	}
	if (op === "mul") {
		// float * float
		return f32Mul(a, b);
	}
	if (op === "div") {
		// float / float
		return f32Div(a, b);
	}
	if (op === "dsum") {
		// d1 + d2, a double
		return f64AddRTZ(a, b);
	}
	// (float) sqrt (float)
	return f32Sqrt(a);
}

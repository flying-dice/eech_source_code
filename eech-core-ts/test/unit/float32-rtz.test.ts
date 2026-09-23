//
// The round-toward-zero float helpers against results recorded from the C
// oracle (test/scenarios/generated/c-reference-float32-rtz.cases.ts, checked
// live by test/c-reference/float32-rtz.cref.test.ts). The Lua 5.1 runner
// replays the same cases.
//

import { describe, expect, it } from "vitest";
import { f32Add, f32Div, f32Mul, f32Sqrt, f32Sub, FLT_MAX, toFloat32RTZ } from "../../src/core/float32";
import { applyFloat32RtzOp } from "../scenarios/float32-rtz";
import { C_REFERENCE_FLOAT32_RTZ_CASES } from "../scenarios/generated/c-reference-float32-rtz.cases";

describe("round-toward-zero float arithmetic", () => {
	it("reproduces every recorded C result bit for bit", () => {
		for (const [op, a, b, expected] of C_REFERENCE_FLOAT32_RTZ_CASES) {
			expect(Object.is(applyFloat32RtzOp(op, a, b), expected), `${op} ${a} ${b}`).toBe(true);
		}
	});

	it("truncates narrowing toward zero", () => {
		expect(toFloat32RTZ(0.1)).toBe(0.09999999403953552);
		expect(toFloat32RTZ(-0.1)).toBe(-0.09999999403953552);
		expect(toFloat32RTZ(16777217)).toBe(16777216);
		expect(toFloat32RTZ(16777219)).toBe(16777218);
		expect(toFloat32RTZ(3.5e38)).toBe(FLT_MAX);
		expect(toFloat32RTZ(-3.5e38)).toBe(-FLT_MAX);
		expect(toFloat32RTZ(1 / 0)).toBe(1 / 0);
		expect(Object.is(toFloat32RTZ(-1e-50), -0)).toBe(true);
		expect(toFloat32RTZ(Number.NaN)).toBeNaN();
	});

	it("steps below a result that is a float in double but not exactly", () => {
		// 1 - 2^-60 rounds to 1 in double; the exact value truncates to the float below 1
		expect(f32Add(1, -Math.pow(2, -60))).toBe(1 - Math.pow(2, -24));
		expect(f32Add(-1, Math.pow(2, -60))).toBe(-(1 - Math.pow(2, -24)));
		// above the power of two the step is the full spacing
		expect(f32Add(3, -Math.pow(2, -60))).toBe(3 - Math.pow(2, -22));
		// in the subnormal range the spacing is fixed
		expect(f32Add(Math.pow(2, -126), -Math.pow(2, -200))).toBe(Math.pow(2, -126) - Math.pow(2, -149));
		expect(f32Add(1, Math.pow(2, -60))).toBe(1);
		expect(f32Sub(100, 0.00001)).toBe(99.99998474121094);
		expect(Object.is(f32Sub(0, 0), 0)).toBe(true);
	});

	it("multiplies, divides and takes square roots toward zero", () => {
		expect(f32Mul(Math.fround(0.1), 3)).toBe(0.29999998211860657);
		expect(f32Div(1, 3)).toBe(0.3333333134651184);
		expect(f32Div(-1, 3)).toBe(-0.3333333134651184);
		expect(f32Div(1, -3)).toBe(-0.3333333134651184);
		expect(f32Div(6, 3)).toBe(2);
		expect(f32Div(1, 0)).toBe(1 / 0);
		expect(f32Div(0, 5)).toBe(0);
		expect(f32Sqrt(2)).toBe(1.4142135381698608); // 0x3fb504f3, as the C gives
		expect(f32Sqrt(4)).toBe(2);
		expect(f32Sqrt(0)).toBe(0);
		expect(f32Sqrt(1 / 0)).toBe(1 / 0);
		// a double sqrt that rounds up onto a float: (1 + 2^-23)^2 truncated to float, whose root is just below 1 + 2^-23
		const justBelow = Math.fround(Math.pow(1 + Math.pow(2, -23), 2));
		expect(f32Sqrt(justBelow)).toBe(1);
	});
});

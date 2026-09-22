//
// Isolated tests of the ported maths helpers. Expectations come from the C
// definitions quoted in src/core/maths (and are cross-checked against the
// executed C in test/c-reference/range.cref.test.ts).
//

import { describe, expect, it } from "vitest";
import { EechAssertionError } from "../../src/core/assert";
import { bound, KILOMETRE } from "../../src/core/maths/miscmath";
import { get2dRange, getApprox2dRange } from "../../src/core/maths/range";

describe("bound (miscmath.h macro)", () => {
	it("clamps below, above and passes through", () => {
		expect(bound(-1, 0, 10)).toBe(0);
		expect(bound(11, 0, 10)).toBe(10);
		expect(bound(5, 0, 10)).toBe(5);
	});

	it("tests LOWER first, so an inverted interval returns UPPER for values above it", () => {
		// ( VALUE ) < ( LOWER ) ? LOWER : ( VALUE ) > ( UPPER ) ? UPPER : VALUE
		expect(bound(70, 0, -5)).toBe(-5);
		expect(bound(-10, 0, -5)).toBe(0);
	});

	it("passes NaN through (both comparisons are false)", () => {
		expect(bound(Number.NaN, 0, 10)).toBeNaN();
	});

	it("KILOMETRE is 1000 * METRE", () => {
		expect(KILOMETRE).toBe(1000);
	});
});

describe("get_2d_range / get_approx_2d_range (range.c)", () => {
	const origin = { x: 0, y: 0, z: 0 };

	it("ignores y", () => {
		expect(get2dRange(origin, { x: 3, y: 99, z: 4 })).toBe(5);
		expect(getApprox2dRange(origin, { x: 3, y: 99, z: 4 })).toBe(4.75);
	});

	it("approximates with the larger axis plus a quarter of the smaller", () => {
		expect(getApprox2dRange(origin, { x: 400, y: 0, z: 300 })).toBe(475);
		expect(getApprox2dRange(origin, { x: 300, y: 0, z: 400 })).toBe(475);
		expect(getApprox2dRange(origin, { x: -300, y: 0, z: -300 })).toBe(375);
	});

	it("asserts on NULL vectors", () => {
		for (const fn of [get2dRange, getApprox2dRange]) {
			expect(() => fn(undefined, origin)).toThrow(new EechAssertionError("v1"));
			expect(() => fn(origin, undefined)).toThrow(new EechAssertionError("v2"));
		}
	});
});

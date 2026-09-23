//
// toFloat32 must equal IEEE 754 binary32 round-to-nearest-even, which the
// JavaScript platform provides as Math.fround. Math.fround is only the test
// oracle: it does not exist in Lua 5.1, where toFloat32 must run unchanged
// (test/lua/conformance.ts repeats a subset under Lua).
//
// toFloat32 is the round-to-nearest conversion for compile-time constants and
// scenario input only. EECH's run-time float arithmetic rounds toward zero; its
// helpers are verified against the C oracle in test/unit/float32-rtz.test.ts
// and test/c-reference/float32-rtz.cref.test.ts (src/core/float32.ts header).
//

import { describe, expect, it } from "vitest";
import { FLT_MAX, toFloat32 } from "../../src/core/float32";
import { FLOAT32_EDGE_CASES } from "../scenarios/float32.cases";

function bitsToDouble(hi: number, lo: number): number {
	const view = new DataView(new ArrayBuffer(8));
	view.setUint32(0, hi);
	view.setUint32(4, lo);
	return view.getFloat64(0);
}

// deterministic xorshift32, no Math.random
function xorshift(seed: number): () => number {
	let x = seed >>> 0;
	return () => {
		x ^= x << 13;
		x >>>= 0;
		x ^= x >>> 17;
		x ^= x << 5;
		x >>>= 0;
		return x;
	};
}

describe("toFloat32", () => {
	it("matches the shared edge cases", () => {
		for (const [input, expected] of FLOAT32_EDGE_CASES) {
			expect(Object.is(toFloat32(input), expected), `${input}`).toBe(true);
			expect(Object.is(Math.fround(input), expected), `oracle ${input}`).toBe(true);
		}
	});

	it("preserves NaN and signed zero", () => {
		expect(toFloat32(Number.NaN)).toBeNaN();
		expect(Object.is(toFloat32(-0), -0)).toBe(true);
		expect(Object.is(toFloat32(0), 0)).toBe(true);
	});

	it("FLT_MAX is the largest finite float", () => {
		expect(FLT_MAX).toBe(Math.fround(3.4028234663852886e38));
		expect(toFloat32(FLT_MAX)).toBe(FLT_MAX);
	});

	it("matches Math.fround for 200000 pseudo-random doubles across the whole exponent range", () => {
		const next = xorshift(0x5eed);
		for (let i = 0; i < 200000; i++) {
			const value = bitsToDouble(next(), next());
			const expected = Math.fround(value);
			const actual = toFloat32(value);
			if (!Object.is(actual, expected) && !(Number.isNaN(actual) && Number.isNaN(expected))) {
				throw new Error(`toFloat32(${value}) = ${actual}, Math.fround = ${expected}`);
			}
		}
	});

	it("matches Math.fround for 200000 pseudo-random doubles in campaign magnitudes", () => {
		const next = xorshift(0xc0ffee);
		for (let i = 0; i < 200000; i++) {
			const value = ((next() / 0xffffffff) * 2 - 1) * 10 ** ((next() % 14) - 4);
			expect(toFloat32(value)).toBe(Math.fround(value));
		}
	});
});

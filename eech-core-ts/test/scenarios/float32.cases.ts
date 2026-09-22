//
// [input, IEEE 754 binary32 result] pairs for toFloat32, shared by the
// vitest suite (checked against Math.fround) and the Lua 5.1 runner.
//
// TSTL-compatible: no Node APIs.
//
// Negative zero is computed, never written as the literal -0: Lua 5.1 stores
// numeric literals in a per-function constant table where 0 and -0 are the
// same key, so a literal -0 silently becomes 0 after transpilation.
//

const NEGATIVE_ZERO = 1 / -(1 / 0);

export const FLOAT32_EDGE_CASES: [number, number][] = [
	[1, 1],
	[-1, -1],
	[0.1, 0.10000000149011612],
	[33.3, 33.29999923706055],
	[99.99999, 99.99999237060547],
	[1000.1, 1000.0999755859375],
	// ties round to even
	[16777217, 16777216],
	[16777219, 16777220],
	[-16777217, -16777216],
	// largest finite float, and the overflow boundary (halfway above FLT_MAX rounds to infinity)
	[3.4028234663852886e38, 3.4028234663852886e38],
	[3.4028235677973362e38, 3.4028234663852886e38],
	[3.4028235677973366e38, 1 / 0],
	[1 / 0, 1 / 0],
	[-1 / 0, -1 / 0],
	// smallest normal, subnormals and underflow
	[1.1754943508222875e-38, 1.1754943508222875e-38],
	[1e-40, 9.99994610111476e-41],
	[1.401298464324817e-45, 1.401298464324817e-45],
	[7.006492321624085e-46, 0],
	[7.006492321624087e-46, 1.401298464324817e-45],
	[-1e-50, NEGATIVE_ZERO],
	// Math.log (x) / Math.LN2 overestimates the exponent just below 2^20 and
	// underestimates it at 2^-125 (in both V8 and Lua 5.1): the correction loops
	[1048575.9999999999, 1048576],
	[2.350988701644575e-38, 2.350988701644575e-38],
];

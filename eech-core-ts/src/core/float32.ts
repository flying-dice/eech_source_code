//
// C `float` storage semantics.
//
// EECH stores campaign quantities (supply levels, positions, ranges) in 32-bit
// `float` variables and struct fields. Every assignment to a float rounds to
// the nearest representable single-precision value (IEEE 754 round to nearest,
// ties to even). JavaScript and Lua 5.1 both compute in doubles, so the port
// narrows explicitly with toFloat32 () wherever C assigns to a float.
//
// A single +, -, *, / or sqrt of float operands computed in double and then
// narrowed equals the float result, so f32 (a op b) models C float arithmetic
// exactly (no double-rounding hazard: 53 >= 2 * 24 + 2).
//
// Math.fround is not available in Lua 5.1, so this is pure arithmetic that
// behaves identically after TSTL transpilation. It is verified against
// Math.fround in test/unit/float32.test.ts.
//

const FLOAT32_OVERFLOW_THRESHOLD = 3.4028235677973366e38; // 2^128 - 2^103: halfway above FLT_MAX

const FLOAT32_MIN_EXPONENT = -126;

const FLOAT32_MANTISSA_BITS = 23;

export const FLT_MAX = 3.4028234663852886e38;

export function toFloat32(value: number): number {
	if (value !== value || value === 0) {
		// NaN and signed zero are preserved
		return value;
	}

	const sign = value < 0 ? -1 : 1;

	const magnitude = value * sign;

	if (magnitude >= FLOAT32_OVERFLOW_THRESHOLD) {
		// includes infinity
		return sign * (1 / 0);
	}

	let exponent = Math.floor(Math.log(magnitude) / Math.LN2);

	// correct any floating point error in the logarithm estimate
	while (Math.pow(2, exponent) > magnitude) {
		exponent -= 1;
	}

	while (Math.pow(2, exponent + 1) <= magnitude) {
		exponent += 1;
	}

	if (exponent < FLOAT32_MIN_EXPONENT) {
		// subnormal range has a fixed quantum
		exponent = FLOAT32_MIN_EXPONENT;
	}

	const quantum = Math.pow(2, exponent - FLOAT32_MANTISSA_BITS);

	const scaled = magnitude / quantum;

	let mantissa = Math.floor(scaled);

	const remainder = scaled - mantissa;

	if (remainder > 0.5 || (remainder === 0.5 && mantissa % 2 === 1)) {
		mantissa += 1;
	}

	return sign * mantissa * quantum;
}

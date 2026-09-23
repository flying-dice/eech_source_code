//
// C provenance: modules/maths/invsqrt.c :: initialise_inverse_square_root_table,
//               get_inverse_square_root; invsqrt.h (LOOKUP_BITS 8, the bit layout macros)
//
// A seed from a 512-entry table indexed by the argument's lowest exponent bit
// and top 8 mantissa bits, then two Newton steps. croute.c's optimise_route
// normalises route legs with it.
//
// The C reads and writes the float's bits through a union. Lua 5.1 has no bit
// operations or typed arrays, so the bits are computed arithmetically, for the
// arguments the port passes: positive, finite floats.
//
// Rounding: each Newton step is ( 3.0 - r * r * x ) * r * 0.5 stored to float.
// r * r and * x are float operations (toward zero); 3.0 - t, * r and * 0.5 are
// double operations that are exact for t near 1 (a 24-bit t, a 26-bit
// difference, a 50-bit product), so the float store truncates the exact value.
// The table is built once at start-up. Its entries keep 8 bits of a rounded
// mantissa, and they are the same whether 1.0 / sqrt is rounded to nearest or
// toward zero (all 512 entries compared in C, docs/slices/supply-task-assignment-route-guide.md),
// so the start-up rounding mode does not matter; the port computes to nearest.
//

import { ASSERT } from "../assert";
import { f32Mul, toFloat32, toFloat32RTZ } from "../float32";

const EXP_POS = 23;
const EXP_BIAS = 127;
const LOOKUP_BITS = 8;
const LOOKUP_POS = EXP_POS - LOOKUP_BITS;
const SEED_POS = EXP_POS - 8;
const TABLE_SIZE = 2 * Math.pow(2, LOOKUP_BITS);

// the biased exponent and 23-bit mantissa field of a positive finite float
function floatFields(f: number): { exponent: number; mantissa: number } {
	let exponent = Math.floor(Math.log(f) / Math.LN2);
	while (Math.pow(2, exponent) > f) {
		exponent -= 1;
	}
	while (Math.pow(2, exponent + 1) <= f) {
		exponent += 1;
	}
	if (exponent < 1 - EXP_BIAS) {
		// subnormal: exponent field 0, mantissa in units of 2^-149
		return { exponent: 0, mantissa: f / Math.pow(2, 1 - EXP_BIAS - EXP_POS) };
	}
	return { exponent: exponent + EXP_BIAS, mantissa: (f / Math.pow(2, exponent) - 1) * Math.pow(2, EXP_POS) };
}

// the float with the given biased exponent (1..254) and mantissa field
function floatFromFields(exponent: number, mantissa: number): number {
	return Math.pow(2, exponent - EXP_BIAS) * (1 + mantissa / Math.pow(2, EXP_POS));
}

let inverse_sqrt_table: number[] = [];

// C provenance: invsqrt.c :: initialise_inverse_square_root_table
export function initialiseInverseSquareRootTable(): void {
	inverse_sqrt_table = [];

	for (let f = 0; f < TABLE_SIZE; f++) {
		// fi.i = ((EXP_BIAS - 1) << EXP_POS) | (f << LOOKUP_POS): f's top bit is the exponent's lowest
		const fi = floatFromFields(EXP_BIAS - 1 + Math.floor(f / 256), (f % 256) * Math.pow(2, LOOKUP_POS));

		// fo.f = 1.0 / sqrt (fi.f)
		const fo = floatFields(toFloat32(1.0 / Math.sqrt(fi)));

		// ((fo.i + (1 << (SEED_POS - 2))) >> SEED_POS) & 0xFF
		const bits = fo.exponent * Math.pow(2, EXP_POS) + fo.mantissa;

		inverse_sqrt_table.push(Math.floor((bits + Math.pow(2, SEED_POS - 2)) / Math.pow(2, SEED_POS)) % 256);
	}

	inverse_sqrt_table[TABLE_SIZE / 2] = 0xff;
}

// C provenance: invsqrt.c :: get_inverse_square_root (x: a positive finite float)
export function getInverseSquareRoot(x: number): number {
	ASSERT(inverse_sqrt_table.length === TABLE_SIZE, "inverse_sqrt_table initialised");

	const a = floatFields(x);

	// GET_EMANT (a): (a >> LOOKUP_POS) & LOOKUP_MASK: the exponent's lowest bit and the top 8 mantissa bits
	const emant = (a.exponent % 2) * 256 + Math.floor(a.mantissa / Math.pow(2, LOOKUP_POS));

	// SET_EXP (((3 * EXP_BIAS) - 1 - GET_EXP (a)) >> 1) | SET_MANTSEED (inverse_sqrt_table [GET_EMANT (a)])
	let r = floatFromFields(Math.floor((3 * EXP_BIAS - 1 - a.exponent) / 2), inverse_sqrt_table[emant] * Math.pow(2, SEED_POS));

	r = toFloat32RTZ((3.0 - f32Mul(f32Mul(r, r), x)) * r * 0.5);

	r = toFloat32RTZ((3.0 - f32Mul(f32Mul(r, r), x)) * r * 0.5);

	return r;
}

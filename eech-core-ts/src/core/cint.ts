//
// C integer semantics that JavaScript and Lua 5.1 numbers do not have.
//
// Both runtimes compute in doubles, and Lua 5.1 has no bitwise operators, so
// these are pure arithmetic and behave identically after TSTL transpilation.
//

import { EechUndefinedBehaviourError } from "./assert";

// C (int) cast of a double or float within int range: truncation toward zero.
export function toCInt(value: number): number {
	return value < 0 ? Math.ceil(value) : Math.floor(value);
}

// C integer division of two ints: the quotient truncated toward zero.
// Division by zero is undefined behaviour (SIGFPE on x86).
export function cIntDivide(numerator: number, denominator: number): number {
	if (denominator === 0) {
		throw new EechUndefinedBehaviourError("integer division by zero");
	}

	return toCInt(numerator / denominator);
}

// A store into an `unsigned int field : bits` bit-field keeps the low `bits`
// bits of the value (two's complement for negative values).
export function storeUnsignedBitfield(value: number, bits: number): number {
	const modulus = Math.pow(2, bits);

	// % truncates in JavaScript and floors in Lua; this form agrees in both
	return ((value % modulus) + modulus) % modulus;
}

// C unsigned int conversion of an int: value modulo 2^32.
export function toCUnsignedInt(value: number): number {
	return storeUnsignedBitfield(value, 32);
}

// C `a & b` for non-negative ints (as the port's bit-field and flag values
// are). Lua 5.1 has no bitwise operators, so this walks the bits.
export function cBitAnd(a: number, b: number): number {
	let result = 0;

	let bit = 1;

	while (a > 0 && b > 0) {
		if (a % 2 === 1 && b % 2 === 1) {
			result += bit;
		}

		a = Math.floor(a / 2);
		b = Math.floor(b / 2);
		bit *= 2;
	}

	return result;
}

// C `1 << bit` for 0 <= bit < 31
export function cBit(bit: number): number {
	return Math.pow(2, bit);
}

// C conversion of an unsigned int (0 .. 2^32 - 1) to int, as passing one
// through an `int` parameter does: the same bits, two's complement.
export function unsignedToCInt(value: number): number {
	return value >= Math.pow(2, 31) ? value - Math.pow(2, 32) : value;
}

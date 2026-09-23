//
// C `float` semantics: the EECH numerical contract (issue #7,
// docs/fidelity/fpu-semantics.md). Ports must follow it.
//
// EECH stores campaign quantities (supply levels, positions, ranges, timers)
// in 32-bit `float` variables and struct fields. JavaScript and Lua 5.1 both
// compute in doubles, so the port models every C float result explicitly.
//
//   Run-time EECH float arithmetic and assignment    ROUND TOWARD ZERO
//     EECH sets the FPU to round toward zero on its campaign thread
//     (startup.c :: set_fpu_rounding_mode_zero, re-asserted after every
//     library initialisation). Every float result EECH computes while running
//     - an operation on floats, a double expression stored as float, an
//     argument passed to a `float` parameter, an int converted to float - is
//     the exact result truncated to the float grid. Use toFloat32RTZ, f32Add,
//     f32Sub, f32Mul, f32Div or f32Sqrt (below), the one that matches the C
//     expression being ported. NEVER use toFloat32 for run-time arithmetic.
//
//   Values fixed at compile time, and scenario input  ROUND TO NEAREST
//     A C constant initialiser (e.g. time.c's `float system_delta_time = 0.1`)
//     is converted by the compiler, to nearest, not by the run-time FPU.
//     Scenario and test input is narrowed to nearest, as the C harness parses
//     it (next_float). Only these use toFloat32.
//
//   float -> int                                    TRUNCATION
//     toCInt (core/cint.ts); convert_float_to_int is fistp under round toward
//     zero, which truncates.
//
//   x87 intermediate evaluation precision           UNRESOLVED
//     The contract evaluates every operation at declared type. Whether the
//     original executable kept some intermediates at 53 or 64 bits, and where
//     it rounded them back to float, is open (it needs the Windows runtime
//     control-word trace and the shipped compiler's instructions). Do not
//     model extended intermediates ad hoc; get_2d_range's sqrt argument is the
//     documented canary.
//
// Math.fround is not available in Lua 5.1, so all of this is pure arithmetic
// that behaves identically after TSTL transpilation. toFloat32 is verified
// against Math.fround (test/unit/float32.test.ts); the round-toward-zero
// helpers bit for bit against the C oracle (test/c-reference/float32-rtz.cref.test.ts)
// and, from recorded results, in JavaScript and Lua 5.1.
//

const FLOAT32_OVERFLOW_THRESHOLD = 3.4028235677973366e38; // 2^128 - 2^103: halfway above FLT_MAX

const FLOAT32_MIN_EXPONENT = -126;

const FLOAT32_MANTISSA_BITS = 23;

export const FLT_MAX = 3.4028234663852886e38;

// Round to nearest (ties to even): compile-time constants and scenario input
// ONLY. Run-time EECH arithmetic rounds toward zero (toFloat32RTZ and the f32*
// helpers below).
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

//
// Round-toward-zero float arithmetic (issue #7, docs/fidelity/fpu-semantics.md).
//
// EECH runs the campaign with the FPU rounding toward zero, so every float
// result EECH computes at run time is the exact result truncated to the float
// grid. JS and Lua 5.1 round to nearest, so these helpers recover the exact
// result's position from an error term:
//
//   - a double result r that is not itself a float truncates to the same
//     float as the exact value (between r and the exact value, which differ
//     by at most half a double ulp, there is no float);
//   - when r IS a float, the sign of (exact - r) decides whether the answer is
//     r or the next float toward zero.
//
// Truncation composes: C evaluating a double expression toward zero and then
// narrowing toward zero gives the exact value truncated to float, so one
// narrowing of the exact value models both steps.
//
// toFloat32 above (round to nearest) is only for values fixed at compile time
// (C constant initialisers, e.g. time.c's 0.1) and for scenario input.
//

const FLOAT32_OVERFLOW = 3.402823669209385e38; // 2^128

// magnitude > 0, finite, below 2^128: the float spacing at magnitude
function float32Quantum(magnitude: number): number {
	let exponent = Math.floor(Math.log(magnitude) / Math.LN2);

	while (Math.pow(2, exponent) > magnitude) {
		exponent -= 1;
	}

	while (Math.pow(2, exponent + 1) <= magnitude) {
		exponent += 1;
	}

	if (exponent < FLOAT32_MIN_EXPONENT) {
		exponent = FLOAT32_MIN_EXPONENT;
	}

	return Math.pow(2, exponent - FLOAT32_MANTISSA_BITS);
}

// C `(float) value` with the FPU rounding toward zero: truncation to the float grid.
// Finite values beyond FLT_MAX give FLT_MAX; infinities, NaN and signed zeros are kept.
export function toFloat32RTZ(value: number): number {
	if (value !== value || value === 0) {
		return value;
	}

	const sign = value < 0 ? -1 : 1;

	const magnitude = value * sign;

	if (magnitude === 1 / 0) {
		return value;
	}

	if (magnitude >= FLOAT32_OVERFLOW) {
		return sign * FLT_MAX;
	}

	const quantum = float32Quantum(magnitude);

	return sign * Math.floor(magnitude / quantum) * quantum;
}

// the float next to f (a nonzero finite float) toward zero
function nextFloat32TowardZero(f: number): number {
	const sign = f < 0 ? -1 : 1;

	const magnitude = f * sign;

	let quantum = float32Quantum(magnitude);

	// at a power of two (above the subnormal range) the spacing below is half
	if (magnitude === Math.pow(2, FLOAT32_MANTISSA_BITS) * quantum && quantum > Math.pow(2, FLOAT32_MIN_EXPONENT - FLOAT32_MANTISSA_BITS)) {
		quantum = quantum / 2;
	}

	return sign * (magnitude - quantum);
}

// truncates the exact value r + (something with the sign of error) to float
function truncateWithError(r: number, error: number): number {
	const f = toFloat32RTZ(r);

	if (f === r && f !== 0 && ((error < 0 && f > 0) || (error > 0 && f < 0))) {
		return nextFloat32TowardZero(f);
	}

	return f;
}

// the rounding error of a + b in double (Knuth's TwoSum): a + b = s + e exactly
function sumError(a: number, b: number, s: number): number {
	const bVirtual = s - a;

	return a - (s - bVirtual) + (b - bVirtual);
}

// (float) (a + b) for doubles a, b: C float addition or subtraction, or a
// double sum narrowed to float. The exact sum truncated to float.
export function f32Add(a: number, b: number): number {
	const s = a + b;

	return truncateWithError(s, sumError(a, b, s));
}

// a - b for floats (see f32Add)
export function f32Sub(a: number, b: number): number {
	return f32Add(a, -b);
}

const DBL_MAX = 1.7976931348623157e308;

const DOUBLE_MANTISSA_BITS = 52;

// the double next to d toward zero, for a normal d above the smallest normal
// (2^-1022): an inexact double sum is never smaller, because sums in and just
// above the subnormal range lie on its fixed 2^-1074 grid and are exact
function nextDoubleTowardZero(d: number): number {
	const sign = d < 0 ? -1 : 1;

	const magnitude = d * sign;

	let exponent = Math.floor(Math.log(magnitude) / Math.LN2);

	while (Math.pow(2, exponent) > magnitude) {
		exponent -= 1;
	}

	while (Math.pow(2, exponent + 1) <= magnitude) {
		exponent += 1;
	}

	let quantum = Math.pow(2, exponent - DOUBLE_MANTISSA_BITS);

	// at a power of two the spacing below is half
	if (magnitude === Math.pow(2, exponent)) {
		quantum = quantum / 2;
	}

	return sign * (magnitude - quantum);
}

// a + b for doubles, rounded toward zero: a C double sum under EECH's FPU
// rounding (e.g. keysite.c's (xmax - xmin) + 1.0). Needed where the double
// result is not stored straight into a float by one truncation of the exact
// value; a sum that is is f32Add.
export function f64AddRTZ(a: number, b: number): number {
	const s = a + b;

	if (s === 1 / 0 || s === -1 / 0) {
		// a finite sum beyond DBL_MAX truncates to DBL_MAX
		return a === s || b === s ? s : s > 0 ? DBL_MAX : -DBL_MAX;
	}

	const error = sumError(a, b, s);

	if (s !== 0 && ((error < 0 && s > 0) || (error > 0 && s < 0))) {
		return nextDoubleTowardZero(s);
	}

	return s;
}

// a * b for floats (or a float and an int of at most 24 bits): the double
// product is exact, so truncating it is exact
export function f32Mul(a: number, b: number): number {
	return toFloat32RTZ(a * b);
}

// a / b for floats (or a float and an int of at most 24 bits), and
// (float) sqrt (x) for a float x (sqrt computes in double).
//
// For these the double result needs no error term. A quotient or square root
// of floats is either exactly a float or further than half a double ulp from
// every float: its distance to a float f is a nonzero multiple of a quantum
// set by the 24-bit operands, at least 2^-49 relative, while the double
// rounding error is at most 2^-53 relative (53 >= 2 * 24 + 2). So the
// correctly rounded double result lies strictly between the same two floats
// as the exact result, or equals it, and truncating it is exact truncation.
// test/c-reference/float32-rtz.cref.test.ts checks both against C.

export function f32Div(a: number, b: number): number {
	return toFloat32RTZ(a / b);
}

export function f32Sqrt(x: number): number {
	return toFloat32RTZ(Math.sqrt(x));
}

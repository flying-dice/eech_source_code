//
// IEEE 754 single-precision bit patterns as the C harness prints them (%08x),
// computed arithmetically so that the same code runs under Lua 5.1 (no
// DataView, no bitwise operators). The value must already be a float32 value.
//

const TWO_POW_23 = 8388608;

const TWO_POW_31 = 2147483648;

export function float32Bits(value: number): number {
	const negative = value < 0 || (value === 0 && 1 / value < 0);

	const magnitude = negative ? -value : value;

	let exponentField = 0;
	let mantissa = 0;

	if (magnitude === 1 / 0) {
		exponentField = 255;
	} else if (magnitude !== 0) {
		let exponent = Math.floor(Math.log(magnitude) / Math.LN2);

		while (Math.pow(2, exponent) > magnitude) {
			exponent -= 1;
		}

		while (Math.pow(2, exponent + 1) <= magnitude) {
			exponent += 1;
		}

		if (exponent < -126) {
			// subnormal: a multiple of 2^-149
			mantissa = magnitude / Math.pow(2, -149);
		} else {
			exponentField = exponent + 127;
			mantissa = (magnitude / Math.pow(2, exponent) - 1) * TWO_POW_23;
		}
	}

	return (negative ? TWO_POW_31 : 0) + exponentField * TWO_POW_23 + mantissa;
}

const HEX_DIGITS = "0123456789abcdef";

export function hex8(value: number): string {
	let remaining = value;
	let text = "";

	for (let i = 0; i < 8; i++) {
		const digit = remaining % 16;

		text = HEX_DIGITS.substring(digit, digit + 1) + text;
		remaining = (remaining - digit) / 16;
	}

	return text;
}

export function float32Hex(value: number): string {
	return hex8(float32Bits(value));
}

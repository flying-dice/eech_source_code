//
// C provenance: modules/maths/miscmath.h, modules/maths/constant.h
//

// C provenance: miscmath.h :: #define bound(VALUE,LOWER,UPPER)
// ( ( VALUE ) < ( LOWER ) ? ( LOWER ) : ( ( VALUE ) > ( UPPER ) ? ( UPPER ) : ( VALUE ) ) )
export function bound(value: number, lower: number, upper: number): number {
	return value < lower ? lower : value > upper ? upper : value;
}

// C provenance: Windows SDK windef.h :: #define max(a,b) (((a) > (b)) ? (a) : (b))
// (EECH relies on the platform macro). Unlike Math.max, max (NaN, 0) is 0.
export function max(a: number, b: number): number {
	return a > b ? a : b;
}

// C provenance: constant.h :: #define METRE (1.0f), #define KILOMETRE (1000 * METRE)
export const METRE = 1.0;

export const KILOMETRE = 1000 * METRE;

//
// INVESTIGATION ONLY (issue #14, F1): how eech_extracted_taskgen.c is built
// for the probe. `-O2` follows the harness's -O0, so it applies to this unit
// only; the rest of the harness is the canonical build.
//

export interface F1Variant {
	name: string;
	model: string;
	unitFlags: string[];
}

export const F1_VARIANTS: F1Variant[] = [
	{ name: "zero-O0", model: "the harness's compatibility pinning (control: must equal the canonical oracle)", unitFlags: ["-ftrivial-auto-var-init=zero"] },
	{ name: "zero-O2", model: "zero-initialised, optimised", unitFlags: ["-ftrivial-auto-var-init=zero", "-O2"] },
	{ name: "pattern-O0", model: "pattern-initialised (a debugging fill, as MSVC /RTC and debug heaps use)", unitFlags: ["-ftrivial-auto-var-init=pattern"] },
	{ name: "pattern-O2", model: "pattern-initialised, optimised", unitFlags: ["-ftrivial-auto-var-init=pattern", "-O2"] },
	{ name: "uninitialised-O0", model: "no initialisation: whatever the stack held (the C as written)", unitFlags: ["-ftrivial-auto-var-init=uninitialized"] },
	{ name: "uninitialised-O2", model: "no initialisation, optimised (register or stack residue)", unitFlags: ["-ftrivial-auto-var-init=uninitialized", "-O2"] },
];

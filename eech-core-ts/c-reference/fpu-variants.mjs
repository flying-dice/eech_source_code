//
// INVESTIGATION ONLY (issue #7): the floating-point environments the C oracle
// is run under by the FPU fidelity spike (npm run spike:fpu). None of these is
// the canonical oracle; see docs/fidelity/fpu-semantics.md.
//
// x87 control word: bits 8-9 precision (0 = 24, 2 = 53, 3 = 64 bits), bits
// 10-11 rounding (0 = nearest, 3 = toward zero), 0x7f = all exceptions masked.
// MXCSR rounding: bits 13-14 (0x6000 = toward zero).
//

const SSE = ["-msse2", "-mfpmath=sse", "-O0", "-ffp-contract=off", "-fno-fast-math"];

// x87 arithmetic, rounded to the declared type at assignments, casts,
// arguments and returns (as MSVC /fp:precise and the C standard specify)
const X87 = ["-mfpmath=387", "-fexcess-precision=standard", "-O0", "-ffp-contract=off", "-fno-fast-math"];

const X87_O2 = ["-mfpmath=387", "-fexcess-precision=standard", "-O2", "-ffp-contract=off", "-fno-fast-math"];

const cw = (value) => `-DHARNESS_X87_CW=0x${value.toString(16).padStart(4, "0")}`;

export const FPU_VARIANTS = [
	{ name: "sse-rn", model: "canonical flags rebuilt through the variant path (control: must equal canonical)", flags: SSE, defines: [] },
	{ name: "sse-rtz", model: "SSE at declared type, round toward zero (rounding mode alone)", flags: SSE, defines: ["-DHARNESS_MXCSR_RC=0x6000", cw(0x0f7f), "-DHARNESS_FISTP"] },
	{ name: "x87-rn-pc64", model: "x87, nearest, 64-bit precision (Linux default control word)", flags: X87, defines: [cw(0x037f)] },
	{ name: "x87-rn-pc53", model: "x87, nearest, 53-bit precision (Windows/MSVC CRT default; EECH without its RTZ calls)", flags: X87, defines: [cw(0x027f)] },
	{ name: "x87-rn-pc53-fistp", model: "as x87-rn-pc53 with convert_float_to_int by fistp", flags: X87, defines: [cw(0x027f), "-DHARNESS_FISTP"] },
	{ name: "x87-rtz-pc53", model: "x87, toward zero, 53-bit precision (EECH application thread per source)", flags: X87, defines: [cw(0x0e7f), "-DHARNESS_FISTP"] },
	{ name: "x87-rtz-pc53-O2", model: "as x87-rtz-pc53, optimised (compiler artefact check)", flags: X87_O2, defines: [cw(0x0e7f), "-DHARNESS_FISTP"] },
	{ name: "x87-rn-pc24", model: "x87, nearest, 24-bit precision (Direct3D 9 default without FPU_PRESERVE)", flags: X87, defines: [cw(0x007f), "-DHARNESS_FISTP"] },
	{ name: "x87-rtz-pc24", model: "x87, toward zero, 24-bit precision (Direct3D precision with EECH's RTZ re-assertion)", flags: X87, defines: [cw(0x0c7f), "-DHARNESS_FISTP"] },
];

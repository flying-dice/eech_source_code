//
// Guards the canonical oracle's floating-point environment against drift.
//
// EECH numerical contract (docs/fidelity/fpu-semantics.md):
//   rounding direction          toward zero (EECH's set_fpu_rounding_mode_zero)
//   declared float operations   IEEE binary32, rounded toward zero
//   float -> int                truncation
//   x87 intermediate precision  unresolved: not modelled (declared type)
//
// The frozen fixtures were recorded under exactly this environment. A
// compiler, flag or platform change that altered it would silently change
// what "the original C" means for every numerical port; a deliberate change
// must update this test and re-record and review the fixtures.
//

import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";
// @ts-expect-error plain ESM build script without type declarations
import { HARNESS_BINARY } from "../../c-reference/build.mjs";

describe("C reference: floating-point environment", () => {
	it("runs the original code with SSE at declared type, rounding toward zero", () => {
		const run = spawnSync(HARNESS_BINARY as string, [], { input: "fpu\n", encoding: "utf8" });

		expect(run.status).toBe(0);
		// x87 control word 0x0f7f: round toward zero (chop), platform default
		// precision, exceptions masked (x87 serves libm's sqrt and fistp);
		// MXCSR rounding 3 = toward zero; floats evaluated at declared type
		expect(run.stdout).toBe("fpu cw 0f7f mxcsr-rc 3 flt-eval-method 0\n");
	});

	it("parses scenario input to nearest, whatever the arithmetic rounding", () => {
		// 0.1 is 0x3dcccccd to nearest and 0x3dcccccc toward zero
		const run = spawnSync(HARNESS_BINARY as string, [], { input: "f32 mul 0.1 1\nf32 narrow 0.1\n", encoding: "utf8" });

		expect(run.stdout).toBe("f32 3dcccccd\nf32 3dcccccc\n");
	});
});

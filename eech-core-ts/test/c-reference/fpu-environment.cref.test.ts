//
// Guards the canonical oracle's floating-point environment against drift.
//
// The frozen fixtures were recorded with EECH arithmetic evaluated by SSE at
// declared type (FLT_EVAL_METHOD 0) under round to nearest. A compiler, flag
// or platform change that altered this would silently change what "the
// original C" means for every numerical port. Whether this is the right
// environment is the subject of issue #7 (docs/fidelity/fpu-semantics.md); a
// change must be made here deliberately, with the fixtures re-recorded and
// reviewed.
//

import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";
// @ts-expect-error plain ESM build script without type declarations
import { HARNESS_BINARY } from "../../c-reference/build.mjs";

describe("C reference: floating-point environment", () => {
	it("runs the original code with SSE at declared type, round to nearest", () => {
		const run = spawnSync(HARNESS_BINARY as string, [], { input: "fpu\n", encoding: "utf8" });

		expect(run.status).toBe(0);
		// x87 control word: the Linux default (unused by SSE code, but fistp and
		// libm x87 paths would read it); MXCSR rounding 0 = nearest
		expect(run.stdout).toBe("fpu cw 037f mxcsr-rc 0 flt-eval-method 0\n");
	});
});

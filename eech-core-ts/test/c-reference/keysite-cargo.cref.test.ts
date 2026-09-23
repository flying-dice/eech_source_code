//
// Slice 4: every keysite cargo expectation must be what the original C
// (keysite.c :: update_keysite_cargo, compiled whole) does, and the
// TypeScript output must equal the C's line for line.
//

import { describe, expect, it } from "vitest";
import { KEYSITE_CARGO_CASES } from "../scenarios/keysite-cargo.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { runCLifecycle } from "./c-harness";

describe("keysite cargo expectations hold for the original EECH C", () => {
	for (const c of KEYSITE_CARGO_CASES) {
		it(c.id, () => {
			const output = runCLifecycle(c.spec);

			expect(firstUnmatchedLine(output, c.expected), output.join("\n")).toBe("");

			for (const prefix of c.absent) {
				expect(output.filter((line) => line.startsWith(prefix)), `absent: ${prefix}`).toEqual([]);
			}

			expect(runLifecycle(c.spec)).toEqual(output);
		});
	}
});

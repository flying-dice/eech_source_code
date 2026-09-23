//
// Slice 5a: the hand-derived expectations of
// test/scenarios/force-low-on-supplies.cases.ts hold for the executed original
// C (fc_msgs.c compiled whole), and TypeScript's whole output equals the C's.
//

import { describe, expect, it } from "vitest";
import { FORCE_LOW_ON_SUPPLIES_CASES } from "../scenarios/force-low-on-supplies.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { runCLifecycle } from "./c-harness";

describe("force low-on-supplies expectations hold for the original EECH C", () => {
	for (const c of FORCE_LOW_ON_SUPPLIES_CASES) {
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

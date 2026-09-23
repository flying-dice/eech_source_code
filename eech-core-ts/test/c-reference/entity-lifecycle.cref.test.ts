import { describe, expect, it } from "vitest";
import { ENTITY_LIFECYCLE_CASES } from "../scenarios/entity-lifecycle.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { runCLifecycle } from "./c-harness";

// Every expected lifecycle line must be what the original C does, and the
// TypeScript output must equal the C's line for line.
describe("entity lifecycle expectations hold for the original EECH C", () => {
	for (const c of ENTITY_LIFECYCLE_CASES) {
		it(c.id, () => {
			const output = runCLifecycle(c.spec);

			expect(firstUnmatchedLine(output, c.expected), output.join("\n")).toBe("");

			expect(runLifecycle(c.spec)).toEqual(output);
		});
	}
});

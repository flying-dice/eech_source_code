//
// Slice 3 behaviour matrix under JavaScript semantics (the Lua 5.1 and C
// reference runners replay the same cases).
//

import { describe, expect, it } from "vitest";
import { ENTITY_LIFECYCLE_CASES } from "../scenarios/entity-lifecycle.cases";
import { C_REFERENCE_RANDOM_LIFECYCLES } from "../scenarios/generated/c-reference-random-lifecycles.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";

describe("entity lifecycle (cargo create / destroy, sectors, heap)", () => {
	for (const c of ENTITY_LIFECYCLE_CASES) {
		it(c.id, () => {
			const output = runLifecycle(c.spec);

			expect(firstUnmatchedLine(output, c.expected), output.join("\n")).toBe("");
		});
	}
});

describe("random lifecycles recorded from the original C", () => {
	it(`all ${C_REFERENCE_RANDOM_LIFECYCLES.length} match`, () => {
		for (const c of C_REFERENCE_RANDOM_LIFECYCLES) {
			expect(runLifecycle(c.spec), c.id).toEqual(c.expected);
		}
	});
});

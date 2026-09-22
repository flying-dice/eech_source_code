import { describe, expect, it } from "vitest";
import { ASSESS_GROUP_SUPPLIES_CASES } from "../scenarios/assess-group-supplies.cases";
import { runCScenario } from "./c-harness";

// Every expected outcome in the behaviour matrix must be what the original C does.
describe("behaviour matrix expectations hold for the original EECH C", () => {
	for (const c of ASSESS_GROUP_SUPPLIES_CASES) {
		it(c.id, () => {
			expect(runCScenario(c.spec)).toEqual(c.expected);
		});
	}
});

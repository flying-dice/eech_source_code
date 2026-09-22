import { describe, expect, it } from "vitest";
import { ASSESS_GROUP_SUPPLIES_CASES } from "../scenarios/assess-group-supplies.cases";
import { runScenario } from "../scenarios/campaign-scenario";
import { C_REFERENCE_RANDOM_CASES } from "../scenarios/generated/c-reference-random.cases";

describe("assess_group_supplies / get_closest_keysite conformance (JavaScript)", () => {
	for (const c of ASSESS_GROUP_SUPPLIES_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			expect(runScenario(c.spec)).toEqual(c.expected);
		});
	}
});

describe("random scenarios recorded from the original C (JavaScript)", () => {
	it(`all ${C_REFERENCE_RANDOM_CASES.length} match`, () => {
		for (const c of C_REFERENCE_RANDOM_CASES) {
			expect(runScenario(c.spec), c.id).toEqual(c.expected);
		}
	});
});

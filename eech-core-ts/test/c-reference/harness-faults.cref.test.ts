//
// The C reference must report EECH's NULL dereferences as outcomes, but must
// never turn any other crash into a result.
//

import { describe, expect, it } from "vitest";
import { ASSESS_GROUP_SUPPLIES_CASES } from "../scenarios/assess-group-supplies.cases";
import { runCInput, runCScenario } from "./c-harness";

describe("C reference harness fault handling", () => {
	it("reports a read inside the NULL page as a NULL dereference", () => {
		expect(runCInput("op fault-null\n").result).toBe("null-dereference");
	});

	it("reports the original code's NULL dereference, with the final state", () => {
		const c = ASSESS_GROUP_SUPPLIES_CASES.find((x) => x.id === "no-force-for-side-dereferences-null");
		expect(c).toBeDefined();
		const outcome = runCScenario(c!.spec);
		expect(outcome.result).toBe("null-dereference");
		expect(outcome).toEqual(c!.expected);
	});

	it("never turns a fault outside the NULL page into an outcome", () => {
		expect(() => runCInput("op fault-unmapped\n")).toThrow(/killed by SIGSEGV/);
	});
});

//
// Slice 6a: the hand-derived expectations of
// test/scenarios/supply-task-assignment.cases.ts hold for the executed original
// C (assign_keysite_tasks, get_suitable_registered_group,
// suitable_group_task_specific_checks and check_group_members_awake from
// assign.c, qs from en_misc.c and assess_group_task_locality_factor from
// group.c extracted; ac_float.c, ac_dbase.c and pi_list.c compiled whole;
// assign_primary_task_to_group trapped), and TypeScript's whole output equals
// the C's.
//

import { describe, expect, it } from "vitest";
import { SUPPLY_TASK_ASSIGNMENT_CASES, supplyTaskAssignmentExpectationFailure } from "../scenarios/supply-task-assignment.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { runCLifecycle } from "./c-harness";

describe("supply task assignment expectations hold for the original EECH C", () => {
	for (const c of SUPPLY_TASK_ASSIGNMENT_CASES) {
		it(c.id, () => {
			const output = runCLifecycle(c.spec);

			expect(supplyTaskAssignmentExpectationFailure(c, output, firstUnmatchedLine), output.join("\n")).toBe("");

			expect(runLifecycle(c.spec)).toEqual(output);
		});
	}
});

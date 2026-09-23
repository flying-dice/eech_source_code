//
// Slice 6b: the hand-derived expectations of
// test/scenarios/supply-task-transaction.cases.ts hold for the executed original
// C (croute.c, the waypoint and guide files, landing.c and ts_vec3d.c compiled
// whole; assign_primary_task_to_group, assign_task_to_group and
// push_task_onto_group_task_stack extracted from assign.c;
// assign_task_to_group_members trapped), and TypeScript's whole output equals
// the C's.
//

import { describe, expect, it } from "vitest";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { SUPPLY_TASK_TRANSACTION_CASES, supplyTaskTransactionExpectationFailure } from "../scenarios/supply-task-transaction.cases";
import { runCLifecycle } from "./c-harness";

describe("supply task transaction expectations hold for the original EECH C", () => {
	for (const c of SUPPLY_TASK_TRANSACTION_CASES) {
		it(c.id, () => {
			const output = runCLifecycle(c.spec);

			expect(supplyTaskTransactionExpectationFailure(c, output, firstUnmatchedLine), output.join("\n")).toBe("");

			expect(runLifecycle(c.spec)).toEqual(output);
		});
	}
});

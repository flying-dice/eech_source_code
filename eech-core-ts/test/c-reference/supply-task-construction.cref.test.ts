//
// Slice 5b: the hand-derived expectations of
// test/scenarios/supply-task-construction.cases.ts hold for the executed
// original C (task.c, ts_creat.c, ts_ptr.c, suitable.c compiled whole;
// create_task and create_supply_task extracted), and TypeScript's whole output
// equals the C's.
//

import { describe, expect, it } from "vitest";
import { SUPPLY_TASK_CONSTRUCTION_CASES, f1CompatibilityPair, f1SemanticRouteFailure, supplyTaskExpectationFailure } from "../scenarios/supply-task-construction.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { runCLifecycle } from "./c-harness";

describe("supply task construction expectations hold for the original EECH C", () => {
	for (const c of SUPPLY_TASK_CONSTRUCTION_CASES) {
		it(c.id, () => {
			const output = runCLifecycle(c.spec);

			expect(supplyTaskExpectationFailure(c, output, firstUnmatchedLine), output.join("\n")).toBe("");

			expect(runLifecycle(c.spec)).toEqual(output);
		});
	}
});

describe("F1 compatibility holds for the original C compiled with zero auto-initialisation", () => {
	it("single player and multiplayer construct the same semantic route", () => {
		const pair = f1CompatibilityPair();

		const singlePlayer = runCLifecycle(pair.singlePlayer);
		const multiplayer = runCLifecycle(pair.multiplayer);

		expect(f1SemanticRouteFailure(singlePlayer, multiplayer)).toBe("");

		expect(runLifecycle(pair.singlePlayer)).toEqual(singlePlayer);
		expect(runLifecycle(pair.multiplayer)).toEqual(multiplayer);
	});
});

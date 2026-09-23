//
// Slice 5a on the frozen Slice 4 corpus.
//
// The frozen Slice 4 fixtures were recorded before the force response was
// ported and do not observe the create_supply_task boundary. They still pass
// byte for byte (keysite-cargo.cref.test.ts, differential.cref.test.ts): the
// response now runs behind the unchanged trace line, and the boundary call
// has no other effect.
//
// Here the same scenarios run again with the boundary observed. The original
// C and TypeScript must agree on the whole output, and removing the
// create-supply-task lines must give back the frozen output exactly: the
// response adds calls and changes nothing else.
//

import { describe, expect, it } from "vitest";
import { C_REFERENCE_RANDOM_KEYSITE_CARGO } from "../scenarios/generated/c-reference-random-keysite-cargo.cases";
import { KEYSITE_CARGO_CASES } from "../scenarios/keysite-cargo.cases";
import { runLifecycle, type LifecycleSpec } from "../scenarios/lifecycle-scenario";
import { runCLifecycle } from "./c-harness";

function observed(spec: LifecycleSpec): LifecycleSpec {
	return { heap: spec.heap, forces: spec.forces, keysites: spec.keysites, ops: [{ kind: "observe-supply-tasks" }, ...spec.ops] };
}

const withoutBoundary = (lines: string[]): string[] => lines.filter((line) => !line.startsWith("create-supply-task "));

describe("Slice 5a on the frozen Slice 4 corpus", () => {
	it("the Slice 4 matrix: C == TS with the boundary observed, and only boundary lines are added", () => {
		let calls = 0;

		for (const c of KEYSITE_CARGO_CASES) {
			const frozen = runCLifecycle(c.spec);
			const output = runCLifecycle(observed(c.spec));

			expect(runLifecycle(observed(c.spec)), c.id).toEqual(output);
			expect(withoutBoundary(output), c.id).toEqual(frozen);

			calls += output.length - frozen.length;
		}

		// the airbase in these cases supplies itself whenever it notifies with crates (F2)
		expect(calls).toBeGreaterThan(0);
	});

	it("the 150 recorded random keysite cargo scenarios: C == TS with the boundary observed, and only boundary lines are added", () => {
		let reaching = 0;

		for (const c of C_REFERENCE_RANDOM_KEYSITE_CARGO) {
			const output = runCLifecycle(observed(c.spec));

			expect(runLifecycle(observed(c.spec)), c.id).toEqual(output);
			expect(withoutBoundary(output), c.id).toEqual(c.expected);

			if (output.length > c.expected.length) {
				reaching++;
			}
		}

		expect(reaching).toBeGreaterThan(0);
	});
});

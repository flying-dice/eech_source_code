//
// The Slice 6a fixture migration (issue #18): a mechanical check of every
// Slice 6a scenario, migrated to Slice 6b's state invariants
// (test/scenarios/slice-6a-migration.ts), against the output the original C
// gave it at Slice 6a (master 81ed32e, test/c-reference/migration/slice-6a-baseline.json:
// its hand matrix and its recorded random fixture, inputs and the 6a C harness's output).
//
// For each scenario, with k the index of its Slice 6a result line:
//
//   - the first k lines (everything before the former boundary) are identical;
//   - a scenario that did not select a SUPPLY task has the identical result
//     line (no selection, an ASSERT, or a selection other than SUPPLY, which
//     stays at the 6a decision boundary);
//   - a scenario that selected a SUPPLY task ends at the Slice 6b boundary for
//     the same group, and the same task (and no other) is ASSIGNED;
//   - TypeScript's whole output equals the C's;
//   - a hand matrix case is exactly the migration of its Slice 6a scenario.
//
// RECORD_CREF=1 (npm run cref:migrate-6a) also writes the migrated random
// fixtures (the selection corpus and the SUPPLY transactions) and the report,
// docs/slices/supply-task-assignment-fixture-migration.md.
//

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { EntitySubTypeTask } from "../../src/generated/c-enums";
import type { LifecycleCase } from "../scenarios/entity-lifecycle.cases";
import { runLifecycle } from "../scenarios/lifecycle-scenario";
import { migrateSlice6aSpec, type LegacySpec } from "../scenarios/slice-6a-migration";
import { SUPPLY_TASK_ASSIGNMENT_CASES } from "../scenarios/supply-task-assignment.cases";
import { runCLifecycle } from "./c-harness";

interface BaselineEntry {
	corpus: "hand" | "random";
	id: string;
	spec: LegacySpec;
	output: string[];
}

interface Migration {
	entry: BaselineEntry;
	transaction: boolean;
	corrections: string[];
	output: string[];
	classification: string;
	failure: string;
	newResult: string;
	spec: LifecycleCase["spec"];
}

const OLD_BOUNDARY = "result boundary assign_primary_task_to_group ";

const NEW_BOUNDARY = "result boundary assign_task_to_group_members ";

const baseline: BaselineEntry[] = JSON.parse(readFileSync(join(__dirname, "migration/slice-6a-baseline.json"), "utf8"));

function resultIndex(output: string[]): number {
	return output.findIndex((line) => line.startsWith("result "));
}

function taskType(spec: LegacySpec, label: string): number {
	for (const op of spec.ops) {
		if (op.kind === "unassigned-task" && op.label === label) {
			return op.subType;
		}
	}

	// a task the scenario constructs (Slice 5b's create_supply_task)
	return EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;
}

function migrate(entry: BaselineEntry): Migration {
	const k = resultIndex(entry.output);

	const oldResult = entry.output[k];

	let selected: string[] = [];

	if (oldResult.startsWith(OLD_BOUNDARY)) {
		selected = oldResult.substring(OLD_BOUNDARY.length).split(" ");
	}

	const transaction = selected.length === 2 && taskType(entry.spec, selected[1]) === EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;

	const migrated = migrateSlice6aSpec(entry.spec, transaction);

	const output = runCLifecycle(migrated.spec);

	let ts: string[];

	try {
		ts = runLifecycle(migrated.spec);
	} catch (e) {
		ts = [`TypeScript threw: ${(e as Error).message}`];
	}

	const newResult = output[resultIndex(output)];

	let failure = "";

	let classification: string;

	if (JSON.stringify(ts) !== JSON.stringify(output)) {
		failure = "TypeScript's output differs from the C's";
	}

	for (let i = 0; i < k && failure === ""; i++) {
		if (output[i] !== entry.output[i]) {
			failure = `line ${i} before the former boundary differs: ${output[i]}`;
		}
	}

	if (!transaction) {
		classification = oldResult.startsWith(OLD_BOUNDARY) ? "selection other than SUPPLY: stays at the 6a decision boundary" : oldResult === "result ok" ? "no selection" : "ASSERT before any selection";

		if (failure === "" && output[k] !== oldResult) {
			failure = `result changed: ${output[k]}`;
		}
	} else {
		classification = "SUPPLY selection: runs the 6b transaction";

		const [group, task] = selected;

		if (failure === "" && !(newResult !== undefined && newResult.startsWith(`${NEW_BOUNDARY}${group} `))) {
			failure = `not the 6b boundary for ${group}: ${newResult}`;
		}

		const assigned = output.filter((line) => line.startsWith("task ") && line.includes(" state 1 ")).map((line) => line.split(" ")[1]);

		if (failure === "" && !(assigned.length === 1 && assigned[0] === task)) {
			failure = `assigned tasks ${assigned.join(",")} are not ${task}`;
		}
	}

	return { entry, transaction, corrections: migrated.corrections, output, classification, failure, newResult, spec: migrated.spec };
}

const migrations = baseline.map(migrate);

describe("Slice 6a fixture migration", () => {
	it("covers the whole Slice 6a corpus", () => {
		expect(baseline.filter((e) => e.corpus === "hand").length).toBe(SUPPLY_TASK_ASSIGNMENT_CASES.length);
		expect(baseline.filter((e) => e.corpus === "random").length).toBe(150);
	});

	for (const m of migrations) {
		it(`${m.entry.corpus} ${m.entry.id}: ${m.classification}`, () => {
			expect(m.failure).toBe("");
		});
	}

	it("the hand matrix is exactly the migration of its Slice 6a scenarios", () => {
		for (const m of migrations.filter((x) => x.entry.corpus === "hand")) {
			const c = SUPPLY_TASK_ASSIGNMENT_CASES.find((x) => x.id === m.entry.id);

			expect(c, m.entry.id).toBeDefined();
			expect(c?.transaction, m.entry.id).toBe(m.transaction);
			expect(c?.spec, m.entry.id).toEqual(m.spec);
		}
	});

	if (process.env.RECORD_CREF === "1") {
		it("records the migrated fixtures and the report", () => {
			expect(migrations.every((m) => m.failure === "")).toBe(true);

			const random = migrations.filter((m) => m.entry.corpus === "random");

			const write = (file: string, name: string, cases: Migration[], what: string): void => {
				const records: LifecycleCase[] = cases.map((m) => ({
					id: m.entry.id,
					c: `random supply task assignment scenario (seed 20261016), migrated to Slice 6b's state invariants (${m.corrections.join("; ")}); output recorded from the original C`,
					spec: m.spec,
					expected: m.output,
				}));

				const text = [
					"//",
					"// GENERATED by `npm run cref:migrate-6a` from the executed original EECH C. Do not edit.",
					`// ${what}`,
					"// Slice 6a's recorded random scenarios (seed 20261016), migrated by",
					"// test/scenarios/slice-6a-migration.ts and checked against their Slice 6a output",
					"// (test/c-reference/slice-6a-migration.cref.test.ts). `expected` is the harness's whole output.",
					"//",
					"",
					'import type { LifecycleCase } from "../entity-lifecycle.cases";',
					"",
					`export const ${name}: LifecycleCase[] = ${JSON.stringify(records, null, "\t")};`,
					"",
				].join("\n");

				writeFileSync(join(__dirname, "../scenarios/generated", file), text);
			};

			write(
				"c-reference-random-supply-task-assignment.cases.ts",
				"C_REFERENCE_RANDOM_SUPPLY_TASK_ASSIGNMENT",
				random.filter((m) => !m.transaction),
				"The Slice 6a selection corpus: scenarios with no selection, an ASSERT, or a selection other than SUPPLY (the 6a decision boundary).",
			);

			write(
				"c-reference-migrated-supply-task-transaction.cases.ts",
				"C_REFERENCE_MIGRATED_SUPPLY_TASK_TRANSACTION",
				random.filter((m) => m.transaction),
				"Slice 6a's SUPPLY selections, now running the Slice 6b transaction to assign_task_to_group_members.",
			);

			const count = (f: (m: Migration) => boolean): number => migrations.filter(f).length;

			const classes: string[] = [];

			for (const m of migrations) {
				if (classes.indexOf(m.classification) < 0) {
					classes.push(m.classification);
				}
			}

			const lines = [
				"# Slice 6a fixture migration (Slice 6b, issue #18)",
				"",
				"GENERATED by `npm run cref:migrate-6a` (test/c-reference/slice-6a-migration.cref.test.ts). Do not edit.",
				"",
				"A test-oracle correction, not a campaign behaviour change. Slice 6a's corpus",
				"was sufficient for the boundary it tested (the decision at",
				"`assign_primary_task_to_group`) but restored states EECH cannot hold: tasks",
				"without their route, scenarios without a world map, aircraft outside the map.",
				"Slice 6b executes past that boundary and reads all of it, so each scenario is",
				"migrated to valid state (test/scenarios/slice-6a-migration.ts) and checked",
				"against the output the original C gave it at Slice 6a (master 81ed32e,",
				"test/c-reference/migration/slice-6a-baseline.json).",
				"",
				"Every row passed these checks, re-run by the test on every `npm run test:cref`:",
				"",
				"- every output line before the former boundary is identical;",
				"- a scenario without a SUPPLY selection has the identical result line;",
				"- a SUPPLY selection ends at `assign_task_to_group_members` for the same group,",
				"  with the same task (and no other) ASSIGNED;",
				"- TypeScript's whole output equals the C's.",
				"",
				"## Summary",
				"",
				"| Classification | Hand matrix | Random fixture |",
				"|---|---|---|",
				...classes.map((c) => `| ${c} | ${count((m) => m.classification === c && m.entry.corpus === "hand")} | ${count((m) => m.classification === c && m.entry.corpus === "random")} |`),
				"",
				"The random fixture is split: the SUPPLY transactions are",
				"test/scenarios/generated/c-reference-migrated-supply-task-transaction.cases.ts;",
				"the rest stay the Slice 6a selection corpus,",
				"test/scenarios/generated/c-reference-random-supply-task-assignment.cases.ts.",
				"",
				"## Scenarios",
				"",
				"| Scenario | Classification | Corrections | Slice 6a result | Slice 6b result |",
				"|---|---|---|---|---|",
				...migrations.map(
					(m) =>
						`| ${m.entry.corpus} ${m.entry.id} | ${m.classification} | ${m.corrections.join("; ")} | \`${m.entry.output[resultIndex(m.entry.output)]}\` | \`${m.newResult}\` |`,
				),
				"",
			];

			writeFileSync(join(__dirname, "../../docs/slices/supply-task-assignment-fixture-migration.md"), lines.join("\n"));
		});
	}
});

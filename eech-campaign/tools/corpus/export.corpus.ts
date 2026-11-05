//
// Exports the eech-core-ts scenario corpus for the native module's
// differential tests (eech-campaign/corpus/*.jsonl.gz).
//
// Each entry is one scenario in the C reference harness's input language,
// with the output of the ORIGINAL C (the canonical 32-bit C reference,
// executed now) and the TSTL port's verdict on the same scenario:
//
//   { "family", "id", "input", "c": [lines], "ts": "agrees" | "differs" | "n/a" }
//
// The native module must reproduce "c" line for line. "ts" records whether
// the TS port (the independent implementation) agrees with the C on that
// scenario, so a three-way comparison needs no Node at `cargo test` time.
//
// Sources: the hand-derived matrices, the recorded random fixtures, and
// (EECH_CORPUS_FRESH=N) N fresh random scenarios per generator.
//

import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import { it } from "vitest";

// @ts-expect-error plain ESM build script without type declarations
import { buildHarness } from "../../../eech-core-ts/c-reference/build.mjs";
import { floatFromBits, formatNumberForC, runCInput, runCTimeline } from "../../../eech-core-ts/test/c-reference/c-harness";
import {
	generateCrateRowOperands,
	generateDoubleSumOperands,
	generateFloat32RtzOperands,
	generateRandomForceLowOnSupplies,
	generateRandomKeysiteCargo,
	generateRandomLifecycles,
	generateRandomScenarios,
	generateRandomSupplyTaskAssignment,
	generateRandomSupplyTaskConstruction,
	generateRandomTimelines,
} from "../../../eech-core-ts/test/c-reference/random-scenarios";
import { ASSESS_GROUP_SUPPLIES_CASES } from "../../../eech-core-ts/test/scenarios/assess-group-supplies.cases";
import { runScenario, serialiseScenario, type ScenarioSpec } from "../../../eech-core-ts/test/scenarios/campaign-scenario";
import { ENTITY_LIFECYCLE_CASES } from "../../../eech-core-ts/test/scenarios/entity-lifecycle.cases";
import { float32Hex } from "../../../eech-core-ts/test/scenarios/float-bits";
import { applyFloat32RtzOp, type Float32RtzOp } from "../../../eech-core-ts/test/scenarios/float32-rtz";
import { FORCE_LOW_ON_SUPPLIES_CASES } from "../../../eech-core-ts/test/scenarios/force-low-on-supplies.cases";
import { KEYSITE_CARGO_CASES } from "../../../eech-core-ts/test/scenarios/keysite-cargo.cases";
import { runLifecycle, serialiseLifecycle, type LifecycleSpec } from "../../../eech-core-ts/test/scenarios/lifecycle-scenario";
import { SUPPLY_TASK_ASSIGNMENT_CASES } from "../../../eech-core-ts/test/scenarios/supply-task-assignment.cases";
import { SUPPLY_TASK_CONSTRUCTION_CASES } from "../../../eech-core-ts/test/scenarios/supply-task-construction.cases";
import { UPDATE_TIMELINE_CASES } from "../../../eech-core-ts/test/scenarios/update-timeline.cases";
import { runTimeline, serialiseTimeline, type TimelineSpec } from "../../../eech-core-ts/test/scenarios/update-timeline";
import { C_REFERENCE_CRATE_ROW_CASES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-crate-row.cases";
import { C_REFERENCE_DOUBLE_SUM_RTZ_CASES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-double-sum-rtz.cases";
import { C_REFERENCE_FLOAT32_RTZ_CASES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-float32-rtz.cases";
import { C_REFERENCE_RANDOM_CASES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random.cases";
import { C_REFERENCE_RANDOM_FORCE_LOW_ON_SUPPLIES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random-force-low-on-supplies.cases";
import { C_REFERENCE_RANDOM_KEYSITE_CARGO } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random-keysite-cargo.cases";
import { C_REFERENCE_RANDOM_LIFECYCLES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random-lifecycles.cases";
import { C_REFERENCE_RANDOM_SUPPLY_TASK_ASSIGNMENT } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random-supply-task-assignment.cases";
import { C_REFERENCE_RANDOM_SUPPLY_TASK_CONSTRUCTION } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random-supply-task-construction.cases";
import { C_REFERENCE_RANDOM_TIMELINES } from "../../../eech-core-ts/test/scenarios/generated/c-reference-random-timelines.cases";

const here = dirname(fileURLToPath(import.meta.url));
const corpusDir = join(here, "../../corpus");

type Ts = "agrees" | "differs" | "n/a";

interface Entry {
	family: string;
	id: string;
	input: string;
	c: string[];
	ts: Ts;
}

let harness = "";

function runC(input: string): string[] {
	const run = spawnSync(harness, [], { input, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 });
	if (run.status !== 0) {
		throw new Error(`C reference failed (${run.signal ?? run.status}): ${run.stderr}\n${input}`);
	}
	const lines = run.stdout.split("\n");
	if (lines[lines.length - 1] === "") lines.pop();
	return lines;
}

function tsVerdict(f: () => boolean): Ts {
	try {
		return f() ? "agrees" : "differs";
	} catch {
		return "differs";
	}
}

function lifecycleEntry(family: string, id: string, spec: LifecycleSpec): Entry {
	const input = serialiseLifecycle(spec, formatNumberForC);
	const c = runC(input);
	return { family, id, input, c, ts: tsVerdict(() => JSON.stringify(runLifecycle(spec)) === JSON.stringify(c)) };
}

function scenarioEntry(family: string, id: string, spec: ScenarioSpec): Entry {
	const input = serialiseScenario(spec, formatNumberForC);
	const c = runC(input);
	return { family, id, input, c, ts: tsVerdict(() => JSON.stringify(runScenario(spec)) === JSON.stringify(runCInput(input))) };
}

function timelineEntry(family: string, id: string, spec: TimelineSpec): Entry {
	const input = serialiseTimeline(spec, formatNumberForC);
	const c = runC(input);
	return { family, id, input, c, ts: tsVerdict(() => JSON.stringify(runTimeline(spec)) === JSON.stringify(runCTimeline(spec))) };
}

const fmt = (n: number): string => (Object.is(n, -0) ? "-0" : String(n));

function float32Entries(family: string, ops: [Float32RtzOp | string, number, number][]): Entry[] {
	const out: Entry[] = [];
	for (let start = 0; start < ops.length; start += 500) {
		const batch = ops.slice(start, start + 500);
		const input = batch.map(([op, a, b]) => `f32 ${op} ${fmt(a)}${op === "narrow" || op === "sqrt" ? "" : ` ${fmt(b)}`}`).join("\n") + "\n";
		const c = runC(input);
		const ts = tsVerdict(() =>
			batch.every(([op, a, b], i) => {
				if (op === "dsum") return true; // double results are checked by eech-core-ts itself
				const value = applyFloat32RtzOp(op as Float32RtzOp, a, b);
				// NaN payloads are not modelled by the TS port (x86 produces the negative default NaN)
				if (Number.isNaN(value) && Number.isNaN(floatFromBits(c[i].split(" ")[1]))) return true;
				return `f32 ${float32Hex(value)}` === c[i];
			}),
		);
		out.push({ family, id: `${family}-${start}`, input, c, ts });
	}
	return out;
}

function crateRowEntries(family: string, ops: [number, number, number][]): Entry[] {
	const out: Entry[] = [];
	for (let start = 0; start < ops.length; start += 500) {
		const batch = ops.slice(start, start + 500);
		const input = batch.map(([x, a, b]) => `f32 crate-row ${fmt(x)} ${fmt(a)} ${fmt(b)}`).join("\n") + "\n";
		out.push({ family, id: `${family}-${start}`, input, c: runC(input), ts: "n/a" });
	}
	return out;
}

function write(name: string, entries: Entry[]): void {
	const text = entries.map((e) => JSON.stringify(e)).join("\n") + "\n";
	writeFileSync(join(corpusDir, `${name}.jsonl.gz`), gzipSync(text, { level: 9 }));
	const differs = entries.filter((e) => e.ts === "differs").length;
	console.log(`${name}: ${entries.length} scenarios, TS differs on ${differs}`);
}

it("exports the corpus", () => {
	harness = buildHarness();
	mkdirSync(corpusDir, { recursive: true });

	const fresh = Number(process.env.EECH_CORPUS_FRESH ?? "0");
	const suffix = fresh > 0 ? "-fresh" : "";

	if (fresh === 0) {
		const recorded: Entry[] = [
			...ASSESS_GROUP_SUPPLIES_CASES.map((k) => scenarioEntry("assess-group-supplies", k.id, k.spec)),
			...C_REFERENCE_RANDOM_CASES.map((k) => scenarioEntry("assess-group-supplies", k.id, k.spec)),
			...UPDATE_TIMELINE_CASES.map((k) => timelineEntry("update-timeline", k.id, k.spec)),
			...C_REFERENCE_RANDOM_TIMELINES.map((k) => timelineEntry("update-timeline", k.id, k.spec)),
			...ENTITY_LIFECYCLE_CASES.map((k) => lifecycleEntry("entity-lifecycle", k.id, k.spec)),
			...C_REFERENCE_RANDOM_LIFECYCLES.map((k) => lifecycleEntry("entity-lifecycle", k.id, k.spec)),
			...KEYSITE_CARGO_CASES.map((k) => lifecycleEntry("keysite-cargo", k.id, k.spec)),
			...C_REFERENCE_RANDOM_KEYSITE_CARGO.map((k) => lifecycleEntry("keysite-cargo", k.id, k.spec)),
			...FORCE_LOW_ON_SUPPLIES_CASES.map((k) => lifecycleEntry("force-low-on-supplies", k.id, k.spec)),
			...C_REFERENCE_RANDOM_FORCE_LOW_ON_SUPPLIES.map((k) => lifecycleEntry("force-low-on-supplies", k.id, k.spec)),
			...SUPPLY_TASK_CONSTRUCTION_CASES.map((k) => lifecycleEntry("supply-task-construction", k.id, k.spec)),
			...C_REFERENCE_RANDOM_SUPPLY_TASK_CONSTRUCTION.map((k) => lifecycleEntry("supply-task-construction", k.id, k.spec)),
			...SUPPLY_TASK_ASSIGNMENT_CASES.map((k) => lifecycleEntry("supply-task-assignment", k.id, k.spec)),
			...C_REFERENCE_RANDOM_SUPPLY_TASK_ASSIGNMENT.map((k) => lifecycleEntry("supply-task-assignment", k.id, k.spec)),
		];
		write("scenarios", recorded);
		write("float32", [
			...float32Entries("float32-rtz", C_REFERENCE_FLOAT32_RTZ_CASES.map(([op, a, b]) => [op, a, b])),
			...float32Entries("double-sum-rtz", C_REFERENCE_DOUBLE_SUM_RTZ_CASES.map(([op, a, b]) => [op, a, b])),
			...crateRowEntries("crate-row", C_REFERENCE_CRATE_ROW_CASES.map(([x, a, b]) => [x, a, b])),
			{ family: "aircraft-database", id: "aircraft-cruise-velocity", input: "aircraft-cruise-velocity\n", c: runC("aircraft-cruise-velocity\n"), ts: "n/a" },
		]);
	} else {
		const seed = Number(process.env.EECH_CORPUS_SEED ?? "20260923");
		const entries: Entry[] = [
			...generateRandomScenarios(seed, fresh).map((s, i) => scenarioEntry("assess-group-supplies", `fresh-${seed}-${i}`, s)),
			...generateRandomTimelines(seed, fresh).map((s, i) => timelineEntry("update-timeline", `fresh-${seed}-${i}`, s)),
			...generateRandomLifecycles(seed, fresh).map((s, i) => lifecycleEntry("entity-lifecycle", `fresh-${seed}-${i}`, s)),
			...generateRandomKeysiteCargo(seed, fresh).map((s, i) => lifecycleEntry("keysite-cargo", `fresh-${seed}-${i}`, s)),
			...generateRandomForceLowOnSupplies(seed, fresh).map((s, i) => lifecycleEntry("force-low-on-supplies", `fresh-${seed}-${i}`, s)),
			...generateRandomSupplyTaskConstruction(seed, fresh).map((s, i) => lifecycleEntry("supply-task-construction", `fresh-${seed}-${i}`, s)),
			...generateRandomSupplyTaskAssignment(seed, fresh).map((s, i) => lifecycleEntry("supply-task-assignment", `fresh-${seed}-${i}`, s)),
		];
		write(`scenarios${suffix}`, entries);
		write(`float32${suffix}`, [
			...float32Entries("float32-rtz", generateFloat32RtzOperands(seed, fresh * 20)),
			...float32Entries("double-sum-rtz", generateDoubleSumOperands(seed, fresh * 4)),
			...crateRowEntries("crate-row", generateCrateRowOperands(seed, fresh * 4)),
		]);
	}
});

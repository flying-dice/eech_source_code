//
// Issue #7, RTZ migration: every change the migration made to a recorded
// fixture, explained.
//
// For each recorded case whose expectation changed between RTZ_BASE (the last
// round-to-nearest fixtures) and the working tree, this test requires:
//   1. the old expectation is exactly what the former canonical oracle
//      (variant `sse-rn`: SSE at declared type, round to nearest) produces;
//   2. the new expectation is exactly what the canonical oracle produces
//      (also checked by test/c-reference/differential.cref.test.ts);
//   3. every changed field belongs to a documented difference class
//      (docs/fidelity/fpu-semantics.md, §3 and §5.1):
//        supply  - assess_group_supplies arithmetic and its consequences (#1-#4)
//        range   - get_2d_range / get_approx_2d_range and the closest-keysite
//                  decisions that read them (#5, #6)
//        timer   - update timers, sub-step delta and update-list membership
//                  that expiry decides (#7, #9)
// and that the Slice 3 lifecycle fixture is byte-identical.
//
// Writes build/fpu-spike/fixture-deltas.json.
//

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import type { ScenarioOutcome, ScenarioSpec } from "../scenarios/campaign-scenario";
import { serialiseScenario } from "../scenarios/campaign-scenario";
import type { TimelineOutcome, TimelineSpec } from "../scenarios/update-timeline";
import { formatNumberForC, runCInput, runCTimeline } from "../c-reference/c-harness";
import { variantBinary } from "./run";

const RTZ_BASE = "5d4e3ec";

const GENERATED = "test/scenarios/generated";

interface Recorded<S, O> {
	id: string;
	spec: S;
	expected: O;
}

function parseFixture<T>(text: string): T[] {
	const start = text.indexOf("= [");
	const end = text.lastIndexOf("];");
	return JSON.parse(text.slice(start + 2, end + 1)) as T[];
}

function atBase(file: string): string {
	return execFileSync("git", ["show", `${RTZ_BASE}:eech-core-ts/${file}`], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

// the paths at which two JSON values differ
function diffPaths(a: unknown, b: unknown, path: string, out: string[]): void {
	if (typeof a === "object" && a !== null && typeof b === "object" && b !== null) {
		const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
		for (const k of keys) {
			diffPaths((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k], `${path}.${k}`, out);
		}
		return;
	}
	if (a !== b) {
		out.push(path);
	}
}

function classifyScenario(path: string): string {
	if (/^\.(transmissions|groupAmmo|groupFuel|keysiteAmmo|keysiteFuel|messages)/.test(path)) return "supply";
	if (/^\.(closest|closestRange)$/.test(path)) return "range";
	return "UNCLASSIFIED";
}

function classifyTimeline(path: string): string {
	if (/^\.steps\.\d+\.(groups\.\d+\.(sleep|assist)|updateList(\.\d+)?|delta)$/.test(path)) return "timer";
	if (/^\.transmissions/.test(path)) return "timer";
	return "UNCLASSIFIED";
}

interface Delta {
	id: string;
	classes: string[];
	paths: string[];
}

describe("RTZ migration: recorded fixture deltas", () => {
	it("explains every changed expectation", () => {
		const deltas: Delta[] = [];

		const scenariosOld = parseFixture<Recorded<ScenarioSpec, ScenarioOutcome>>(atBase(`${GENERATED}/c-reference-random.cases.ts`));
		const scenariosNew = parseFixture<Recorded<ScenarioSpec, ScenarioOutcome>>(readFileSync(`${GENERATED}/c-reference-random.cases.ts`, "utf8"));
		expect(scenariosNew.map((c) => JSON.stringify(c.spec))).toEqual(scenariosOld.map((c) => JSON.stringify(c.spec)));

		scenariosOld.forEach((old, i) => {
			const now = scenariosNew[i];
			const paths: string[] = [];
			diffPaths(old.expected, now.expected, "", paths);
			if (paths.length === 0) return;
			const input = serialiseScenario(old.spec, formatNumberForC);
			expect(JSON.parse(JSON.stringify(runCInput(input, variantBinary("sse-rn")))), `${old.id} former oracle`).toEqual(old.expected);
			expect(JSON.parse(JSON.stringify(runCInput(input))), `${old.id} canonical oracle`).toEqual(now.expected);
			deltas.push({ id: old.id, classes: [...new Set(paths.map(classifyScenario))], paths });
		});

		const timelinesOld = parseFixture<Recorded<TimelineSpec, TimelineOutcome>>(atBase(`${GENERATED}/c-reference-random-timelines.cases.ts`));
		const timelinesNew = parseFixture<Recorded<TimelineSpec, TimelineOutcome>>(readFileSync(`${GENERATED}/c-reference-random-timelines.cases.ts`, "utf8"));
		expect(timelinesNew.map((c) => JSON.stringify(c.spec))).toEqual(timelinesOld.map((c) => JSON.stringify(c.spec)));

		timelinesOld.forEach((old, i) => {
			const now = timelinesNew[i];
			const paths: string[] = [];
			diffPaths(old.expected, now.expected, "", paths);
			if (paths.length === 0) return;
			expect(JSON.parse(JSON.stringify(runCTimeline(old.spec, variantBinary("sse-rn")))), `${old.id} former oracle`).toEqual(old.expected);
			expect(JSON.parse(JSON.stringify(runCTimeline(old.spec))), `${old.id} canonical oracle`).toEqual(now.expected);
			deltas.push({ id: old.id, classes: [...new Set(paths.map(classifyTimeline))], paths });
		});

		// Slice 3 is byte-identical
		const lifecycles = `${GENERATED}/c-reference-random-lifecycles.cases.ts`;
		expect(readFileSync(lifecycles, "utf8")).toBe(atBase(lifecycles));

		mkdirSync("build/fpu-spike", { recursive: true });
		writeFileSync("build/fpu-spike/fixture-deltas.json", JSON.stringify(deltas, null, "\t"));

		const unclassified = deltas.filter((d) => d.classes.includes("UNCLASSIFIED"));
		expect(unclassified).toEqual([]);
		expect(deltas.length).toBeGreaterThan(0);
	});
});

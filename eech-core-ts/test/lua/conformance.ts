//
// Lua 5.1 conformance runner.
//
// Transpiled by TSTL together with src/ and executed by a real Lua 5.1
// interpreter (scripts/run-lua.mjs). It replays the shared behaviour matrix
// (expectations verified against the original C) and the float32 edge cases,
// so the port is checked under Lua semantics - integer/float formatting, 0 is
// truthy, 1-based tables, pcall-based exceptions - not only JavaScript.
//

import { toFloat32 } from "../../src/core/float32";
import { ASSESS_GROUP_SUPPLIES_CASES } from "../scenarios/assess-group-supplies.cases";
import { runScenario } from "../scenarios/campaign-scenario";
import { FLOAT32_EDGE_CASES } from "../scenarios/float32.cases";
import { C_REFERENCE_RANDOM_CASES } from "../scenarios/generated/c-reference-random.cases";
import { C_REFERENCE_RANDOM_TIMELINES } from "../scenarios/generated/c-reference-random-timelines.cases";
import { UPDATE_TIMELINE_CASES } from "../scenarios/update-timeline.cases";
import { ENTITY_LIFECYCLE_CASES } from "../scenarios/entity-lifecycle.cases";
import { C_REFERENCE_RANDOM_LIFECYCLES } from "../scenarios/generated/c-reference-random-lifecycles.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { runTimeline } from "../scenarios/update-timeline";

declare const _VERSION: string;

function sameNumber(a: number, b: number): boolean {
	if (a !== a) {
		return b !== b;
	}
	// distinguishes 0 and -0
	return a === b && (a !== 0 || 1 / a === 1 / b);
}

function describe(value: unknown): string {
	if (typeof value === "object" && value !== null) {
		const parts: string[] = [];
		for (const key in value as Record<string, unknown>) {
			parts.push(`${key}=${describe((value as Record<string, unknown>)[key])}`);
		}
		parts.sort();
		return `{${parts.join(",")}}`;
	}
	if (typeof value === "number") {
		return string.format("%.17g", value);
	}
	return `${value as string}`;
}

function deepEqual(a: unknown, b: unknown): boolean {
	if (typeof a === "number" && typeof b === "number") {
		return sameNumber(a, b);
	}
	if (typeof a === "object" && typeof b === "object" && a !== null && b !== null) {
		const ra = a as Record<string, unknown>;
		const rb = b as Record<string, unknown>;
		for (const key in ra) {
			if (!deepEqual(ra[key], rb[key])) {
				return false;
			}
		}
		for (const key in rb) {
			if (!(key in ra)) {
				return false;
			}
		}
		return true;
	}
	return a === b;
}

let failures = 0;
let passes = 0;

function check(name: string, actual: unknown, expected: unknown): void {
	if (deepEqual(actual, expected)) {
		passes += 1;
	} else {
		failures += 1;
		print(`FAIL ${name}`);
		print(`  expected ${describe(expected)}`);
		print(`  actual   ${describe(actual)}`);
	}
}

if (_VERSION !== "Lua 5.1") {
	print(`expected Lua 5.1, running ${_VERSION}`);
	os.exit(1);
}

for (const [input, expected] of FLOAT32_EDGE_CASES) {
	check(`toFloat32(${string.format("%.17g", input)})`, toFloat32(input), expected);
}

for (const c of ASSESS_GROUP_SUPPLIES_CASES) {
	check(c.id, runScenario(c.spec), c.expected);
}

for (const c of C_REFERENCE_RANDOM_CASES) {
	check(c.id, runScenario(c.spec), c.expected);
}

for (const c of UPDATE_TIMELINE_CASES) {
	check(c.id, runTimeline(c.spec), c.expected);
}

for (const c of C_REFERENCE_RANDOM_TIMELINES) {
	check(c.id, runTimeline(c.spec), c.expected);
}

for (const c of ENTITY_LIFECYCLE_CASES) {
	check(c.id, firstUnmatchedLine(runLifecycle(c.spec), c.expected), "");
}

for (const c of C_REFERENCE_RANDOM_LIFECYCLES) {
	check(c.id, runLifecycle(c.spec), c.expected);
}

print(`${_VERSION}: ${passes} passed, ${failures} failed`);

os.exit(failures === 0 ? 0 : 1);

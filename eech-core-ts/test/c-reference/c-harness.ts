//
// Runs scenarios through the original EECH C (c-reference/harness.c) and
// converts its output into a ScenarioOutcome comparable with the TS port.
//

import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { existsSync } from "node:fs";
// @ts-expect-error plain ESM build script without type declarations
import { HARNESS_BINARY } from "../../c-reference/build.mjs";
import { serialiseScenario, type ScenarioOutcome, type ScenarioSpec } from "../scenarios/campaign-scenario";
import { serialiseLifecycle, type LifecycleSpec } from "../scenarios/lifecycle-scenario";
import { serialiseTimeline, type TimelineOutcome, type TimelineSpec, type TimelineStepState } from "../scenarios/update-timeline";

// Built once by test/c-reference/global-setup.ts; never rebuilt from a worker.
function harness(): string {
	if (!existsSync(HARNESS_BINARY as string)) {
		throw new Error(`C harness ${HARNESS_BINARY as string} is missing; run through vitest.cref.config.ts (global setup builds it)`);
	}
	return HARNESS_BINARY as string;
}

// A run that did not exit normally with status 0 is never an outcome: in
// particular a process killed by a signal (e.g. a SIGSEGV outside the NULL page,
// which the harness deliberately does not handle) is a harness failure.
function describeFailure(run: SpawnSyncReturns<string>): string {
	if (run.error) {
		return run.error.message;
	}
	if (run.signal !== null) {
		return `killed by ${run.signal}: ${run.stderr}`;
	}
	return `exit ${run.status}: ${run.stderr}`;
}

export function formatNumberForC(n: number): string {
	// shortest round-trip representation; strtod reads it back exactly
	return String(n);
}

export function doubleFromBits(hex: string): number {
	const view = new DataView(new ArrayBuffer(8));
	view.setUint32(0, Number.parseInt(hex.substring(0, 8), 16));
	view.setUint32(4, Number.parseInt(hex.substring(8), 16));
	return view.getFloat64(0);
}

export function floatFromBits(hex: string): number {
	const view = new DataView(new ArrayBuffer(4));
	view.setUint32(0, Number.parseInt(hex, 16));
	return view.getFloat32(0);
}

// A lifecycle scenario through the original C: the output lines, as runLifecycle () produces them.
export function runCLifecycle(spec: LifecycleSpec): string[] {
	const input = serialiseLifecycle(spec, formatNumberForC);

	const run = spawnSync(harness(), [], { input, encoding: "utf8" });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${describeFailure(run)})\ninput:\n${input}`);
	}

	const lines = run.stdout.split("\n");

	if (lines[lines.length - 1] === "") {
		lines.pop();
	}

	return lines;
}

export function runCScenario(spec: ScenarioSpec): ScenarioOutcome {
	return runCInput(serialiseScenario(spec, formatNumberForC));
}

// `binary`: an investigation variant of the harness (test/fpu-spike); the canonical oracle by default
export function runCInput(input: string, binary: string = harness()): ScenarioOutcome {
	const run = spawnSync(binary, [], { input, encoding: "utf8" });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${describeFailure(run)})\ninput:\n${input}`);
	}

	const outcome: ScenarioOutcome = {
		result: "",
		closest: "",
		closestRange: undefined,
		messages: [],
		transmissions: [],
		groupAmmo: 0,
		groupFuel: 0,
		keysiteAmmo: [],
		keysiteFuel: [],
	};

	for (const line of run.stdout.split("\n")) {
		const w = line.split(" ");

		if (w[0] === "transmit") {
			outcome.transmissions.push({ entity: w[1], floatType: Number(w[2]), value: floatFromBits(w[3]) });
		} else if (w[0] === "message") {
			outcome.messages.push({ receiver: w[1], sender: w[2], message: Number(w[3]), arg: Number(w[4]) });
		} else if (w[0] === "closest") {
			outcome.closest = w[1];
			outcome.closestRange = w[2] === "-" ? undefined : floatFromBits(w[2]);
		} else if (w[0] === "result") {
			outcome.result = w[1] === "assert" ? `assert:${w.slice(2).join(" ")}` : w[1];
		} else if (w[0] === "final" && w[1] === "group") {
			outcome.groupAmmo = floatFromBits(w[2]);
			outcome.groupFuel = floatFromBits(w[3]);
		} else if (w[0] === "final" && w[1] === "keysite") {
			outcome.keysiteAmmo.push(floatFromBits(w[2]));
			outcome.keysiteFuel.push(floatFromBits(w[3]));
		} else if (line !== "") {
			throw new Error(`unexpected C harness output: ${line}`);
		}
	}

	return outcome;
}

export function runCRange(x1: number, z1: number, x2: number, z2: number): { range: number; approx: number } {
	const input = `op range ${[x1, z1, x2, z2].map(formatNumberForC).join(" ")}\n`;
	const run = spawnSync(harness(), [], { input, encoding: "utf8" });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${describeFailure(run)})`);
	}

	const w = run.stdout.split("\n")[0].split(" ");

	return { range: floatFromBits(w[1]), approx: floatFromBits(w[2]) };
}

export function runCTimeline(spec: TimelineSpec, binary: string = harness()): TimelineOutcome {
	const input = serialiseTimeline(spec, formatNumberForC);
	const run = spawnSync(binary, [], { input, encoding: "utf8" });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${describeFailure(run)})\ninput:\n${input}`);
	}

	const outcome: TimelineOutcome = { result: "", transmissions: [], steps: [] };
	let current: TimelineStepState | undefined;

	for (const line of run.stdout.split("\n")) {
		const w = line.split(" ");

		if (w[0] === "transmit") {
			outcome.transmissions.push({ step: outcome.steps.length, entity: w[1], floatType: Number(w[2]), value: floatFromBits(w[3]) });
		} else if (w[0] === "step") {
			current = { delta: floatFromBits(w[2]), updateList: w[3] === "-" ? [] : w[3].split(","), groups: [] };
			outcome.steps.push(current);
		} else if (w[0] === "timer") {
			current?.groups.push({ sleep: floatFromBits(w[1]), assist: floatFromBits(w[2]) });
		} else if (w[0] === "result") {
			outcome.result = w[1] === "assert" ? `assert:${w.slice(2).join(" ")}` : w[1];
		} else if (line !== "") {
			throw new Error(`unexpected C harness output: ${line}`);
		}
	}

	return outcome;
}

// Float operations through the harness `f32` command (canonical environment),
// one process for all of them: the result bit patterns, in order.
export function runCFloat32(ops: [string, number, number][]): string[] {
	const fmt = (n: number): string => (Object.is(n, -0) ? "-0" : String(n));
	const input = ops.map(([op, a, b]) => `f32 ${op} ${fmt(a)}${op === "narrow" || op === "sqrt" ? "" : ` ${fmt(b)}`}`).join("\n") + "\n";
	// dsum results are double bit patterns (16 hex digits), the others float (8)
	const run = spawnSync(harness(), [], { input, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${describeFailure(run)})`);
	}

	const lines = run.stdout.split("\n").filter((l) => l !== "");

	return lines.map((l) => l.split(" ")[1]);
}

// keysite.c's crate-row step through the harness `f32 crate-row` command
// (canonical environment), one process for all: the stored x as float bits.
export function runCCrateRow(operands: [number, number, number][]): string[] {
	const fmt = (n: number): string => (Object.is(n, -0) ? "-0" : String(n));
	const input = operands.map(([x, xmin, xmax]) => `f32 crate-row ${fmt(x)} ${fmt(xmin)} ${fmt(xmax)}`).join("\n") + "\n";
	const run = spawnSync(harness(), [], { input, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${describeFailure(run)})`);
	}

	return run.stdout
		.split("\n")
		.filter((l) => l !== "")
		.map((l) => l.split(" ")[1]);
}

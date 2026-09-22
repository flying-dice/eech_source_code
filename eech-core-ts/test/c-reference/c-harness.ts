//
// Runs scenarios through the original EECH C (c-reference/harness.c) and
// converts its output into a ScenarioOutcome comparable with the TS port.
//

import { spawnSync } from "node:child_process";
// @ts-expect-error plain ESM build script without type declarations
import { buildHarness } from "../../c-reference/build.mjs";
import { serialiseScenario, type ScenarioOutcome, type ScenarioSpec } from "../scenarios/campaign-scenario";

let harnessPath: string | undefined;

function harness(): string {
	if (harnessPath === undefined) {
		harnessPath = buildHarness() as string;
	}
	return harnessPath;
}

export function formatNumberForC(n: number): string {
	// shortest round-trip representation; strtod reads it back exactly
	return String(n);
}

export function floatFromBits(hex: string): number {
	const view = new DataView(new ArrayBuffer(4));
	view.setUint32(0, Number.parseInt(hex, 16));
	return view.getFloat32(0);
}

export function runCScenario(spec: ScenarioSpec): ScenarioOutcome {
	return runCInput(serialiseScenario(spec, formatNumberForC));
}

export function runCInput(input: string): ScenarioOutcome {
	const run = spawnSync(harness(), [], { input, encoding: "utf8" });

	if (run.status !== 0) {
		throw new Error(`C harness failed (${run.status}): ${run.stderr}\ninput:\n${input}`);
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
		throw new Error(`C harness failed (${run.status}): ${run.stderr}`);
	}

	const w = run.stdout.split("\n")[0].split(" ");

	return { range: floatFromBits(w[1]), approx: floatFromBits(w[2]) };
}

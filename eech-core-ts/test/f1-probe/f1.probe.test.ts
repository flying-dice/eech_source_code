//
// INVESTIGATION ONLY (issue #14, F1): prepare.y and finish.y in
// create_supply_task are never initialised. This probe runs the F1 single
// player / multiplayer pair through the otherwise unchanged original C built
// with zero-, pattern- and un-initialised automatic variables in that one
// unit, and reports the heights create_task stored and what multiplayer
// packing made of them. It is the evidence that 0.0 is a choice the port
// makes for undefined behaviour, not the value EECH had
// (docs/slices/supply-task-construction.md, F1).
//

import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { serialiseLifecycle } from "../scenarios/lifecycle-scenario";
import { f1CompatibilityPair } from "../scenarios/supply-task-construction.cases";
import { formatNumberForC, runCLifecycle } from "../c-reference/c-harness";
import { F1_VARIANTS } from "./variants";

function run(binary: string, input: string): string[] {
	const result = spawnSync(binary, [], { input, encoding: "utf8" });
	const lines = result.stdout.split("\n");
	if (lines[lines.length - 1] === "") {
		lines.pop();
	}
	if (result.status !== 0 || result.signal !== null) {
		lines.push(`exit ${result.status} ${result.signal ?? ""}`);
	}
	return lines;
}

// the stored heights of the prepare (node 1) and finish (node 3) points
function heights(output: string[]): string {
	const route = output.find((line) => line.startsWith("route "));
	if (route === undefined) {
		return "- -";
	}
	const fields = route.split(" ");
	return `${fields[2 + 6 + 1]} ${fields[2 + 18 + 1]}`;
}

function result(output: string[]): string {
	const line = output.find((l) => l.startsWith("result ") || l.startsWith("exit "));
	return line === undefined ? "none" : line;
}

describe("F1: uninitialised prepare.y / finish.y", () => {
	const pair = f1CompatibilityPair();
	const sp = serialiseLifecycle(pair.singlePlayer, formatNumberForC);
	const mp = serialiseLifecycle(pair.multiplayer, formatNumberForC);

	const rows: string[] = ["| build | prepare.y, finish.y (single player) | single player | multiplayer |", "|---|---|---|---|"];

	for (const variant of F1_VARIANTS) {
		it(variant.name, () => {
			const binary = join("build", "c-reference-f1", variant.name, "harness");
			const spOut = run(binary, sp);
			const mpOut = run(binary, mp);

			rows.push(`| ${variant.name} | ${heights(spOut)} | ${result(spOut)} | ${result(mpOut)} |`);

			if (variant.name === "zero-O0") {
				// the control: the canonical oracle's pinning
				expect(spOut).toEqual(runCLifecycle(pair.singlePlayer));
				expect(mpOut).toEqual(runCLifecycle(pair.multiplayer));
			}

			if (variant.name.startsWith("zero")) {
				expect(heights(spOut)).toBe("00000000 00000000");
			}

			// single player never packs, so it completes whatever the heights are
			expect(result(spOut)).toBe("result ok");
		});
	}

	it("report", () => {
		process.stderr.write(`\n${rows.join("\n")}\n`);
	});
});

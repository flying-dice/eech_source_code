//
// INVESTIGATION ONLY (issue #7), Investigation 5: adversarial numerical inputs
// aimed at each float operation of Slices 1-3 (docs/fidelity/fpu-semantics.md,
// "Sensitivity inventory"). Kept apart from the frozen fixtures: nothing here
// is recorded into test/scenarios/generated.
//
// Writes build/fpu-spike/adversarial.json.
//

import { mkdirSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { differential, runCanonical, type CorpusResult } from "./run";

const f32 = new Float32Array(1);
const u32 = new Uint32Array(f32.buffer);

function fround(x: number): number {
	f32[0] = x;
	return f32[0];
}

// the adjacent float towards +infinity / -infinity
function nextUp(x: number): number {
	f32[0] = x;
	if (x === 0) {
		u32[0] = 1;
	} else if (x > 0) {
		u32[0] += 1;
	} else {
		u32[0] -= 1;
	}
	return f32[0];
}

function nextDown(x: number): number {
	return -nextUp(-x);
}

// round a double toward zero to float
function rtz32(x: number): number {
	const r = fround(x);
	return Math.abs(r) > Math.abs(x) ? (x > 0 ? nextDown(r) : nextUp(r)) : r;
}

const fmt = (n: number): string => String(n);

function neighbours(x: number): number[] {
	return [nextDown(nextDown(x)), nextDown(x), fround(x), nextUp(x), nextUp(nextUp(x))];
}

// Slice 3: sector index = convert_float_to_int (x) / side; map extent and midpoint
function lifecycleInputs(): string[] {
	const inputs: string[] = [];
	const prefix = ["heap 40", "session 1", "force 1", "keysite 1 8 1 0 0 100 100"];
	const probes = [0.5, 1.5, 2.5, 255.5, 510.5, 511.5, 512.5, 1023.5, 1024.5, ...neighbours(512), ...neighbours(511.5), ...neighbours(1024)];

	// crates at half-integers and ULP neighbours of sector boundaries (side 512)
	for (const x of probes) {
		for (const z of [0.25, 511.5, 767.5]) {
			inputs.push([...prefix, "map 2 2 512", `create c0 4 -1 parent 6 keysite0 vec3d 8 ${fmt(x)} 0 ${fmt(z)} end`, "end"].join("\n") + "\n");
		}
	}

	// map sizes where n * side - 1.0 is not a float (side must be a power of
	// two; the extent is observable through a crate's default position, the
	// map midpoint)
	for (const [n, side] of [
		[1, 16777216],
		[2, 16777216],
		[3, 8388608],
		[5, 4194304],
		[3, 16777216],
		[1, 33554432],
		[2, 8388608],
	]) {
		inputs.push([...prefix, `map ${n} ${n} ${side}`, "create c0 4 -1 parent 6 keysite0 end", "end"].join("\n") + "\n");
	}

	return inputs;
}

// Slice 2: timer expiry and accumulation, update-loop subdivision
function timelineInputs(): string[] {
	const inputs: string[] = [];

	// accumulated timer deltas: sleep and assist timers counted down by repeated deltas
	for (const [sleep, delta] of [
		[1.0, 0.1],
		[0.3, 0.1],
		[0.7, 0.1],
		[1.0, 1 / 3],
		[2.0, 0.2],
		[1.0, 0.01],
		[5.0, 0.3],
		[nextUp(0.3), 0.1],
		[nextDown(0.3), 0.1],
		[0.6, 0.2],
	]) {
		const frames = Math.min(60, Math.ceil(sleep / delta) + 2);
		const lines = ["timeline 1", `tgroup 0 1 ${fmt(fround(sleep))} ${fmt(fround(sleep))} 1`];
		for (let i = 0; i < frames; i++) {
			lines.push(`frame ${fmt(fround(delta))} 0 1`);
		}
		lines.push("end");
		inputs.push(lines.join("\n") + "\n");
	}

	// subdivision: iterations = (int) (delta * rate + 1.0) where the float
	// product rounds differently in each model
	let found = 0;
	for (let rate = 1; rate <= 100 && found < 60; rate++) {
		for (let k = 1; k <= 4 * rate && found < 60; k++) {
			for (const d of neighbours(k / rate)) {
				const exact = d * rate; // exact in double: 24-bit by 7-bit
				const sse = Math.trunc(fround(exact) + 1.0);
				const x87 = Math.trunc(exact + 1.0);
				const rtz = Math.trunc(rtz32(exact) + 1.0);
				if (sse !== x87 || sse !== rtz) {
					found++;
					inputs.push([`timeline ${rate}`, `tgroup 0 1 ${fmt(fround(4 * d))} 0 1`, `frame ${fmt(d)} 0 1`, `frame ${fmt(d)} 0 2`, "end"].join("\n") + "\n");
				}
			}
		}
	}

	return inputs;
}

// Slice 1: supply arithmetic and thresholds; ranges and the closest-keysite threshold
function scenarioInputs(): string[] {
	const inputs: string[] = [];

	const levels = [1e-5, 3e-6, 0.1, 0.3, 33.333, 99.99999, nextDown(100), 99.9, 50.00001, 1e-30, 7e-8];
	for (const ammo of levels) {
		for (const keysiteLevel of [100, 0.3, 1e-5, nextUp(100 - fround(ammo)), fround(100 - fround(ammo)), rtz32(100 - fround(ammo))]) {
			inputs.push(
				[
					"session 1",
					"force 1",
					`keysite 1 1 1 0 0 ${fmt(fround(keysiteLevel))} ${fmt(fround(keysiteLevel))}`,
					`group 22 1 ${fmt(fround(ammo))} ${fmt(fround(ammo))} 1 0 0 0 0 0`,
					"op assess",
				].join("\n") + "\n",
			);
		}
	}

	// ranges with large, inexact coordinates
	const coords = [0.1, 1234.567, -30000.123, 29999.87, 16777.217, 3e4 + 0.0625, nextUp(1e4), 7.77e3, -0.3];
	for (const x1 of coords) {
		for (const z2 of coords) {
			inputs.push(`op range ${fmt(fround(x1))} ${fmt(fround(z2 / 3))} ${fmt(fround(-z2))} ${fmt(fround(x1 / 7))}\n`);
		}
	}

	// get_closest_keysite: min_range at the canonical range and its ULP neighbours
	for (const [kx, kz, px, pz] of [
		[1000.3, 2000.7, 10.1, -3.3],
		[-7027.003, 362.58, 1414.948, -6811.288],
		[12345.678, -9876.543, 1.1, 2.2],
		[0.3, 0.7, 0.1, 0.2],
	]) {
		const canon = runCanonical(`op range ${fmt(fround(kx))} ${fmt(fround(kz))} ${fmt(fround(px))} ${fmt(fround(pz))}\n`).stdout.split(" ");
		f32[0] = 0;
		u32[0] = Number.parseInt(canon[2], 16);
		const approx = f32[0];
		for (const minRange of neighbours(approx)) {
			inputs.push(
				[
					"session 1",
					"force 1",
					`keysite 1 1 1 ${fmt(fround(kx))} ${fmt(fround(kz))} 100 100`,
					`op closest 1 1 1 ${fmt(fround(px))} ${fmt(fround(pz))} ${fmt(minRange)} 1 0 -1`,
				].join("\n") + "\n",
			);
		}
	}

	return inputs;
}

describe("FPU spike: adversarial inputs", () => {
	it("runs every adversarial corpus through every variant", () => {
		const results: CorpusResult[] = [
			differential("adversarial-slice1", scenarioInputs(), 12),
			differential("adversarial-slice2", timelineInputs(), 12),
			differential("adversarial-slice3", lifecycleInputs(), 12),
		];

		mkdirSync("build/fpu-spike", { recursive: true });
		writeFileSync("build/fpu-spike/adversarial.json", JSON.stringify(results, null, "\t"));

		for (const r of results) {
			const summary = Object.entries(r.byVariant).map(([name, v]) => `${name}: ${v.differing}/${r.count} ${JSON.stringify(v.kinds)}`);
			console.log(`${r.corpus}\n  ${summary.join("\n  ")}`);
			expect(r.byVariant["sse-rn"].differing).toBe(0);
		}
	});
});

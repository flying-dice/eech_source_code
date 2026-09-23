//
// Slice 6b: the compiled databases the route and the guide read, and the
// inverse square root, against the original C.
//
//   - src/generated/c-waypoint-database.ts, c-guide-database.ts,
//     c-route-biasing-database.ts, the aircraft cruise altitudes and the task
//     database's route flags equal the C tables as compiled (harness
//     `database-6b`), bit for bit;
//   - invsqrt.c :: get_inverse_square_root (the 512-entry seed table built at
//     start-up, two Newton steps under RTZ) equals the port's for arguments
//     across the whole positive float range, subnormals included.
//

import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";
// @ts-expect-error plain ESM build script without type declarations
import { HARNESS_BINARY } from "../../c-reference/build.mjs";
import { initialiseCampaignCore } from "../../src";
import { getInverseSquareRoot } from "../../src/core/maths/invsqrt";
import { AIRCRAFT_DATABASE_CRUISE_ALTITUDE } from "../../src/generated/c-aircraft-database";
import { GUIDE_DATABASE_CRITERIA } from "../../src/generated/c-guide-database";
import * as R from "../../src/generated/c-route-biasing-database";
import { TASK_DATABASE_ADD_START_WAYPOINT, TASK_DATABASE_ASSESS_LANDING, TASK_DATABASE_TASK_ROUTE_SEARCH } from "../../src/generated/c-task-database";
import * as W from "../../src/generated/c-waypoint-database";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { ScriptedClock } from "../adapters/scripted-clock";
import { float32Hex } from "../scenarios/float-bits";
import { floatFromBits, runCInverseSquareRoot } from "./c-harness";
import { mulberry32 } from "./random-scenarios";

function databaseDump(): string[] {
	const run = spawnSync(HARNESS_BINARY as string, [], { input: "database-6b\n", encoding: "utf8" });
	expect(run.status).toBe(0);
	return run.stdout.split("\n").filter((line) => line !== "");
}

function portDump(): string[] {
	const lines: string[] = [];
	const block = (name: string, w: number, b: number): string =>
		` ${name} ${float32Hex(W.WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE[w][b])} ${float32Hex(W.WAYPOINT_DATABASE_REACHED_RADIUS[w][b])} ${float32Hex(W.WAYPOINT_DATABASE_VELOCITY[w][b])} ` +
		`${W.WAYPOINT_DATABASE_CRITERIA_LAST_TO_REACH[w][b]} ${W.WAYPOINT_DATABASE_CRITERIA_TRANSMIT_RECON[w][b]} ${W.WAYPOINT_DATABASE_POSITION_TYPE[w][b]}`;
	for (let w = 0; w < W.WAYPOINT_DATABASE_GUIDE_SUB_TYPE.length; w++) {
		lines.push(`waypoint ${w} guide ${W.WAYPOINT_DATABASE_GUIDE_SUB_TYPE[w]}${block("fw", w, 0)}${block("hc", w, 1)}${block("rv", w, 2)}${block("sh", w, 3)}`);
	}
	GUIDE_DATABASE_CRITERIA.forEach((criteria, g) => {
		lines.push(`guide ${g}${criteria.map(([valid, value]) => ` ${valid}:${float32Hex(value)}`).join("")}`);
	});
	AIRCRAFT_DATABASE_CRUISE_ALTITUDE.forEach((altitude, a) => lines.push(`aircraft ${a} altitude ${float32Hex(altitude)}`));
	TASK_DATABASE_ADD_START_WAYPOINT.forEach((start, t) => lines.push(`task ${t} start ${start} landing ${TASK_DATABASE_ASSESS_LANDING[t]} search ${TASK_DATABASE_TASK_ROUTE_SEARCH[t]}`));
	R.ROUTE_BIASING_ELEVATION_BIAS.forEach((_e, m) =>
		lines.push(
			`route-biasing ${m} elevation ${float32Hex(R.ROUTE_BIASING_ELEVATION_BIAS[m])} range ${float32Hex(R.ROUTE_BIASING_RANGE_BIAS[m])} side ${float32Hex(R.ROUTE_BIASING_SIDE_BIAS[m])} ` +
				`min-range ${float32Hex(R.ROUTE_BIASING_MIN_ROUTE_RANGE[m])} deviation ${float32Hex(R.ROUTE_BIASING_ROUTE_DEVIATION_SIZE[m])} samples ${float32Hex(R.ROUTE_BIASING_NUM_ROUTE_SAMPLES[m])} ` +
				`tolerance ${float32Hex(R.ROUTE_BIASING_OPTIMISE_TOLERANCE[m])}`,
		),
	);
	return lines;
}

describe("C reference: the route and guide databases and the inverse square root", () => {
	it("the generated waypoint, guide, route biasing, cruise altitude and task route columns are the compiled C tables", () => {
		expect(portDump()).toEqual(databaseDump());
	});

	it("get_inverse_square_root matches the original C bit for bit, subnormal arguments included", () => {
		initialiseCampaignCore({
			mobilePhysicalState: new InMemoryMobilePhysicalState(),
			entityReplication: new RecordingEntityReplication(),
			clock: new ScriptedClock(),
			object3DMetadata: new InMemoryObject3DMetadata(),
			campaignEvents: new RecordingCampaignEvents(),
			terrainElevation: new GridTerrainElevation(),
			roadNetwork: new InMemoryRoadNetwork(),
		});
		const rnd = mulberry32(0x15a7);
		const hex = (n: number): string => n.toString(16).padStart(8, "0");
		const args: string[] = [];
		// every seed-table entry (exponent parity x top 8 mantissa bits), then random positive finite floats
		for (let e = 0; e < 2; e++) {
			for (let m = 0; m < 256; m++) {
				args.push(hex((126 + e) * 0x800000 + m * 0x8000 + Math.floor(rnd() * 0x8000)));
			}
		}
		for (let i = 0; i < 3584; i++) {
			args.push(hex(1 + Math.floor(rnd() * 0x7f7fffff)));
		}
		const c = runCInverseSquareRoot(args);
		expect(c.length).toBe(args.length);
		args.forEach((a, i) => {
			expect(float32Hex(getInverseSquareRoot(floatFromBits(a))), a).toBe(c[i]);
		});
	});
});

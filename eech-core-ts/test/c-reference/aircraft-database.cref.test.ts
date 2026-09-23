//
// Slice 6a: the generated aircraft cruise velocities
// (src/generated/c-aircraft-database.ts, knots_to_metres_per_second folded at
// float precision by scripts/gen-c-sources.mjs) equal the compiled original
// ac_dbase.c aircraft_database bit for bit.
//

import { describe, expect, it } from "vitest";
import { AIRCRAFT_DATABASE_CRUISE_VELOCITY } from "../../src/generated/c-aircraft-database";
import { EntitySubTypeAircraft } from "../../src/generated/c-enums";
import { runCAircraftCruiseVelocity } from "./c-harness";

function floatBits(value: number): string {
	const view = new DataView(new ArrayBuffer(4));
	view.setFloat32(0, value);
	return view.getUint32(0).toString(16).padStart(8, "0");
}

describe("aircraft database cruise velocity matches the compiled C", () => {
	it("every aircraft sub type, bit for bit", () => {
		const c = runCAircraftCruiseVelocity();

		expect(c).toHaveLength(EntitySubTypeAircraft.NUM_ENTITY_SUB_TYPE_AIRCRAFT);
		expect(AIRCRAFT_DATABASE_CRUISE_VELOCITY.map(floatBits)).toEqual(c);
		for (const v of AIRCRAFT_DATABASE_CRUISE_VELOCITY) {
			expect(Math.fround(v)).toBe(v);
		}
	});
});

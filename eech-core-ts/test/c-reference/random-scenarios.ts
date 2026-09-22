//
// Seeded random scenario generator for C-vs-TS differential testing.
// It only produces campaign state and an operation; outcomes always come from
// executing the original C.
//

import { EntitySide, EntitySubTypeGroup, EntitySubTypeKeysite } from "../../src/generated/c-enums";
import type { GroupParentSpec, KeysiteSpec, PositionSpec, ScenarioSpec } from "../scenarios/campaign-scenario";

export function mulberry32(seed: number): () => number {
	let a = seed >>> 0;
	return () => {
		a = (a + 0x6d2b79f5) >>> 0;
		let t = a;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

export function generateRandomScenarios(seed: number, count: number): ScenarioSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const real = (lo: number, hi: number) => Math.round((lo + rnd() * (hi - lo)) * 1000) / 1000;
	const sides = [EntitySide.ENTITY_SIDE_BLUE_FORCE, EntitySide.ENTITY_SIDE_RED_FORCE];
	const side = () => sides[int(2)];
	const coordinate = () => (chance(0.4) ? real(-1500, 1500) : real(-20000, 20000));
	const position = (): PositionSpec => ({ kind: "at", x: coordinate(), z: coordinate() });
	const level = (lo: number, hi: number) => (chance(0.25) ? 100 : chance(0.1) ? 0 : real(lo, hi));

	const scenarios: ScenarioSpec[] = [];

	for (let n = 0; n < count; n++) {
		const forces: EntitySide[] = chance(0.1) ? [] : chance(0.2) ? [side()] : chance(0.5) ? [sides[0], sides[1]] : [sides[1], sides[0]];

		const keysites: KeysiteSpec[] = [];
		const numKeysites = int(6);
		for (let i = 0; i < numKeysites; i++) {
			keysites.push({
				side: side(),
				subType: int(EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES),
				inUse: chance(0.8),
				x: coordinate(),
				z: coordinate(),
				ammo: chance(0.1) ? 0 : real(-10, 1000),
				fuel: chance(0.1) ? 0 : real(-10, 1000),
			});
		}

		const groupSide = side();
		let parent: GroupParentSpec = { kind: "none" };
		const parentRoll = int(3);
		if (parentRoll === 1 && keysites.length > 0) {
			parent = { kind: "keysite", keysite: int(keysites.length) };
		} else if (parentRoll === 2 && forces.indexOf(groupSide) >= 0) {
			parent = { kind: "independent" };
		}

		const group = {
			subType: int(EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS),
			side: groupSide,
			ammo: level(-20, 120),
			fuel: level(-20, 120),
			parent,
			busy: chance(0.2),
			leader: chance(0.85) ? position() : ({ kind: "none" } as PositionSpec),
		};

		const op: ScenarioSpec["op"] = chance(0.75)
			? { kind: "assess" }
			: {
					kind: "closest",
					type: int(EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES + 1),
					side: side(),
					pos: chance(0.1) ? { kind: "none" } : position(),
					minRange: chance(0.1) ? 0 : real(0, 3000),
					wantRange: chance(0.7),
					outside: chance(0.7),
					exclude: keysites.length > 0 && chance(0.3) ? int(keysites.length) : -1,
			  };

		scenarios.push({ session: chance(0.95), forces, keysites, group, op });
	}

	return scenarios;
}

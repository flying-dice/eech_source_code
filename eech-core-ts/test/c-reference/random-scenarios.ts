//
// Seeded random scenario generator for C-vs-TS differential testing.
// It only produces campaign state and an operation; outcomes always come from
// executing the original C.
//

import { EntitySide, EntitySubTypeGroup, EntitySubTypeKeysite, EntityType, FloatType, IntType, ListType, Vec3dType } from "../../src/generated/c-enums";
import type { GroupParentSpec, KeysiteSpec, PositionSpec, ScenarioSpec } from "../scenarios/campaign-scenario";
import type { LifecycleAttribute, LifecycleOp, LifecycleSpec } from "../scenarios/lifecycle-scenario";
import type { TimelineSpec, TimelineStep } from "../scenarios/update-timeline";

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

//
// Random update timelines: groups restored with or without timers and
// update-list membership, then a random mix of timer sets and frames.
//
export function generateRandomTimelines(seed: number, count: number): TimelineSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const real = (lo: number, hi: number) => Math.round((lo + rnd() * (hi - lo)) * 1000) / 1000;
	const timer = () => (chance(0.35) ? 0 : chance(0.1) ? real(-2, 0) : real(0, 5));

	const timelines: TimelineSpec[] = [];

	for (let n = 0; n < count; n++) {
		const groups = [];
		const numGroups = 1 + int(4);
		for (let i = 0; i < numGroups; i++) {
			groups.push({
				subType: int(EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS),
				side: chance(0.5) ? EntitySide.ENTITY_SIDE_BLUE_FORCE : EntitySide.ENTITY_SIDE_RED_FORCE,
				sleep: timer(),
				assist: timer(),
				onUpdateList: chance(0.5),
			});
		}

		const steps: TimelineStep[] = [];
		const numSteps = 1 + int(10);
		for (let i = 0; i < numSteps; i++) {
			if (chance(0.35)) {
				steps.push({
					kind: "set",
					group: int(numGroups),
					floatType: chance(0.5) ? FloatType.FLOAT_TYPE_SLEEP : FloatType.FLOAT_TYPE_ASSIST_TIMER,
					value: chance(0.2) ? 0 : chance(0.1) ? real(-3, 0) : real(0, 8),
				});
			} else {
				steps.push({
					kind: "frame",
					delta: chance(0.2) ? [0.04, 0.1, 0.25, 0.5, 1][int(5)] : real(0.001, 3),
					locked: chance(0.15),
					count: chance(0.1) ? 0 : 1 + int(3),
				});
			}
		}

		timelines.push({ entityUpdateFrameRate: chance(0.1) ? [0, 101, 1, 100][int(4)] : 1 + int(8), groups, steps });
	}

	return timelines;
}

//
// Random entity lifecycles: a map (usually valid), then creates and destroys
// of cargo with random attribute lists. Positions are multiples of 0.25, so
// they are exact floats whatever the rounding mode (see
// docs/slices/entity-lifecycle-cargo.md). Parents are keysites, NULL, or live
// sectors for the sector list; destroys name created entities, destroyed or not.
//
export function generateRandomLifecycles(seed: number, count: number): LifecycleSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const sides = [EntitySide.ENTITY_SIDE_BLUE_FORCE, EntitySide.ENTITY_SIDE_RED_FORCE];
	const quarter = (lo: number, hi: number) => Math.round((lo + rnd() * (hi - lo)) * 4) / 4;

	const lifecycles: LifecycleSpec[] = [];

	for (let n = 0; n < count; n++) {
		const forces: EntitySide[] = chance(0.5) ? [sides[0], sides[1]] : [sides[int(2)]];

		const keysites: KeysiteSpec[] = [];
		const numKeysites = int(4);
		for (let i = 0; i < numKeysites; i++) {
			keysites.push({ side: sides[int(2)], subType: int(EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES), inUse: chance(0.8), x: 0, z: 0, ammo: 100, fuel: 100 });
		}

		const xSectors = chance(0.03) ? 0 : 1 + int(4);
		const zSectors = 1 + int(4);
		const sideLength = chance(0.03) ? 1000 : [256, 512, 1024][int(3)];
		const numSectors = xSectors * zSectors;
		const maxX = xSectors * sideLength - 1;
		const maxZ = zSectors * sideLength - 1;

		const ops: LifecycleOp[] = [{ kind: "map", xSectors, zSectors, sideLength }];

		// heap: session, update, forces, keysites, sectors, then a few free entries (sometimes too few)
		const restored = 2 + forces.length + keysites.length;
		const used = restored + numSectors;
		const heap = chance(0.08) ? Math.max(restored, used - 1 - int(3)) : used + 1 + int(10);

		const labels: string[] = [];
		const numOps = int(12);

		for (let i = 0; i < numOps; i++) {
			const roll = rnd();

			if (roll < 0.25 && labels.length > 0) {
				ops.push({ kind: "destroy", label: labels[int(labels.length)] });
				continue;
			}

			if (roll >= 0.25 && roll < 0.26) {
				ops.push({ kind: "map", xSectors: 1 + int(2), zSectors: 1, sideLength: 512 });
				continue;
			}

			const attributes: LifecycleAttribute[] = [];
			const numAttributes = int(6);

			for (let a = 0; a < numAttributes; a++) {
				const kind = int(10);

				if (kind < 3) {
					attributes.push({
						kind: "vec3d",
						type: Vec3dType.VEC3D_TYPE_POSITION,
						x: chance(0.05) ? quarter(-64, 0) : chance(0.1) ? maxX + [0, 0.5, 1][int(3)] : quarter(0, maxX),
						y: quarter(-100, 500),
						z: chance(0.05) ? maxZ + 0.25 : quarter(0, maxZ),
					});
				} else if (kind < 5) {
					attributes.push({ kind: "int", type: IntType.INT_TYPE_SIDE, value: int(7) - 1 });
				} else if (kind < 6) {
					attributes.push({ kind: "int", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: int(40) - 1 });
				} else if (kind < 7) {
					attributes.push({ kind: "int", type: IntType.INT_TYPE_ALIVE, value: int(4) });
				} else if (kind < 9) {
					const target = keysites.length > 0 && chance(0.85) ? `keysite${int(keysites.length)}` : "NULL";
					attributes.push({ kind: "parent", type: ListType.LIST_TYPE_CARGO, target });
				} else if (chance(0.5) && labels.length > 0) {
					attributes.push({ kind: "pred", type: ListType.LIST_TYPE_CARGO, target: labels[int(labels.length)] });
				} else if (numSectors > 0 && sideLength !== 1000) {
					attributes.push({ kind: "parent", type: ListType.LIST_TYPE_SECTOR, target: `sector${int(xSectors)}_${int(zSectors)}` });
				}
			}

			const label = `c${labels.length}`;
			const type = chance(0.03) ? [EntityType.ENTITY_TYPE_UNKNOWN, EntityType.NUM_ENTITY_TYPES][int(2)] : EntityType.ENTITY_TYPE_CARGO;

			ops.push({ kind: "create", label, type, index: chance(0.03) ? int(heap) : -1, attributes });
			labels.push(label);
		}

		lifecycles.push({ heap, forces, keysites, ops });
	}

	return lifecycles;
}

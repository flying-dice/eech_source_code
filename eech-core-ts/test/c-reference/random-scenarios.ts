//
// Seeded random scenario generator for C-vs-TS differential testing.
// It only produces campaign state and an operation; outcomes always come from
// executing the original C.
//

import { OBJECT_3D_SINGLE_CRATE } from "../../src/generated/c-constants";
import { EntitySide, EntitySubTypeGroup, EntitySubTypeKeysite, EntitySubTypeTask, EntityType, FloatType, GameStatusType, IntType, ListType, Vec3dType } from "../../src/generated/c-enums";
import type { GroupParentSpec, KeysiteSpec, PositionSpec, ScenarioSpec } from "../scenarios/campaign-scenario";
import type { LifecycleAttribute, LifecycleOp, LifecycleSpec } from "../scenarios/lifecycle-scenario";
import type { TimelineSpec, TimelineStep } from "../scenarios/update-timeline";
import type { Float32RtzOp } from "../scenarios/float32-rtz";

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
// Some operations allocate a specific heap index.
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
		const destroyed: Record<string, boolean> = {};
		let allocated = false;
		const numOps = int(12);

		for (let i = 0; i < numOps; i++) {
			const roll = rnd();

			// once an index may have been allocated, destroying a crate a second time
			// could reach the allocated group through the stale pointer
			const destroyable = allocated ? labels.filter((l) => !destroyed[l]) : labels;

			if (roll < 0.25 && destroyable.length > 0) {
				const label = destroyable[int(destroyable.length)];
				destroyed[label] = true;
				ops.push({ kind: "destroy", label });
				continue;
			}

			if (roll >= 0.25 && roll < 0.26) {
				ops.push({ kind: "map", xSectors: 1 + int(2), zSectors: 1, sideLength: 512 });
				continue;
			}

			// get_free_entity with a specific index: usually a free entry, sometimes
			// one in use, outside the heap, or ENTITY_INDEX_DONT_CARE. Allocated
			// entries are never destroyed or referenced (their type is not ported).
			if (roll >= 0.26 && roll < 0.36) {
				allocated = true;
				ops.push({ kind: "allocate", label: `g${i}`, index: chance(0.05) ? -1 : chance(0.05) ? heap + int(2) : int(heap) });
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

//
// Operands for the round-toward-zero float helpers (src/core/float32.ts),
// aimed at their correction paths: results that are exactly a float in double
// but not in exact arithmetic, power-of-two boundaries, cancellation, the
// subnormal range and overflow. mul, div and sqrt take floats; narrow and sum
// take doubles.
//
export function generateFloat32RtzOperands(seed: number, count: number): [Float32RtzOp, number, number][] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const pick = <T>(values: T[]): T => values[int(values.length)];
	const sign = () => (rnd() < 0.5 ? -1 : 1);

	// a float: mostly campaign magnitudes, sometimes anywhere in the range
	const float = (): number => {
		const exponent = rnd() < 0.8 ? int(40) - 25 : int(276) - 149;
		const mantissa = rnd() < 0.2 ? Math.pow(2, 23) : Math.pow(2, 23) + int(Math.pow(2, 23));
		return Math.fround(sign() * mantissa * Math.pow(2, exponent - 23));
	};

	// a double that is rarely a float
	const double = (): number => {
		const kind = int(5);
		if (kind === 0) return sign() * Math.round(rnd() * 1e7) / 1000;
		if (kind === 1) return float();
		if (kind === 2) return float() * (1 + sign() * Math.pow(2, -30 - int(23)));
		if (kind === 3) return sign() * rnd() * Math.pow(2, int(300) - 150);
		return pick([100, 0.1, 0.3, 1 / 3, 3.4028235677973366e38, 3.5e38, 1e-46, 7.006492321624085e-46, 1 / 0]) * sign();
	};

	const out: [Float32RtzOp, number, number][] = [];
	while (out.length < count) {
		const op = pick<Float32RtzOp>(["narrow", "sum", "sum", "mul", "div", "div", "sqrt"]);
		if (op === "narrow") {
			out.push([op, double(), 0]);
		} else if (op === "sum") {
			const a = double();
			const kind = int(4);
			// the exact sum is a float plus or minus a tiny amount; cancellation; unrelated
			const b =
				kind === 0 ? sign() * Math.abs(a) * Math.pow(2, -30 - int(40)) : kind === 1 ? -a * (1 + sign() * Math.pow(2, -int(30))) : kind === 2 ? -Math.fround(a) : double();
			out.push([op, a, b]);
		} else if (op === "mul") {
			out.push([op, float(), rnd() < 0.3 ? int(101) : float()]);
		} else if (op === "div") {
			const b = rnd() < 0.4 ? 1 + int(100) : float();
			// sometimes an exact quotient
			const a = rnd() < 0.2 ? Math.fround(float() * b) : float();
			out.push([op, a, b]);
		} else {
			const r = Math.abs(float());
			const kind = int(3);
			// a perfect square, its float neighbours, or anything
			const a = kind === 0 ? Math.fround(Math.fround(r) * Math.fround(r)) : kind === 1 ? Math.fround(r * r * (1 + sign() * Math.pow(2, -23))) : Math.abs(float());
			out.push([op, a, 0]);
		}
	}
	return out;
}

//
// Operands for f64AddRTZ (src/core/float32.ts), the double sum of
// keysite.c :: update_keysite_cargo's (xmax - xmin) + 1.0: crate widths plus
// 1.0, tiny and huge widths where the sum is inexact, and sums whose exact
// value lies within a double ulp of a double.
//
export function generateDoubleSumOperands(seed: number, count: number): [Float32RtzOp, number, number][] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const sign = () => (rnd() < 0.5 ? -1 : 1);
	const out: [Float32RtzOp, number, number][] = [];
	while (out.length < count) {
		const kind = int(4);
		const a =
			kind === 0
				? Math.fround(sign() * rnd() * Math.pow(2, int(24) - 8))
				: kind === 1
					? sign() * Math.pow(2, -int(60) - 20) * (1 + rnd())
					: kind === 2
						? sign() * Math.pow(2, 50 + int(10)) * (1 + rnd())
						: sign() * rnd() * Math.pow(2, int(200) - 100);
		const b = rnd() < 0.5 ? 1.0 : sign() * Math.abs(a) * Math.pow(2, -int(70));
		out.push(["dsum", a, b]);
	}
	return out;
}

//
// Slice 4: keysite.c :: update_keysite_cargo on the Slice 3 lifecycle.
// Keysites (alive or not, in use or not, of every type), the crate's bounds
// (realistic, fractional, degenerate), the game status, crates restored as a
// saved game holds them, and runs of updates whose levels sit on and around
// crate multiples and the request threshold. Heaps are sometimes too small
// for the crates asked for (the original's debug_fatal).
//
export function generateRandomKeysiteCargo(seed: number, count: number): LifecycleSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const pick = <T>(values: T[]): T => values[int(values.length)];
	const sides = [EntitySide.ENTITY_SIDE_BLUE_FORCE, EntitySide.ENTITY_SIDE_RED_FORCE];
	const real = (lo: number, hi: number) => Math.round((lo + rnd() * (hi - lo)) * 1000) / 1000;
	const quarter = (lo: number, hi: number) => Math.round((lo + rnd() * (hi - lo)) * 4) / 4;

	const specs: LifecycleSpec[] = [];

	for (let n = 0; n < count; n++) {
		const forces: EntitySide[] = chance(0.1) ? [] : chance(0.4) ? [sides[0], sides[1]] : [sides[int(2)]];

		const xSectors = 1 + int(4);
		const zSectors = 1 + int(4);
		const sideLength = pick([256, 512, 1024]);
		const maxX = xSectors * sideLength - 1;
		const maxZ = zSectors * sideLength - 1;

		const keysites: KeysiteSpec[] = [];
		const numKeysites = 1 + int(3);
		for (let i = 0; i < numKeysites; i++) {
			keysites.push({
				side: sides[int(2)],
				subType: int(EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES),
				inUse: chance(0.9),
				// crate rows extend in +x and +z; some start near the far edges
				x: chance(0.5) ? quarter(0, maxX - 60) : real(0, maxX - 60),
				z: chance(0.5) ? quarter(0, maxZ - 20) : real(0, maxZ - 20),
				ammo: 100,
				fuel: 100,
			});
		}

		const ops: LifecycleOp[] = [];

		for (let i = 0; i < numKeysites; i++) {
			ops.push({ kind: "keysite-state", keysite: `keysite${i}`, alive: chance(0.92) ? 1 : chance(0.5) ? 0 : 2, y: chance(0.5) ? quarter(-10, 300) : real(-10, 300) });
		}

		const width = chance(0.1) ? 0 : chance(0.5) ? quarter(0.25, 3) : real(0.01, 3);
		const depth = chance(0.1) ? 0 : chance(0.5) ? quarter(0.25, 3) : real(0.01, 3);
		const height = chance(0.5) ? quarter(0, 2) : real(0, 2);
		const xmin = chance(0.3) ? -width / 2 : -real(0, width);
		const zmin = chance(0.3) ? -depth / 2 : -real(0, depth);
		const ymin = chance(0.2) ? 0 : -real(0, height);
		ops.push({ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin, xmax: xmin + width, ymin, ymax: ymin + height, zmin, zmax: zmin + depth });

		ops.push({ kind: "map", xSectors, zSectors, sideLength });

		if (!chance(0.1)) {
			ops.push({ kind: "game-status", status: chance(0.9) ? GameStatusType.GAME_STATUS_INITIALISED : pick([GameStatusType.GAME_STATUS_UNINITIALISED, GameStatusType.GAME_STATUS_INITIALISING]) });
		}

		// crates a saved game restores (anywhere near their keysite, any sub type)
		const numRestored = chance(0.3) ? int(5) : 0;
		for (let r = 0; r < numRestored; r++) {
			const k = int(numKeysites);
			ops.push({
				kind: "create",
				label: `r${r}`,
				type: EntityType.ENTITY_TYPE_CARGO,
				index: -1,
				attributes: [
					{ kind: "parent", type: ListType.LIST_TYPE_CARGO, target: `keysite${k}` },
					{ kind: "int", type: IntType.INT_TYPE_SIDE, value: keysites[k].side },
					{ kind: "int", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: chance(0.9) ? int(2) : 2 },
					{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x: keysites[k].x + quarter(0, 30), y: 0, z: keysites[k].z + quarter(0, 10) },
				],
			});
		}

		const numUpdates = 1 + int(10);
		let crates = numRestored;
		for (let u = 0; u < numUpdates; u++) {
			if (chance(0.05)) {
				ops.push({ kind: "game-status", status: pick([GameStatusType.GAME_STATUS_INITIALISED, GameStatusType.GAME_STATUS_INITIALISING]) });
			}

			const size = chance(0.8) ? 10 : chance(0.5) ? pick([0.1, 1.1, 2.5, 7.5, 12]) : real(0.5, 12);
			const levelKind = int(6);
			const level =
				levelKind === 0
					? size * int(11)
					: levelKind === 1
						? pick([75, 75.00001, 74.99999, 0, 100])
						: levelKind === 2
							? size * int(11) + pick([0.001, -0.001, 0.5])
							: levelKind === 3
								? real(-5, 5)
								: real(0, 100);
			ops.push({ kind: "update-cargo", keysite: `keysite${int(numKeysites)}`, level, subType: chance(0.95) ? int(2) : 2, size });
			crates += Math.max(0, Math.ceil(level / size));
		}

		// session, update, forces, keysites, sectors, restored crates; then room
		// for the crates asked for, or (rarely) too little
		const used = 2 + forces.length + numKeysites + xSectors * zSectors + numRestored;
		const heap = chance(0.05) ? used + int(4) : used + Math.min(crates, 400) + 8;

		specs.push({ heap, forces, keysites, ops });
	}

	return specs;
}

//
// Slice 4 review (#11): operands for keysite.c's crate-row step,
// position.x += (bounding_box->xmax - bounding_box->xmin) + 1.0, as
// [x, xmin, xmax] floats. Aimed at each of its rounding points: the float
// difference (cancellation, huge and tiny widths, overflow), the double
// + 1.0 (widths below 2^-29 or at least 2^53, where it is inexact), and the
// stored sum (x large or far from the width's scale); plus realistic crate
// rows along a keysite.
//
export function generateCrateRowOperands(seed: number, count: number): [number, number, number][] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const sign = () => (rnd() < 0.5 ? -1 : 1);
	const f = Math.fround;
	const float = (lo: number, hi: number) => f(sign() * rnd() * Math.pow(2, lo + int(hi - lo)));
	const out: [number, number, number][] = [];
	while (out.length < count) {
		const kind = int(7);
		if (kind === 0) {
			// a realistic crate beside a keysite
			const xmin = f(-rnd() * 3);
			out.push([f(rnd() * 60000), xmin, f(xmin + rnd() * 3)]);
		} else if (kind === 1) {
			// a tiny width: + 1.0 is inexact in double
			const xmin = float(-40, 10);
			out.push([float(-10, 20), xmin, f(xmin + float(-150, -30))]);
		} else if (kind === 2) {
			// a huge width: the difference or + 1.0 loses bits, or overflows
			out.push([float(-10, 30), f(-rnd() * Math.pow(2, 100 + int(28))), f(rnd() * Math.pow(2, 100 + int(28)))]);
		} else if (kind === 3) {
			// cancellation in the difference
			const xmin = float(-20, 40);
			out.push([float(-10, 30), xmin, f(xmin * (1 + sign() * Math.pow(2, -int(24))))]);
		} else if (kind === 4) {
			// x far larger than the spacing: the stored sum truncates
			out.push([float(24, 60), float(-10, 3), float(-10, 3)]);
		} else if (kind === 5) {
			// a negative or zero width
			const xmax = float(-10, 5);
			out.push([float(-10, 20), f(xmax + Math.abs(float(-10, 5))), rnd() < 0.2 ? f(xmax + Math.abs(float(-10, 5))) : xmax]);
		} else {
			out.push([float(-149, 128), float(-149, 128), float(-149, 128)]);
		}
	}
	return out;
}

//
// Slice 5a (issue #12): fc_msgs.c :: response_to_force_low_on_supplies on the
// real Slice 1-4 entity graph. Keysites of every sub type cluster around a
// requester, some at offsets on either side of get_closest_keysite's 10 km
// early exit and where the approximate and exact ranges disagree; real
// update_keysite_cargo calls stock them; restored tasks sit on requesters'
// LIST_TYPE_TASK_DEPENDENT lists; groups request through assess_group_supplies.
// Every scenario observes the create_supply_task boundary from its start.
//
export function generateRandomForceLowOnSupplies(seed: number, count: number): LifecycleSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const pick = <T>(values: T[]): T => values[int(values.length)];
	const sides = [EntitySide.ENTITY_SIDE_BLUE_FORCE, EntitySide.ENTITY_SIDE_RED_FORCE];
	const clamp = (v: number) => Math.min(32000, Math.max(0, v));

	// supplier offsets: near, around the 10 km early exit, far, and pairs where
	// get_approx_2d_range and get_2d_range fall on different sides of it
	const offsets: [number, number][] = [
		[0, 0],
		[3000, 0],
		[9500, 2000],
		[9800, 1000],
		[7900, 7900],
		[10000, 0],
		[10500, 0],
		[12000, 0],
		[-12000, 0],
		[20000, 0],
		[0, -15000],
	];

	const groupTypes = [
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ANTI_AIRCRAFT,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_SECONDARY_FRONTLINE,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_SELF_PROPELLED_ARTILLERY,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_SELF_PROPELLED_MLRS,
	];

	const specs: LifecycleSpec[] = [];

	for (let n = 0; n < count; n++) {
		const forces: EntitySide[] = chance(0.03) ? [] : chance(0.5) ? [sides[0], sides[1]] : [sides[0]];

		const cx = 1000 * (6 + int(20));
		const cz = 1000 * (6 + int(20));

		const keysites: KeysiteSpec[] = [];
		const numKeysites = 1 + int(6);
		for (let i = 0; i < numKeysites; i++) {
			const [dx, dz] = i === 0 ? [0, 0] : chance(0.7) ? pick(offsets) : [int(30000) - 15000, int(30000) - 15000];
			const flip = chance(0.5) ? -1 : 1;
			keysites.push({
				side: chance(0.8) ? sides[0] : sides[1],
				subType: i === 0 ? pick([0, 3, 4, 6, int(9)]) : pick([0, 2, 7, int(9)]),
				inUse: chance(0.92),
				x: clamp(cx + flip * dx),
				z: clamp(cz + dz),
				ammo: 100,
				fuel: 100,
			});
		}

		const ops: LifecycleOp[] = [{ kind: "observe-supply-tasks" }];

		for (let i = 0; i < numKeysites; i++) {
			ops.push({ kind: "keysite-state", keysite: `keysite${i}`, alive: chance(0.95) ? 1 : 0, y: 0 });
		}

		ops.push(
			{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: -0.5, ymax: 0.5, zmin: -1.5, zmax: 1.5 },
			{ kind: "map", xSectors: 4, zSectors: 4, sideLength: 8192 },
			{ kind: "game-status", status: chance(0.95) ? GameStatusType.GAME_STATUS_INITIALISED : GameStatusType.GAME_STATUS_UNINITIALISED },
		);

		// a comms client (rarely) makes one last request, of keysite0 for ammo at
		// level 5: keysite0 never holds ammo crates in such a scenario, so that
		// request neither creates nor destroys (client-side creation and
		// destruction are not ported)
		const client = chance(0.05);

		// stock suppliers (and requesters) through the real update_keysite_cargo
		for (let i = 0; i < numKeysites; i++) {
			for (const subType of [0, 1]) {
				if (chance(0.6) && !(client && i === 0 && subType === 0)) {
					ops.push({ kind: "update-cargo", keysite: `keysite${i}`, level: pick([0, 5, 15, 35, 55]), subType, size: 10 });
				}
			}
		}

		const groups: string[] = [];
		const numGroups = chance(0.4) ? 1 + int(2) : 0;
		for (let g = 0; g < numGroups; g++) {
			const leader: PositionSpec = chance(0.9) ? { kind: "at", x: clamp(cx + int(4000) - 2000), z: clamp(cz + int(4000) - 2000) } : { kind: "none" };
			ops.push({
				kind: "restore-group",
				label: `g${g}`,
				subType: chance(0.9) ? pick(groupTypes) : int(EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS),
				side: forces.length > 0 && chance(0.9) ? forces[0] : sides[1],
				ammo: pick([100, 99.5, 50, 0]),
				fuel: pick([100, 99.5, 50, 0]),
				parent: chance(0.7) ? "NULL" : chance(0.5) ? "independent" : `keysite${int(numKeysites)}`,
				busy: chance(0.2),
				leader,
			});
			groups.push(`g${g}`);
		}

		const objectives = [...keysites.map((_, i) => `keysite${i}`), ...groups];
		const numTasks = chance(0.5) ? 1 + int(3) : 0;
		for (let t = 0; t < numTasks; t++) {
			ops.push({
				kind: "task",
				label: `t${t}`,
				objective: chance(0.7) ? objectives[0] : pick(objectives),
				subType: chance(0.8) ? EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY : int(EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS),
				side: chance(0.75) ? sides[0] : sides[1],
				state: int(3),
				userData: pick([0, 1, 2, 0.5]),
			});
		}

		// the requests: keysites below and above one crate's worth, and groups
		const numRequests = 1 + int(4);
		for (let r = 0; r < numRequests; r++) {
			if (groups.length > 0 && chance(0.4)) {
				ops.push({ kind: "assess-group", group: pick(groups) });
			} else {
				const keysite = chance(0.7) ? 0 : int(numKeysites);
				const subType = client && keysite === 0 ? 1 : int(2);
				ops.push({ kind: "update-cargo", keysite: `keysite${keysite}`, level: pick([0, 5, 10, 35, 60, 75, 80]), subType, size: 10 });
			}
		}

		// route waypoints on requesters' task-dependent lists, RECON (21, the
		// value of ENTITY_SUB_TYPE_TASK_SUPPLY) among them
		const numWaypoints = chance(0.3) ? 1 + int(2) : 0;
		for (let w = 0; w < numWaypoints; w++) {
			ops.push({ kind: "waypoint", label: `w${w}`, dependent: chance(0.8) ? objectives[0] : pick(objectives), subType: chance(0.5) ? 21 : int(40) });
		}

		if (client) {
			ops.push({ kind: "comms-model", model: 1 }, { kind: "update-cargo", keysite: "keysite0", level: 5, subType: 0, size: 10 });
		}

		specs.push({ heap: 400, forces, keysites, ops });
	}

	return specs;
}

//
// Slice 5b: random supply task construction scenarios (taskgen.c ::
// create_supply_task -> create_task). Requests come from real senders through
// the real Slice 5a response. Candidate start keysites vary in type, use,
// landing types, usable state and based groups (type, alive, busy); sectors
// in side and surface-to-air defence; keysites in height (some beyond the
// map volume, which multiplayer packing refuses); the game type; single
// player or multiplayer; and the force's supply task counter (around the
// 12-bit id's wrap). One map in five is 16 x 16 sectors, where start
// keysites can be 100 km away. Every position is on the map.
//
export function generateRandomSupplyTaskConstruction(seed: number, count: number): LifecycleSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const pick = <T>(values: T[]): T => values[int(values.length)];
	const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
	const RED = EntitySide.ENTITY_SIDE_RED_FORCE;

	const groupTypes = [
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_HEAVY_LIFT_TRANSPORT_HELICOPTER,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_HEAVY_LIFT_TRANSPORT_HELICOPTER,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_AIRCRAFT,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_HEAVY_LIFT_TRANSPORT_AIRCRAFT,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER,
		EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_HELICOPTER,
	];

	const specs: LifecycleSpec[] = [];

	for (let n = 0; n < count; n++) {
		const big = chance(0.2);
		const sectors = big ? 16 : 4;
		const extent = sectors * 8192 - 1;
		// crates are created up to ~10 m east of and 3 m north of their keysite
		const clamp = (v: number) => Math.min(extent - 100, Math.max(100, v));
		const forces: EntitySide[] = chance(0.3) ? [BLUE, RED] : [BLUE];
		const sideOf = (): EntitySide => (forces.length > 1 && chance(0.1) ? RED : BLUE);

		// requesters and suppliers near a centre; start keysite candidates anywhere
		const cx = 1000 * (4 + int(big ? 120 : 25));
		const cz = 1000 * (4 + int(big ? 120 : 25));
		const spread = big ? 60000 : 16000;

		const keysites: KeysiteSpec[] = [];
		const numKeysites = 2 + int(6);
		for (let i = 0; i < numKeysites; i++) {
			const near = i < 2 || chance(0.4);
			keysites.push({
				side: chance(0.03) ? RED : sideOf(),
				// keysite0 requests (FARP, airbase or another), keysite1 supplies (factory or refinery), then airbases mostly
				subType: i === 0 ? pick([3, 3, 0, int(9)]) : i === 1 ? pick([2, 2, 7]) : chance(0.7) ? 0 : int(9),
				inUse: chance(0.93),
				x: clamp(near ? cx + int(12000) - 6000 : cx + int(2 * spread) - spread),
				z: clamp(near ? cz + int(12000) - 6000 : cz + int(2 * spread) - spread),
				ammo: 100,
				fuel: 100,
			});
		}

		const ops: LifecycleOp[] = [{ kind: "observe-supply-tasks" }, { kind: "observe-tasks" }];

		for (let i = 0; i < numKeysites; i++) {
			const y = chance(0.85) ? 0 : pick([3000, 250.5, 65535, 70000, -9000]);
			ops.push({ kind: "keysite-state", keysite: `keysite${i}`, alive: chance(0.97) ? 1 : 0, y });
		}

		ops.push(
			{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: chance(0.1) ? 0 : -0.5, ymax: 0.5, zmin: -1.5, zmax: 1.5 },
			{ kind: "map", xSectors: sectors, zSectors: sectors, sideLength: 8192 },
			{ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED },
			{ kind: "game-type", type: pick([2, 2, 2, 3, 1, 0, 4]) },
		);

		if (chance(0.3)) {
			ops.push({ kind: "single-player" });
		}

		if (chance(0.15)) {
			for (const i of forces.map((_, f) => f)) {
				ops.push({ kind: "task-counter", force: `force${i}`, subType: EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY, created: pick([4093, 4094, 4095, 8189, 8190, 12285, 100000]) });
			}
		}

		// start keysite candidates: landing types, usable state and based groups
		const groups: string[] = [];
		for (let i = 0; i < numKeysites; i++) {
			if (chance(0.85)) {
				ops.push({ kind: "keysite-landing", keysite: `keysite${i}`, landingTypes: chance(0.75) ? pick([4, 6, 2]) : int(16), usableState: chance(0.8) ? 0 : 1 + int(2) });
			}
			const numBased = chance(0.8) ? 1 + int(chance(0.2) ? 7 : 3) : 0;
			for (let g = 0; g < numBased; g++) {
				const label = `k${i}g${g}`;
				ops.push(
					{
						kind: "restore-group",
						label,
						subType: chance(0.9) ? pick(groupTypes) : int(EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS),
						side: keysites[i].side,
						ammo: 100,
						fuel: 100,
						parent: `keysite${i}`,
						busy: chance(0.25),
						leader: { kind: "none" },
					},
					{ kind: "group-alive", group: label, alive: chance(0.9) ? 1 : 0 },
				);
				groups.push(label);
			}
		}

		// sector presence and defences on some cells
		const numSectorStates = int(big ? 30 : 10);
		for (let s = 0; s < numSectorStates; s++) {
			ops.push({
				kind: "sector-state",
				sector: `sector${int(sectors)}_${int(sectors)}`,
				blue: pick([0, 1, 2, 0.5]),
				red: pick([0, 1, 2, 0.5]),
				samNeutral: chance(0.2) ? 1 : 0,
				samBlue: chance(0.3) ? pick([1, 0.25]) : 0,
				samRed: chance(0.3) ? pick([1, 0.25]) : 0,
			});
		}

		// stock suppliers (and requesters) through the real update_keysite_cargo
		for (let i = 0; i < numKeysites; i++) {
			for (const subType of [0, 1]) {
				if (i === 1 ? chance(0.9) : chance(0.3)) {
					ops.push({ kind: "update-cargo", keysite: `keysite${i}`, level: pick([15, 35, 55]), subType, size: 10 });
				}
			}
		}

		// front line groups that resupply through supply tasks
		const frontline: string[] = [];
		if (chance(0.3)) {
			const label = "line";
			ops.push({
				kind: "restore-group",
				label,
				subType: EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE,
				side: forces[0],
				ammo: pick([50, 0]),
				fuel: pick([100, 50]),
				parent: "NULL",
				busy: false,
				leader: { kind: "at", x: clamp(cx + int(8000) - 4000), z: clamp(cz + int(8000) - 4000) },
			});
			frontline.push(label);
		}

		// the requests: keysites low on supplies, and front line groups
		const numRequests = 1 + int(4);
		for (let r = 0; r < numRequests; r++) {
			if (frontline.length > 0 && chance(0.3)) {
				ops.push({ kind: "assess-group", group: frontline[0] });
			} else {
				ops.push({ kind: "update-cargo", keysite: `keysite${chance(0.7) ? 0 : int(numKeysites)}`, level: pick([5, 10, 35, 60, 75]), subType: int(2), size: 10 });
			}
		}

		specs.push({ heap: big ? 700 : 400, forces, keysites, ops });
	}

	return specs;
}

//
// Slice 6a: random supply task assignment scenarios (assign.c ::
// assign_keysite_tasks up to assign_primary_task_to_group). Keysites (types,
// sides) hold restored unassigned tasks (mostly SUPPLY; ESCORT of a group,
// TROOP_INSERTION of a keysite, other types and categories; priorities with
// ties, critical or not, expiry around KEYSITE_TASK_ASSIGN_TIMER and the
// locality ETAs) and based groups (types, sides, alive, busy, sleep, air
// registration, members: helicopters or, for jet groups, fixed wing, with
// member counts equal to the members restored, their aircraft and distances).
// Pilots lock some tasks and groups. In one scenario in four the real Slice
// 4 - 5b path first constructs a supply task. Keysites are then assigned in
// random order and categories; the first boundary ends the scenario.
//
export function generateRandomSupplyTaskAssignment(seed: number, count: number): LifecycleSpec[] {
	const rnd = mulberry32(seed);
	const int = (n: number) => Math.floor(rnd() * n);
	const chance = (p: number) => rnd() < p;
	const pick = <T>(values: T[]): T => values[int(values.length)];
	const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
	const RED = EntitySide.ENTITY_SIDE_RED_FORCE;
	const G = EntitySubTypeGroup;
	const T = EntitySubTypeTask;

	const groupTypes = [
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_HEAVY_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_HEAVY_LIFT_TRANSPORT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_AIRCRAFT,
		G.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_MARINE_ATTACK_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_ASSAULT_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_RECON_HELICOPTER,
		G.ENTITY_SUB_TYPE_GROUP_CARRIER_BORNE_INTERCEPTOR,
		G.ENTITY_SUB_TYPE_GROUP_MULTI_ROLE_FIGHTER,
		G.ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP,
	];
	// groups whose members are fixed wing (gp_dbase.c default_entity_type)
	const jets = [
		G.ENTITY_SUB_TYPE_GROUP_MULTI_ROLE_FIGHTER,
		G.ENTITY_SUB_TYPE_GROUP_CARRIER_BORNE_ATTACK_AIRCRAFT,
		G.ENTITY_SUB_TYPE_GROUP_CARRIER_BORNE_INTERCEPTOR,
		G.ENTITY_SUB_TYPE_GROUP_CLOSE_AIR_SUPPORT_AIRCRAFT,
		G.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_AIRCRAFT,
		G.ENTITY_SUB_TYPE_GROUP_HEAVY_LIFT_TRANSPORT_AIRCRAFT,
	];
	const otherTasks = [T.ENTITY_SUB_TYPE_TASK_BAI, T.ENTITY_SUB_TYPE_TASK_RECON, T.ENTITY_SUB_TYPE_TASK_REPAIR, T.ENTITY_SUB_TYPE_TASK_TRANSFER_HELICOPTER, T.ENTITY_SUB_TYPE_TASK_BARCAP];

	const specs: LifecycleSpec[] = [];

	for (let n = 0; n < count; n++) {
		const forces: EntitySide[] = chance(0.3) ? [BLUE, RED] : [BLUE];
		const social = chance(0.25);

		const keysites: KeysiteSpec[] = [];
		const numKeysites = social ? 3 + int(3) : 1 + int(4);
		for (let i = 0; i < numKeysites; i++) {
			keysites.push({
				side: chance(0.1) ? RED : BLUE,
				subType: social && i < 3 ? [3, 2, 0][i] : chance(0.6) ? 0 : int(9),
				inUse: true,
				x: 2000 + int(28000),
				z: 2000 + int(28000),
				ammo: 100,
				fuel: 100,
			});
		}

		const ops: LifecycleOp[] = [];
		for (let i = 0; i < numKeysites; i++) {
			ops.push({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: 0 });
		}

		if (social) {
			ops.push(
				{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: -0.5, ymax: 0.5, zmin: -1.5, zmax: 1.5 },
				{ kind: "map", xSectors: 4, zSectors: 4, sideLength: 8192 },
				{ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED },
				{ kind: "game-type", type: 2 },
			);
			for (let i = 0; i < numKeysites; i++) {
				ops.push({ kind: "keysite-landing", keysite: `keysite${i}`, landingTypes: pick([4, 6, 0]), usableState: 0 });
			}
		}

		const client = chance(0.05);

		const pilots: string[] = [];
		if (chance(0.25)) {
			ops.push({ kind: "pilot", label: "p0" });
			pilots.push("p0");
		}

		// groups
		const groups: string[] = [];
		const numGroups = int(8);
		for (let g = 0; g < numGroups; g++) {
			const label = `g${g}`;
			const subType = chance(0.92) ? pick(groupTypes) : int(EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS);
			const jet = jets.indexOf(subType) >= 0;
			const keysite = chance(0.9) ? (chance(0.6) ? (social ? 2 : 0) : int(numKeysites)) : -1;
			const home = keysite >= 0 ? keysites[keysite] : { x: 16000, z: 16000 };
			const members = pick([0, 1, 1, 1, 2, 3]);
			const offset = (): number => (chance(0.6) ? 0 : int(100000) - 50000 + pick([0, 0.5, 0.25]));
			const aircraft = (): number => int(33);
			const side = chance(0.9) ? home === keysites[keysite] ? keysites[keysite].side : BLUE : pick([BLUE, RED]);
			const helicopterLeader = members > 0 && !jet;
			ops.push(
				{
					kind: "restore-group",
					label,
					subType,
					side,
					ammo: 100,
					fuel: 100,
					parent: keysite >= 0 ? `keysite${keysite}` : "NULL",
					busy: chance(0.1),
					leader: helicopterLeader ? { kind: "at", x: home.x + offset(), z: home.z } : { kind: "none" },
				},
				{ kind: "group-alive", group: label, alive: chance(0.85) ? 1 : 0 },
				{ kind: "member-count", group: label, count: members },
			);
			if (helicopterLeader) {
				ops.push({ kind: "aircraft-type", member: `${label}.leader`, subType: aircraft() });
			}
			for (let m = helicopterLeader ? 1 : 0; m < members; m++) {
				ops.push({
					kind: "add-member",
					label: `${label}.m${m}`,
					group: label,
					type: jet ? EntityType.ENTITY_TYPE_FIXED_WING : EntityType.ENTITY_TYPE_HELICOPTER,
					subType: aircraft(),
					x: home.x + offset(),
					z: home.z,
				});
			}
			if (forces.indexOf(side) >= 0 && chance(0.85)) {
				ops.push({ kind: "air-register", group: label });
			}
			if (chance(0.05)) {
				ops.push({ kind: "group-sleep", group: label, sleep: pick([5, 0.5]) });
			}
			if (pilots.length > 0 && chance(0.15)) {
				ops.push({ kind: "pilot-lock", entity: label, pilot: "p0" });
			}
			groups.push(label);
		}

		// the social chain's supply task, constructed by the real Slice 4 - 5b path
		if (social) {
			ops.push(
				{ kind: "update-cargo", keysite: "keysite1", level: 35, subType: 0, size: 10 },
				{ kind: "update-cargo", keysite: "keysite0", level: pick([5, 0]), subType: 0, size: 10 },
			);
		}

		// restored unassigned tasks
		const numTasks = chance(0.9) ? 1 + int(5) : 0;
		for (let t = 0; t < numTasks; t++) {
			const r = rnd();
			let subType: number;
			let objective = "NULL";
			if (r < 0.55) {
				subType = T.ENTITY_SUB_TYPE_TASK_SUPPLY;
			} else if (r < 0.7) {
				subType = T.ENTITY_SUB_TYPE_TASK_ESCORT;
				objective = groups.length > 0 && chance(0.85) ? pick(groups) : "NULL";
			} else if (r < 0.8) {
				subType = T.ENTITY_SUB_TYPE_TASK_TROOP_INSERTION;
				objective = chance(0.85) ? `keysite${int(numKeysites)}` : "NULL";
			} else if (r < 0.95) {
				subType = pick(otherTasks);
			} else {
				subType = int(EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS);
			}
			ops.push({
				kind: "unassigned-task",
				label: `t${t}`,
				keysite: `keysite${chance(0.6) ? (social ? 2 : 0) : int(numKeysites)}`,
				objective,
				subType,
				side: chance(0.9) ? BLUE : RED,
				critical: chance(0.7) ? 1 : 0,
				priority: pick([4, 4, 4, 1, 2.5, 8, 0]),
				expire: pick([1200, 1200, 180, 180.5, 100, 600, 3000]),
			});
			if (pilots.length > 0 && chance(0.15)) {
				ops.push({ kind: "pilot-lock", entity: `t${t}`, pilot: "p0" });
			}
		}

		ops.push({ kind: "observe-tasks" });

		if (client) {
			ops.push({ kind: "comms-model", model: 1 });
		}

		const numAssigns = 1 + int(3);
		for (let a = 0; a < numAssigns; a++) {
			ops.push({ kind: "assign-tasks", keysite: `keysite${chance(0.6) ? (social ? 2 : 0) : int(numKeysites)}`, category: chance(0.7) ? 2 : int(4) });
		}

		specs.push({ heap: 200, forces, keysites, ops });
	}

	return specs;
}

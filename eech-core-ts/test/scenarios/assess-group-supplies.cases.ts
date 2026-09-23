//
// Behaviour matrix for group.c :: assess_group_supplies and
// keysite.c :: get_closest_keysite, as conformance cases.
//
// Expected outcomes are derived by reading the original C (the `c` field names
// the branch); none were produced by running the TypeScript. They are checked
// three ways:
//   - test/unit/conformance.test.ts        TS port under JavaScript semantics
//   - test/lua/conformance.ts              TS port transpiled to Lua 5.1
//   - test/c-reference/*.cref.test.ts      the original C, executed
//
// TSTL-compatible: no Node APIs.
//

import { FLT_MAX } from "../../src/core/float32";
import { EntityMessage, EntitySide, EntitySubTypeCargo, EntitySubTypeGroup, EntitySubTypeKeysite, FloatType } from "../../src/generated/c-enums";
import type { GroupSpec, KeysiteSpec, PositionSpec, ScenarioOutcome, ScenarioSpec, TransmitEvent } from "./campaign-scenario";

export interface ConformanceCase {
	id: string;
	c: string;
	spec: ScenarioSpec;
	expected: ScenarioOutcome;
}

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const RED = EntitySide.ENTITY_SIDE_RED_FORCE;

const AMMO = FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL;
const FUEL = FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL;

const LOW_ON_SUPPLIES = EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES;
const CARGO_AMMO = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO;
const CARGO_FUEL = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_FUEL;

// group_database .resupply_source (gp_dbase.c): KEYSITE, GROUP and NONE respectively
const HELICOPTERS = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER;
const FRONTLINE = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE;
const INFANTRY = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_INFANTRY;

const ORIGIN: PositionSpec = { kind: "at", x: 0, z: 0 };
const NO_POSITION: PositionSpec = { kind: "none" };

const AIRBASE = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE;
const FARP = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP;
const ANY_KEYSITE = EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES;

function keysite(x: number, z: number, ammo: number, fuel: number, extra: Partial<KeysiteSpec> = {}): KeysiteSpec {
	return { side: BLUE, subType: AIRBASE, inUse: true, x, z, ammo, fuel, ...extra };
}

function group(ammo: number, fuel: number, extra: Partial<GroupSpec> = {}): GroupSpec {
	return { subType: HELICOPTERS, side: BLUE, ammo, fuel, parent: { kind: "keysite", keysite: 0 }, busy: false, leader: ORIGIN, ...extra };
}

function assess(keysites: KeysiteSpec[], g: GroupSpec, world: Partial<ScenarioSpec> = {}): ScenarioSpec {
	return { session: true, forces: [BLUE, RED], keysites, group: g, op: { kind: "assess" }, ...world };
}

function outcome(spec: ScenarioSpec, expected: Partial<ScenarioOutcome>): ScenarioOutcome {
	const keysiteAmmo: number[] = [];
	const keysiteFuel: number[] = [];

	for (const k of spec.keysites) {
		keysiteAmmo.push(k.ammo);
		keysiteFuel.push(k.fuel);
	}

	return {
		result: "ok",
		closest: "",
		closestRange: undefined,
		messages: [],
		transmissions: [],
		groupAmmo: spec.group === undefined ? 0 : spec.group.ammo,
		groupFuel: spec.group === undefined ? 0 : spec.group.fuel,
		keysiteAmmo,
		keysiteFuel,
		...expected,
	};
}

function tx(entity: string, floatType: FloatType, value: number): TransmitEvent {
	return { entity, floatType, value };
}

function kase(id: string, c: string, spec: ScenarioSpec, expected: Partial<ScenarioOutcome>): ConformanceCase {
	return { id, c, spec, expected: outcome(spec, expected) };
}

function closest(keysites: KeysiteSpec[], op: Partial<Extract<ScenarioSpec["op"], { kind: "closest" }>>): ScenarioSpec {
	return {
		session: true,
		forces: [BLUE, RED],
		keysites,
		group: undefined,
		op: { kind: "closest", type: ANY_KEYSITE, side: BLUE, pos: ORIGIN, minRange: 1000, wantRange: true, outside: true, exclude: -1, ...op },
	};
}

export const ASSESS_GROUP_SUPPLIES_CASES: ConformanceCase[] = [
	//
	// RESUPPLY_SOURCE_GROUP: request a supply mission from the side's force
	//
	kase(
		"group-low-ammo-requests-ammo",
		"RESUPPLY_SOURCE_GROUP, ammo_supply_level < 100.0",
		assess([], group(50, 50, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION })),
		{ messages: [{ receiver: "force0", sender: "group", message: LOW_ON_SUPPLIES, arg: CARGO_AMMO }] },
	),
	kase(
		"group-low-fuel-requests-fuel",
		"RESUPPLY_SOURCE_GROUP, ammo >= 100.0, fuel_supply_level < 100.0",
		assess([], group(100, 99.5, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION })),
		{ messages: [{ receiver: "force0", sender: "group", message: LOW_ON_SUPPLIES, arg: CARGO_FUEL }] },
	),
	kase(
		"group-low-ammo-and-fuel-requests-ammo-only",
		"RESUPPLY_SOURCE_GROUP: else if - one request per assessment, ammo first",
		assess([], group(10, 10, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION })),
		{ messages: [{ receiver: "force0", sender: "group", message: LOW_ON_SUPPLIES, arg: CARGO_AMMO }] },
	),
	kase(
		"group-full-requests-nothing",
		"RESUPPLY_SOURCE_GROUP, ammo == 100.0 and fuel == 100.0 are not < 100.0",
		assess([], group(100, 100, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION })),
		{},
	),
	kase(
		"group-over-full-requests-nothing",
		"RESUPPLY_SOURCE_GROUP, levels above 100.0",
		assess([], group(150, 101, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION })),
		{},
	),
	kase(
		"group-just-below-100-requests",
		"RESUPPLY_SOURCE_GROUP, float 99.99999 < 100.0",
		assess([], group(99.99999, 100, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION })),
		{
			groupAmmo: 99.99999237060547,
			messages: [{ receiver: "force0", sender: "group", message: LOW_ON_SUPPLIES, arg: CARGO_AMMO }],
		},
	),
	kase(
		"group-request-goes-to-force-of-own-side",
		"get_local_force_entity: first force in LIST_TYPE_FORCE whose INT_TYPE_SIDE matches",
		assess([], group(50, 100, { subType: FRONTLINE, side: RED, parent: { kind: "none" }, leader: NO_POSITION })),
		{ messages: [{ receiver: "force1", sender: "group", message: LOW_ON_SUPPLIES, arg: CARGO_AMMO }] },
	),
	kase(
		"group-busy-still-requests",
		"RESUPPLY_SOURCE_GROUP does not consult INT_TYPE_GROUP_MODE",
		assess([], group(50, 100, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION, busy: true })),
		{ messages: [{ receiver: "force0", sender: "group", message: LOW_ON_SUPPLIES, arg: CARGO_AMMO }] },
	),
	kase(
		"group-without-force-asserts",
		"get_local_force_entity returns NULL; notify_local_entity: ASSERT (receiver)",
		assess([], group(50, 100, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION }), { forces: [RED] }),
		{ result: "assert:receiver" },
	),
	kase(
		"group-without-session-asserts",
		"get_local_force_entity: !get_session_entity () returns NULL; ASSERT (receiver)",
		assess([], group(50, 100, { subType: FRONTLINE, parent: { kind: "none" }, leader: NO_POSITION }), { session: false }),
		{ result: "assert:receiver" },
	),

	//
	// RESUPPLY_SOURCE_NONE
	//
	kase(
		"none-source-does-nothing",
		"RESUPPLY_SOURCE_NONE matches neither branch",
		assess([keysite(0, 0, 500, 500)], group(0, 0, { subType: INFANTRY }), { session: false, forces: [] }),
		{},
	),

	//
	// RESUPPLY_SOURCE_KEYSITE: rearm / refuel an idle group from keysite stock
	//
	kase(
		"keysite-rearm-and-refuel-limited-by-stock",
		"KEYSITE, IDLE, parent keysite; required = bound (100 - level, 0, keysite level)",
		assess([keysite(0, 0, 50, 50)], group(30, 40)),
		{
			transmissions: [tx("keysite0", AMMO, 0), tx("group", AMMO, 80), tx("keysite0", FUEL, 0), tx("group", FUEL, 90)],
			groupAmmo: 80,
			groupFuel: 90,
			keysiteAmmo: [0],
			keysiteFuel: [0],
		},
	),
	kase(
		"keysite-rearm-only",
		"KEYSITE, IDLE, ammo < 100.0, fuel == 100.0",
		assess([keysite(0, 0, 500, 500)], group(30, 100)),
		{
			transmissions: [tx("keysite0", AMMO, 430), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [430],
		},
	),
	kase(
		"keysite-refuel-only",
		"KEYSITE, IDLE, ammo == 100.0, fuel < 100.0",
		assess([keysite(0, 0, 500, 500)], group(100, 20)),
		{
			transmissions: [tx("keysite0", FUEL, 420), tx("group", FUEL, 100)],
			groupFuel: 100,
			keysiteFuel: [420],
		},
	),
	kase(
		"keysite-busy-group-not-resupplied",
		"KEYSITE, INT_TYPE_GROUP_MODE == GROUP_MODE_BUSY (guide stack not empty)",
		assess([keysite(0, 0, 500, 500)], group(10, 10, { busy: true })),
		{},
	),
	kase(
		"keysite-full-group-not-resupplied",
		"KEYSITE, IDLE, ammo and fuel == 100.0",
		assess([keysite(0, 0, 500, 500)], group(100, 100)),
		{},
	),
	kase(
		"keysite-empty-stock-still-transmits",
		"KEYSITE, keysite level 0: required bounds to 0, values are still set (and transmitted)",
		assess([keysite(0, 0, 0, 0)], group(30, 40)),
		{
			transmissions: [tx("keysite0", AMMO, 0), tx("group", AMMO, 30), tx("keysite0", FUEL, 0), tx("group", FUEL, 40)],
		},
	),
	kase(
		"keysite-negative-stock-drains-group",
		"bound (required, 0.0, level) with level < 0 returns level: the group loses supplies and the keysite returns to 0",
		assess([keysite(0, 0, -5, -1)], group(30, 40)),
		{
			transmissions: [tx("keysite0", AMMO, 0), tx("group", AMMO, 25), tx("keysite0", FUEL, 0), tx("group", FUEL, 39)],
			groupAmmo: 25,
			groupFuel: 39,
			keysiteAmmo: [0],
			keysiteFuel: [0],
		},
	),
	kase(
		"keysite-negative-group-level-tops-up-to-100",
		"required = 100.0 - (level * ACCELERATOR) exceeds 100 for negative group levels",
		assess([keysite(0, 0, 500, 500)], group(-20, 100)),
		{
			transmissions: [tx("keysite0", AMMO, 380), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [380],
		},
	),
	kase(
		"keysite-float-arithmetic",
		"float storage: 100.0 - 33.3f in double, narrowed to float; float - float; float + float (all toward zero)",
		// Last-ulp values confirmed by executing the original C (c-reference harness);
		// the first hand estimate was wrong, which is why these are C-checked.
		// Rounding toward zero (docs/fidelity/fpu-semantics.md, class "supply"):
		// 33.3f + (100.0 - 33.3f) truncates to 99.99999f, not 100.0f, so the
		// group's ammo is still below 100 after the refill.
		assess([keysite(0, 0, 1000.1, 12.7)], group(33.3, 87.4)),
		{
			transmissions: [
				tx("keysite0", AMMO, 933.3999633789062),
				tx("group", AMMO, 99.99999237060547),
				tx("keysite0", FUEL, 0.10000133514404297),
				tx("group", FUEL, 100),
			],
			groupAmmo: 99.99999237060547,
			groupFuel: 100,
			keysiteAmmo: [933.3999633789062],
			keysiteFuel: [0.10000133514404297],
		},
	),

	//
	// RESUPPLY_SOURCE_KEYSITE without a keysite parent: get_closest_keysite (NUM_ENTITY_SUB_TYPE_KEYSITES,
	// side, group position, 1.0 * KILOMETRE, NULL, TRUE, NULL)
	//
	kase(
		"closest-keysite-within-1km",
		"!keysite: get_closest_keysite early out (range <= min_range)",
		assess([keysite(5000, 0, 500, 500), keysite(300, 0, 500, 500)], group(50, 100, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite1", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [500, 450],
		},
	),
	kase(
		"closest-keysite-early-out-is-first-in-list-not-nearest",
		"get_closest_keysite returns the first in-range keysite in LIST_TYPE_KEYSITE_FORCE order",
		assess([keysite(900, 0, 500, 500), keysite(10, 0, 500, 500)], group(50, 100, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite0", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [450, 500],
		},
	),
	kase(
		"closest-keysite-beyond-1km-picks-nearest",
		"get_closest_keysite: outside_of_range keeps range < best_range",
		assess([keysite(5000, 0, 500, 500), keysite(0, -3000, 500, 500)], group(50, 100, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite1", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [500, 450],
		},
	),
	kase(
		"closest-keysite-tie-keeps-first",
		"get_closest_keysite: strict < on equal approximate ranges",
		assess([keysite(3000, 0, 500, 500), keysite(0, 3000, 500, 500)], group(50, 100, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite0", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [450, 500],
		},
	),
	kase(
		"closest-keysite-skips-unused",
		"get_closest_keysite: INT_TYPE_IN_USE == 0 is skipped (C int truthiness)",
		assess([keysite(100, 0, 500, 500, { inUse: false }), keysite(4000, 0, 500, 500)], group(50, 100, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite1", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [500, 450],
		},
	),
	kase(
		"closest-keysite-only-own-side",
		"get_closest_keysite walks the side's force LIST_TYPE_KEYSITE_FORCE only",
		assess([keysite(10, 0, 500, 500, { side: RED }), keysite(8000, 0, 500, 500)], group(50, 100, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite1", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [500, 450],
		},
	),
	kase(
		"independent-group-parent-is-force-not-keysite",
		"group_link is shared: parent (LIST_TYPE_KEYSITE_GROUP) is the force; type != ENTITY_TYPE_KEYSITE",
		assess([keysite(200, 0, 500, 500)], group(50, 100, { parent: { kind: "independent" } })),
		{
			transmissions: [tx("keysite0", AMMO, 450), tx("group", AMMO, 100)],
			groupAmmo: 100,
			keysiteAmmo: [450],
		},
	),
	kase(
		"closest-keysite-rearm-and-refuel-look-up-twice",
		"re-arm and re-fuel each resolve the keysite",
		assess([keysite(2000, 0, 60, 500)], group(50, 70, { parent: { kind: "none" } })),
		{
			transmissions: [tx("keysite0", AMMO, 10), tx("group", AMMO, 100), tx("keysite0", FUEL, 470), tx("group", FUEL, 100)],
			groupAmmo: 100,
			groupFuel: 100,
			keysiteAmmo: [10],
			keysiteFuel: [470],
		},
	),
	kase(
		"no-keysite-found-asserts",
		"get_closest_keysite returns NULL; ASSERT (keysite)",
		assess([keysite(10, 0, 500, 500, { side: RED })], group(50, 100, { parent: { kind: "none" } })),
		{ result: "assert:keysite" },
	),
	kase(
		"no-leader-asserts-in-range",
		"group position is NULL without members; get_approx_2d_range: ASSERT (v2)",
		assess([keysite(10, 0, 500, 500)], group(50, 100, { parent: { kind: "none" }, leader: NO_POSITION })),
		{ result: "assert:v2" },
	),
	kase(
		"no-leader-no-candidates-asserts-keysite",
		"no in-use keysite: the NULL position is never used; ASSERT (keysite)",
		assess([keysite(10, 0, 500, 500, { inUse: false })], group(50, 100, { parent: { kind: "none" }, leader: NO_POSITION })),
		{ result: "assert:keysite" },
	),
	kase(
		"no-force-for-side-dereferences-null",
		"get_closest_keysite: get_local_entity_first_child (NULL force, ...) dereferences NULL",
		assess([], group(50, 100, { parent: { kind: "none" } }), { forces: [RED] }),
		{ result: "null-dereference" },
	),

	//
	// get_closest_keysite called directly (parameters assess_group_supplies does not vary)
	//
	kase(
		"closest-filters-sub-type",
		"get_closest_keysite: INT_TYPE_ENTITY_SUB_TYPE == type",
		closest([keysite(10, 0, 0, 0), keysite(5000, 0, 0, 0, { subType: FARP })], { type: FARP }),
		{ closest: "keysite1", closestRange: 5000 },
	),
	kase(
		"closest-early-out-reports-approximate-range",
		"get_closest_keysite: early out writes the get_approx_2d_range value",
		closest([keysite(300, 400, 0, 0)], {}),
		{ closest: "keysite0", closestRange: 400 + 300 / 4 },
	),
	kase(
		"closest-outside-reports-exact-range",
		"get_closest_keysite: the closest keysite's range is recomputed with get_2d_range",
		closest([keysite(3000, 4000, 0, 0)], {}),
		{ closest: "keysite0", closestRange: 5000 },
	),
	kase(
		"closest-none-reports-flt-max",
		"get_closest_keysite: best_range starts at FLT_MAX",
		closest([], {}),
		{ closest: "NULL", closestRange: FLT_MAX },
	),
	kase(
		"closest-inside-only-returns-null",
		"get_closest_keysite: outside_of_range FALSE ignores keysites beyond min_range",
		closest([keysite(3000, 0, 0, 0)], { outside: false }),
		{ closest: "NULL", closestRange: FLT_MAX },
	),
	kase(
		"closest-without-range-output",
		"get_closest_keysite: actual_range NULL",
		closest([keysite(3000, 0, 0, 0), keysite(10, 0, 0, 0)], { wantRange: false }),
		{ closest: "keysite1" },
	),
	kase(
		"closest-excludes-keysite",
		"get_closest_keysite: current_keysite != exclude_keysite",
		closest([keysite(10, 0, 0, 0), keysite(3000, 0, 0, 0)], { exclude: 0 }),
		{ closest: "keysite1", closestRange: 3000 },
	),
	kase(
		"closest-null-position-asserts",
		"get_closest_keysite: pos NULL reaches get_approx_2d_range: ASSERT (v2)",
		closest([keysite(10, 0, 0, 0)], { pos: NO_POSITION }),
		{ result: "assert:v2" },
	),
	kase(
		"closest-min-range-precondition",
		"get_closest_keysite: ASSERT (min_range > 0.0 || outside_of_range)",
		closest([keysite(10, 0, 0, 0)], { minRange: 0, outside: false }),
		{ result: "assert:min_range > 0.0 || outside_of_range" },
	),
];

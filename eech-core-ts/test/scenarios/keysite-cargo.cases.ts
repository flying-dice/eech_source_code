//
// Slice 4 behaviour matrix: keysite.c :: update_keysite_cargo, organised by
// crate-count transitions, on the real Slice 3 lifecycle (a keysite, a sector
// grid and its cargo list; creation and destruction through the original
// cargo path).
//
// Every case runs three ways: JavaScript (test/unit/keysite-cargo.test.ts),
// Lua 5.1 (test/lua/conformance.ts) and the executed original C
// (test/c-reference/keysite-cargo.cref.test.ts), where the whole output of
// TypeScript must also equal the C's.
//
// `expected` lists lines derived by hand from the C source; they must appear
// in the output in this order (an expected line also matches an output line
// that continues it after a space). `absent` lists line prefixes that must not
// appear at all (e.g. a notification the case proves is not sent).
//
// Common layout, unless a case says otherwise: heap 40; session 0, update 1,
// force0 (blue) 2, keysite0 3 (blue airbase at x 100, z 200, y 5, alive, in
// use); map 2 x 2 sectors of 1024 (sectors 4-7, all crates fall in
// sector0_0); game status INITIALISED; the crate's bounds are x -1..1,
// y -0.5..0.5, z -1.5..1.5. So a crate is 2 wide (spacing 2 + 1.0 = 3) and 3
// deep (a sub type row is 3 + 1 = 4 further in z): ammo crates stand at
// x 100, 103, 106, ... (y 5.5, z 200) and fuel crates at z 204. The first
// crate created gets index 8, and crates the original creates are labelled
// crate<index>.
//
// Airbase supply usage (ks_dbase.c): ammo -0.2, fuel -0.4, so both notify.
//
// Float bit patterns: 100 42c80000, 103 42ce0000, 106 42d40000, 109 42da0000,
// 112 42e00000, 115 42e60000, 118 42ec0000, 5.5 40b00000, 200 43480000,
// 204 434c0000, 208 43500000.
//

import { OBJECT_3D_SINGLE_CRATE } from "../../src/generated/c-constants";
import { EntityMessage, EntitySide, EntitySubTypeCargo, EntitySubTypeKeysite, GameStatusType } from "../../src/generated/c-enums";
import type { KeysiteSpec } from "./campaign-scenario";
import type { LifecycleOp, LifecycleSpec } from "./lifecycle-scenario";

export interface KeysiteCargoCase {
	id: string;
	// C provenance of the behaviour under test
	c: string;
	spec: LifecycleSpec;
	expected: string[];
	absent: string[];
}

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const RED = EntitySide.ENTITY_SIDE_RED_FORCE;

const AMMO = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO;
const FUEL = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_FUEL;
const SUPPLIES = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_SUPPLIES;

const LOW = EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES;

const MESSAGE = "message force0 keysite0";

function keysite(side: EntitySide, subType: EntitySubTypeKeysite, x: number, z: number): KeysiteSpec {
	return { side, subType, inUse: true, x, z, ammo: 100, fuel: 100 };
}

const CRATE_BOUNDS: LifecycleOp = { kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: -0.5, ymax: 0.5, zmin: -1.5, zmax: 1.5 };

const MAP: LifecycleOp = { kind: "map", xSectors: 2, zSectors: 2, sideLength: 1024 };

const RUNNING: LifecycleOp = { kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED };

function alive(label: string, value: number, y: number): LifecycleOp {
	return { kind: "keysite-state", keysite: label, alive: value, y };
}

function update(level: number, subType: number = AMMO, size = 10, label = "keysite0"): LifecycleOp {
	return { kind: "update-cargo", keysite: label, level, subType, size };
}

function spec(ops: LifecycleOp[], subType: EntitySubTypeKeysite = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE): LifecycleSpec {
	return { heap: 40, forces: [BLUE], keysites: [keysite(BLUE, subType, 100, 200)], ops: [alive("keysite0", 1, 5), CRATE_BOUNDS, MAP, RUNNING, ...ops] };
}

// transmit-create of an ammo (sub type 0) or fuel (1) crate of keysite0
function created(index: number, subType: number, xBits: string, zBits: string): string {
	return `transmit-create 4 ${index} parent 6 keysite0 int 191 ${BLUE} int 53 ${subType} vec3d 8 ${xBits} 40b00000 ${zBits} end`;
}

function ammo(index: number, xBits: string): string {
	return created(index, AMMO, xBits, "43480000");
}

function fuel(index: number, xBits: string): string {
	return created(index, FUEL, xBits, "434c0000");
}

const THREE_AMMO_CRATES = [update(35)]; // crate8 x 100, crate9 x 103, crate10 x 106; list crate10 crate9 crate8

export const KEYSITE_CARGO_CASES: KeysiteCargoCase[] = [
	// ---- creation
	{
		id: "creates-nothing-when-the-level-equals-one-crate",
		c: "creation needs temp_cargo_level > cargo_size: 10 > 10 is false; 10 <= 75 notifies",
		spec: spec([update(10)]),
		expected: [`${MESSAGE} ${LOW} ${AMMO}`, "result ok", "keysite keysite0 -"],
		absent: ["transmit-create"],
	},
	{
		id: "creates-one-crate",
		c: "15 > 10: one crate at the keysite position (y - ymin, z + 0 rows); 5 is left",
		spec: spec([update(15)]),
		expected: [ammo(8, "42c80000"), "result ok", "cargo crate8 8 1 0 1 42c80000 40b00000 43480000 keysite0 sector0_0", "keysite keysite0 crate8"],
		absent: ["message", "transmit-create 4 9"],
	},
	{
		id: "creates-several-crates-in-a-row-newest-first-in-the-list",
		c: "35: crates at x, x + 3, x + 6; cg_creat.c inserts each at the head of the cargo list",
		spec: spec(THREE_AMMO_CRATES),
		expected: [ammo(8, "42c80000"), ammo(9, "42ce0000"), ammo(10, "42d40000"), "result ok", "keysite keysite0 crate10 crate9 crate8"],
		absent: ["message", "transmit-create 4 11"],
	},
	{
		id: "an-exact-multiple-creates-one-crate-fewer",
		c: "30 from no crates: 30 > 10, 20 > 10, then 10 > 10 is false: two crates, not three",
		spec: spec([update(30)]),
		expected: [ammo(8, "42c80000"), ammo(9, "42ce0000"), "result ok", "keysite keysite0 crate9 crate8"],
		absent: ["transmit-create 4 10", "message"],
	},
	// ---- retention and no change
	{
		id: "an-exact-multiple-keeps-every-existing-crate",
		c: "three crates at 30: 20, 10 and 0 are all >= 0, so all are kept (retention is not creation); 30 <= 75 notifies",
		spec: spec([...THREE_AMMO_CRATES, update(30)]),
		expected: [ammo(10, "42d40000"), `${MESSAGE} ${LOW} ${AMMO}`, "result ok", "keysite keysite0 crate10 crate9 crate8"],
		absent: ["transmit-destroy"],
	},
	{
		id: "no-change-above-the-threshold-and-no-notification",
		c: "80 creates seven crates (80 ... 20 > 10); a second 80 keeps them (10 remains, not > 10) and 80 > 75 does not notify",
		spec: spec([update(80), update(80)]),
		expected: [ammo(14, "42ec0000"), "result ok", "keysite keysite0 crate14 crate13 crate12 crate11 crate10 crate9 crate8"],
		absent: ["transmit-create 4 15", "transmit-destroy", "message"],
	},
	{
		id: "a-steady-state-below-the-threshold-notifies-on-every-call",
		c: "three crates at 35: all kept, 5 is not > 10, 35 <= 75: notifies; no latch, so a second call notifies again",
		spec: spec([...THREE_AMMO_CRATES, update(35), update(35)]),
		expected: [`${MESSAGE} ${LOW} ${AMMO}`, `${MESSAGE} ${LOW} ${AMMO}`, "result ok", "keysite keysite0 crate10 crate9 crate8"],
		absent: ["transmit-destroy"],
	},
	// ---- destruction
	{
		id: "destroys-the-oldest-crate-first",
		c: "three crates at 25: the walk meets crate10 (15), crate9 (5), crate8 (-5): the oldest is destroyed; destroying does not stop the notification",
		spec: spec([...THREE_AMMO_CRATES, update(25)]),
		expected: ["transmit-destroy crate8", `${MESSAGE} ${LOW} ${AMMO}`, "result ok", "keysite keysite0 crate10 crate9", "sector sector0_0 4 0 0 crate10 crate9"],
		absent: ["transmit-destroy crate9", "transmit-destroy crate10"],
	},
	{
		id: "destroys-several-crates-in-list-order",
		c: "three crates at 12: crate10 kept (2); crate9 (-8) and crate8 (-18) destroyed, the successor taken before each destruction",
		spec: spec([...THREE_AMMO_CRATES, update(12)]),
		expected: ["transmit-destroy crate9", "transmit-destroy crate8", `${MESSAGE} ${LOW} ${AMMO}`, "result ok", "keysite keysite0 crate10"],
		absent: ["transmit-destroy crate10"],
	},
	{
		id: "destroys-every-crate",
		c: "three crates at 5: -5, -15, -25: all destroyed, newest first",
		spec: spec([...THREE_AMMO_CRATES, update(5)]),
		expected: ["transmit-destroy crate10", "transmit-destroy crate9", "transmit-destroy crate8", `${MESSAGE} ${LOW} ${AMMO}`, "result ok", "keysite keysite0 -"],
		absent: ["transmit-create 4 11"],
	},
	{
		id: "a-later-creation-lands-on-a-surviving-crate",
		c: "positions are by count: after crate8 (x 100) is destroyed, two crates are kept, so the next crate goes to x 106, where crate10 already stands; it reuses index 8",
		spec: spec([...THREE_AMMO_CRATES, update(25), update(35)]),
		expected: ["transmit-destroy crate8", ammo(8, "42d40000"), "result ok", "cargo crate8 8 1 0 1 42d40000", "cargo crate10 10 1 0 1 42d40000", "keysite keysite0 crate8 crate10 crate9"],
		absent: [],
	},
	// ---- notification
	{
		id: "a-missing-crate-is-materialised-instead-of-notifying",
		c: "one crate at 25: 15 remains > 10, so a crate is created, and the else-if notification is not reached although 25 <= 75",
		spec: spec([update(15), update(25)]),
		expected: [ammo(8, "42c80000"), ammo(9, "42ce0000"), "result ok", "keysite keysite0 crate9 crate8"],
		absent: ["message"],
	},
	{
		id: "the-threshold-itself-notifies",
		c: "seven crates at 75: 5 remains, nothing to create, and 75 <= KEYSITE_SUPPLY_REQUEST_THRESHOLD (75.0) notifies",
		spec: spec([update(80), update(75)]),
		expected: [ammo(14, "42ec0000"), `${MESSAGE} ${LOW} ${AMMO}`, "result ok"],
		absent: ["transmit-destroy"],
	},
	{
		id: "just-above-the-threshold-does-not-notify",
		c: "seven crates at 75.00001f: nothing to create and 75.00001 > 75: no notification",
		spec: spec([update(80), update(75.00001)]),
		expected: [ammo(14, "42ec0000"), "result ok"],
		absent: ["message", "transmit-destroy"],
	},
	{
		id: "fuel-notifies-with-its-sub-type",
		c: "fuel at 5: nothing to create; the airbase's fuel usage (-0.4) is negative: the message argument is ENTITY_SUB_TYPE_CARGO_FUEL",
		spec: spec([update(5, FUEL)]),
		expected: [`${MESSAGE} ${LOW} ${FUEL}`, "result ok"],
		absent: ["transmit-create"],
	},
	{
		id: "a-factory-never-notifies",
		c: "factory usage: ammo +1.0, fuel 0.0 (neither < 0.0)",
		spec: spec([update(5, AMMO), update(5, FUEL)], EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY),
		expected: ["result ok"],
		absent: ["message"],
	},
	{
		id: "a-power-station-notifies-for-fuel-only",
		c: "power station usage: ammo 0.0 (no), fuel -0.5 (yes)",
		spec: spec([update(5, AMMO), update(5, FUEL)], EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_POWER_STATION),
		expected: [`${MESSAGE} ${LOW} ${FUEL}`, "result ok"],
		absent: [`${MESSAGE} ${LOW} ${AMMO}`],
	},
	{
		id: "no-force-for-the-side-fails-the-receiver-assert",
		c: "get_local_force_entity returns NULL for a red keysite with only a blue force: notify_local_entity ASSERT (receiver)",
		spec: { heap: 40, forces: [BLUE], keysites: [keysite(RED, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, 100, 200)], ops: [alive("keysite0", 1, 5), CRATE_BOUNDS, MAP, RUNNING, update(5)] },
		expected: ["result assert receiver"],
		absent: ["message"],
	},
	// ---- ammo and fuel independently
	{
		id: "ammo-and-fuel-stand-in-separate-rows",
		c: "fuel crates are in the row z + 1 * ((zmax - zmin) + 1) = 204; ammo crates ignore them: not counted, and they do not move x",
		spec: spec([update(25, FUEL), update(15, AMMO)]),
		expected: [fuel(8, "42c80000"), fuel(9, "42ce0000"), ammo(10, "42c80000"), "result ok", "keysite keysite0 crate10 crate9 crate8"],
		absent: ["transmit-destroy", "message"],
	},
	{
		id: "destroying-ammo-leaves-fuel-crates",
		c: "ammo at 0 destroys only ammo crates; fuel crates are skipped in the walk",
		spec: spec([update(25, FUEL), update(15, AMMO), update(0, AMMO)]),
		expected: ["transmit-destroy crate10", "result ok", "keysite keysite0 crate9 crate8"],
		absent: ["transmit-destroy crate9", "transmit-destroy crate8"],
	},
	{
		id: "supplies-crates-use-the-third-row-and-never-notify",
		c: "ENTITY_SUB_TYPE_CARGO_SUPPLIES (2): z + 2 * 4 = 208; the notification switch has no arm for it",
		spec: spec([update(15, SUPPLIES), update(5, SUPPLIES)]),
		expected: [created(8, SUPPLIES, "42c80000", "43500000"), "transmit-destroy crate8", "result ok"],
		absent: ["message"],
	},
	// ---- guards
	{
		id: "nothing-happens-while-the-game-is-initialising",
		c: "get_game_status () == GAME_STATUS_INITIALISING returns first (a restored session brings its packed cargo)",
		spec: spec([{ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISING }, update(50), update(5)]),
		expected: ["result ok", "keysite keysite0 -"],
		absent: ["transmit-create", "message"],
	},
	{
		id: "an-uninitialised-game-still-updates",
		c: "only INITIALISING returns: the zero-initialised status (UNINITIALISED) proceeds",
		spec: spec([{ kind: "game-status", status: GameStatusType.GAME_STATUS_UNINITIALISED }, update(15)]),
		expected: [ammo(8, "42c80000"), "result ok"],
		absent: [],
	},
	{
		id: "a-dead-keysite-does-nothing",
		c: "!raw->alive returns",
		spec: spec([alive("keysite0", 0, 5), update(50), update(5)]),
		expected: ["result ok", "keysite keysite0 -"],
		absent: ["transmit-create", "message"],
	},
	{
		id: "a-keysite-not-in-use-does-nothing",
		c: "!raw->in_use returns",
		spec: { heap: 40, forces: [BLUE], keysites: [{ ...keysite(BLUE, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, 100, 200), inUse: false }], ops: [alive("keysite0", 1, 5), CRATE_BOUNDS, MAP, RUNNING, update(50), update(5)] },
		expected: ["result ok", "keysite keysite0 -"],
		absent: ["transmit-create", "message"],
	},
	{
		id: "the-alive-bit-field-keeps-the-low-bit",
		c: "alive : 1 stores 2 as 0: the keysite is dead",
		spec: spec([alive("keysite0", 2, 5), update(50)]),
		expected: ["result ok", "keysite keysite0 -"],
		absent: ["transmit-create"],
	},
	// ---- positioning and bounds
	{
		id: "a-zero-size-crate-is-spaced-by-one",
		c: "xmin = xmax = 0: spacing (0 + 1.0) = 1; ymin 0 leaves y; depth 0: the fuel row is z + 1",
		spec: spec([{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: 0, xmax: 0, ymin: 0, ymax: 0, zmin: 0, zmax: 0 }, update(25, FUEL)]),
		expected: [
			`transmit-create 4 8 parent 6 keysite0 int 191 ${BLUE} int 53 ${FUEL} vec3d 8 42c80000 40a00000 43490000 end`,
			`transmit-create 4 9 parent 6 keysite0 int 191 ${BLUE} int 53 ${FUEL} vec3d 8 42ca0000 40a00000 43490000 end`,
			"result ok",
		],
		absent: [],
	},
	{
		id: "fractional-bounds-and-position-round-toward-zero",
		c: "keysite x 0.1, bounds 0.05 wide: every position step is (x + (w + 1.0)) truncated to float (checked against the C)",
		spec: {
			heap: 40,
			forces: [BLUE],
			keysites: [keysite(BLUE, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, 0.1, 0.3)],
			ops: [alive("keysite0", 1, 0.7), { kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -0.025, xmax: 0.025, ymin: -0.1, ymax: 0.1, zmin: -0.3, zmax: 0.3 }, MAP, RUNNING, update(55, FUEL)],
		},
		expected: ["transmit-create 4 8", "transmit-create 4 12", "result ok"],
		absent: ["transmit-create 4 13"],
	},
	{
		id: "a-crate-row-crosses-a-sector-boundary",
		c: "crates are inserted into the sector of their position (cg_creat.c): a row from x 1018 crosses into sector1_0 at 1024",
		spec: { heap: 40, forces: [BLUE], keysites: [keysite(BLUE, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, 1018, 200)], ops: [alive("keysite0", 1, 5), CRATE_BOUNDS, MAP, RUNNING, update(35)] },
		expected: ["result ok", "cargo crate10 10 1 0 1 44800000", "sector sector0_0 4 0 0 crate9 crate8", "sector sector1_0 5 1 0 crate10"],
		absent: [],
	},
	{
		id: "a-negative-size-refills-the-remainder-after-destroying",
		c: "size -10 at -25: crate10 (-15) and crate9 (-5) are destroyed without advancing x, crate8 (5) is kept (x 103); 5 > -10 forever, so crates are created from x 103 until the heap is full",
		spec: { ...spec([...THREE_AMMO_CRATES, update(-25, AMMO, -10)]), heap: 17 },
		expected: [
			"transmit-destroy crate10",
			"transmit-destroy crate9",
			// the free list is last-in first-out: index 9 (freed last) is reused first
			ammo(9, "42ce0000"),
			ammo(10, "42d40000"),
			"result fatal EN_CREATE: CREATE_CLIENT_SERVER_ENTITY : unable to create entity %s. Limit of %d reached",
		],
		absent: ["message"],
	},
	// ---- RTZ-sensitive counting (rounding to nearest would differ)
	{
		id: "fractional-level-and-size-count-toward-zero",
		c: "0.3f / 0.1f: 0.3 - 0.1 truncates below 0.2, so the second remainder is not > 0.1: two crates (three to nearest)",
		spec: spec([update(0.3, AMMO, 0.1)]),
		expected: [ammo(8, "42c80000"), ammo(9, "42ce0000"), "result ok"],
		absent: ["transmit-create 4 10"],
	},
	{
		id: "a-larger-fractional-count-toward-zero",
		c: "7.7f / 1.1f: six crates toward zero (seven to nearest)",
		spec: spec([update(7.7, AMMO, 1.1)]),
		expected: ["transmit-create 4 13", "result ok"],
		absent: ["transmit-create 4 14"],
	},
];

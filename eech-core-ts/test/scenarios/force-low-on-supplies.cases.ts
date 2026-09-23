//
// Slice 5a behaviour matrix: fc_msgs.c :: response_to_force_low_on_supplies,
// run socially on the frozen Slice 1-4 entity graph (issue #12).
//
// Nothing about the requester, supplier or cargo is mocked. Keysites are
// restored raw, crates come from the real Slice 4 update_keysite_cargo, and
// the message comes from a real sender: update_keysite_cargo (keysites) or
// assess_group_supplies (groups). Existing tasks are restored raw onto their
// objective's LIST_TYPE_TASK_DEPENDENT list, as a saved game holds them.
// The only new observation is the create_supply_task call (Slice 5b), printed
// as
//
//   create-supply-task <requester> <supplier> <cargo> <movement type> <priority bits> <start keysite> <end keysite>
//
// with MOVEMENT_TYPE_AIR (1), task_database [SUPPLY].task_priority (4.0f,
// 40800000) and NULL, NULL for every call this response makes.
//
// Every case runs three ways: JavaScript (test/unit/force-low-on-supplies.test.ts),
// Lua 5.1 (test/lua/conformance.ts) and the executed original C
// (test/c-reference/force-low-on-supplies.cref.test.ts), where TypeScript's
// whole output must also equal the C's.
//
// `expected` lists lines derived by hand from the C source. They must appear
// in the output in this order; a line also matches an output line that
// continues it after a space. `absent` lists line prefixes that must not appear.
//
// Common layout: heap 200; forces as listed (force0, force1); keysites as
// listed (keysite0 ...); crate bounds x -1..1, y -0.5..0.5, z -1.5..1.5; map
// 4 x 4 sectors of 8192 (0 .. 32768 in x and z); game status INITIALISED.
// Heap indices: session 0, update 1, the forces, the keysites, the 16
// sectors, then entities in the order the operations create them. Stocking a
// keysite with update_keysite_cargo (level 35, size 10) creates three crates of
// that sub type; the cargo list holds them newest first.
//
// Keysite sub types: AIRBASE 0, FACTORY 2, FARP 3, OIL_REFINERY 7. A FARP's
// supply usage is negative for ammo and fuel (ks_dbase.c), so a FARP at level
// 5 (no crate: 5 > 10 is false) always notifies; a factory's and a refinery's
// is not, so stocking them never notifies.
//

import { OBJECT_3D_SINGLE_CRATE } from "../../src/generated/c-constants";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeCargo,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntitySubTypeWaypoint,
	GameStatusType,
	TaskStateType,
} from "../../src/generated/c-enums";
import type { KeysiteSpec, PositionSpec } from "./campaign-scenario";
import type { LifecycleOp, LifecycleSpec } from "./lifecycle-scenario";

export interface ForceLowOnSuppliesCase {
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

const AIRBASE = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE;
const FACTORY = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY;
const FARP = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP;
const REFINERY = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY;

const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;
const ENGAGE = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE;

const UNASSIGNED = TaskStateType.TASK_STATE_UNASSIGNED;
const ASSIGNED = TaskStateType.TASK_STATE_ASSIGNED;
const COMPLETED = TaskStateType.TASK_STATE_COMPLETED;

// a group that resupplies through a supply task (gp_dbase.c: RESUPPLY_SOURCE_GROUP)
const FRONTLINE = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE;

const LOW = EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES;

const CREATE = "create-supply-task";

// MOVEMENT_TYPE_AIR, task_database [ENTITY_SUB_TYPE_TASK_SUPPLY].task_priority (4.0f), start and end keysite NULL
const CALL_TAIL = "1 40800000 NULL NULL";

interface Site {
	side: EntitySide;
	subType: EntitySubTypeKeysite;
	x: number;
	z: number;
	inUse?: boolean;
}

//
// Builds one scenario and tracks heap indices, so that expectations can name
// the crates the original creates (crate<index>).
//
class Scenario {
	public readonly ops: LifecycleOp[] = [];

	private next: number;

	public constructor(
		private readonly forces: EntitySide[],
		private readonly sites: Site[],
	) {
		this.next = 2 + forces.length + sites.length + 16;
		this.ops.push(
			{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: -0.5, ymax: 0.5, zmin: -1.5, zmax: 1.5 },
			{ kind: "map", xSectors: 4, zSectors: 4, sideLength: 8192 },
			{ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED },
		);
	}

	// update_keysite_cargo (keysite, 35, sub_type, 10) from no crates of that
	// sub type: three crates; returns their labels, newest (list head) first
	public stock(keysite: number, subType: number): string[] {
		this.ops.push({ kind: "update-cargo", keysite: `keysite${keysite}`, level: 35, subType, size: 10 });
		const labels = [`crate${this.next}`, `crate${this.next + 1}`, `crate${this.next + 2}`];
		this.next += 3;
		return labels.reverse();
	}

	public task(label: string, objective: string, subType: number, side: number, state: number, userData: number): void {
		this.ops.push({ kind: "task", label, objective, subType, side, state, userData });
		this.next += 1;
	}

	public waypoint(label: string, dependent: string, subType: number): void {
		this.ops.push({ kind: "waypoint", label, dependent, subType });
		this.next += 1;
	}

	public group(label: string, ammo: number, fuel: number, leader: PositionSpec): void {
		this.ops.push({ kind: "restore-group", label, subType: FRONTLINE, side: BLUE, ammo, fuel, parent: "NULL", busy: false, leader });
		this.next += leader.kind === "at" ? 2 : 1;
	}

	public op(op: LifecycleOp): void {
		this.ops.push(op);
	}

	// the notification under test, from a real sender
	public request(keysite: number, level: number, subType: number): void {
		this.ops.push({ kind: "observe-supply-tasks" }, { kind: "update-cargo", keysite: `keysite${keysite}`, level, subType, size: 10 });
	}

	public assess(group: string): void {
		this.ops.push({ kind: "observe-supply-tasks" }, { kind: "assess-group", group });
	}

	public spec(): LifecycleSpec {
		const keysites: KeysiteSpec[] = this.sites.map((s) => ({
			side: s.side,
			subType: s.subType,
			inUse: s.inUse !== false,
			x: s.x,
			z: s.z,
			ammo: 100,
			fuel: 100,
		}));

		return { heap: 200, forces: this.forces, keysites, ops: [...keysites.map((_, i): LifecycleOp => ({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: 0 })), ...this.ops] };
	}
}

function message(sender: string, subType: number, force = "force0"): string {
	return `message ${force} ${sender} ${LOW} ${subType}`;
}

function call(requester: string, supplier: string, cargo: string): string {
	return `${CREATE} ${requester} ${supplier} ${cargo} ${CALL_TAIL}`;
}

// a FARP (keysite0) at (8000, 16000) and a factory (keysite1) 20 km east with three ammo crates
function farpWithFactory(): { s: Scenario; crates: string[] } {
	const s = new Scenario([BLUE], [
		{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
		{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
	]);
	return { s, crates: s.stock(1, AMMO) };
}

function build(): ForceLowOnSuppliesCase[] {
	const cases: ForceLowOnSuppliesCase[] = [];

	// ---- guard

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 }]);
		s.stock(0, AMMO);
		s.op({ kind: "game-status", status: GameStatusType.GAME_STATUS_UNINITIALISED });
		s.request(0, 35, AMMO);
		cases.push({
			id: "returns-unless-the-game-is-initialised",
			c: "get_game_status () != GAME_STATUS_INITIALISED -> return FALSE (update_keysite_cargo itself only stops while INITIALISING)",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 }]);
		s.stock(0, AMMO);
		s.op({ kind: "comms-model", model: CommsModelType.COMMS_MODEL_CLIENT });
		s.request(0, 35, AMMO);
		cases.push({
			id: "returns-on-a-comms-client",
			c: "get_comms_model () == COMMS_MODEL_CLIENT -> return FALSE",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	// ---- F2: an airbase is its own supplier

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 }]);
		const crates = s.stock(0, AMMO);
		s.request(0, 35, AMMO);
		cases.push({
			id: "an-airbase-supplies-itself-from-its-newest-crate",
			c: "get_closest_keysite (AIRBASE, side, own position, 10 km, ..., exclude NULL) returns the requester at range 0; the cargo walk takes the head of its own cargo list",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite0", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 19000, z: 16000 },
		]);
		s.stock(1, AMMO);
		const own = s.stock(0, AMMO);
		s.request(0, 35, AMMO);
		cases.push({
			id: "an-airbase-supplies-itself-even-beside-a-stocked-factory",
			c: "the factory 3 km away is found (range 3000), then airbase_actual_range 0 < 3000 replaces it with the requester",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite0", own[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 19000, z: 16000 },
		]);
		s.stock(1, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "an-airbase-without-crates-requests-nothing-even-beside-a-stocked-factory",
			c: "self-selection still wins (range 0); the requester holds no ammo crate, so cargo is NULL and create_supply_task is not called",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 }]);
		const crates = s.stock(0, FUEL);
		s.request(0, 35, FUEL);
		cases.push({
			id: "an-airbase-supplies-itself-with-fuel",
			c: "case ENTITY_SUB_TYPE_CARGO_FUEL: the same airbase comparison",
			spec: s.spec(),
			expected: [message("keysite0", FUEL), call("keysite0", "keysite0", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: AIRBASE, x: 16000, z: 16000 }]);
		const ammo = s.stock(0, AMMO);
		s.stock(0, FUEL);
		s.request(0, 35, AMMO);
		cases.push({
			id: "the-cargo-walk-skips-crates-of-the-other-sub-type",
			c: "the fuel crates are newer (list head); the walk passes them to the newest ammo crate",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite0", ammo[0]), "result ok"],
			absent: [],
		});
	}

	// ---- supplier selection for a requester that is not an airbase

	{
		const { s, crates } = farpWithFactory();
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-farp-is-supplied-by-a-factory",
			c: "ammo: get_closest_keysite (FACTORY) beyond 10 km returns the closest, exact range 20000; no airbase (FLT_MAX)",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: REFINERY, x: 28000, z: 16000 },
		]);
		const crates = s.stock(1, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "ammo-falls-back-to-an-oil-refinery",
			c: "ammo: no factory, so get_closest_keysite (OIL_REFINERY)",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
			{ side: BLUE, subType: REFINERY, x: 13000, z: 16000 },
		]);
		const factory = s.stock(1, AMMO);
		s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "ammo-prefers-a-factory-over-a-closer-refinery",
			c: "ammo: the refinery is searched only when no factory is found",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", factory[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 13000, z: 16000 },
			{ side: BLUE, subType: REFINERY, x: 28000, z: 16000 },
		]);
		s.stock(1, FUEL);
		const refinery = s.stock(2, FUEL);
		s.request(0, 5, FUEL);
		cases.push({
			id: "fuel-prefers-a-refinery-over-a-closer-factory",
			c: "fuel: the factory is searched only when no oil refinery is found",
			spec: s.spec(),
			expected: [message("keysite0", FUEL), call("keysite0", "keysite2", refinery[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
		]);
		const crates = s.stock(1, FUEL);
		s.request(0, 5, FUEL);
		cases.push({
			id: "fuel-falls-back-to-a-factory",
			c: "fuel: no oil refinery, so get_closest_keysite (FACTORY)",
			spec: s.spec(),
			expected: [message("keysite0", FUEL), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
			{ side: BLUE, subType: AIRBASE, x: 23000, z: 16000 },
		]);
		s.stock(1, AMMO);
		const airbase = s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-strictly-closer-airbase-replaces-the-factory",
			c: "airbase_actual_range 15000 < factory_actual_range 20000",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite2", airbase[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 16000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
			{ side: BLUE, subType: AIRBASE, x: 4000, z: 16000 },
		]);
		const factory = s.stock(1, AMMO);
		s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "an-equidistant-airbase-does-not-replace-the-factory",
			c: "both exact ranges are 12000 (get_2d_range); 12000 < 12000 is false",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", factory[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 17000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 10000, z: 16000 },
		]);
		const first = s.stock(1, AMMO);
		s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "the-first-listed-factory-within-10-km-wins-over-a-nearer-one",
			c: "get_closest_keysite returns the first keysite in LIST_TYPE_KEYSITE_FORCE order with range <= 10 km (9000), not the nearest (2000)",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", first[0]), "result ok"],
			absent: [],
		});
	}

	{
		// FARP at (8000, 16000). Airbase offset (9500, 2000): approximate range
		// 9500 + 2000 / 4 = 10000 <= 10 km, an early exit reporting 10000 (exact
		// 9708.2). Factory offset (9800, 1000): approximate 10050 > 10 km, so the
		// exact range 9850.9 is reported. 10000 < 9850.9 is false: the factory is
		// kept although the airbase is nearer.
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 17800, z: 17000 },
			{ side: BLUE, subType: AIRBASE, x: 17500, z: 18000 },
		]);
		const factory = s.stock(1, AMMO);
		s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "an-approximate-airbase-range-keeps-a-farther-factory",
			c: "airbase_actual_range is get_approx_2d_range (early exit), factory_actual_range is get_2d_range; the comparison mixes them",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", factory[0]), "result ok"],
			absent: [],
		});
	}

	{
		// FARP at (8000, 8000). Airbase offset (7900, 7900): approximate range
		// 7900 + 7900 / 4 = 9875 <= 10 km (exact 11172.3). Factory offset
		// (10500, 0): approximate and exact 10500. 9875 < 10500: the airbase
		// replaces the factory although it is farther.
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 8000 },
			{ side: BLUE, subType: FACTORY, x: 18500, z: 8000 },
			{ side: BLUE, subType: AIRBASE, x: 15900, z: 15900 },
		]);
		s.stock(1, AMMO);
		const airbase = s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "an-approximate-airbase-range-replaces-a-nearer-factory",
			c: "the approximate range underestimates a diagonal: 9875 < 10500 though the airbase is 11172 away",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite2", airbase[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: FARP, x: 8000, z: 16000 }]);
		s.request(0, 5, AMMO);
		cases.push({
			id: "no-supplier-no-task",
			c: "no factory, refinery or airbase: both ranges FLT_MAX, factory NULL",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
		]);
		s.stock(1, FUEL);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-supplier-without-that-cargo-gives-no-task",
			c: "the factory holds only fuel crates: the ammo walk ends with cargo NULL",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const s = new Scenario([BLUE, RED], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: RED, subType: FACTORY, x: 11000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
		]);
		s.stock(1, AMMO);
		const blue = s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "only-keysites-of-the-requesters-side-supply",
			c: "get_closest_keysite walks the force of the requester's side (INT_TYPE_SIDE of the sender)",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite2", blue[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FARP, x: 8000, z: 16000 },
			{ side: BLUE, subType: FACTORY, x: 11000, z: 16000, inUse: false },
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
		]);
		const inUse = s.stock(2, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-keysite-not-in-use-does-not-supply",
			c: "get_closest_keysite skips keysites whose INT_TYPE_IN_USE is 0",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite2", inUse[0]), "result ok"],
			absent: [],
		});
	}

	// ---- the duplicate-task decision

	{
		const { s } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, BLUE, UNASSIGNED, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "an-active-supply-task-for-the-same-cargo-blocks",
			c: "entity_is_object_of_task counts it (1); the walk finds TASK_USER_DATA == sub_type -> return FALSE",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const { s, crates } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, BLUE, UNASSIGNED, FUEL);
		s.request(0, 5, AMMO);
		cases.push({
			id: "an-active-supply-task-for-the-other-cargo-does-not-block",
			c: "counted (1), but the walk finds no TASK_USER_DATA == AMMO",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const { s, crates } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, BLUE, COMPLETED, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-completed-supply-task-alone-does-not-block",
			c: "entity_is_object_of_task skips TASK_STATE_COMPLETED: count 0, the walk is not entered",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const { s } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, BLUE, COMPLETED, AMMO);
		s.task("t1", "keysite0", SUPPLY, BLUE, UNASSIGNED, FUEL);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-completed-task-blocks-once-any-supply-task-is-active",
			c: "the active fuel task makes the count 1; the walk does not check the state and matches the completed ammo task -> return FALSE",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const { s, crates } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, RED, UNASSIGNED, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "another-sides-supply-task-alone-does-not-block",
			c: "entity_is_object_of_task counts only tasks of the sender's side: count 0",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const { s } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, RED, UNASSIGNED, AMMO);
		s.task("t1", "keysite0", SUPPLY, BLUE, ASSIGNED, FUEL);
		s.request(0, 5, AMMO);
		cases.push({
			id: "another-sides-task-blocks-once-an-own-supply-task-is-assigned",
			c: "the assigned blue fuel task counts (1); the walk does not check the side and matches the red ammo task",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const { s, crates } = farpWithFactory();
		s.task("t0", "keysite0", ENGAGE, BLUE, UNASSIGNED, AMMO);
		s.task("t1", "keysite0", SUPPLY, BLUE, UNASSIGNED, FUEL);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-task-of-another-type-never-matches",
			c: "count 1 (the fuel supply task); the walk checks ENTITY_SUB_TYPE_TASK_SUPPLY before TASK_USER_DATA, so the engage task (user data 0) is passed",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	// ---- route waypoints on the requester's LIST_TYPE_TASK_DEPENDENT list
	//      (croute.c links a route's waypoints to their dependents)

	{
		const { s, crates } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, BLUE, UNASSIGNED, FUEL);
		s.waypoint("w0", "keysite0", EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-supply-route-waypoint-is-passed",
			c: "entity_is_object_of_task skips non-tasks (count 1, the fuel task); the walk reads the drop-off waypoint's sub type (6), which is not ENTITY_SUB_TYPE_TASK_SUPPLY (21)",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const { s } = farpWithFactory();
		s.task("t0", "keysite0", SUPPLY, BLUE, UNASSIGNED, FUEL);
		s.waypoint("w0", "keysite0", EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_RECON);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-recon-waypoint-reads-as-an-ammo-supply-task",
			c: "the walk does not check the entity type: ENTITY_SUB_TYPE_WAYPOINT_RECON is 21, the value of ENTITY_SUB_TYPE_TASK_SUPPLY, and a waypoint's FLOAT_TYPE_TASK_USER_DATA is en_float.c's default 0.0, which equals AMMO -> return FALSE",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const { s, crates } = farpWithFactory();
		s.waypoint("w0", "keysite0", EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_RECON);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-recon-waypoint-alone-does-not-block",
			c: "entity_is_object_of_task counts tasks only (count 0), so the walk is not entered",
			spec: s.spec(),
			expected: [message("keysite0", AMMO), call("keysite0", "keysite1", crates[0]), "result ok"],
			absent: [],
		});
	}

	// ---- group senders (group.c :: assess_group_supplies, RESUPPLY_SOURCE_GROUP)

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: FACTORY, x: 16000, z: 16000 }]);
		const crates = s.stock(0, AMMO);
		s.group("g0", 50, 100, { kind: "at", x: 20000, z: 16000 });
		s.assess("g0");
		cases.push({
			id: "a-group-is-supplied-by-a-factory",
			c: "ammo 50 < 100: the group's position (its leader's) is the search origin; the factory 4 km away is the supplier",
			spec: s.spec(),
			expected: [message("g0", AMMO), call("g0", "keysite0", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: FACTORY, x: 16000, z: 16000 }]);
		const crates = s.stock(0, FUEL);
		s.group("g0", 100, 50, { kind: "at", x: 20000, z: 16000 });
		s.assess("g0");
		cases.push({
			id: "a-group-low-on-fuel-is-supplied-with-fuel",
			c: "ammo 100, fuel 50 < 100: FUEL; no refinery, so the factory",
			spec: s.spec(),
			expected: [message("g0", FUEL), call("g0", "keysite0", crates[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [
			{ side: BLUE, subType: FACTORY, x: 28000, z: 16000 },
			{ side: BLUE, subType: AIRBASE, x: 13000, z: 16000 },
		]);
		s.stock(0, AMMO);
		const airbase = s.stock(1, AMMO);
		s.group("g0", 50, 100, { kind: "at", x: 8000, z: 16000 });
		s.assess("g0");
		cases.push({
			id: "a-group-near-an-airbase-is-supplied-by-it",
			c: "airbase 5000 (early exit) < factory 20000",
			spec: s.spec(),
			expected: [message("g0", AMMO), call("g0", "keysite1", airbase[0]), "result ok"],
			absent: [],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: FACTORY, x: 16000, z: 16000 }]);
		s.stock(0, AMMO);
		s.group("g0", 50, 100, { kind: "at", x: 20000, z: 16000 });
		s.task("t0", "g0", SUPPLY, BLUE, ASSIGNED, AMMO);
		s.assess("g0");
		cases.push({
			id: "a-group-with-a-supply-task-for-that-cargo-is-not-resupplied-again",
			c: "the duplicate-task decision on the group's own LIST_TYPE_TASK_DEPENDENT list",
			spec: s.spec(),
			expected: [message("g0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: FACTORY, x: 16000, z: 16000 }]);
		s.stock(0, AMMO);
		s.group("g0", 50, 100, { kind: "none" });
		s.assess("g0");
		cases.push({
			id: "a-group-without-members-has-no-position",
			c: "gp_vec3d.c returns NULL without a leader; get_closest_keysite passes it to get_approx_2d_range, which asserts v2",
			spec: s.spec(),
			expected: [message("g0", AMMO), "result assert v2"],
			absent: [CREATE],
		});
	}

	{
		const s = new Scenario([BLUE], [{ side: BLUE, subType: FARP, x: 16000, z: 16000 }]);
		s.group("g0", 50, 100, { kind: "none" });
		s.assess("g0");
		cases.push({
			id: "a-group-without-members-and-no-supplier-requests-nothing",
			c: "with no keysite of the searched types the NULL position is never read: factory NULL",
			spec: s.spec(),
			expected: [message("g0", AMMO), "result ok"],
			absent: [CREATE],
		});
	}

	return cases;
}

export const FORCE_LOW_ON_SUPPLIES_CASES: ForceLowOnSuppliesCase[] = build();

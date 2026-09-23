//
// Slice 6a behaviour matrix: assign.c :: assign_keysite_tasks, the decision
// (issue #16), up to assign_primary_task_to_group, the boundary.
//
// Nothing is mocked. Keysites, groups, members, pilots and tasks are restored
// raw, as a saved game holds them (a group's member count is gp_pack.c ::
// unpack_local_data's raw restore; its live maintenance is not ported); the
// social chain constructs its task through the real Slice 4 - 5b path. Every
// case ends with either
//
//   result boundary assign_primary_task_to_group <group> <task>
//
// (the selected pair: production throws UnportedBoundaryError there, the C
// harness traps the same call) or "result ok" when no task is assigned.
//
// Every case runs three ways: JavaScript (test/unit/supply-task-assignment.test.ts),
// Lua 5.1 (test/lua/conformance.ts) and the executed original C
// (test/c-reference/supply-task-assignment.cref.test.ts), where TypeScript's
// whole output must also equal the C's.
//
// `expected` lists lines derived by hand from the C source, in order; a line
// also matches an output line that continues it after a space. `absent` lists
// line prefixes that must not appear.
//
// Common layout (matrix): heap 200; force0 BLUE; keysite0 an AIRBASE at
// (1000, 16000) (ks_dbase.c: assign 3 tasks, reserve 2); keysite1 a FARP at
// (9000, 16000) (assign 1, reserve 1). Groups are medium lift transport
// helicopters (gp_dbase.c: minimum idle count 0) led by a UH-60 (cruise
// velocity 85 knots, 43.80644226... m/s) at their keysite, member count 1,
// on the force's air registry. Tasks are critical SUPPLY tasks (ts_dbase.c:
// TASK_CATEGORY_SUPPORT, minimum member count 1), priority 4, expiring in
// 1200 s. With the compiled databases every nonzero group-to-task suitability
// is exactly 1.0 (suitable.c: a group passes the critical factors only with
// at least the task's strengths, so each min (a / b, 1.0) is 1.0), so the
// least suitable group is the first qualifying one in LIST_TYPE_KEYSITE_GROUP
// order.
//

import {
	EntitySide,
	EntitySubTypeAircraft,
	EntitySubTypeCargo,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntityType,
	GameStatusType,
	GameType,
	KeysiteUsableState,
	TaskCategoryType,
} from "../../src/generated/c-enums";
import { OBJECT_3D_SINGLE_CRATE } from "../../src/generated/c-constants";
import type { KeysiteSpec } from "./campaign-scenario";
import type { LifecycleOp, LifecycleSpec } from "./lifecycle-scenario";

export interface SupplyTaskAssignmentCase {
	id: string;
	// C provenance of the behaviour under test
	c: string;
	spec: LifecycleSpec;
	expected: string[];
	absent: string[];
}

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const RED = EntitySide.ENTITY_SIDE_RED_FORCE;

const AIRBASE = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE;
const FARP = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP;
const FACTORY = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY;

const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;
const ESCORT = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ESCORT;
const TROOP_INSERTION = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_TROOP_INSERTION;
const BAI = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_BAI;

const SUPPORT = TaskCategoryType.TASK_CATEGORY_SUPPORT;
const STRIKE = TaskCategoryType.TASK_CATEGORY_STRIKE;
const RECON = TaskCategoryType.TASK_CATEGORY_RECON;

const MLT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER;
const ATTACK = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER;
const MARINE_ATTACK = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MARINE_ATTACK_HELICOPTER;
const ASSAULT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_HELICOPTER;
const RECON_HELICOPTER = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_RECON_HELICOPTER;
const INTERCEPTOR = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_CARRIER_BORNE_INTERCEPTOR;
const FIGHTER = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MULTI_ROLE_FIGHTER;
const ASSAULT_SHIP = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP;

const UH60 = EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK;
const FA18 = EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_FA18_HORNET;

const BOUNDARY = "result boundary assign_primary_task_to_group";

// ETA at 85 knots from 1000 m to 1000 + d on the x axis: eta = d / 43.80644226...f,
// truncated to float. These distances are consecutive floats (steps of 2^-8 m):
//   52567.73046875 -> eta 1200 - 2^-13 (just below the 1200 s expiry)
//   52567.734375   -> eta 1200 exactly (eta > expire is false: eligible)
//   52567.73828125 -> eta 1200 + 2^-13 (just above: rejected)
const D_BELOW = 52567.73046875;
const D_EQUAL = 52567.734375;
const D_ABOVE = 52567.73828125;

interface Site {
	subType: EntitySubTypeKeysite;
	x: number;
	z: number;
	side?: EntitySide;
}

interface GroupOptions {
	subType?: EntitySubTypeGroup;
	side?: EntitySide;
	alive?: number;
	busy?: boolean;
	// the helicopter leader's offset along x from its keysite, or "none" for no leader
	leader?: number | "none";
	aircraft?: EntitySubTypeAircraft;
	members?: number;
	register?: boolean;
	sleep?: number;
}

interface TaskOptions {
	subType?: EntitySubTypeTask;
	objective?: string;
	side?: EntitySide;
	critical?: number;
	priority?: number;
	expire?: number;
}

class Scenario {
	public readonly ops: LifecycleOp[] = [];

	public constructor(private readonly sites: Site[] = [
		{ subType: AIRBASE, x: 1000, z: 16000 },
		{ subType: FARP, x: 9000, z: 16000 },
	]) {}

	// a group based at keysite `keysite` (or "NULL"), restored raw
	public group(label: string, keysite: number | "NULL", options: GroupOptions = {}): void {
		const site = keysite === "NULL" ? { x: 0, z: 0 } : this.sites[keysite];
		const leader = options.leader ?? 0;
		this.ops.push({
			kind: "restore-group",
			label,
			subType: options.subType ?? MLT,
			side: options.side ?? BLUE,
			ammo: 100,
			fuel: 100,
			parent: keysite === "NULL" ? "NULL" : `keysite${keysite}`,
			busy: options.busy === true,
			leader: leader === "none" ? { kind: "none" } : { kind: "at", x: site.x + leader, z: site.z },
		});
		this.ops.push({ kind: "group-alive", group: label, alive: options.alive ?? 1 });
		this.ops.push({ kind: "member-count", group: label, count: options.members ?? 1 });
		if (leader !== "none") {
			this.ops.push({ kind: "aircraft-type", member: `${label}.leader`, subType: options.aircraft ?? UH60 });
		}
		if (options.register !== false) {
			this.ops.push({ kind: "air-register", group: label });
		}
		if (options.sleep !== undefined) {
			this.ops.push({ kind: "group-sleep", group: label, sleep: options.sleep });
		}
	}

	// a fixed wing member of `group` at keysite `keysite`
	public fixedWing(label: string, group: string, keysite: number, aircraft: EntitySubTypeAircraft): void {
		const site = this.sites[keysite];
		this.ops.push({ kind: "add-member", label, group, type: EntityType.ENTITY_TYPE_FIXED_WING, subType: aircraft, x: site.x, z: site.z });
	}

	public task(label: string, keysite: number, options: TaskOptions = {}): void {
		this.ops.push({
			kind: "unassigned-task",
			label,
			keysite: `keysite${keysite}`,
			objective: options.objective ?? "NULL",
			subType: options.subType ?? SUPPLY,
			side: options.side ?? BLUE,
			critical: options.critical ?? 1,
			priority: options.priority ?? 4,
			expire: options.expire ?? 1200,
		});
	}

	public op(op: LifecycleOp): void {
		this.ops.push(op);
	}

	public assign(keysite: number, category: TaskCategoryType = SUPPORT): void {
		this.ops.push({ kind: "observe-tasks" }, { kind: "assign-tasks", keysite: `keysite${keysite}`, category });
	}

	public spec(): LifecycleSpec {
		const keysites: KeysiteSpec[] = this.sites.map((s) => ({ side: s.side ?? BLUE, subType: s.subType, inUse: true, x: s.x, z: s.z, ammo: 100, fuel: 100 }));

		return {
			heap: 200,
			forces: [BLUE],
			keysites,
			ops: [...this.sites.map((_s, i): LifecycleOp => ({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: 0 })), ...this.ops],
		};
	}
}

function boundary(group: string, task: string): string {
	return `${BOUNDARY} ${group} ${task}`;
}

//
// The social chain: Slice 5b's base layout (a FARP at (8000, 16000) asks for
// ammo, a factory 5 km east holds three crates, an airbase at (22000, 16000)
// takes helicopters and bases the idle medium lift group g0 led from the
// airbase). The real response constructs task27 on the airbase's unassigned
// list; the airbase's SUPPORT assignment then selects g0 for it. Heap: session
// 0, update 1, force0 2, keysites 3-5, 16 sectors 6-21, g0 22, g0.leader 23,
// crates 24-26, the task 27.
//
function socialChain(): LifecycleSpec {
	const ops: LifecycleOp[] = [
		{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: -0.5, ymax: 0.5, zmin: -1.5, zmax: 1.5 },
		{ kind: "map", xSectors: 4, zSectors: 4, sideLength: 8192 },
		{ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED },
		{ kind: "game-type", type: GameType.GAME_TYPE_CAMPAIGN },
		{ kind: "keysite-landing", keysite: "keysite2", landingTypes: 4, usableState: KeysiteUsableState.KEYSITE_STATE_USABLE },
		{ kind: "restore-group", label: "g0", subType: MLT, side: BLUE, ammo: 100, fuel: 100, parent: "keysite2", busy: false, leader: { kind: "at", x: 22000, z: 16000 } },
		{ kind: "group-alive", group: "g0", alive: 1 },
		{ kind: "update-cargo", keysite: "keysite1", level: 35, subType: EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO, size: 10 },
		{ kind: "observe-supply-tasks" },
		{ kind: "observe-tasks" },
		{ kind: "update-cargo", keysite: "keysite0", level: 5, subType: EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO, size: 10 },
		{ kind: "member-count", group: "g0", count: 2 },
		{ kind: "air-register", group: "g0" },
		{ kind: "aircraft-type", member: "g0.leader", subType: UH60 },
		{ kind: "assign-tasks", keysite: "keysite2", category: SUPPORT },
	];
	const sites: [EntitySubTypeKeysite, number][] = [
		[FARP, 8000],
		[FACTORY, 13000],
		[AIRBASE, 22000],
	];

	return {
		heap: 200,
		forces: [BLUE],
		keysites: sites.map(([subType, x]) => ({ side: BLUE, subType, inUse: true, x, z: 16000, ammo: 100, fuel: 100 })),
		ops: [...sites.map((_s, i): LifecycleOp => ({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: 0 })), ...ops],
	};
}

function build(): SupplyTaskAssignmentCase[] {
	const cases: SupplyTaskAssignmentCase[] = [];

	const add = (id: string, c: string, s: Scenario | LifecycleSpec, expected: string[], absent: string[] = []): void => {
		cases.push({ id, c, spec: s instanceof Scenario ? s.spec() : s, expected, absent });
	};

	// ---- the social chain

	add(
		"social-chain-cargo-to-supply-task-to-selected-group",
		"keysite cargo -> FORCE_LOW_ON_SUPPLIES -> create_supply_task -> create_task (unassigned at the airbase) -> assign_keysite_tasks (SUPPORT) -> get_suitable_registered_group -> assign_primary_task_to_group (g0, task27)",
		socialChain(),
		[
			"transmit-switch-parent task27 40 keysite2",
			boundary("g0", "task27"),
			"task task27 27 sub 21 side 1 state 0",
			"unassigned keysite2 task27",
		],
	);

	// ---- which group: locality (group.c :: assess_group_task_locality_factor)

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.task("t0", 0);
		s.assign(0);
		add("zero-distance-with-real-cruise-velocity-is-eligible", "eta = 0 / 43.8f = 0 <= 1200; check_group_members_awake: the UH-60's sleep is en_float.c's default 0.0", s, [boundary("g0", "t0"), "task t0", "unassigned keysite0 t0"]);
	}

	for (const [id, d, eligible] of [
		["eta-just-below-the-expiry-is-eligible", D_BELOW, true],
		["eta-equal-to-the-expiry-is-eligible", D_EQUAL, true],
		["eta-just-above-the-expiry-is-rejected", D_ABOVE, false],
	] as [string, number, boolean][]) {
		const s = new Scenario();
		s.group("g0", 0, { leader: d });
		s.task("t0", 0);
		s.assign(0);
		add(id, `group.c: eta (${d} m / 85 knots, truncated to float) > expire_timer (1200) rejects`, s, eligible ? [boundary("g0", "t0")] : ["result ok"], eligible ? [] : [BOUNDARY]);
	}

	// ---- which group: the list order and the qualification gates

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("equal-suitability-the-first-group-wins", "assign.c: result < best_result (strict); every nonzero suitability is 1.0", s, [boundary("g0", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { alive: 0 });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("a-dead-group-remains-a-candidate", "get_suitable_registered_group never reads INT_TYPE_ALIVE", s, [boundary("g0", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { register: false });
		s.group("g1", 1);
		s.task("t0", 0);
		s.assign(0);
		add(
			"the-force-wide-registered-idle-count-qualifies-an-unregistered-local-group",
			"assign.c: idle_group_count [MLT] counts the force's LIST_TYPE_AIR_REGISTRY (g1, based elsewhere) = 1 > minimum_idle_count 0",
			s,
			[boundary("g0", "t0")],
		);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { register: false });
		s.task("t0", 0);
		s.assign(0);
		add("no-registered-idle-group-of-the-type-no-group-qualifies", "idle_group_count [MLT] = 0, not > 0", s, ["result ok"], [BOUNDARY]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { register: false });
		s.group("g1", 1, { busy: true });
		s.task("t0", 0);
		s.assign(0);
		add("a-busy-registered-group-is-not-counted-idle", "idle counts only GROUP_MODE_IDLE registry groups; g1 busy -> 0", s, ["result ok"], [BOUNDARY]);
	}

	{
		// escort of a slow objective (the medium lift group g9, not based at a keysite):
		// attack helicopters (minimum idle count 2) qualify only with more than 2 idle registered
		for (const registered of [2, 3]) {
			const s = new Scenario();
			s.group("g9", "NULL", { register: false });
			s.group("g0", 0, { subType: ATTACK });
			for (let i = 1; i < registered; i++) {
				s.group(`g${i}`, 1, { subType: ATTACK });
			}
			s.task("t0", 0, { subType: ESCORT, objective: "g9" });
			s.assign(0);
			add(
				`minimum-idle-count-${registered}-idle-attack-groups`,
				`gp_dbase.c: ATTACK_HELICOPTER minimum_idle_count 2; idle_count ${registered} > 2 is ${registered > 2}`,
				s,
				registered > 2 ? [boundary("g0", "t0")] : ["result ok"],
				registered > 2 ? [] : [BOUNDARY],
			);
		}
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { busy: true });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("a-busy-group-is-skipped", "INT_TYPE_GROUP_MODE != GROUP_MODE_IDLE", s, [boundary("g1", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { sleep: 5 });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("a-sleeping-group-is-skipped", "group_raw->sleep == 0.0 is false", s, [boundary("g1", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { side: RED, register: false });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("a-group-of-another-side-is-skipped", "group_raw->side != the task's side", s, [boundary("g1", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { members: 0 });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("a-group-below-the-minimum-member-count-is-skipped", "INT_TYPE_MEMBER_COUNT 0 < INT_TYPE_MINIMUM_MEMBER_COUNT 1", s, [boundary("g1", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { subType: ATTACK });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("an-unsuitable-group-is-skipped", "get_group_to_task_suitability (ATTACK_HELICOPTER, SUPPLY) = 0.0", s, [boundary("g1", "t0")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0, { subType: ASSAULT_SHIP, register: false });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("an-assault-ship-is-never-assigned", "stop carriers being assigned: ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP -> next group", s, [boundary("g1", "t0")]);
	}

	{
		const s = new Scenario();
		s.op({ kind: "pilot", label: "p0" });
		s.group("g0", 0);
		s.op({ kind: "pilot-lock", entity: "g0", pilot: "p0" });
		s.group("g1", 0);
		s.task("t0", 0);
		s.assign(0);
		add("a-pilot-locked-group-is-skipped", "get_local_entity_parent (group, LIST_TYPE_PILOT_LOCK)", s, [boundary("g1", "t0")]);
	}

	// ---- suitable_group_task_specific_checks

	{
		// ESCORT of a fast objective (a multi-role fighter group, speed 10): the slow
		// marine attack helicopters (3) are rejected, the carrier interceptors (10) pass
		const s = new Scenario();
		s.group("g9", "NULL", { subType: FIGHTER, register: false, leader: "none" });
		s.group("g0", 0, { subType: MARINE_ATTACK });
		s.group("g1", 0, { subType: INTERCEPTOR, leader: "none" });
		s.fixedWing("g1.leader", "g1", 0, FA18);
		s.task("t0", 0, { subType: ESCORT, objective: "g9" });
		s.assign(0);
		add("escort-of-a-fast-objective-takes-a-fast-group", "assign.c ESCORT: objective speed >= 5 and group speed < 5 -> FALSE", s, [boundary("g1", "t0")]);
	}

	{
		// ESCORT of a slow objective (medium lift, speed 1): the interceptors (10) are rejected
		const s = new Scenario();
		s.group("g9", "NULL", { register: false });
		s.group("g0", 0, { subType: INTERCEPTOR, leader: "none" });
		s.fixedWing("g0.leader", "g0", 0, FA18);
		s.group("g1", 0, { subType: MARINE_ATTACK });
		s.task("t0", 0, { subType: ESCORT, objective: "g9" });
		s.assign(0);
		add("escort-of-a-slow-objective-rejects-a-fast-group", "assign.c ESCORT: objective speed < 5 and group speed >= 5 -> FALSE", s, [boundary("g1", "t0")]);
	}

	{
		// ESCORT of a recon helicopter group (speed 4): marine attack (3) is slower
		const s = new Scenario();
		s.group("g9", "NULL", { subType: RECON_HELICOPTER, register: false });
		s.group("g0", 0, { subType: MARINE_ATTACK });
		s.task("t0", 0, { subType: ESCORT, objective: "g9" });
		s.assign(0);
		add("escort-rejects-a-group-slower-than-its-objective", "assign.c ESCORT (magitek): objective speed 4 > group speed 3 -> FALSE", s, ["result ok"], [BOUNDARY]);
	}

	{
		// TROOP_INSERTION against an enemy airbase (keysite2, RED): a one-member
		// assault group is rejected, a two-member one qualifies
		const s = new Scenario([
			{ subType: AIRBASE, x: 1000, z: 16000 },
			{ subType: FARP, x: 9000, z: 16000 },
			{ subType: AIRBASE, x: 30000, z: 16000, side: RED },
		]);
		s.group("g0", 0, { subType: ASSAULT });
		s.group("g1", 0, { subType: ASSAULT, members: 2 });
		s.task("t0", 0, { subType: TROOP_INSERTION, objective: "keysite2" });
		s.assign(0, STRIKE);
		add("troop-insertion-against-an-enemy-airbase-needs-two-members", "assign.c TROOP_INSERTION: airbase objective, member count < 2, other side -> FALSE", s, [boundary("g1", "t0")]);
	}

	{
		// the same against a friendly airbase: the one-member group qualifies
		const s = new Scenario();
		s.group("g0", 0, { subType: ASSAULT });
		s.task("t0", 0, { subType: TROOP_INSERTION, objective: "keysite0" });
		s.assign(0, STRIKE);
		add("troop-insertion-against-a-friendly-airbase-takes-one-member", "assign.c TROOP_INSERTION: same side -> TRUE", s, [boundary("g0", "t0")]);
	}

	// ---- which task: category, order, reservation, pilot lock

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.task("t0", 0, { subType: BAI, priority: 9 });
		s.task("t1", 0, { priority: 1 });
		s.assign(0);
		add("only-the-category-s-tasks-are-considered", "assign.c: INT_TYPE_TASK_CATEGORY == category (BAI is STRIKE)", s, [boundary("g0", "t1")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.task("t0", 0);
		s.assign(0, RECON);
		add("no-task-of-the-category-assigns-nothing", "assign.c: task_count == 0 -> return", s, ["result ok", "unassigned keysite0 t0"], [BOUNDARY]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.assign(0);
		add("no-unassigned-task-assigns-nothing", "assign.c: no LIST_TYPE_UNASSIGNED_TASK child -> return", s, ["result ok"], [BOUNDARY]);
	}

	for (const [count, first] of [
		[2, "t1"],
		[3, "t2"],
		[5, "t3"],
	] as [number, string][]) {
		const s = new Scenario();
		s.group("g0", 0);
		for (let i = 0; i < count; i++) {
			s.task(`t${i}`, 0);
		}
		s.assign(0);
		add(
			`equal-priority-quicksort-order-of-${count}-tasks`,
			`en_misc.c :: qs (descending, middle pivot, not stable): ${count} equal keys put ${first} first`,
			s,
			[boundary("g0", first)],
		);
	}

	{
		// a critical task's priority doubles (3 -> 6) and outranks a non-critical 5
		// (expiring in 100 s, so not reserved)
		const s = new Scenario();
		s.group("g0", 0);
		s.task("t0", 0, { critical: 0, priority: 5, expire: 100 });
		s.task("t1", 0, { priority: 3 });
		s.assign(0);
		add("a-critical-task-s-priority-doubles", "assign.c: sort_order *= 2.0 for INT_TYPE_CRITICAL_TASK", s, [boundary("g0", "t1")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.task("t0", 0, { critical: 0, priority: 3 });
		s.task("t1", 0, { critical: 0, priority: 2 });
		s.task("t2", 0, { critical: 0, priority: 1 });
		s.assign(0);
		add("an-airbase-reserves-its-first-two-non-critical-tasks-for-players", "ks_dbase.c: reserve_task_count 2; non-critical, expire 1200 > KEYSITE_TASK_ASSIGN_TIMER (180)", s, [boundary("g0", "t2")]);
	}

	{
		const s = new Scenario();
		s.group("g1", 1);
		s.task("t0", 1, { critical: 0, priority: 2 });
		s.task("t1", 1, { critical: 0, priority: 1 });
		s.assign(1);
		add("a-farp-reserves-one", "ks_dbase.c: FARP reserve_task_count 1", s, [boundary("g1", "t1")]);
	}

	{
		const s = new Scenario();
		s.group("g0", 0);
		s.task("t0", 0, { critical: 0, priority: 3, expire: 180 });
		s.assign(0);
		add("a-non-critical-task-expiring-within-the-assign-timer-is-not-reserved", "expire 180 > 180.0 is false", s, [boundary("g0", "t0")]);
	}

	{
		const s = new Scenario();
		s.op({ kind: "pilot", label: "p0" });
		s.group("g0", 0);
		s.task("t0", 0, { priority: 9 });
		s.op({ kind: "pilot-lock", entity: "t0", pilot: "p0" });
		s.task("t1", 0);
		s.assign(0);
		add("a-pilot-locked-task-is-skipped", "get_local_entity_parent (task, LIST_TYPE_PILOT_LOCK) -> continue", s, [boundary("g0", "t1")]);
	}

	{
		// the first task (an escort, priority 9) finds no group; the loop moves on
		const s = new Scenario();
		s.group("g9", "NULL", { register: false });
		s.group("g0", 0);
		s.task("t0", 0, { subType: ESCORT, objective: "g9", priority: 9 });
		s.task("t1", 0);
		s.assign(0);
		add("a-task-without-a-group-does-not-stop-the-loop", "get_suitable_registered_group returns NULL -> next task", s, [boundary("g0", "t1")]);
	}

	return cases;
}

export const SUPPLY_TASK_ASSIGNMENT_CASES: SupplyTaskAssignmentCase[] = build();

// "" when the output meets the case's expectations
export function supplyTaskAssignmentExpectationFailure(c: SupplyTaskAssignmentCase, output: string[], firstUnmatchedLine: (output: string[], expected: string[]) => string): string {
	const missing = firstUnmatchedLine(output, c.expected);

	if (missing !== "") {
		return `missing: ${missing}`;
	}

	for (const prefix of c.absent) {
		for (const line of output) {
			if (line.substring(0, prefix.length) === prefix) {
				return `unexpected: ${line}`;
			}
		}
	}

	return "";
}

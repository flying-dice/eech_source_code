//
// Slice 5b behaviour matrix: taskgen.c :: create_supply_task -> create_task
// through its return (issue #14), run socially on the frozen Slice 1-5a
// entity graph.
//
// Nothing is mocked. Keysites, groups and sectors are restored raw, as a
// saved game holds them; crates come from the real Slice 4
// update_keysite_cargo; the request comes from a real sender and the real
// Slice 5a response. What 5b adds to the output:
//
//   transmit-create 34 ...       ENTITY_COMMS_CREATE of the task (ts_creat.c :: create_remote)
//   transmit-task-pointers ...   ENTITY_COMMS_SET_TASK_POINTERS: the route as packed
//   campaign mission-created ... the campaign screen's MISSION_CREATED (CampaignEvents.missionCreated)
//   transmit-switch-parent ...   ENTITY_COMMS_SWITCH_PARENT onto the start keysite's unassigned list
//
// and, once a scenario observes tasks, the graph's tasks (values, route,
// lists), the forces' supply task counters, and the keysites' unassigned and
// task dependent lists and the sectors' task lists.
//
// Every case runs three ways: JavaScript (test/unit/supply-task-construction.test.ts),
// Lua 5.1 (test/lua/conformance.ts) and the executed original C
// (test/c-reference/supply-task-construction.cref.test.ts), where TypeScript's
// whole output must also equal the C's.
//
// `expected` lists lines derived by hand from the C source, in order; a line
// also matches an output line that continues it after a space. `fragments`
// must each appear inside some output line. `absent` lists line prefixes that
// must not appear.
//
// Common layout: heap 200; force0 BLUE; crate bounds x -1..1, y -0.5..0.5,
// z -1.5..1.5; map 4 x 4 sectors of 8192 (0 .. 32767; the adjusted area is
// 5000 .. 27767); game status INITIALISED; game type CAMPAIGN. Heap indices:
// session 0, update 1, force0 2, the keysites, the 16 sectors, then entities
// in the order the operations create them. Stocking a keysite with ammo
// (level 35) creates three crates in a row at x + 0, 3, 6, height y + 0.5;
// the newest (x + 6) heads the cargo list and is the one a request takes.
//
// Supply tasks (ts_dbase.c): primary, MOVEMENT_TYPE_AIR, landing types
// FIXED_WING_TRANSPORT | HELICOPTER (6), keysite capacity LARGE (only an
// airbase has it, ks_dbase.c), cargo space 5. A medium lift transport
// helicopter group (gp_dbase.c: air, helicopter landing, cargo space 8) suits
// them; a keysite takes helicopters when its landing types hold bit 2 (4).
// No sector state is restored unless a case says so: every sector is then
// RED (sector_side blue > red is false) and undefended.
//

import { OBJECT_3D_SINGLE_CRATE } from "../../src/generated/c-constants";
import {
	EntitySide,
	EntitySubTypeCargo,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	GameStatusType,
	GameType,
	KeysiteUsableState,
} from "../../src/generated/c-enums";
import type { KeysiteSpec, PositionSpec } from "./campaign-scenario";
import { float32Hex } from "./float-bits";
import type { LifecycleOp, LifecycleSpec } from "./lifecycle-scenario";

export interface SupplyTaskConstructionCase {
	id: string;
	// C provenance of the behaviour under test
	c: string;
	spec: LifecycleSpec;
	expected: string[];
	fragments: string[];
	absent: string[];
}

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;

const AMMO = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO;
const FUEL = EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_FUEL;

const AIRBASE = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE;
const FACTORY = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY;
const FARP = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP;

const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;

const MLT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER;
const ATTACK = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER;
const FRONTLINE = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE;

// landing types: bits of ENTITY_SUB_TYPE_LANDING_FIXED_WING (1), _FIXED_WING_TRANSPORT (2), _HELICOPTER (4)
const HELICOPTERS = 4;

const TRANSMIT_CREATE_TASK = "transmit-create 34";
const POINTERS = "transmit-task-pointers";
const MISSION = "campaign mission-created";
const SWITCH = "transmit-switch-parent";

// LIST_TYPE_UNASSIGNED_TASK
const UNASSIGNED_LIST = 40;

// waypoint and formation types of a supply route (taskgen.c): PICK_UP 19,
// PREPARE_FOR_DROP_OFF 18, DROP_OFF 6, FINISH_DROP_OFF 9, then the terminator
// (NUM_ENTITY_SUB_TYPE_WAYPOINTS 38, FORMATION_NONE 0); FORMATION_ROW_LEFT 2
const WAYPOINTS = [19, 18, 6, 9, 38];
const FORMATIONS = [2, 2, 2, 2, 0];

// the float bits of a whole number of metres
function hex(value: number): string {
	return float32Hex(value);
}

interface Site {
	subType: EntitySubTypeKeysite;
	x: number;
	z: number;
	y?: number;
	inUse?: boolean;
}

interface GroupOptions {
	subType?: EntitySubTypeGroup;
	alive?: number;
	busy?: boolean;
}

//
// Builds one scenario and tracks heap indices, so that expectations can name
// the crates and tasks the original creates (crate<index>, task<index>).
//
class Scenario {
	public readonly ops: LifecycleOp[] = [];

	private next: number;

	public constructor(
		private readonly sites: Site[],
		private readonly heap = 200,
		xSectors = 4,
		bounds: { ymin: number; ymax: number } = { ymin: -0.5, ymax: 0.5 },
	) {
		this.next = 3 + sites.length + xSectors * xSectors;
		this.ops.push(
			{ kind: "bounds", object: OBJECT_3D_SINGLE_CRATE, xmin: -1, xmax: 1, ymin: bounds.ymin, ymax: bounds.ymax, zmin: -1.5, zmax: 1.5 },
			{ kind: "map", xSectors, zSectors: xSectors, sideLength: 8192 },
			{ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED },
			{ kind: "game-type", type: GameType.GAME_TYPE_CAMPAIGN },
		);
	}

	// update_keysite_cargo (keysite, level, sub_type, 10) from no crates of that
	// sub type; returns their labels, newest (list head) first
	public stock(keysite: number, subType: number, level = 35): string[] {
		this.ops.push({ kind: "update-cargo", keysite: `keysite${keysite}`, level, subType, size: 10 });
		const labels: string[] = [];
		for (let remaining = level; remaining > 10; remaining -= 10) {
			labels.unshift(`crate${this.next}`);
			this.next += 1;
		}
		return labels;
	}

	public landing(keysite: number, landingTypes: number, usableState: number = KeysiteUsableState.KEYSITE_STATE_USABLE): void {
		this.ops.push({ kind: "keysite-landing", keysite: `keysite${keysite}`, landingTypes, usableState });
	}

	// a group based at a keysite, without members
	public basedGroup(label: string, keysite: number, options: GroupOptions = {}): void {
		const busy = options.busy === true;
		this.ops.push(
			{ kind: "restore-group", label, subType: options.subType ?? MLT, side: BLUE, ammo: 100, fuel: 100, parent: `keysite${keysite}`, busy, leader: { kind: "none" } },
			{ kind: "group-alive", group: label, alive: options.alive ?? 1 },
		);
		this.next += busy ? 2 : 1;
	}

	// a front line group that resupplies through supply tasks, led from `leader`
	public frontline(label: string, ammo: number, leader: PositionSpec): void {
		this.ops.push({ kind: "restore-group", label, subType: FRONTLINE, side: BLUE, ammo, fuel: 100, parent: "NULL", busy: false, leader });
		this.next += leader.kind === "at" ? 2 : 1;
	}

	public op(op: LifecycleOp): void {
		this.ops.push(op);
	}

	// the request under test, from a real sender; returns the label the task
	// gets if one is created
	public request(keysite: number, level: number, subType: number): string {
		this.ops.push(
			{ kind: "observe-supply-tasks" },
			{ kind: "observe-tasks" },
			{ kind: "update-cargo", keysite: `keysite${keysite}`, level, subType, size: 10 },
		);
		return `task${this.next}`;
	}

	public assess(group: string): string {
		this.ops.push({ kind: "observe-supply-tasks" }, { kind: "observe-tasks" }, { kind: "assess-group", group });
		return `task${this.next}`;
	}

	// a created task takes the next heap entry
	public created(): void {
		this.next += 1;
	}

	public spec(): LifecycleSpec {
		const keysites: KeysiteSpec[] = this.sites.map((s) => ({
			side: BLUE,
			subType: s.subType,
			inUse: s.inUse !== false,
			x: s.x,
			z: s.z,
			ammo: 100,
			fuel: 100,
		}));

		return {
			heap: this.heap,
			forces: [BLUE],
			keysites,
			ops: [...this.sites.map((s, i): LifecycleOp => ({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: s.y ?? 0 })), ...this.ops],
		};
	}
}

// create_supply_task's call from the 5a response: MOVEMENT_TYPE_AIR, priority 4.0f, NULL, NULL
function call(requester: string, supplier: string, cargo: string): string {
	return `create-supply-task ${requester} ${supplier} ${cargo} 1 40800000 NULL NULL`;
}

// ENTITY_COMMS_CREATE for a supply task: parent TASK_DEPENDENT (39) objective,
// sub type (53) 21, task id (224), expire timer (37) 1200.0 (44960000),
// priority (140) 4.0, critical (36) 1, movement (147) AIR 1, route length (183) 4, side (191) BLUE 1
function createTask(index: string, objective: string, id = 1): string {
	return `${TRANSMIT_CREATE_TASK} ${index} parent 39 ${objective} int 53 21 int 224 ${id} float 37 44960000 float 140 40800000 int 36 1 int 147 1 int 183 4 int 191 1 end`;
}

interface Node {
	x: number;
	y: number;
	z: number;
	dependent: string;
}

// the route as the graph prints it: the four nodes, then the terminator
function route(task: string, nodes: Node[]): string {
	let text = `route ${task}`;
	const all = [...nodes, { x: -1, y: -1, z: -1, dependent: "NULL" }];
	all.forEach((n, i) => {
		text += ` ${hex(n.x)} ${hex(n.y)} ${hex(n.z)} ${WAYPOINTS[i]} ${FORMATIONS[i]} ${n.dependent}`;
	});
	return `${text} return NULL`;
}

// the transmitted route: the four nodes (no terminator)
function pointers(task: string, nodes: Node[]): string {
	let text = `${POINTERS} ${task} nodes`;
	for (const n of nodes) {
		text += ` ${hex(n.x)} ${hex(n.y)} ${hex(n.z)}`;
	}
	text += ` formations ${FORMATIONS.slice(0, 4).join(" ")} waypoints ${WAYPOINTS.slice(0, 4).join(" ")} dependents`;
	for (const n of nodes) {
		text += ` ${n.dependent}`;
	}
	return `${text} return NULL`;
}

// task line up to its difficulty: sub SUPPLY, side BLUE, state UNASSIGNED, critical, AIR, 4 nodes
function taskLine(task: string, id: number, difficulty: number): string {
	return `task ${task} ${task.substring(4)} sub ${SUPPLY} side 1 state 0 id ${id} critical 1 movement 1 length 4 difficulty ${difficulty} expire 44960000 priority 40800000`;
}

// the base layout: a FARP (keysite0) at (8000, 16000) asks for ammo; a
// factory (keysite1) 5 km east holds three crates; an airbase (keysite2) at
// (22000, 16000) takes helicopters and bases an idle medium lift group
function base(extraSites: Site[] = []): { s: Scenario; crates: string[] } {
	const s = new Scenario([
		{ subType: FARP, x: 8000, z: 16000 },
		{ subType: FACTORY, x: 13000, z: 16000 },
		{ subType: AIRBASE, x: 22000, z: 16000 },
		...extraSites,
	]);
	s.landing(2, HELICOPTERS);
	s.basedGroup("g0", 2);
	return { s, crates: s.stock(1, AMMO) };
}

// the base route: pick up at the newest crate (13006, 0.5 -> 1, 16000); the
// direction to the FARP is (-5006, -0.5, 0) normalised, x -1 to float
// precision, so prepare is 4 km short (12000) and finish 2 km past (6000),
// both at the F1 height 0.0
function baseNodes(): Node[] {
	return [
		{ x: 13006, y: 1, z: 16000, dependent: "keysite1" },
		{ x: 12000, y: 0, z: 16000, dependent: "NULL" },
		{ x: 8000, y: 0, z: 16000, dependent: "keysite0" },
		{ x: 6000, y: 0, z: 16000, dependent: "NULL" },
	];
}

function build(): SupplyTaskConstructionCase[] {
	const cases: SupplyTaskConstructionCase[] = [];

	// ---- construction

	{
		const { s, crates } = base();
		const task = s.request(0, 5, AMMO);
		s.created();
		const index = task.substring(4);
		cases.push({
			id: "a-farp-request-constructs-a-supply-task",
			c: "create_supply_task -> get_task_start_keysite (airbase) -> create_task: counter, route, create, pointers, parent switch (MISSION_CREATED), difficulty, sector list",
			spec: s.spec(),
			expected: [
				call("keysite0", "keysite1", crates[0]),
				createTask(index, "keysite0"),
				pointers(task, baseNodes()),
				`${MISSION} ${task}`,
				`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`,
				"result ok",
				// every sector is RED: keysite2 -> node 0 crosses sectors 2 and 1 of row 1,
				// node 0 -> 1 stays in 1, 1 -> 2 crosses 1 and 0, 2 -> 3 stays in 0, and the
				// last node (index 3) is sector 0 again: 7 enemy sectors, 7 >> 1 = 3
				`${taskLine(task, 1, 3)} user 00000000 objective keysite0 keysite keysite2 sector sector0_1 update update`,
				route(task, baseNodes()),
				"force force0 supply-tasks-created 1",
				"unassigned keysite0 -",
				`dependents keysite0 ${task}`,
				"unassigned keysite1 -",
				"dependents keysite1 -",
				`unassigned keysite2 ${task}`,
				`sector-tasks sector0_1 ${task}`,
			],
			fragments: [],
			absent: [],
		});
	}

	{
		// the start keysite (sector 3, 1) is two sectors from the first route node (sector 1, 1)
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 30000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		const crates = s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "difficulty-starts-at-the-start-keysite-the-parent-switch-attached",
			c: [
				"create_task: set_client_server_entity_parent (UNASSIGNED_TASK, start keysite) runs before assess_task_difficulty, which starts at raw->task_link.parent.",
				"Order: ENTITY_COMMS_CREATE, ENTITY_COMMS_SET_TASK_POINTERS, then the switch: insert (LINK_PARENT: UNASSIGNED, MISSION_CREATED) before ENTITY_COMMS_SWITCH_PARENT is sent.",
				"From keysite2 (3, 1): 3 -> 1 visits 3 sectors; node 0 -> 1: 1; 1 -> 2 (sector 0): 2; 2 -> 3: 1; last node: 1. 8 enemy sectors >> 1 = 4.",
				"Started at node 0 instead it would be 5 >> 1 = 2.",
			].join(" "),
			spec: s.spec(),
			expected: [
				call("keysite0", "keysite1", crates[0]),
				createTask(task.substring(4), "keysite0"),
				pointers(task, baseNodes()),
				`${MISSION} ${task}`,
				`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`,
				"result ok",
				`${taskLine(task, 1, 4)} user 00000000 objective keysite0 keysite keysite2 sector sector0_1 update update`,
				`unassigned keysite2 ${task}`,
			],
			fragments: [],
			absent: [],
		});
	}

	{
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		const crates = s.stock(1, FUEL);
		const task = s.request(0, 5, FUEL);
		s.created();
		cases.push({
			id: "a-fuel-task-carries-user-data-one",
			c: "FLOAT_TYPE_TASK_USER_DATA = ENTITY_SUB_TYPE_CARGO_FUEL (1.0); a fuel crate row is 3 m further along z (z + 1 * (zmax - zmin + 1))",
			spec: s.spec(),
			expected: [call("keysite0", "keysite1", crates[0]), "result ok", `${taskLine(task, 1, 3)} user 3f800000`],
			fragments: [],
			absent: [],
		});
	}

	{
		const { s, crates } = base([{ subType: FARP, x: 8000, z: 12000 }]);
		const first = s.request(0, 5, AMMO);
		s.created();
		const second = s.request(3, 5, AMMO);
		s.created();
		cases.push({
			id: "task-ids-count-a-forces-supply-tasks",
			c: "force_raw->task_generation [SUPPLY].created ++ gives ids 1 and 2; each joins the head of the start keysite's unassigned list",
			spec: s.spec(),
			expected: [
				call("keysite0", "keysite1", crates[0]),
				`${SWITCH} ${first} ${UNASSIGNED_LIST} keysite2`,
				call("keysite3", "keysite1", crates[0]),
				`${SWITCH} ${second} ${UNASSIGNED_LIST} keysite2`,
				"result ok",
				"force force0 supply-tasks-created 2",
				`unassigned keysite2 ${second} ${first}`,
			],
			fragments: [` id 2 `],
			absent: [],
		});
	}

	// ---- task id semantics

	for (const [restored, id] of [
		[4094, 4095],
		[4095, 1],
		[8189, 4095],
		[8190, 1],
	]) {
		const { s } = base();
		s.op({ kind: "task-counter", force: "force0", subType: SUPPLY, created: restored });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: `task-id-after-${restored}-created-is-${id}`,
			c: "id = ++created; while (id > (1 << NUM_TASK_ID_BITS) - 1) id -= 4095: the id wraps by 4095, not 4096, and 0 never occurs",
			spec: s.spec(),
			expected: ["result ok", taskLine(task, id, 3), `force force0 supply-tasks-created ${restored + 1}`],
			fragments: [` int 224 ${id} `],
			absent: [],
		});
	}

	// ---- no start keysite: nothing is created

	const noTask = (id: string, c: string, configure: (s: Scenario) => void, sites?: Site[]): void => {
		const s = new Scenario(
			sites ?? [
				{ subType: FARP, x: 8000, z: 16000 },
				{ subType: FACTORY, x: 13000, z: 16000 },
				{ subType: AIRBASE, x: 22000, z: 16000 },
			],
		);
		configure(s);
		const crates = s.stock(1, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id,
			c,
			spec: s.spec(),
			expected: [call("keysite0", "keysite1", crates[0]), "result ok", "force force0 supply-tasks-created 0", "unassigned keysite2 -"],
			fragments: [],
			absent: [TRANSMIT_CREATE_TASK, POINTERS, MISSION, SWITCH, "task ", "route "],
		});
	};

	noTask("no-keysite-takes-helicopters-or-transports", "INT_TYPE_LANDING_TYPES & task landing types is 0 (a restored keysite's landing types default to 0)", (s) => {
		s.basedGroup("g0", 2);
	});

	noTask("fixed-wing-landing-alone-does-not-suit", "landing types FIXED_WING (1) do not intersect FIXED_WING_TRANSPORT | HELICOPTER (6)", (s) => {
		s.landing(2, 1);
		s.basedGroup("g0", 2);
	});

	noTask("a-keysite-without-groups-is-skipped", "get_local_entity_first_child (keysite, LIST_TYPE_KEYSITE_GROUP) is NULL", (s) => {
		s.landing(2, HELICOPTERS);
	});

	noTask(
		"a-keysite-not-in-use-is-skipped",
		"INT_TYPE_IN_USE is 0",
		(s) => {
			s.landing(2, HELICOPTERS);
			s.basedGroup("g0", 2);
		},
		[
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000, inUse: false },
		],
	);

	noTask(
		"a-farp-lacks-the-air-force-capacity",
		"check_capacity: keysite_database [FARP].air_force_capacity SMALL < task_database [SUPPLY].keysite_air_force_capacity LARGE",
		(s) => {
			s.landing(2, HELICOPTERS);
			s.basedGroup("g0", 2);
		},
		[
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: FARP, x: 22000, z: 16000 },
		],
	);

	noTask("a-dead-group-scores-nothing", "INT_TYPE_ALIVE is 0: score stays 0.0, so the keysite is skipped", (s) => {
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2, { alive: 0 });
	});

	noTask("an-unsuitable-group-scores-nothing", "get_group_to_task_suitability (ATTACK_HELICOPTER, SUPPLY) is 0.0 (cargo space 0 < 5)", (s) => {
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2, { subType: ATTACK });
	});

	{
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(2, 2);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "fixed-wing-transport-landing-suits",
			c: "landing types FIXED_WING_TRANSPORT (2) intersect the task's 6; the group's own landing type is not checked here",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	// ---- F2: the self-supplying airbase

	{
		const s = new Scenario([{ subType: AIRBASE, x: 16000, z: 16000 }]);
		s.landing(0, HELICOPTERS);
		s.basedGroup("g0", 0);
		const crates = s.stock(0, AMMO);
		s.request(0, 35, AMMO);
		cases.push({
			id: "an-airbase-that-supplies-itself-and-starts-the-task-gets-none",
			c: "F2: supplier == requester; get_task_start_keysite chooses the requester, and start_ks == requester returns NULL before create_task",
			spec: s.spec(),
			expected: [call("keysite0", "keysite0", crates[0]), "result ok", "force force0 supply-tasks-created 0"],
			fragments: [],
			absent: [TRANSMIT_CREATE_TASK, SWITCH],
		});
	}

	{
		const s = new Scenario([
			{ subType: AIRBASE, x: 16000, z: 16000 },
			{ subType: AIRBASE, x: 20000, z: 16000 },
		]);
		s.landing(1, HELICOPTERS);
		s.basedGroup("g0", 1);
		const crates = s.stock(0, AMMO);
		const task = s.request(0, 35, AMMO);
		s.created();
		cases.push({
			id: "a-self-supplying-airbase-gets-a-task-from-another-airbase",
			c: "F2: supplier == requester, but the start keysite is the other airbase: a route from the requester's own crate to itself",
			spec: s.spec(),
			expected: [call("keysite0", "keysite0", crates[0]), `${SWITCH} ${task} ${UNASSIGNED_LIST} keysite1`, "result ok", "force force0 supply-tasks-created 1"],
			fragments: [` 19 2 keysite0 `, ` 6 2 keysite0 `],
			absent: [],
		});
	}

	{
		// with ymin 0 a single crate (level 15) sits exactly on the keysite
		const s = new Scenario(
			[
				{ subType: AIRBASE, x: 16000, z: 16000 },
				{ subType: AIRBASE, x: 20000, z: 16000 },
			],
			200,
			4,
			{ ymin: 0, ymax: 1 },
		);
		s.landing(1, HELICOPTERS);
		s.basedGroup("g0", 1);
		const crates = s.stock(0, AMMO, 15);
		const task = s.request(0, 15, AMMO);
		s.created();
		const at = { x: 16000, y: 0, z: 16000 };
		cases.push({
			id: "cargo-on-the-requester-gives-a-zero-direction",
			c: "normalise_any_3d_vector: length 0 -> direction (0, 0, 0), so prepare and finish are the drop-off point",
			spec: s.spec(),
			expected: [
				call("keysite0", "keysite0", crates[0]),
				"result ok",
				route(task, [
					{ ...at, dependent: "keysite0" },
					{ ...at, dependent: "NULL" },
					{ ...at, dependent: "keysite0" },
					{ ...at, dependent: "NULL" },
				]),
			],
			fragments: [],
			absent: [],
		});
	}

	// ---- start keysite scoring

	{
		const { s } = base([{ subType: AIRBASE, x: 15000, z: 16000 }]);
		s.landing(3, HELICOPTERS);
		s.basedGroup("g1", 3, { busy: true });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "an-idle-group-outweighs-a-nearer-busy-one",
			c: "busy 0.5 * 2 x^4 (x = 1994 / 100 km - 1, x^4 = 0.92) = 0.92 < idle 5.0 * 2 x^4 (x = 8994 / 100 km - 1, x^4 = 0.69) = 6.9",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	{
		const { s } = base([{ subType: AIRBASE, x: 15000, z: 16000 }]);
		s.landing(3, HELICOPTERS);
		s.basedGroup("g1", 3);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "the-nearer-keysite-wins",
			c: "range bias 2 x^4: 1994 m (9.2) beats 8994 m (6.9) for equal group scores",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite3`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	{
		const { s } = base([{ subType: AIRBASE, x: 15000, z: 16000 }]);
		s.landing(3, HELICOPTERS, KeysiteUsableState.KEYSITE_STATE_REPAIRING);
		s.basedGroup("g1", 3);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-keysite-under-repair-scores-half",
			c: "INT_TYPE_KEYSITE_USABLE_STATE != KEYSITE_STATE_USABLE: score *= 0.5 (9.2 -> 4.6 < 6.9)",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	{
		// two airbases 5 km north and south of the crate (13006, 16000): equal ranges
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 13006, z: 11000 },
			{ subType: AIRBASE, x: 13006, z: 21000 },
		]);
		s.landing(2, HELICOPTERS);
		s.landing(3, HELICOPTERS);
		for (let i = 0; i < 3; i++) {
			s.basedGroup(`a${i}`, 2);
		}
		for (let i = 0; i < 5; i++) {
			s.basedGroup(`b${i}`, 3);
		}
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "group-scores-are-capped-at-twelve",
			c: "score = min (score, 12.0f): 3 idle groups (15) and 5 (25) both score 12; the tie keeps the first keysite (score > best_score)",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	{
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 13006, z: 11000 },
			{ subType: AIRBASE, x: 13006, z: 21000 },
		]);
		s.landing(2, HELICOPTERS);
		s.landing(3, HELICOPTERS);
		s.basedGroup("a0", 2);
		s.basedGroup("b0", 3);
		s.basedGroup("b1", 3, { busy: true });
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-busy-group-still-adds-to-a-keysite",
			c: "idle 5.0 + busy 0.5 = 5.5 beats a lone idle group's 5.0 at an equal range",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite3`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	{
		// the second request's nearer airbase already holds one unassigned supply task
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 15000, z: 16000 },
			{ subType: AIRBASE, x: 17000, z: 16000 },
			{ subType: FARP, x: 8000, z: 12000 },
		]);
		s.landing(2, HELICOPTERS);
		s.landing(3, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.basedGroup("g1", 3);
		s.stock(1, AMMO);
		const first = s.request(0, 5, AMMO);
		s.created();
		const second = s.request(4, 5, AMMO);
		s.created();
		cases.push({
			id: "unassigned-tasks-of-the-type-bias-against-a-keysite",
			c: "task_count = max (1.0 - 1 * 0.2f, 0.2f) = 0.8: 9.2 * 0.8 = 7.4 < 8.5 (the airbase 3994 m away)",
			spec: s.spec(),
			expected: [`${SWITCH} ${first} ${UNASSIGNED_LIST} keysite2`, `${SWITCH} ${second} ${UNASSIGNED_LIST} keysite3`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	// ---- range: a 16 x 16 map (0 .. 131071)

	{
		const s = new Scenario(
			[
				{ subType: FARP, x: 8000, z: 16000 },
				{ subType: FACTORY, x: 13000, z: 16000 },
				{ subType: AIRBASE, x: 115000, z: 16000 },
			],
			400,
			16,
		);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		s.request(0, 5, AMMO);
		cases.push({
			id: "a-keysite-100-km-away-is-out-of-range",
			c: "range >= KEYSITE_TASK_MAX_RANGE_BIAS (100 km): approximate range 101994 skips the keysite",
			spec: s.spec(),
			expected: ["result ok", "force force0 supply-tasks-created 0"],
			fragments: [],
			absent: [TRANSMIT_CREATE_TASK],
		});
	}

	{
		const s = new Scenario(
			[
				{ subType: FARP, x: 8000, z: 16000 },
				{ subType: FACTORY, x: 13000, z: 16000 },
				{ subType: AIRBASE, x: 100000, z: 16000 },
			],
			400,
			16,
		);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		for (let x = 1; x <= 12; x++) {
			s.op({ kind: "sector-state", sector: `sector${x}_1`, blue: 0, red: 1, samNeutral: 0, samBlue: 0, samRed: 1 });
		}
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-far-route-caps-both-difficulty-terms-at-five",
			c: "87 km from the crate the score is tiny but positive; the route from sector 12 crosses 13 defended enemy sectors: min (n >> 1, 5) twice = 10",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok", taskLine(task, 1, 10)],
			fragments: [],
			absent: [],
		});
	}

	// ---- difficulty

	{
		const { s } = base();
		// sector2_1: blue presence 1 = red 1 (a tie is RED), neutral and blue SAMs only
		s.op({ kind: "sector-state", sector: "sector2_1", blue: 1, red: 1, samNeutral: 5, samBlue: 5, samRed: 0 });
		s.op({ kind: "sector-state", sector: "sector1_1", blue: 2, red: 1, samNeutral: 0, samBlue: 0, samRed: 1 });
		s.op({ kind: "sector-state", sector: "sector0_1", blue: 1, red: 0, samNeutral: 0, samBlue: 0, samRed: 0 });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "difficulty-counts-enemy-held-and-enemy-defended-sectors",
			c: "assess_task_sector_difficulty: enemy SAM levels exclude the task's side and neutral; INT_TYPE_SECTOR_SIDE is RED unless blue > red. Visits: sector2_1 once (enemy, undefended), sector1_1 three times (blue, red SAM), sector0_1 three times (blue): air 3 >> 1 = 1, enemy 1 >> 1 = 0",
			spec: s.spec(),
			expected: ["result ok", taskLine(task, 1, 1)],
			fragments: [],
			absent: [],
		});
	}

	{
		// a north-south route: the Bresenham walk's z-major arm
		const s = new Scenario([
			{ subType: FARP, x: 16000, z: 26000 },
			{ subType: FACTORY, x: 16000, z: 9000 },
			{ subType: AIRBASE, x: 16000, z: 2000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-north-south-route-walks-sectors-along-z",
			c: "ax <= az: the z-major Bresenham arm. keysite2 (row 0) -> crate (row 1): 2; -> prepare (22000, row 2): 2; -> drop-off (row 3): 2; -> finish (27767 bounded, row 3): 1; last node: 1. 8 enemy sectors >> 1 = 4",
			spec: s.spec(),
			expected: ["result ok", taskLine(task, 1, 4)],
			fragments: [` ${hex(27767)} 9 2 NULL `],
			absent: [],
		});
	}

	{
		// a diagonal route
		const s = new Scenario([
			{ subType: FARP, x: 26000, z: 26000 },
			{ subType: FACTORY, x: 9000, z: 9000 },
			{ subType: AIRBASE, x: 9000, z: 2000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-diagonal-route-steps-both-axes",
			c: "Bresenham, ax == az: the z-major arm steps x as well (d >= 0) each time. keysite2 (1, 0) -> crate (1, 1): 2; -> prepare (2, 2) diagonally: 2; -> drop-off (3, 3): 2; -> finish (3, 3): 1; last node: 1. 8 enemy sectors >> 1 = 4",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok", taskLine(task, 1, 4)],
			fragments: [],
			absent: [],
		});
	}

	// ---- the route's derived points

	{
		const s = new Scenario([
			{ subType: FARP, x: 6000, z: 16000 },
			{ subType: FACTORY, x: 11000, z: 16000 },
			{ subType: AIRBASE, x: 20000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "finish-is-kept-off-the-map-perimeter",
			c: "bound_position_to_adjusted_map_area: finish x 6000 - 2000 = 4000 < MIN_MAP_X + MAP_PERIMETER_SIZE -> 5000",
			spec: s.spec(),
			expected: ["result ok"],
			fragments: [` ${hex(6000)} 00000000 ${hex(16000)} 6 2 keysite0 ${hex(5000)} 00000000 ${hex(16000)} 9 2 NULL `],
			absent: [],
		});
	}

	{
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000, y: 3000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-height-difference-shortens-the-horizontal-offsets",
			c: "direction.y = stop.y - start.y is normalised with x and z: x = -5006 / 5832.6, so prepare is 3433 m short, not 4 km; the drop-off keeps the keysite's height, prepare and finish the F1 0.0",
			spec: s.spec(),
			expected: ["result ok"],
			fragments: [` ${hex(8000)} ${hex(3000)} ${hex(16000)} 6 2 keysite0 `],
			absent: [],
		});
	}

	{
		// the FARP is just inside sector 1; its finish point (6500) is in sector 0
		const s = new Scenario([
			{ subType: FARP, x: 8500, z: 16000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "the-task-joins-the-sector-of-its-objective",
			c: "position = get_local_entity_vec3d_ptr (task_objective, VEC3D_TYPE_POSITION): the FARP's sector (1, 1), not the last route node's (finish at 6500: sector 0)",
			spec: s.spec(),
			expected: ["result ok", `sector-tasks sector1_1 ${task}`],
			fragments: [` ${hex(6500)} 00000000 ${hex(16000)} 9 2 NULL `],
			absent: ["sector-tasks sector0_1"],
		});
	}

	// ---- group requesters

	{
		const s = new Scenario([
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(1, HELICOPTERS);
		s.basedGroup("g0", 1);
		const crates = s.stock(0, AMMO);
		s.frontline("line", 50, { kind: "at", x: 8000, z: 16000 });
		const task = s.assess("line");
		s.created();
		cases.push({
			id: "a-front-line-group-is-the-task-objective",
			c: "assess_group_supplies -> the response -> create_supply_task (group requester): the group's position (its leader's) is the drop-off; the task joins the group's task dependent list (gp_msgs.c LINK_CHILD: no arm) and the sector under the group",
			spec: s.spec(),
			expected: [call("line", "keysite0", crates[0]), `${SWITCH} ${task} ${UNASSIGNED_LIST} keysite1`, "result ok", `sector-tasks sector0_1 ${task}`],
			fragments: [` objective line keysite keysite1 sector sector0_1 `, ` ${hex(8000)} 00000000 ${hex(16000)} 6 2 line `],
			absent: [],
		});
	}

	// ---- the campaign screen and replication

	{
		const { s } = base();
		s.op({ kind: "game-type", type: GameType.GAME_TYPE_FREE_FLIGHT });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "free-flight-does-not-notify-the-campaign-screen",
			c: "notify_campaign_screen returns FALSE unless the game type is CAMPAIGN or SKIRMISH; the task is still unassigned",
			spec: s.spec(),
			expected: [`${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok", taskLine(task, 1, 3)],
			fragments: [],
			absent: [MISSION],
		});
	}

	{
		const { s } = base();
		s.op({ kind: "game-type", type: GameType.GAME_TYPE_SKIRMISH });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "a-skirmish-notifies-the-campaign-screen",
			c: "notify_campaign_screen: GAME_TYPE_SKIRMISH passes the guard",
			spec: s.spec(),
			expected: [`${MISSION} ${task}`, `${SWITCH} ${task} ${UNASSIGNED_LIST} keysite2`, "result ok"],
			fragments: [],
			absent: [],
		});
	}

	{
		const { s, crates } = base();
		s.op({ kind: "single-player" });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "single-player-transmits-nothing",
			c: "transmit_entity_comms_message returns when direct_play_get_comms_mode () == DIRECT_PLAY_COMMS_MODE_NONE: no create, pointers or switch parent; the local task is the same",
			spec: s.spec(),
			expected: [
				call("keysite0", "keysite1", crates[0]),
				`${MISSION} ${task}`,
				"result ok",
				`${taskLine(task, 1, 3)} user 00000000 objective keysite0 keysite keysite2 sector sector0_1 update update`,
				route(task, baseNodes()),
			],
			fragments: [],
			absent: [TRANSMIT_CREATE_TASK, POINTERS, SWITCH],
		});
	}

	{
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000, y: 70000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		const task = s.request(0, 5, AMMO);
		cases.push({
			id: "multiplayer-packing-asserts-on-a-drop-off-above-the-map",
			c: "pack_vec3d (VEC3D_TYPE_POSITION): ASSERT (point_inside_map_volume (v) || v->y == -10000) fails for the drop-off node at y 70000 > MAX_MAP_Y 65535",
			spec: s.spec(),
			expected: [createTask(task.substring(4), "keysite0"), "result assert point_inside_map_volume (v) || v->y == -10000"],
			fragments: [],
			absent: [POINTERS, MISSION, SWITCH],
		});
	}

	{
		const s = new Scenario([
			{ subType: FARP, x: 8000, z: 16000, y: 70000 },
			{ subType: FACTORY, x: 13000, z: 16000 },
			{ subType: AIRBASE, x: 22000, z: 16000 },
		]);
		s.landing(2, HELICOPTERS);
		s.basedGroup("g0", 2);
		s.stock(1, AMMO);
		s.op({ kind: "single-player" });
		const task = s.request(0, 5, AMMO);
		s.created();
		cases.push({
			id: "single-player-keeps-a-drop-off-above-the-map",
			c: "the same route in single player: nothing is packed, so nothing checks it; the drop-off node keeps y 70000",
			spec: s.spec(),
			expected: [`${MISSION} ${task}`, "result ok", taskLine(task, 1, 3)],
			fragments: [` ${hex(8000)} ${hex(70000)} ${hex(16000)} 6 2 keysite0 `],
			absent: [TRANSMIT_CREATE_TASK, POINTERS, SWITCH, "result assert"],
		});
	}

	return cases;
}

export const SUPPLY_TASK_CONSTRUCTION_CASES: SupplyTaskConstructionCase[] = build();

// Checks a case's expectations against an output; returns the first failure, or "".
export function supplyTaskExpectationFailure(c: SupplyTaskConstructionCase, output: string[], firstUnmatchedLine: (output: string[], expected: string[]) => string): string {
	const unmatched = firstUnmatchedLine(output, c.expected);

	if (unmatched !== "") {
		return `expected line not found: ${unmatched}`;
	}

	for (const fragment of c.fragments) {
		if (!output.some((line) => line.includes(fragment))) {
			return `fragment not found: ${fragment}`;
		}
	}

	for (const prefix of c.absent) {
		for (const line of output) {
			if (line.startsWith(prefix)) {
				return `absent prefix present: ${line}`;
			}
		}
	}

	return "";
}

//
// F1 acceptance (issue #14): under the compatibility rule (prepare.y =
// finish.y = 0.0) single player and multiplayer construct the same semantic
// route. In EECH the undefined heights could make the two modes diverge:
// only a transmitting session packs the route, and pack_vec3d checks (debug)
// or clamps in place (release) a node outside the map volume. With 0.0 both
// derived nodes are inside the volume, so packing changes nothing.
//
// The pair is the base scenario, once as a multiplayer server (transmitting)
// and once as a single player session.
//
export function f1CompatibilityPair(): { singlePlayer: LifecycleSpec; multiplayer: LifecycleSpec } {
	const build = (singlePlayer: boolean): LifecycleSpec => {
		const { s } = base();
		if (singlePlayer) {
			s.op({ kind: "single-player" });
		}
		s.request(0, 5, AMMO);
		return s.spec();
	};

	return { singlePlayer: build(true), multiplayer: build(false) };
}

// Returns "" when the pair's outputs show the same semantic route, else why not.
export function f1SemanticRouteFailure(singlePlayer: string[], multiplayer: string[]): string {
	const graph = (output: string[]): string[] => {
		const at = output.indexOf("result ok");
		return at < 0 ? [] : output.slice(at);
	};

	const spGraph = graph(singlePlayer);
	const mpGraph = graph(multiplayer);

	if (spGraph.length === 0 || mpGraph.length === 0) {
		return "a mode did not complete (result ok)";
	}

	if (spGraph.join("\n") !== mpGraph.join("\n")) {
		return "the modes' entity graphs differ";
	}

	let route = "";

	for (const line of mpGraph) {
		if (line.startsWith("route ")) {
			route = line;
		}
	}

	if (route === "") {
		return "no route in the graph";
	}

	// route <task> then 6 fields per node: x y z waypoint formation dependent
	const fields = route.split(" ");
	const task = fields[1];
	let nodes = "";

	for (let n = 0; n < 4; n++) {
		const at = 2 + n * 6;
		nodes += ` ${fields[at]} ${fields[at + 1]} ${fields[at + 2]}`;
	}

	const transmitted = `${POINTERS} ${task} nodes${nodes} formations`;

	if (!multiplayer.some((line) => line.startsWith(transmitted))) {
		return "the multiplayer transmission does not carry the stored route";
	}

	if (singlePlayer.some((line) => line.startsWith(POINTERS) || line.startsWith(TRANSMIT_CREATE_TASK) || line.startsWith(SWITCH))) {
		return "single player transmitted the task";
	}

	return "";
}

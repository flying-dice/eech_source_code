//
// Slice 6b behaviour matrix (issue #18): assign.c's assignment transaction for
// a SUPPLY task, assign_primary_task_to_group -> assign_task_to_group ->
// create_generic_waypoint_route -> push_task_onto_group_task_stack, ending
// before assign_task_to_group_members:
//
//   result boundary assign_task_to_group_members <group> <guide> <valid members>
//
// Every case runs three ways: JavaScript (test/unit/supply-task-transaction.test.ts),
// Lua 5.1 (test/lua/conformance.ts) and the executed original C
// (test/c-reference/supply-task-transaction.cref.test.ts), where TypeScript's
// whole output must also equal the C's. `expected` lines are derived by hand
// from the C source and the compiled databases (in order; a line also matches
// an output line that continues it after a space); `absent` lists prefixes
// that must not appear.
//
// State is valid campaign state: a world map, the terrain and road network
// (TerrainElevation, RoadNetwork), tasks restored with their route
// (persisted-task, ts_pack.c) or constructed by the real Slice 5b path, and
// aircraft inside the map.
//
// Common layout ("short route"): heap 200; force0 BLUE; a 4 x 4 map of 8192 m
// sectors; keysite0 an AIRBASE at (10000, 16000), keysite1 a FARP at
// (14000, 16000); a campaign running (the campaign screen listens); a flat
// terrain (0 m); road nodes n0 (0, 0, 0) without links, n1 (10000, 0, 16000),
// n2 (13000, 0, 16000), n3 (14002, 0, 16000). g0, a medium lift transport
// helicopter group at keysite0 led by a UH-60 at (10000, 16000) (cruise
// 43.80644226 m/s, 0x422f39cc; cruise altitude 24 m, 0x41c00000), registered
// and idle. t0, a critical SUPPLY task at keysite0 (objective keysite1) with
// create_supply_task's route shape: PICK_UP (10000, 0, 16000) for keysite0,
// PREPARE_FOR_DROP_OFF (12000, 0, 16000), DROP_OFF (14000, 0, 16000) for
// keysite1, FINISH_DROP_OFF (15000, 0, 16000), no return keysite.
//
// Heap: session 0, update 1, force0 2, keysites 3-4, sectors 5-20, g0 21,
// g0.leader 22, t0 23, then the waypoints (24 ...) and the guide.
//
// Derivation of the short route (croute.c, ts_dbase.c SUPPLY: add_start_waypoint,
// task_route_search, assess_landing; gp_dbase.c MLT: MOVEMENT_TYPE_AIR,
// ENTITY_TYPE_HELICOPTER):
//   - assess_landing: no return keysite and the group's keysite0 is its start,
//     so the return keysite becomes keysite0 (a LAND waypoint at (10000, 0, 16000));
//   - specified route: start (the leader, ceil) NAVIGATION, PICK_UP,
//     PREPARE, DROP_OFF, FINISH, LAND; every leg is at most 5000 m (the last is
//     exactly 5000: 25e6 > 25e6 is false), so the route search adds nothing;
//   - checksum: every node but the first and the last, (int) x + y + z:
//     26000 + 28000 + 30000 + 31000 = 115000 = 56 (mod 256);
//   - waypoints: y bounded to 1; route nodes: n0 has no links, the first node
//     within 5 m wins, else the strictly closest (approximate 2D range): 1, 1,
//     2, 3 (2 m), 3, 1;
//   - flight times: 2D range / 43.80644226 truncated: 0, 0, 2000 -> 0x42369f1d,
//     2000, 1000 -> 0x41b69f1d, 5000 -> 0x42e446e4;
//   - tags (wp_char.c, re-tagged at every link and sub type set): NAVIGATION
//     'A' 65; PICK_UP 'X' 88, PREPARE 'Y' 89, DROP_OFF 'Z' 90, FINISH wraps
//     to 'X' 88; LAND sets 'W' 87;
//   - guide: sub type waypoint_database [NAVIGATION].guide_sub_type (0,
//     NAVIGATION_DIRECT, no guide criteria); RADIUS := hc reached radius 50
//     (0x42480000); TRANSMIT_DATA unchanged (0, 0.0); LAST_TO_REACH := hc 1;
//     velocity := 1.0 x cruise velocity (0x422f39cc).
//

import { OBJECT_3D_SINGLE_CRATE } from "../../src/generated/c-constants";
import {
	EntitySide,
	EntitySubTypeAircraft,
	EntitySubTypeCargo,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntitySubTypeWaypoint,
	EntityType,
	FormationType,
	GameStatusType,
	GameType,
	IntType,
	KeysiteUsableState,
	TaskCategoryType,
} from "../../src/generated/c-enums";
import type { LifecycleOp, LifecycleSpec, PersistedRouteNode } from "./lifecycle-scenario";

export interface SupplyTaskTransactionCase {
	id: string;
	// C provenance of the behaviour under test
	c: string;
	spec: LifecycleSpec;
	expected: string[];
	absent: string[];
}

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;

const AIRBASE = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE;
const FARP = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP;
const FACTORY = EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY;

const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;
const SUPPORT = TaskCategoryType.TASK_CATEGORY_SUPPORT;

const MLT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER;
const MLT_AIRCRAFT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_AIRCRAFT;

const UH60 = EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK;
const C130 = EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_C130J_HERCULES_II;

const W = EntitySubTypeWaypoint;
const ROW_LEFT = FormationType.FORMATION_ROW_LEFT;

const BOUNDARY = "result boundary assign_task_to_group_members";

const CHECKSUM = IntType.INT_TYPE_ROUTE_CHECK_SUM;

interface Site {
	subType: EntitySubTypeKeysite;
	x: number;
	z: number;
}

interface TransactionOptions {
	sites?: Site[];
	leader?: { x: number; z: number } | "none";
	route?: PersistedRouteNode[];
	roads?: boolean;
	terrain?: LifecycleOp;
	campaign?: boolean;
	singlePlayer?: boolean;
	groupType?: EntitySubTypeGroup;
	environment?: boolean;
	extra?: LifecycleOp[];
	members?: LifecycleOp[];
}

function node(x: number, z: number, waypointType: number, dependent: string): PersistedRouteNode {
	return { x, y: 0, z, waypointType, formation: ROW_LEFT, dependent };
}

// create_supply_task's route shape for keysite0 -> keysite1
const SHORT_ROUTE: PersistedRouteNode[] = [
	node(10000, 16000, W.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, "keysite0"),
	node(12000, 16000, W.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF, "NULL"),
	node(14000, 16000, W.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, "keysite1"),
	node(15000, 16000, W.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF, "NULL"),
];

const ROAD_NODES: LifecycleOp[] = [
	{ kind: "road-node", x: 0, y: 0, z: 0, links: 0 },
	{ kind: "road-node", x: 10000, y: 0, z: 16000, links: 1 },
	{ kind: "road-node", x: 13000, y: 0, z: 16000, links: 1 },
	{ kind: "road-node", x: 14002, y: 0, z: 16000, links: 2 },
];

const FLAT: LifecycleOp = { kind: "terrain", defaultElevation: 0, cellSize: 1, cellsX: 0, cellsZ: 0, cells: [] };

function transaction(options: TransactionOptions = {}): LifecycleSpec {
	const sites = options.sites ?? [
		{ subType: AIRBASE, x: 10000, z: 16000 },
		{ subType: FARP, x: 14000, z: 16000 },
	];

	const leader = options.leader ?? { x: 10000, z: 16000 };

	const ops: LifecycleOp[] = [...sites.map((_s, i): LifecycleOp => ({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: 0 })), { kind: "map", xSectors: 4, zSectors: 4, sideLength: 8192 }];

	if (options.campaign !== false) {
		ops.push({ kind: "game-status", status: GameStatusType.GAME_STATUS_INITIALISED }, { kind: "game-type", type: GameType.GAME_TYPE_CAMPAIGN });
	}

	ops.push(options.terrain ?? FLAT);

	if (options.roads !== false) {
		ops.push(...ROAD_NODES);
	}

	ops.push(
		{
			kind: "restore-group",
			label: "g0",
			subType: options.groupType ?? MLT,
			side: BLUE,
			ammo: 100,
			fuel: 100,
			parent: "keysite0",
			busy: false,
			leader: leader === "none" ? { kind: "none" } : { kind: "at", x: leader.x, z: leader.z },
		},
		{ kind: "group-alive", group: "g0", alive: 1 },
		{ kind: "member-count", group: "g0", count: 1 },
	);

	if (leader !== "none") {
		ops.push({ kind: "aircraft-type", member: "g0.leader", subType: UH60 });
	}

	ops.push(...(options.members ?? []), { kind: "air-register", group: "g0" });

	ops.push({
		kind: "persisted-task",
		label: "t0",
		keysite: "keysite0",
		objective: "keysite1",
		subType: SUPPLY,
		side: BLUE,
		critical: 1,
		priority: 4,
		expire: 1200,
		route: options.route ?? SHORT_ROUTE,
		returnKeysite: "NULL",
	});

	ops.push(...(options.extra ?? []));

	if (options.singlePlayer === true) {
		ops.push({ kind: "single-player" });
	}

	if (options.environment === true) {
		ops.push({ kind: "observe-environment" });
	}

	ops.push({ kind: "observe-assignment" }, { kind: "assign-tasks", keysite: "keysite0", category: SUPPORT });

	return {
		heap: 200,
		forces: [BLUE],
		keysites: sites.map((s) => ({ side: BLUE, subType: s.subType, inUse: true, x: s.x, z: s.z, ammo: 100, fuel: 100 })),
		ops,
	};
}

// The transmissions and events of the short route, in the transaction's order
const SHORT_ROUTE_TRANSACTION = [
	// the checksum, set client-server before any waypoint exists
	`transmit-int t0 ${CHECKSUM} 56`,
	// every waypoint was created locally; clients rebuild them from this message
	"transmit-waypoint-route t0 g0 keysite0 start 461c4000 00000000 467a0000 stop 461c4000 00000000 467a0000 checksum 56 waypoints wp24 wp25 wp26 wp27 wp28 wp29",
	// the guide: task LIST_TYPE_GUIDE (16), first waypoint LIST_TYPE_CURRENT_WAYPOINT (9), sub type 0,
	// VALID_GUIDE_MEMBERS 0xffffffff passed as an int (-1), the first waypoint's position
	"transmit-create 13 30 parent 16 t0 parent 9 wp24 int 53 0 int 244 -1 vec3d 8 461c4000 3f800000 467a0000 end",
	// onto the group's guide stack (LIST_TYPE_GUIDE_STACK 17)
	"transmit-switch-list guide30 17 g0 17",
	// criteria that change from the zeroed guide: RADIUS 50.0, LAST_TO_REACH
	"transmit-guide-criteria guide30 0 1 42480000",
	"transmit-guide-criteria guide30 4 1 00000000",
	// FLOAT_TYPE_VELOCITY (149)
	"transmit guide30 149 422f39cc",
	// the task joins the assigned list: TASK_STATE_ASSIGNED, the campaign screen's MISSION_ASSIGNED
	"campaign mission-assigned t0",
	// LIST_TYPE_UNASSIGNED_TASK (40) -> LIST_TYPE_ASSIGNED_TASK (3)
	"transmit-switch-list t0 40 keysite0 3",
	`${BOUNDARY} g0 guide30 ffffffff`,
];

// The assignment graph of the short route (the heap's used list, newest first)
const SHORT_ROUTE_GRAPH = [
	"assigned keysite0 t0",
	"assigned keysite1 -",
	"guide guide30 30 task t0 current wp24 stack g0 sub 0 valid ffffffff position 461c4000 3f800000 467a0000 velocity 422f39cc update update criteria 1:42480000 0:00000000 0:00000000 0:00000000 1:00000000 0:00000000",
	"waypoint wp29 29 task t0 sub 13 formation 2 position-type 0 461c4000 3f800000 467a0000 route-node 1 altitude 41c00000 flight-time 42e446e4 tag 87 dependent keysite0",
	"waypoint wp28 28 task t0 sub 9 formation 2 position-type 0 466a6000 3f800000 467a0000 route-node 3 altitude 41c00000 flight-time 41b69f1d tag 88 dependent NULL",
	"waypoint wp27 27 task t0 sub 6 formation 2 position-type 0 465ac000 3f800000 467a0000 route-node 3 altitude 41c00000 flight-time 42369f1d tag 90 dependent keysite1",
	"waypoint wp26 26 task t0 sub 18 formation 2 position-type 0 463b8000 3f800000 467a0000 route-node 2 altitude 41c00000 flight-time 42369f1d tag 89 dependent NULL",
	"waypoint wp25 25 task t0 sub 19 formation 2 position-type 0 461c4000 3f800000 467a0000 route-node 1 altitude 41c00000 flight-time 00000000 tag 88 dependent keysite0",
	"waypoint wp24 24 task t0 sub 17 formation 2 position-type 0 461c4000 3f800000 467a0000 route-node 1 altitude 41c00000 flight-time 00000000 tag 65 dependent NULL",
	"task-assignment t0 state 1 checksum 56 return keysite0 waypoints wp24 wp25 wp26 wp27 wp28 wp29 guides guide30",
	"group g0 mode 1 guides guide30",
];

//
// The social chain (Slice 5b's base layout): a FARP at (8000, 16000) asks for
// ammo, the factory 5 km east supplies it, and the real Slice 4 - 5b path
// constructs task27 on the airbase's (22000, 16000) unassigned list; the
// airbase's SUPPORT assignment selects g0 (Slice 6a) and runs the transaction
// to the boundary. A terrain grid of 4096 m cells and three road nodes (one
// without links); every terrain lookup is printed.
//
function socialChain(): LifecycleSpec {
	const sites: [EntitySubTypeKeysite, number][] = [
		[FARP, 8000],
		[FACTORY, 13000],
		[AIRBASE, 22000],
	];

	const ops: LifecycleOp[] = [
		...sites.map((_s, i): LifecycleOp => ({ kind: "keysite-state", keysite: `keysite${i}`, alive: 1, y: 0 })),
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
		{ kind: "aircraft-type", member: "g0.leader", subType: EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK },
		{
			kind: "terrain",
			defaultElevation: 0,
			cellSize: 4096,
			cellsX: 8,
			cellsZ: 8,
			cells: [
				0, 10, 20, 30, 40, 50, 60, 70, 5, 15, 25, 35, 45, 55, 65, 75, 100, 200, 300, 400, 500, 600, 700, 800, 0, 0, 0, 0, 0, 0, 0, 0, 250, 250, 250, 250, 250, 250, 250, 250, 10, 20, 10, 20, 10,
				20, 10, 20, 0, 0, 0, 0, 0, 0, 0, 0, 1000, 900, 800, 700, 600, 500, 400, 300,
			],
		},
		{ kind: "road-node", x: 1000, y: 0, z: 1000, links: 2 },
		{ kind: "road-node", x: 20000, y: 0, z: 16000, links: 1 },
		{ kind: "road-node", x: 9000, y: 0, z: 15990, links: 0 },
		{ kind: "observe-assignment" },
		{ kind: "observe-environment" },
		{ kind: "assign-tasks", keysite: "keysite2", category: SUPPORT },
	];

	return {
		heap: 200,
		forces: [BLUE],
		keysites: sites.map(([subType, x]) => ({ side: BLUE, subType, inUse: true, x, z: 16000, ammo: 100, fuel: 100 })),
		ops,
	};
}

//
// Route-choice canaries for #9 (the x87 intermediate precision row of the
// numerical contract). A 7000 m first leg from the leader at (1500, 1000) to
// the pick-up at keysite0 (1500, 8000) is split once; its samples lie at
// x = 334, 626, 918, 1210, 1502, 1794, 2086, 2378 (z 4500), one 256 m terrain
// cell each. The middle six samples stand on 1e9 m, so the choice is between
// sample 1 (x 334, rating 6.1875 x the average of its elevations) and sample 8
// (x 2378, 6.25 x its average), both far enough off the leg to survive
// optimise_route. The elevations make the two ratings agree to within their
// last bits, so the order of the float operations and their rounding decide.
// The expectations are the canonical oracle's choice; the choices of the
// investigation variants (c-reference/fpu-variants.mjs) are recorded beside
// each and checked by test/fpu-spike/route-choice.fpu.test.ts.
//
export interface RouteChoiceCanary {
	id: string;
	elevations: number[];
	// the sample x (1 or 8) each floating-point model chooses
	choices: { canonical: number; "sse-rn": number; "x87-rtz-pc53": number; "x87-rn-pc53": number };
}

export const ROUTE_CHOICE_CANARIES: RouteChoiceCanary[] = [
	{ id: "rounding-mode-decides", elevations: [6133517, 6133519, 1e9, 1e9, 1e9, 1e9, 6072180, 6072183], choices: { canonical: 1, "sse-rn": 8, "x87-rtz-pc53": 1, "x87-rn-pc53": 8 } },
	{ id: "x87-intermediate-precision-decides", elevations: [28418379, 28418382, 1e9, 1e9, 1e9, 1e9, 28134198, 28134196], choices: { canonical: 1, "sse-rn": 1, "x87-rtz-pc53": 8, "x87-rn-pc53": 1 } },
	{ id: "every-other-model-disagrees", elevations: [9122246, 9122243, 1e9, 1e9, 1e9, 1e9, 9031020, 9031024], choices: { canonical: 8, "sse-rn": 1, "x87-rtz-pc53": 1, "x87-rn-pc53": 1 } },
];

export function routeChoiceCanarySpec(elevations: number[]): LifecycleSpec {
	const cells: number[] = [];
	for (let i = 0; i < 10 * 18; i++) {
		cells.push(0);
	}
	const columns = [1, 2, 3, 4, 5, 7, 8, 9];
	for (let k = 0; k < 8; k++) {
		cells[17 * 10 + columns[k]] = elevations[k];
	}
	return {
		heap: 200,
		forces: [BLUE],
		keysites: [
			{ side: BLUE, subType: AIRBASE, inUse: true, x: 1500, z: 8000, ammo: 100, fuel: 100 },
			{ side: BLUE, subType: FARP, inUse: true, x: 1500, z: 12000, ammo: 100, fuel: 100 },
		],
		ops: [
			{ kind: "keysite-state", keysite: "keysite0", alive: 1, y: 0 },
			{ kind: "keysite-state", keysite: "keysite1", alive: 1, y: 0 },
			{ kind: "map", xSectors: 4, zSectors: 4, sideLength: 8192 },
			{ kind: "terrain", defaultElevation: 0, cellSize: 256, cellsX: 10, cellsZ: 18, cells },
			{ kind: "road-node", x: 1500, y: 0, z: 8000, links: 1 },
			{ kind: "restore-group", label: "g0", subType: MLT, side: BLUE, ammo: 100, fuel: 100, parent: "keysite0", busy: false, leader: { kind: "at", x: 1500, z: 1000 } },
			{ kind: "group-alive", group: "g0", alive: 1 },
			{ kind: "member-count", group: "g0", count: 1 },
			{ kind: "aircraft-type", member: "g0.leader", subType: UH60 },
			{ kind: "air-register", group: "g0" },
			{
				kind: "persisted-task",
				label: "t0",
				keysite: "keysite0",
				objective: "keysite1",
				subType: SUPPLY,
				side: BLUE,
				critical: 1,
				priority: 4,
				expire: 1200,
				route: [
					node(1500, 8000, W.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, "keysite0"),
					node(1500, 10000, W.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF, "NULL"),
					node(1500, 12000, W.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, "keysite1"),
					node(1500, 13000, W.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF, "NULL"),
				],
				returnKeysite: "NULL",
			},
			{ kind: "observe-assignment" },
			{ kind: "assign-tasks", keysite: "keysite0", category: SUPPORT },
		],
	};
}

// the generated waypoint (wp25) at sample 1 (x 334) or sample 8 (x 2378), z 4500
export function routeChoiceWaypoint(sample: number): string {
	return `waypoint wp25 25 task t0 sub 17 formation 2 position-type 0 ${sample === 1 ? "43a70000" : "4514a000"} 3f800000 458ca000`;
}

function build(): SupplyTaskTransactionCase[] {
	const cases: SupplyTaskTransactionCase[] = [];

	const add = (id: string, c: string, spec: LifecycleSpec, expected: string[], absent: string[] = []): void => {
		cases.push({ id, c, spec, expected, absent });
	};

	// ---- acceptance: the social chain

	add(
		"social-chain-cargo-to-assigned-supply-task",
		"keysite cargo -> FORCE_LOW_ON_SUPPLIES -> create_supply_task (task27) -> assign_keysite_tasks -> assign_primary_task_to_group (g0, task27) -> assign_task_to_group -> create_generic_waypoint_route -> push_task_onto_group_task_stack -> assign_task_to_group_members (boundary)",
		socialChain(),
		[
			"campaign mission-created task27",
			"transmit-switch-parent task27 40 keysite2",
			// the route search's terrain samples come before the checksum is transmitted
			"terrain",
			`transmit-int task27 ${CHECKSUM} 95`,
			// then one (discarded) lookup per waypoint
			"terrain",
			"transmit-waypoint-route task27 g0 keysite2 start 46abe000 00000000 467a0000 stop 46abe000 00000000 467a0000 checksum 95 waypoints wp28 wp29 wp30 wp31 wp32 wp33",
			"transmit-create 13 34 parent 16 task27 parent 9 wp28 int 53 0 int 244 -1 vec3d 8 46abe000 3f800000 467a0000 end",
			"transmit-switch-list guide34 17 g0 17",
			"transmit-guide-criteria guide34 0 1 42480000",
			"transmit-guide-criteria guide34 4 1 00000000",
			"transmit guide34 149 422f39cc",
			"campaign mission-assigned task27",
			"transmit-switch-list task27 40 keysite2 3",
			`${BOUNDARY} g0 guide34 ffffffff`,
			"task task27 27 sub 21 side 1 state 1",
			"unassigned keysite2 -",
			"assigned keysite2 task27",
			"guide guide34 34 task task27 current wp28 stack g0 sub 0 valid ffffffff",
			"waypoint wp33 33 task task27 sub 13",
			"task-assignment task27 state 1 checksum 95 return keysite2 waypoints wp28 wp29 wp30 wp31 wp32 wp33 guides guide34",
			"group g0 mode 1 guides guide34",
		],
		["result boundary assign_primary_task_to_group"],
	);

	// ---- the transaction, derived by hand (see the header)

	add(
		"short-route-transaction-in-order",
		"assign_task_to_group -> create_generic_waypoint_route (checksum INT_VALUE, local waypoints, CREATE_WAYPOINT_ROUTE) -> push_task_onto_group_task_stack (guide CREATE, SWITCH_LIST, criteria, velocity, ASSIGNED, MISSION_ASSIGNED, task SWITCH_LIST)",
		transaction(),
		[...SHORT_ROUTE_TRANSACTION, ...SHORT_ROUTE_GRAPH],
		["result boundary assign_primary_task_to_group"],
	);

	add(
		"the-discarded-terrain-lookup-per-waypoint",
		"decision D4: create_generic_waypoint_route reads get_3d_terrain_elevation at every waypoint and discards it; no leg is long enough to sample the route search",
		transaction({ environment: true }),
		[
			`transmit-int t0 ${CHECKSUM} 56`,
			"terrain 461c4000 467a0000 00000000",
			"terrain 461c4000 467a0000 00000000",
			"terrain 463b8000 467a0000 00000000",
			"terrain 465ac000 467a0000 00000000",
			"terrain 466a6000 467a0000 00000000",
			"terrain 461c4000 467a0000 00000000",
			"transmit-waypoint-route t0 g0 keysite0",
		],
	);

	add(
		"a-map-without-a-road-table-asserts",
		"decision D3: ai_misc.c :: get_closest_road_node ASSERT (road_node_positions), at the first waypoint, after the checksum",
		transaction({ roads: false }),
		[`transmit-int t0 ${CHECKSUM} 56`, "result assert road_node_positions"],
		["transmit-waypoint-route", "campaign mission-assigned", BOUNDARY],
	);

	add(
		"single-player-transmits-nothing",
		"en_comms.c: no transmission in a single player session; the campaign screen still hears MISSION_ASSIGNED",
		transaction({ singlePlayer: true }),
		["campaign mission-assigned t0", `${BOUNDARY} g0 guide30 ffffffff`, ...SHORT_ROUTE_GRAPH],
		["transmit"],
	);

	add(
		"the-campaign-screen-hears-only-a-running-campaign",
		"ca_msgs.c :: notify_campaign_screen: nothing unless the game status is INITIALISED and the game a campaign or skirmish",
		transaction({ campaign: false }),
		["transmit-switch-list guide30 17 g0 17", "transmit-switch-list t0 40 keysite0 3", `${BOUNDARY} g0 guide30 ffffffff`, "task-assignment t0 state 1"],
		["campaign mission-assigned"],
	);

	//
	// Decision D2. A 7000 m first leg from the leader at (9208, 16000) to the
	// pick-up at keysite0 (9208, 23000) is split once. Its 8 samples lie on the
	// perpendicular through (9208, 19500), 7000 / 24 = 291.66665649 m apart
	// from ceil (9208 - 4 x 291.66665649) = 8042: x = 8042, 8334, 8626, 8918,
	// 9210, 9502, 9794, 10086. The terrain is 0 m below x = 8700 (one 8700 m
	// cell column) and 1000 m elsewhere, so only samples 1 and 2 have a zero
	// average (sample 1 averages samples 1, 1, 2; sample 2 samples 1, 2, 3).
	// Their ratings are the range bias 0.5 x |2s - 8| / 16 (0.1875, 0.125)
	// plus the side bias 1.0 x (sector side != side). Sample 1 is in sector
	// (0, 2), BLUE; sample 2 in sector (1, 2), RED. The group's
	// INT_TYPE_SECTOR_SIDE is en_int.c's default 0 (NEUTRAL), so both sides
	// differ: 1.1875 against 1.125, and sample 2 (8334, 0, 19500) wins. Were
	// the group's own side (BLUE) compared, sample 1 would (0.1875 < 1.125).
	// The node survives optimise_route (|dot| 0.8826 <= 0.94). Checksum:
	// 27834 + 4 x 9208 + 103000 = 167666 = 242 (mod 256). Terrain lookups: the
	// 8 samples when the route is created, the same 8 in the second pass, then
	// one per waypoint.
	//
	{
		const route = [
			node(9208, 23000, W.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, "keysite0"),
			node(9208, 25000, W.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF, "NULL"),
			node(9208, 27000, W.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, "keysite1"),
			node(9208, 28000, W.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF, "NULL"),
		];
		const samples = ["45fb5000 46985800 00000000", "46023800 46985800 00000000", "4606c800 46985800 00000000", "460b5800 46985800 447a0000", "460fe800 46985800 447a0000", "46147800 46985800 447a0000", "46190800 46985800 447a0000", "461d9800 46985800 447a0000"].map((s) => `terrain ${s}`);
		add(
			"the-side-bias-compares-with-the-neutral-default",
			"decision D2: croute.c :: get_route_point_rating compares sector sides with get_local_entity_int_value (group, INT_TYPE_SECTOR_SIDE), which no group file overloads (en_int.c default 0)",
			transaction({
				sites: [
					{ subType: AIRBASE, x: 9208, z: 23000 },
					{ subType: FARP, x: 9208, z: 27000 },
				],
				leader: { x: 9208, z: 16000 },
				route,
				terrain: { kind: "terrain", defaultElevation: 1000, cellSize: 8700, cellsX: 1, cellsZ: 3, cells: [0, 0, 0] },
				environment: true,
				extra: [
					{ kind: "sector-state", sector: "sector0_2", blue: 1, red: 0, samNeutral: 0, samBlue: 0, samRed: 0 },
					{ kind: "sector-state", sector: "sector1_2", blue: 0, red: 1, samNeutral: 0, samBlue: 0, samRed: 0 },
				],
			}),
			[
				...samples,
				...samples,
				`transmit-int t0 ${CHECKSUM} 242`,
				"terrain 460fe000 467a0000 447a0000",
				"terrain 46023800 46985800 00000000",
				"terrain 460fe000 46b3b000 447a0000",
				"transmit-waypoint-route t0 g0 keysite0 start 460fe000 00000000 467a0000 stop 460fe000 00000000 46b3b000 checksum 242 waypoints wp24 wp25 wp26 wp27 wp28 wp29 wp30",
				`${BOUNDARY} g0 guide31 ffffffff`,
				// the pick-up and the generated waypoint (NAVIGATION, 'B'), each 3607.4748 m from the one before;
				// road node n1 (10000, 0, 16000) is the closest linked node to both
				"waypoint wp26 26 task t0 sub 19 formation 2 position-type 0 460fe000 3f800000 46b3b000 route-node 1 altitude 41c00000 flight-time 42a4b35e tag 88 dependent keysite0",
				"waypoint wp25 25 task t0 sub 17 formation 2 position-type 0 46023800 3f800000 46985800 route-node 1 altitude 41c00000 flight-time 42a4b35e tag 66 dependent NULL",
			],
		);
	}

	//
	// A fixed wing supply group (gp_dbase.c: MEDIUM_LIFT_TRANSPORT_AIRCRAFT,
	// ENTITY_TYPE_FIXED_WING) led by a C-130J (cruise 260 knots, 133.99617 m/s,
	// 0x4305ff05; cruise altitude 700 m, 0x442f0000): the waypoint database's
	// fixed wing column gives RADIUS 150 (0x43160000) and no LAST_TO_REACH, so
	// only RADIUS is transmitted; flight times 2000 m -> 0x416ed013.
	//
	add(
		"a-fixed-wing-group-takes-the-fixed-wing-waypoint-column",
		"guide.c :: initialise_guide_criteria and attach_group_to_guide_entity: get_waypoint_database_* by group_database [].default_entity_type",
		transaction({
			groupType: MLT_AIRCRAFT,
			leader: "none",
			members: [{ kind: "add-member", label: "g0.m0", group: "g0", type: EntityType.ENTITY_TYPE_FIXED_WING, subType: C130, x: 10000, z: 16000 }],
		}),
		[
			"transmit-switch-list guide30 17 g0 17",
			"transmit-guide-criteria guide30 0 1 43160000",
			"transmit guide30 149 4305ff05",
			`${BOUNDARY} g0 guide30 ffffffff`,
			"waypoint wp26 26 task t0 sub 18 formation 2 position-type 0 463b8000 3f800000 467a0000 route-node 2 altitude 442f0000 flight-time 416ed013",
		],
		["transmit-guide-criteria guide30 4"],
	);

	// ---- route-choice canaries (#9)

	for (const canary of ROUTE_CHOICE_CANARIES) {
		add(
			`route-choice-canary-${canary.id}`,
			`croute.c :: get_best_point: the strictly lower rating (first of equals) of samples 1 and 8 in near-tie; variants choose canonical ${canary.choices.canonical}, sse-rn ${canary.choices["sse-rn"]}, x87-rtz-pc53 ${canary.choices["x87-rtz-pc53"]}, x87-rn-pc53 ${canary.choices["x87-rn-pc53"]}`,
			routeChoiceCanarySpec(canary.elevations),
			[`${BOUNDARY} g0 guide31 ffffffff`, routeChoiceWaypoint(canary.choices.canonical)],
		);
	}

	return cases;
}

export const SUPPLY_TASK_TRANSACTION_CASES: SupplyTaskTransactionCase[] = build();

// "" when the output meets the case's expectations
export function supplyTaskTransactionExpectationFailure(c: SupplyTaskTransactionCase, output: string[], firstUnmatchedLine: (output: string[], expected: string[]) => string): string {
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

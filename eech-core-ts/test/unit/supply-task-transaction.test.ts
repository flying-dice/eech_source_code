//
// Slice 6b: the transaction matrix (test/scenarios/supply-task-transaction.cases.ts)
// and the recorded random and migrated scenarios under JavaScript semantics;
// the production boundary (assign_task_to_group_members fails loudly with the
// group, guide and valid members); the paths no valid supply scenario takes,
// hand-derived; the undefined behaviour the port surfaces; and the mechanical
// invariants behind the slice's coverage exclusions and the corpus's state.
// The same cases run against the original C in
// test/c-reference/supply-task-transaction.cref.test.ts and under Lua 5.1.
//

import { describe, expect, it } from "vitest";
import { initialiseCampaignCore } from "../../src";
import { assignKeysiteTasks, assignPrimaryTaskToGroup, assignTaskToGroup, pushTaskOntoGroupTaskStack } from "../../src/ai/taskgen/assign";
import { createGenericWaypointRoute, generateRouteCheckSum, optimiseRoute, parserTaskWaypointRoute, secondPastRoute, type RouteNode } from "../../src/ai/taskgen/croute";
import { getClosestRoadNode } from "../../src/ai/ai_misc/ai_misc";
import { EechAssertionError, EechFatalError, EechNullDereferenceError, EechUndefinedBehaviourError, UnportedBehaviourError, UnportedBoundaryError } from "../../src/core/assert";
import { getInverseSquareRoot } from "../../src/core/maths/invsqrt";
import { getApprox3dRange } from "../../src/core/maths/range";
import type { AircraftRaw } from "../../src/entity/mobile/aircraft/ac_float";
import { clearedTaskGeneration, type ForceRaw } from "../../src/entity/special/force/force";
import { clearedGuideRaw, createClientServerGuideEntity, getGuideCriteriaValid, initialiseGuideCriteria, setClientServerGuideCriteriaValid, type GuideRaw } from "../../src/entity/special/guide/guide";
import type { GroupRaw } from "../../src/entity/special/group/group";
import type { KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { getLocalEntityLandingEntity } from "../../src/entity/special/landing/landing";
import { createLocalSectorEntities, getLocalRawSectorEntity, type SectorRaw } from "../../src/entity/special/sector/sector";
import { clearedTaskRaw, getLocalGroupPrimaryTask, type TaskRaw } from "../../src/entity/special/task/task";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { clearedWaypointRaw, type WaypointRaw } from "../../src/entity/special/waypoint/waypoint";
import { getWaypointSubTypeTag, resetWaypointTags, updateLocalEntityWaypointListTags } from "../../src/entity/special/waypoint/wp_char";
import { getWaypointDatabaseVelocityValue } from "../../src/entity/special/waypoint/wp_dbase";
import { setCommsModel } from "../../src/entity/system/comms";
import { createLocalEntity } from "../../src/entity/system/en_creat";
import { createLocalEntityRaw, ENTITY_INDEX_DONT_CARE } from "../../src/entity/system/en_heap";
import { deleteLocalEntityFromParentsChildList, getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { notifyLocalEntity } from "../../src/entity/system/en_msgs";
import {
	getLocalEntityCharValue,
	getLocalEntityFloatValue,
	getLocalEntityIntValue,
	setLocalEntityFloatValue,
	setLocalEntityIntValue,
	setLocalEntityRawFloatValue,
	setLocalEntityRawIntValue,
} from "../../src/entity/system/en_values";
import { boundPositionToAdjustedMapVolume, getWorldMap, MAP_PERIMETER_SIZE, setEntityWorldMapSize } from "../../src/entity/system/en_world";
import { getEntityHeapState, getLocalEntityData, setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { AIRCRAFT_DATABASE_CRUISE_VELOCITY } from "../../src/generated/c-aircraft-database";
import { GROUP_DATABASE_DEFAULT_LANDING_TYPE } from "../../src/generated/c-group-database";
import { TASK_ASSIGN_ALL_MEMBERS } from "../../src/generated/c-constants";
import {
	CharType,
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeAircraft,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntitySubTypeWaypoint,
	EntityType,
	FloatType,
	FormationType,
	GuideCriteriaType,
	IntType,
	ListType,
	MovementType,
	PositionType,
	TaskCategoryType,
	TaskStateType,
} from "../../src/generated/c-enums";
import { TASK_DATABASE_ADD_START_WAYPOINT, TASK_DATABASE_ASSESS_LANDING, TASK_DATABASE_PRIMARY_TASK, TASK_DATABASE_TASK_ROUTE_SEARCH } from "../../src/generated/c-task-database";
import { WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE } from "../../src/generated/c-waypoint-database";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { ScriptedClock } from "../adapters/scripted-clock";
import { generateRandomSupplyTaskTransaction, scenarioAircraftPositions } from "../c-reference/random-scenarios";
import { C_REFERENCE_MIGRATED_SUPPLY_TASK_TRANSACTION } from "../scenarios/generated/c-reference-migrated-supply-task-transaction.cases";
import { C_REFERENCE_RANDOM_SUPPLY_TASK_TRANSACTION } from "../scenarios/generated/c-reference-random-supply-task-transaction.cases";
import { firstUnmatchedLine, runLifecycle, type LifecycleSpec } from "../scenarios/lifecycle-scenario";
import { SUPPLY_TASK_ASSIGNMENT_CASES } from "../scenarios/supply-task-assignment.cases";
import { SUPPLY_TASK_TRANSACTION_CASES, supplyTaskTransactionExpectationFailure } from "../scenarios/supply-task-transaction.cases";

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const MLT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER;
const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;
const W = EntitySubTypeWaypoint;

describe("assign.c's transaction for a SUPPLY task, to assign_task_to_group_members", () => {
	for (const c of SUPPLY_TASK_TRANSACTION_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			const output = runLifecycle(c.spec);

			expect(supplyTaskTransactionExpectationFailure(c, output, firstUnmatchedLine), output.join("\n")).toBe("");
		});
	}
});

describe("recorded supply task transactions (outputs of the original C)", () => {
	for (const c of [...C_REFERENCE_RANDOM_SUPPLY_TASK_TRANSACTION, ...C_REFERENCE_MIGRATED_SUPPLY_TASK_TRANSACTION]) {
		it(c.id, () => {
			expect(runLifecycle(c.spec)).toEqual(c.expected);
		});
	}
});

//
// Every scenario of the Slice 6b corpus restores or constructs aircraft
// inside the world map: an aircraft off the map is not a state this campaign
// path can hold (the route search then never terminates, and EECH has no
// guard). The corpus: the transaction matrix, the 6a matrix cases that run the
// transaction, the recorded and migrated fixtures, and fresh generated
// scenarios.
//
describe("the Slice 6b corpus holds every aircraft inside the map", () => {
	const inMap = (id: string, spec: LifecycleSpec): void => {
		const { maxX, maxZ, positions } = scenarioAircraftPositions(spec);

		expect(maxX, `${id}: a world map`).toBeGreaterThan(0);

		for (const p of positions) {
			expect(p.x >= 0 && p.x <= maxX && p.z >= 0 && p.z <= maxZ, `${id}: (${p.x}, ${p.z}) in 0..${maxX} x 0..${maxZ}`).toBe(true);
		}
	};

	it("the hand matrices and the recorded fixtures", () => {
		for (const c of SUPPLY_TASK_TRANSACTION_CASES) {
			inMap(c.id, c.spec);
		}
		for (const c of SUPPLY_TASK_ASSIGNMENT_CASES.filter((x) => x.transaction)) {
			inMap(c.id, c.spec);
		}
		for (const c of [...C_REFERENCE_RANDOM_SUPPLY_TASK_TRANSACTION, ...C_REFERENCE_MIGRATED_SUPPLY_TASK_TRANSACTION]) {
			inMap(c.id, c.spec);
		}
	});

	it("fresh generated scenarios", () => {
		generateRandomSupplyTaskTransaction(0x6b1a, 500).forEach((spec, i) => inMap(`fresh ${i}`, spec));
	});
});

describe("the database facts behind the slice's coverage exclusions", () => {
	it("SUPPLY is a primary task that adds a start waypoint, assesses landing and searches its route", () => {
		// the port assigns SUPPLY tasks only (assign.ts): these make the other arms of
		// create_generic_waypoint_route and assign_task_to_group unreachable
		expect(TASK_DATABASE_PRIMARY_TASK[SUPPLY]).toBe(1);
		expect(TASK_DATABASE_ADD_START_WAYPOINT[SUPPLY]).toBe(1);
		expect(TASK_DATABASE_ASSESS_LANDING[SUPPLY]).toBe(1);
		expect(TASK_DATABASE_TASK_ROUTE_SEARCH[SUPPLY]).toBe(1);
	});

	it("every aircraft has a positive cruise velocity (a member's flight time is range / cruise velocity)", () => {
		for (let i = 0; i < AIRCRAFT_DATABASE_CRUISE_VELOCITY.length; i++) {
			expect(AIRCRAFT_DATABASE_CRUISE_VELOCITY[i], `aircraft ${i}`).toBeGreaterThan(0);
		}
	});

	it("every sector is BLUE or RED, never the group's side 0 (decision D2): the route side bias always applies", () => {
		world();
		const sector = getLocalRawSectorEntity(0, 0) as Entity;
		const raw = getLocalEntityData<SectorRaw>(sector);
		for (const sides of [
			[0, 0],
			[1, 0],
			[0, 1],
			[1, 1],
		]) {
			raw.sector_side[BLUE] = sides[0];
			raw.sector_side[EntitySide.ENTITY_SIDE_RED_FORCE] = sides[1];
			expect(getLocalEntityIntValue(sector, IntType.INT_TYPE_SECTOR_SIDE)).not.toBe(EntitySide.ENTITY_SIDE_NEUTRAL);
		}
	});

	it("NAVIGATION has no minimum previous waypoint distance in any column (the parser's second arm, decision D5)", () => {
		expect(WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE[W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION]).toEqual([0, 0, 0, 0]);
	});

	it("no waypoint type of a supply route has one either: the parser never moves a supply waypoint", () => {
		for (const type of [W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, W.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, W.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF, W.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, W.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF, W.ENTITY_SUB_TYPE_WAYPOINT_LAND]) {
			expect(WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE[type], W[type]).toEqual([0, 0, 0, 0]);
		}
	});
});

//
// A valid restored world, built directly: a BLUE force; a 4 x 4 map of 8192 m
// sectors; a flat terrain; a road node at (10000, 0, 16000); an airbase at
// (10000, 16000) and a FARP at (14000, 16000); an idle, registered medium
// lift group at the airbase led by a UH-60 there; a critical SUPPLY task at
// the airbase with the short route of the transaction matrix.
//

interface World {
	replication: RecordingEntityReplication;
	events: RecordingCampaignEvents;
	roads: InMemoryRoadNetwork;
	force: Entity;
	airbase: Entity;
	farp: Entity;
	group: Entity;
	leader: Entity;
	task: Entity;
}

function point(x: number, z: number): { x: number; y: number; z: number } {
	return { x, y: 0, z };
}

function world(roads = true, leaderAboard = true): World {
	const physical = new InMemoryMobilePhysicalState();
	const replication = new RecordingEntityReplication();
	const events = new RecordingCampaignEvents();
	const network = new InMemoryRoadNetwork();
	initialiseCampaignCore(
		{
			mobilePhysicalState: physical,
			entityReplication: replication,
			clock: new ScriptedClock(),
			object3DMetadata: new InMemoryObject3DMetadata(),
			campaignEvents: events,
			terrainElevation: new GridTerrainElevation(),
			roadNetwork: network,
		},
		{ numberOfEntities: 128 },
	);
	if (roads) {
		network.addRoadNode(point(10000, 16000), 1);
	}
	const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
	setSessionEntityRaw(session);
	setUpdateEntity(createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {}));
	setEntityWorldMapSize(4, 4, 8192);
	createLocalSectorEntities();
	const forceRaw: ForceRaw = { side: BLUE, task_generation: clearedTaskGeneration() };
	const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, forceRaw);
	insertLocalEntityIntoParentsChildListRaw(force, ListType.LIST_TYPE_FORCE, session, undefined);
	const keysite = (subType: EntitySubTypeKeysite, x: number): Entity => {
		const raw: KeysiteRaw = {
			sub_type: subType,
			side: BLUE,
			alive: 1,
			in_use: 1,
			position: point(x, 16000),
			supplies: { ammo_supply_level: 100, fuel_supply_level: 100 },
			landing_types: 4,
			keysite_usable_state: 0,
		};
		const en = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, raw);
		insertLocalEntityIntoParentsChildListRaw(en, ListType.LIST_TYPE_KEYSITE_FORCE, force, undefined);
		return en;
	};
	const airbase = keysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, 10000);
	const farp = keysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP, 14000);
	const groupRaw: GroupRaw = {
		sub_type: MLT,
		side: BLUE,
		alive: 1,
		supplies: { ammo_supply_level: 100, fuel_supply_level: 100 },
		sleep: 0,
		assist_timer: 0,
		member_count: 1,
		group_list_type: ListType.LIST_TYPE_KEYSITE_GROUP,
	};
	const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, groupRaw);
	insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_KEYSITE_GROUP, airbase, undefined);
	insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_AIR_REGISTRY, force, undefined);
	const leaderRaw: AircraftRaw = { mob: { sub_type: EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK } };
	const leader = createLocalEntityRaw(EntityType.ENTITY_TYPE_HELICOPTER, leaderRaw);
	if (leaderAboard) {
		insertLocalEntityIntoParentsChildListRaw(leader, ListType.LIST_TYPE_MEMBER, group, undefined);
	}
	physical.setMobilePosition(leader.index, point(10000, 16000));
	const taskRaw: TaskRaw = clearedTaskRaw();
	taskRaw.sub_type = SUPPLY;
	taskRaw.side = BLUE;
	taskRaw.task_state = TaskStateType.TASK_STATE_UNASSIGNED;
	taskRaw.critical_task = 1;
	taskRaw.task_priority = 4;
	taskRaw.expire_timer = 1200;
	taskRaw.route_length = 4;
	taskRaw.route_nodes = [point(10000, 16000), point(12000, 16000), point(14000, 16000), point(15000, 16000)];
	taskRaw.route_waypoint_types = [W.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, W.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF, W.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, W.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF];
	taskRaw.route_formation_types = [2, 2, 2, 2];
	taskRaw.route_dependents = [airbase, undefined, farp, undefined];
	const task = createLocalEntityRaw(EntityType.ENTITY_TYPE_TASK, taskRaw);
	insertLocalEntityIntoParentsChildListRaw(task, ListType.LIST_TYPE_UNASSIGNED_TASK, airbase, undefined);
	return { replication, events, roads: network, force, airbase, farp, group, leader, task };
}

function boundaryOf(f: () => unknown): UnportedBoundaryError {
	try {
		f();
	} catch (e) {
		if (e instanceof UnportedBoundaryError) {
			return e;
		}
		throw e;
	}
	throw new Error("no boundary");
}

describe("the slice 6b boundary: assign_task_to_group_members fails loudly", () => {
	it("production throws UnportedBoundaryError carrying the group, the guide and the valid members, with the task ASSIGNED", () => {
		const w = world();

		const e = boundaryOf(() => assignKeysiteTasks(w.airbase, TaskCategoryType.TASK_CATEGORY_SUPPORT));

		expect(e).toBeInstanceOf(UnportedBehaviourError);
		expect(e.boundary).toBe("assign.c :: assign_task_to_group_members");
		const guide = getLocalEntityFirstChild(w.group, ListType.LIST_TYPE_GUIDE_STACK) as Entity;
		expect(e.args).toEqual([w.group, guide, TASK_ASSIGN_ALL_MEMBERS]);
		expect(getLocalEntityParent(guide, ListType.LIST_TYPE_GUIDE)).toBe(w.task);
		expect(getLocalEntityIntValue(w.task, IntType.INT_TYPE_TASK_STATE)).toBe(TaskStateType.TASK_STATE_ASSIGNED);
		expect(getLocalEntityFirstChild(w.airbase, ListType.LIST_TYPE_ASSIGNED_TASK)).toBe(w.task);
		expect(getLocalEntityFirstChild(w.airbase, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBe(undefined);
		// no member follows the guide: the member assignment did not start
		expect(getLocalEntityFirstChild(guide, ListType.LIST_TYPE_FOLLOWER)).toBe(undefined);
		expect(getLocalGroupPrimaryTask(w.group)).toBe(w.task);
		// the return keysite is the group's keysite
		expect(getLocalEntityData<TaskRaw>(w.task).return_keysite).toBe(w.airbase);
	});

	it("a group whose guide stack already holds a primary task is an ASSERT", () => {
		const w = world();
		boundaryOf(() => assignKeysiteTasks(w.airbase, TaskCategoryType.TASK_CATEGORY_SUPPORT));
		expect(() => assignPrimaryTaskToGroup(w.group, w.task)).toThrow(EechAssertionError);
		expect(() => assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).toThrow(EechAssertionError);
	});
});

describe("assign.c :: assign_task_to_group paths no valid supply selection takes", () => {
	it("returns FALSE for a group without members, and assign_primary_task_to_group then returns FALSE", () => {
		const w = world(true, false);
		expect(assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).toBe(false);
		expect(assignPrimaryTaskToGroup(w.group, w.task)).toBe(0);
		expect(getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT)).toBe(undefined);
	});

	it("returns FALSE for an assault ship given anything but ENGAGE, and for the landing and takeoff tasks", () => {
		const w = world();
		getLocalEntityData<GroupRaw>(w.group).sub_type = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP;
		expect(assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).toBe(false);
		getLocalEntityData<GroupRaw>(w.group).sub_type = MLT;
		for (const type of [EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_LANDING, EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_LANDING_HOLDING, EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_TAKEOFF, EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING]) {
			getLocalEntityData<TaskRaw>(w.task).sub_type = type;
			expect(assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS), EntitySubTypeTask[type]).toBe(false);
		}
		// an assault ship given ENGAGE passes that check (ENGAGE assesses no landing: fails at its route, not ported here)
		getLocalEntityData<GroupRaw>(w.group).sub_type = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP;
		getLocalEntityData<TaskRaw>(w.task).sub_type = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE;
		expect(() => assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).toThrow();
	});

	it("a return keysite other than the group's own asks for its free landing sites: not ported", () => {
		const w = world();
		getLocalEntityData<TaskRaw>(w.task).return_keysite = w.farp;
		expect(() => assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).toThrow(new UnportedBehaviourError(`landing.c :: get_keysite_landing_sites_available (landing type ${GROUP_DATABASE_DEFAULT_LANDING_TYPE[MLT]})`));
	});

	it("the group's own keysite as the return keysite needs no landing site check", () => {
		const w = world();
		getLocalEntityData<TaskRaw>(w.task).return_keysite = w.airbase;
		expect(boundaryOf(() => assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).boundary).toBe("assign.c :: assign_task_to_group_members");
	});

	it("a group not listed at a keysite has no start keysite: get_closest_keysite is not ported", () => {
		const w = world();
		getLocalEntityData<GroupRaw>(w.group).group_list_type = ListType.LIST_TYPE_INDEPENDENT_GROUP;
		expect(() => assignTaskToGroup(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS)).toThrow(UnportedBehaviourError);
	});

	it("a keysite with landing entities is not ported (landing.c)", () => {
		const w = world();
		const landing = createLocalEntityRaw(EntityType.ENTITY_TYPE_LANDING, {});
		expect(() => getLocalEntityLandingEntity(w.airbase, 0)).not.toThrow();
		expect(getLocalEntityLandingEntity(w.airbase, 0)).toBe(undefined);
		insertLocalEntityIntoParentsChildListRaw(landing, ListType.LIST_TYPE_LANDING_SITE, w.airbase, undefined);
		expect(() => getLocalEntityLandingEntity(w.airbase, 0)).toThrow(UnportedBehaviourError);
	});
});

describe("assign.c :: push_task_onto_group_task_stack", () => {
	it("moves only a task on a keysite's unassigned list; the guide is created and attached either way", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		deleteLocalEntityFromParentsChildList(w.task, ListType.LIST_TYPE_UNASSIGNED_TASK);
		const guide = pushTaskOntoGroupTaskStack(w.group, w.task, TASK_ASSIGN_ALL_MEMBERS);
		expect(getLocalEntityFirstChild(w.group, ListType.LIST_TYPE_GUIDE_STACK)).toBe(guide);
		expect(getLocalEntityIntValue(w.task, IntType.INT_TYPE_TASK_STATE)).toBe(TaskStateType.TASK_STATE_UNASSIGNED);
		expect(getLocalEntityFirstChild(w.airbase, ListType.LIST_TYPE_ASSIGNED_TASK)).toBe(undefined);

		// an ASSIGNED task (another member's) stays where it is; its new guide goes on top of the stack
		const other = world();
		createGenericWaypointRoute(other.group, other.task, other.airbase);
		getLocalEntityData<TaskRaw>(other.task).task_state = TaskStateType.TASK_STATE_ASSIGNED;
		const first = pushTaskOntoGroupTaskStack(other.group, other.task, 1);
		getLocalEntityData<TaskRaw>(other.task).sub_type = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ESCORT;
		const second = pushTaskOntoGroupTaskStack(other.group, other.task, 2);
		expect(getLocalEntityFirstChild(other.group, ListType.LIST_TYPE_GUIDE_STACK)).toBe(second);
		expect(first).not.toBe(second);
		expect(getLocalEntityParent(other.task, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBe(other.airbase);
	});

	it("a primary task twice on a guide stack is an ASSERT (get_local_group_primary_task: count <= 1)", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		pushTaskOntoGroupTaskStack(w.group, w.task, 1);
		pushTaskOntoGroupTaskStack(w.group, w.task, 2);
		expect(() => getLocalGroupPrimaryTask(w.group)).toThrow(new EechAssertionError("count <= 1"));
	});
});

describe("croute.c :: create_generic_waypoint_route paths no valid supply selection takes", () => {
	it("a task that already has its waypoint route is left as it is", () => {
		const w = world();
		expect(createGenericWaypointRoute(w.group, w.task, w.airbase)).toBe(true);
		const first = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT);
		const transmitted = w.replication.events.length;
		expect(createGenericWaypointRoute(w.group, w.task, w.airbase)).toBe(true);
		expect(getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT)).toBe(first);
		expect(w.replication.events.length).toBe(transmitted);
	});

	it("the client's rebuild from ENTITY_COMMS_CREATE_WAYPOINT_ROUTE is not ported", () => {
		const w = world();
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		expect(() => createGenericWaypointRoute(w.group, w.task, w.airbase)).toThrow(UnportedBehaviourError);
		setCommsModel(CommsModelType.COMMS_MODEL_SERVER);
	});

	it("a task without route nodes reads specified_route uninitialised (undefined behaviour)", () => {
		const w = world();
		const raw = getLocalEntityData<TaskRaw>(w.task);
		raw.route_length = 0;
		expect(() => createGenericWaypointRoute(w.group, w.task, w.airbase)).toThrow(EechUndefinedBehaviourError);
		// with a length but no arrays, the arrays are NULL pointers
		raw.route_length = 1;
		raw.route_nodes = undefined;
		expect(() => createGenericWaypointRoute(w.group, w.task, w.airbase)).toThrow(EechNullDereferenceError);
	});

	it("a route of one point has no route to checksum: generate_route_check_sum dereferences NULL", () => {
		expect(() => generateRouteCheckSum(undefined)).toThrow(EechNullDereferenceError);
	});

	it("a map without a road table asserts at the first waypoint (decision D3), after the checksum", () => {
		const w = world(false);
		expect(() => createGenericWaypointRoute(w.group, w.task, w.airbase)).toThrow(new EechAssertionError("road_node_positions"));
		expect(w.replication.events.map((e) => e.kind)).toEqual(["int_value"]);
	});
});

describe("croute.c :: second_past_route reads best_point uninitialised when get_best_point fails first", () => {
	const routeOf = (points: [number, number][]): RouteNode => {
		const nodes: RouteNode[] = points.map(([x, z]) => ({ type: W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, formation: 0, dependent: undefined, position: point(x, z), next: undefined, prev: undefined }));
		for (let i = 0; i < nodes.length; i++) {
			nodes[i].next = nodes[i + 1];
			nodes[i].prev = nodes[i - 1];
		}
		return nodes[0];
	};

	it("the first interior node between points at most 5000 m apart: undefined behaviour", () => {
		world();
		expect(() => secondPastRoute(routeOf([[10000, 10000], [11000, 10000], [12000, 10000]]), 0, 1)).toThrow(EechUndefinedBehaviourError);
	});

	it("a later failure reuses the previous iteration's point (defined)", () => {
		// iteration 1: 1000 .. 8000 (7000 m) re-places the node at 7000; iteration 2: from that
		// point to 9000 is under 5000 m, so the node at 8000 takes the same point again
		world();
		const route = routeOf([[1000, 10000], [7000, 10000], [8000, 10000], [9000, 10000]]);
		secondPastRoute(route, 0, 1);
		const second = route.next as RouteNode;
		const third = second.next as RouteNode;
		expect(second.position.x).toBeLessThan(7000);
		expect(third.position).toEqual(second.position);
	});
});

describe("croute.c :: parser_task_waypoint_route (ATTACK waypoints: the only kind with a spacing)", () => {
	// helicopter columns: ATTACK needs 5000 m after its previous waypoint
	const route = (points: [number, EntitySubTypeWaypoint][]): { w: World; waypoints: Entity[] } => {
		const w = world();
		const waypoints: Entity[] = [];
		for (const [x, type] of points) {
			const wp = createLocalEntity(EntityType.ENTITY_TYPE_WAYPOINT, ENTITY_INDEX_DONT_CARE, [
				{ kind: "parent", type: ListType.LIST_TYPE_WAYPOINT, entity: w.task },
				{ kind: "child_pred", type: ListType.LIST_TYPE_WAYPOINT, entity: waypoints[waypoints.length - 1] },
				{ kind: "vec3d", type: 8, x, y: 1, z: 10000 },
				{ kind: "int_value", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: type },
			]);
			waypoints.push(wp);
		}
		return { w, waypoints };
	};
	const x = (wp: Entity): number => getLocalEntityData<WaypointRaw>(wp).position.x;

	it("no waypoint, one or two: nothing to space", () => {
		for (const n of [0, 1, 2]) {
			const { w, waypoints } = route(
				[
					[10000, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION],
					[11000, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK],
				].slice(0, n) as [number, EntitySubTypeWaypoint][],
			);
			parserTaskWaypointRoute(w.group, w.task);
			expect(waypoints.map(x)).toEqual([10000, 11000].slice(0, n));
		}
	});

	it("a NAVIGATION waypoint 1000 m before an ATTACK moves to the midpoint of its neighbours, still too close, then 5000 m back from the ATTACK", () => {
		// last 10000, this 12000, next 13000 (ATTACK): the midpoint 11500 is 1500 m away; the half leg
		// (1500, 0, 0) normalised is 1500 x (1.0 / 1500 truncated) = 1 - 2^-24, x 5000 = 4999.99951171875,
		// and ceil (13000 - 4999.99951171875) = 8001
		const { w, waypoints } = route([
			[10000, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION],
			[12000, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION],
			[13000, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK],
		]);
		parserTaskWaypointRoute(w.group, w.task);
		expect(waypoints.map(x)).toEqual([10000, 8001, 13000]);
		expect(getLocalEntityData<WaypointRaw>(waypoints[1]).position).toEqual({ x: 8001, y: 1, z: 10000 });
	});

	it("... or stays at the midpoint when that is far enough", () => {
		// last 2000, this 12000, next 13000: the midpoint 7500 is 5500 m from the ATTACK
		const { w, waypoints } = route([
			[2000, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION],
			[12000, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION],
			[13000, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK],
		]);
		parserTaskWaypointRoute(w.group, w.task);
		expect(waypoints.map(x)).toEqual([2000, 7500, 13000]);
	});

	it("a waypoint too close to the next but not NAVIGATION, before a waypoint that is not NAVIGATION either, stays", () => {
		const { w, waypoints } = route([
			[10000, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION],
			[12000, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK],
			[13000, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK],
		]);
		parserTaskWaypointRoute(w.group, w.task);
		expect(waypoints.map(x)).toEqual([10000, 12000, 13000]);
	});
});

describe("the waypoint entity", () => {
	it("wp_char.c: navigation tags wrap after 'V'; target tags wrap after 'Z'; a landing tag 'W' leads the next target tag", () => {
		resetWaypointTags();
		const nav: number[] = [];
		for (let i = 0; i < 23; i++) {
			nav.push(getWaypointSubTypeTag(W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION));
		}
		expect(nav[21]).toBe(86);
		expect(nav[22]).toBe(65);
		expect([W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK, W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK].map(getWaypointSubTypeTag)).toEqual([88, 89, 90, 88]);
		expect(getWaypointSubTypeTag(W.ENTITY_SUB_TYPE_WAYPOINT_LAND)).toBe(87);
		expect(getWaypointSubTypeTag(W.ENTITY_SUB_TYPE_WAYPOINT_ATTACK)).toBe(87);
		expect(() => getWaypointSubTypeTag(W.NUM_ENTITY_SUB_TYPE_WAYPOINTS)).toThrow(EechFatalError);
		expect(() => updateLocalEntityWaypointListTags(undefined)).toThrow(EechAssertionError);
	});

	it("leaving the task's waypoint list re-tags it; the raw setter does not; the formation ASSERT is the raw setter's", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		const first = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT) as Entity;
		const second = getLocalEntityData<{ sub_type: number }>(first) as unknown as WaypointRaw;
		expect(second.tag).toBe(65);
		deleteLocalEntityFromParentsChildList(first, ListType.LIST_TYPE_WAYPOINT);
		const pick = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT) as Entity;
		expect(getLocalEntityCharValue(pick, CharType.CHAR_TYPE_TAG)).toBe(88);
		setLocalEntityRawIntValue(pick, IntType.INT_TYPE_ENTITY_SUB_TYPE, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION);
		expect(getLocalEntityCharValue(pick, CharType.CHAR_TYPE_TAG)).toBe(88);
		expect(() => setLocalEntityRawIntValue(pick, IntType.INT_TYPE_WAYPOINT_FORMATION, FormationType.NUM_FORMATION_TYPES + 1)).toThrow(EechAssertionError);
		expect(getLocalEntityIntValue(pick, IntType.INT_TYPE_WAYPOINT_GUIDE_TYPE)).toBe(0);
		expect(getLocalEntityFloatValue(pick, FloatType.FLOAT_TYPE_HEADING)).toBe(0);
	});

	it("a VIRTUAL waypoint's position (its dependent's) is not ported; a waypoint created outside the map volume is not either", () => {
		const w = world();
		const raw = clearedWaypointRaw();
		raw.position_type = PositionType.POSITION_TYPE_VIRTUAL;
		const virtual = createLocalEntityRaw(EntityType.ENTITY_TYPE_WAYPOINT, raw);
		expect(() => getLocalEntityCharValue(virtual, CharType.CHAR_TYPE_TAG)).not.toThrow();
		expect(() => createClientServerGuideEntity(w.task, undefined, 1)).toThrow(EechAssertionError);
		insertLocalEntityIntoParentsChildListRaw(virtual, ListType.LIST_TYPE_WAYPOINT, w.task, undefined);
		expect(() => createClientServerGuideEntity(w.task, undefined, 1)).toThrow(UnportedBehaviourError);
		expect(() =>
			createLocalEntity(EntityType.ENTITY_TYPE_WAYPOINT, ENTITY_INDEX_DONT_CARE, [
				{ kind: "parent", type: ListType.LIST_TYPE_WAYPOINT, entity: w.task },
				{ kind: "vec3d", type: 8, x: -1, y: 1, z: 1 },
			]),
		).toThrow(UnportedBehaviourError);
		// a dependent given at creation: the dependent's LIST_TYPE_TASK_DEPENDENT list
		const wp = createLocalEntity(EntityType.ENTITY_TYPE_WAYPOINT, ENTITY_INDEX_DONT_CARE, [
			{ kind: "parent", type: ListType.LIST_TYPE_WAYPOINT, entity: w.task },
			{ kind: "parent", type: ListType.LIST_TYPE_TASK_DEPENDENT, entity: w.farp },
		]);
		expect(getLocalEntityFirstChild(w.farp, ListType.LIST_TYPE_TASK_DEPENDENT)).toBe(wp);
		expect(() => getWaypointDatabaseVelocityValue(W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, EntityType.ENTITY_TYPE_KEYSITE)).toThrow(EechFatalError);
		for (const type of [EntityType.ENTITY_TYPE_ROUTED_VEHICLE, EntityType.ENTITY_TYPE_SHIP_VEHICLE]) {
			expect(getWaypointDatabaseVelocityValue(W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, type)).toBe(1);
		}
	});
});

describe("the guide entity", () => {
	it("a first waypoint given must be the task's", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		const second = world();
		const foreign = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT) as Entity;
		expect(() => createClientServerGuideEntity(second.task, foreign, 1)).toThrow(EechAssertionError);
		const own = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT) as Entity;
		const guide = createClientServerGuideEntity(w.task, own, 1) as Entity;
		expect(getLocalEntityParent(guide, ListType.LIST_TYPE_CURRENT_WAYPOINT)).toBe(own);
		expect(getLocalEntityIntValue(guide, IntType.INT_TYPE_VALID_GUIDE_MEMBERS)).toBe(1);
	});

	it("initialise_guide_criteria: a guide type's criteria are set, and every criterion the type lacks is cleared (transmitted as FALSE, 0.0)", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		const guide = createClientServerGuideEntity(w.task, undefined, 1) as Entity;
		insertLocalEntityIntoParentsChildListRaw(guide, ListType.LIST_TYPE_GUIDE_STACK, w.group, undefined);
		setClientServerGuideCriteriaValid(guide, GuideCriteriaType.GUIDE_CRITERIA_HEADING, 1, 2.5);
		// the LANDED guide sets HEADING (2 pi) and ALTITUDE (0.5)
		getLocalEntityData<GuideRaw>(guide).sub_type = 5;
		initialiseGuideCriteria(guide);
		expect(getLocalEntityData<GuideRaw>(guide).criteria[GuideCriteriaType.GUIDE_CRITERIA_ALTITUDE]).toEqual({ valid: 1, value: 0.5 });
		// back to NAVIGATION_DIRECT: HEADING and ALTITUDE are cleared
		getLocalEntityData<GuideRaw>(guide).sub_type = 0;
		const before = w.replication.events.length;
		initialiseGuideCriteria(guide);
		expect(getGuideCriteriaValid(guide, GuideCriteriaType.GUIDE_CRITERIA_HEADING)).toBe(0);
		// the guide database loop clears every criterion NAVIGATION_DIRECT lacks, RADIUS and LAST_TO_REACH
		// included; the waypoint database then sets those two again: each is transmitted twice
		const criterion = (type: GuideCriteriaType, valid: number, value: number) => ({ kind: "guide_criteria", guideIndex: guide.index, type, valid, value });
		expect(w.replication.events.slice(before)).toEqual([
			criterion(GuideCriteriaType.GUIDE_CRITERIA_RADIUS, 0, 0),
			criterion(GuideCriteriaType.GUIDE_CRITERIA_HEADING, 0, 0),
			criterion(GuideCriteriaType.GUIDE_CRITERIA_ALTITUDE, 0, 0),
			criterion(GuideCriteriaType.GUIDE_CRITERIA_LAST_TO_REACH, 0, 0),
			criterion(GuideCriteriaType.GUIDE_CRITERIA_RADIUS, 1, 50),
			criterion(GuideCriteriaType.GUIDE_CRITERIA_LAST_TO_REACH, 1, 0),
		]);
		expect(clearedGuideRaw().criteria.length).toBe(GuideCriteriaType.NUM_GUIDE_CRITERIA_TYPES);
		expect(getLocalEntityFloatValue(guide, FloatType.FLOAT_TYPE_VELOCITY)).toBe(0);
	});
});

describe("environment ports", () => {
	it("get_closest_road_node: nodes without links are skipped; the first within the error wins; node 0 when none has links", () => {
		const w = world(false);
		w.roads.addRoadNode(point(10000, 16000), 0);
		expect(getClosestRoadNode(point(10000, 16000), 5.0)).toBe(0);
		w.roads.addRoadNode(point(12000, 16000), 1);
		w.roads.addRoadNode(point(10003, 16000), 2);
		expect(getClosestRoadNode(point(10000, 16000), 5.0)).toBe(2);
		expect(getClosestRoadNode(point(10000, 16000), 1.0)).toBe(2);
		expect(() => getClosestRoadNode(undefined, 5.0)).toThrow(EechAssertionError);
	});

	it("the terrain adapter rejects a malformed grid", () => {
		expect(() => new GridTerrainElevation().setGrid(0, 0, 1, 1, [0])).toThrow();
		expect(() => new GridTerrainElevation().setGrid(0, 1, 2, 1, [0])).toThrow();
	});
});

describe("invsqrt.c :: get_inverse_square_root", () => {
	it("1 / sqrt (x) to within its two Newton steps, for powers of four and others", () => {
		world();
		expect(getInverseSquareRoot(4)).toBeCloseTo(0.5, 6);
		expect(getInverseSquareRoot(1e6)).toBeCloseTo(0.001, 9);
		expect(getInverseSquareRoot(2)).toBeCloseTo(Math.SQRT1_2, 6);
	});
});

it("group side: every group's INT_TYPE_SECTOR_SIDE is en_int.c's default 0 (decision D2)", () => {
	const w = world();
	expect(getLocalEntityIntValue(w.group, IntType.INT_TYPE_SECTOR_SIDE)).toBe(0);
});

//
// Paths of the ported functions that no valid supply transaction reaches,
// hand-derived.
//

function fillHeap(): void {
	while (getEntityHeapState().firstFreeEntity !== -1) {
		createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {});
	}
}

describe("a full entity heap", () => {
	it("gd_creat.c: create_local and create_server find no free entity; the client-server create is fatal", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		fillHeap();
		expect(() => createClientServerGuideEntity(w.task, undefined, 1)).toThrow(EechFatalError);
	});

	it("wp_creat.c: create_local finds no free entity; the local create is fatal", () => {
		const w = world();
		fillHeap();
		expect(() => createLocalEntity(EntityType.ENTITY_TYPE_WAYPOINT, ENTITY_INDEX_DONT_CARE, [{ kind: "parent", type: ListType.LIST_TYPE_WAYPOINT, entity: w.task }])).toThrow(EechFatalError);
	});
});

describe("the waypoint's values", () => {
	it("wp_int.c, wp_float.c: every raw and local setter and getter; POSITION_TYPE and ROUTE_NODE are bitfields", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		const wp = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT) as Entity;
		for (const set of [setLocalEntityRawIntValue, setLocalEntityIntValue]) {
			set(wp, IntType.INT_TYPE_POSITION_TYPE, 1);
			expect(getLocalEntityIntValue(wp, IntType.INT_TYPE_POSITION_TYPE)).toBe(1);
			set(wp, IntType.INT_TYPE_ROUTE_NODE, 3);
			expect(getLocalEntityIntValue(wp, IntType.INT_TYPE_ROUTE_NODE)).toBe(3);
			set(wp, IntType.INT_TYPE_WAYPOINT_FORMATION, 2);
			expect(getLocalEntityIntValue(wp, IntType.INT_TYPE_WAYPOINT_FORMATION)).toBe(2);
		}
		// the local setter has no formation ASSERT
		setLocalEntityIntValue(wp, IntType.INT_TYPE_WAYPOINT_FORMATION, FormationType.NUM_FORMATION_TYPES + 1);
		expect(getLocalEntityIntValue(wp, IntType.INT_TYPE_WAYPOINT_FORMATION)).toBe(FormationType.NUM_FORMATION_TYPES + 1);
		for (const set of [setLocalEntityRawFloatValue, setLocalEntityFloatValue]) {
			set(wp, FloatType.FLOAT_TYPE_ALTITUDE, 150);
			set(wp, FloatType.FLOAT_TYPE_FLIGHT_TIME, 60);
			set(wp, FloatType.FLOAT_TYPE_HEADING, 1.5);
			expect([FloatType.FLOAT_TYPE_ALTITUDE, FloatType.FLOAT_TYPE_FLIGHT_TIME, FloatType.FLOAT_TYPE_HEADING].map((t) => getLocalEntityFloatValue(wp, t))).toEqual([150, 60, 1.5]);
		}
	});

	it("wp_msgs.c: a LINK_PARENT or UNLINK_PARENT from no sender re-tags nothing", () => {
		const w = world();
		createGenericWaypointRoute(w.group, w.task, w.airbase);
		const wp = getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT) as Entity;
		const tag = getLocalEntityCharValue(wp, CharType.CHAR_TYPE_TAG);
		resetWaypointTags();
		getWaypointSubTypeTag(W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION);
		for (const message of [EntityMessage.ENTITY_MESSAGE_LINK_PARENT, EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT]) {
			expect(notifyLocalEntity(message, wp, undefined, ListType.LIST_TYPE_WAYPOINT)).toBe(1);
		}
		expect(getLocalEntityCharValue(wp, CharType.CHAR_TYPE_TAG)).toBe(tag);
		expect(getWaypointSubTypeTag(W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION)).toBe(66);
	});

	it("wp_char.c: every waypoint sub type is a navigation, target or landing tag", () => {
		const navigation = [W.ENTITY_SUB_TYPE_WAYPOINT_CAP_START, W.ENTITY_SUB_TYPE_WAYPOINT_END, W.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION];
		const landing = [
			W.ENTITY_SUB_TYPE_WAYPOINT_APPROACH,
			W.ENTITY_SUB_TYPE_WAYPOINT_CONVOY,
			W.ENTITY_SUB_TYPE_WAYPOINT_HOLDING,
			W.ENTITY_SUB_TYPE_WAYPOINT_HOLDING_LOOP,
			W.ENTITY_SUB_TYPE_WAYPOINT_LAND,
			W.ENTITY_SUB_TYPE_WAYPOINT_LANDED,
			W.ENTITY_SUB_TYPE_WAYPOINT_LIFT_OFF,
			W.ENTITY_SUB_TYPE_WAYPOINT_LOWER_UNDERCARRIAGE,
			W.ENTITY_SUB_TYPE_WAYPOINT_REVERSE_CONVOY,
			W.ENTITY_SUB_TYPE_WAYPOINT_RAISE_UNDERCARRIAGE,
			W.ENTITY_SUB_TYPE_WAYPOINT_START_UP,
			W.ENTITY_SUB_TYPE_WAYPOINT_SUB_ROUTE_NAVIGATION,
			W.ENTITY_SUB_TYPE_WAYPOINT_TAXI,
			W.ENTITY_SUB_TYPE_WAYPOINT_TOUCH_DOWN,
			W.ENTITY_SUB_TYPE_WAYPOINT_TAKEN_OFF,
		];
		for (let type = 0; type < W.NUM_ENTITY_SUB_TYPE_WAYPOINTS; type++) {
			resetWaypointTags();
			const expected = navigation.indexOf(type) >= 0 ? 65 : landing.indexOf(type) >= 0 ? 87 : 88;
			expect(getWaypointSubTypeTag(type), W[type]).toBe(expected);
		}
	});
});

describe("the task's route checksum and the group's primary task", () => {
	it("ts_int.c: INT_TYPE_ROUTE_CHECK_SUM is an 8-bit unsigned bitfield", () => {
		const w = world();
		setLocalEntityIntValue(w.task, IntType.INT_TYPE_ROUTE_CHECK_SUM, 0x1a5);
		expect(getLocalEntityIntValue(w.task, IntType.INT_TYPE_ROUTE_CHECK_SUM)).toBe(0xa5);
	});

	it("get_local_group_primary_task skips a guide whose task is not primary", () => {
		const w = world();
		const engage = clearedTaskRaw();
		engage.sub_type = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE;
		expect(TASK_DATABASE_PRIMARY_TASK[engage.sub_type]).toBe(0);
		const task = createLocalEntityRaw(EntityType.ENTITY_TYPE_TASK, engage);
		const guide = createLocalEntityRaw(EntityType.ENTITY_TYPE_GUIDE, clearedGuideRaw());
		insertLocalEntityIntoParentsChildListRaw(guide, ListType.LIST_TYPE_GUIDE, task, undefined);
		insertLocalEntityIntoParentsChildListRaw(guide, ListType.LIST_TYPE_GUIDE_STACK, w.group, undefined);
		expect(getLocalGroupPrimaryTask(w.group)).toBeUndefined();
	});
});

describe("route geometry", () => {
	const chain = (points: [number, number][]): RouteNode => {
		const nodes: RouteNode[] = points.map(([x, z]) => ({ type: 0, formation: 0, dependent: undefined, position: point(x, z), next: undefined, prev: undefined }));
		for (let i = 0; i < nodes.length; i++) {
			nodes[i].next = nodes[i + 1];
			nodes[i].prev = nodes[i - 1];
		}
		return nodes[0];
	};

	const positions = (first: RouteNode): number[][] => {
		const out: number[][] = [];
		for (let n: RouteNode | undefined = first; n; n = n.next) {
			out.push([n.position.x, n.position.z]);
		}
		return out;
	};

	it("croute.c :: optimise_route removes a node on a zero-length leg on either side, and a straight-through node", () => {
		world();
		const first = chain([
			[0, 0],
			[0, 0],
			[1000, 0],
			[1000, 0],
			[2000, 0],
			[2000, 1000],
		]);
		optimiseRoute(first, MovementType.MOVEMENT_TYPE_AIR);
		expect(positions(first)).toEqual([
			[0, 0],
			[2000, 0],
			[2000, 1000],
		]);
	});

	it("croute.c :: generate_route_check_sum: a single node has no interior; interior nodes sum (int) x + y + z modulo 256", () => {
		expect(generateRouteCheckSum(chain([[5, 7]]))).toBe(0);
		expect(generateRouteCheckSum(chain([[1, 1], [200.9, 100.5], [1, 1]]))).toBe((200 + 0 + 100) % 256);
	});

	it("maths: get_approx_3d_range orders the three deltas: max + (med + min) / 4", () => {
		expect(getApprox3dRange(point(0, 0), { x: 1, y: 4, z: 2 })).toBe(4.75);
		expect(getApprox3dRange(point(0, 0), { x: 1, y: 2, z: 4 })).toBe(4.75);
		expect(getApprox3dRange(point(0, 0), { x: 4, y: 2, z: 1 })).toBe(4.75);
	});

	it("en_world.c :: bound_position_to_adjusted_map_volume: the perimeter in x and z, the volume in y", () => {
		world();
		const map = getWorldMap();
		const inside = { x: 10000, y: 0, z: 10000 };
		expect(boundPositionToAdjustedMapVolume(inside)).toBe(false);
		expect(inside).toEqual({ x: 10000, y: 0, z: 10000 });
		const low = { x: 0, y: -9000, z: 0 };
		expect(boundPositionToAdjustedMapVolume(low)).toBe(true);
		expect(low).toEqual({ x: map.min_map_x + MAP_PERIMETER_SIZE, y: map.min_map_y, z: map.min_map_z + MAP_PERIMETER_SIZE });
		const high = { x: map.max_map_x, y: 70000, z: map.max_map_z };
		expect(boundPositionToAdjustedMapVolume(high)).toBe(true);
		expect(high).toEqual({ x: map.max_map_x - MAP_PERIMETER_SIZE, y: map.max_map_y, z: map.max_map_z - MAP_PERIMETER_SIZE });
		for (const p of [
			{ x: 0, y: 0, z: 10000 },
			{ x: 10000, y: 70000, z: 10000 },
			{ x: 10000, y: 0, z: map.max_map_z },
		]) {
			expect(boundPositionToAdjustedMapVolume(p)).toBe(true);
		}
	});
});

//
// get_inverse_square_root against the original C (the harness's `invsqrt`
// command, test/c-reference/route-databases.cref.test.ts): float bits in, float
// bits out. Subnormal arguments, exact powers of two, and the largest float.
//
describe("invsqrt.c :: get_inverse_square_root at the float edges (bits from the original C)", () => {
	const bits = (hex: string): number => {
		const view = new DataView(new ArrayBuffer(4));
		view.setUint32(0, Number.parseInt(hex, 16));
		return view.getFloat32(0);
	};
	for (const [x, r] of [
		["00000001", "5fcb9ffc"],
		["00400000", "5f3465ad"],
		["007fffff", "5f000000"],
		["3f800000", "3f7fffff"],
		["40800000", "3effffff"],
		["41000000", "3eb504f3"],
		["3e800000", "3fffffff"],
		["7f7fffff", "1f800000"],
		["3fc00000", "3f5105ec"],
	]) {
		it(`${x} -> ${r}`, () => {
			world();
			expect(getInverseSquareRoot(bits(x))).toBe(bits(r));
		});
	}
});

describe("en_valid.c :: assert_local_create_entity_index under the client comms model", () => {
	it("a client never creates at ENTITY_INDEX_DONT_CARE; a given index depends on the comms data flow (not ported)", () => {
		const w = world();
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		const create = (index: number) => () => createLocalEntity(EntityType.ENTITY_TYPE_WAYPOINT, index, [{ kind: "parent", type: ListType.LIST_TYPE_WAYPOINT, entity: w.task }]);
		expect(create(ENTITY_INDEX_DONT_CARE)).toThrow(new EechAssertionError("assert_local_create_entity_index ((index))"));
		expect(create(100)).toThrow(UnportedBehaviourError);
		setCommsModel(CommsModelType.COMMS_MODEL_SERVER);
	});
});

//
// Slice 5b behaviour matrix (test/scenarios/supply-task-construction.cases.ts)
// under JavaScript semantics, the F1 single player / multiplayer acceptance
// case, and the create_task and task entity paths the supply path does not
// take (other task types reach them; they are ported with create_task and
// fail loudly where their callees are not ported). The same cases run against
// the original C in test/c-reference/supply-task-construction.cref.test.ts
// and under Lua 5.1.
//

import { describe, expect, it } from "vitest";
import { SUPPLY_TASK_CONSTRUCTION_CASES, f1CompatibilityPair, f1SemanticRouteFailure, supplyTaskExpectationFailure } from "../scenarios/supply-task-construction.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { C_REFERENCE_RANDOM_SUPPLY_TASK_CONSTRUCTION } from "../scenarios/generated/c-reference-random-supply-task-construction.cases";
import { initialiseCampaignCore } from "../../src";
import { createSupplyTask, createTask, getTaskStartKeysite, observeCreateSupplyTask, terminator_point } from "../../src/ai/taskgen/taskgen";
import { EechAssertionError, EechUndefinedBehaviourError, UnportedBehaviourError } from "../../src/core/assert";
import { setGameStatus } from "../../src/core/game-status";
import { setGameType } from "../../src/core/game-type";
import { calculateGroupToTaskSuitability } from "../../src/ai/highlevl/suitable";
import { clearedTaskGeneration, type ForceRaw } from "../../src/entity/special/force/force";
import type { GroupRaw } from "../../src/entity/special/group/group";
import type { KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { createLocalSectorEntities, getLocalRawSectorEntity } from "../../src/entity/special/sector/sector";
import { assessTaskDifficulty, type TaskRaw } from "../../src/entity/special/task/task";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { setCommsModel } from "../../src/entity/system/comms";
import { setEntityCommsTransmission } from "../../src/entity/system/en_comms";
import { createClientServerEntity } from "../../src/entity/system/en_creat";
import { createLocalEntityRaw, ENTITY_INDEX_DONT_CARE } from "../../src/entity/system/en_heap";
import { getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw, setClientServerEntityParent } from "../../src/entity/system/en_list";
import { notifyLocalEntity } from "../../src/entity/system/en_msgs";
import { setClientServerEntityFloatValue } from "../../src/entity/system/en_values";
import { setEntityWorldMapSize } from "../../src/entity/system/en_world";
import { getLocalEntityData, setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { notifyCampaignScreenMissionCreated } from "../../src/ui_menu/campaign/ca_msgs";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntityType,
	FloatType,
	GameStatusType,
	GameType,
	IntType,
	ListType,
	MovementType,
	TaskStateType,
} from "../../src/generated/c-enums";
import {
	GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED,
	GROUP_DATABASE_AI_STATS_MOVEMENT_STEALTH,
	GROUP_DATABASE_DEFAULT_LANDING_TYPE,
	GROUP_DATABASE_MOVEMENT_TYPE,
} from "../../src/generated/c-group-database";
import { TASK_DATABASE_AI_STATS_MOVEMENT_SPEED, TASK_DATABASE_AI_STATS_MOVEMENT_STEALTH, TASK_DATABASE_LANDING_TYPES, TASK_DATABASE_MOVEMENT_TYPE } from "../../src/generated/c-task-database";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { ScriptedClock } from "../adapters/scripted-clock";

describe("taskgen.c :: create_supply_task -> create_task", () => {
	for (const c of SUPPLY_TASK_CONSTRUCTION_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			const output = runLifecycle(c.spec);

			expect(supplyTaskExpectationFailure(c, output, firstUnmatchedLine), output.join("\n")).toBe("");
		});
	}
});

describe("F1 compatibility: single player and multiplayer construct the same route", () => {
	it("prepare.y = finish.y = 0.0 leaves nothing for multiplayer packing to change", () => {
		const pair = f1CompatibilityPair();

		expect(f1SemanticRouteFailure(runLifecycle(pair.singlePlayer), runLifecycle(pair.multiplayer))).toBe("");
	});
});

describe("recorded random supply task construction scenarios (outputs of the original C)", () => {
	for (const c of C_REFERENCE_RANDOM_SUPPLY_TASK_CONSTRUCTION) {
		it(c.id, () => {
			expect(runLifecycle(c.spec)).toEqual(c.expected);
		});
	}
});

//
// create_task and the task entity along paths the supply route does not take.
// A minimal restored world: a BLUE force, an airbase that takes helicopters
// with an idle medium lift group, a 4 x 4 map of 8192 m sectors.
//


const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const RED = EntitySide.ENTITY_SIDE_RED_FORCE;
const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;

interface World {
	replication: RecordingEntityReplication;
	events: RecordingCampaignEvents;
	session: Entity;
	force: Entity;
	airbase: Entity;
}

function keysite(world: { session: Entity }, force: Entity | undefined, subType: EntitySubTypeKeysite, side: EntitySide, x: number, landing: number): Entity {
	const raw: KeysiteRaw = {
		sub_type: subType,
		side,
		alive: 1,
		in_use: 1,
		position: { x, y: 0, z: 16000 },
		supplies: { ammo_supply_level: 100, fuel_supply_level: 100 },
		landing_types: landing,
		keysite_usable_state: 0,
	};
	const en = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, raw);
	if (force !== undefined) {
		insertLocalEntityIntoParentsChildListRaw(en, ListType.LIST_TYPE_KEYSITE_FORCE, force, undefined);
	}
	return en;
}

function basedGroup(at: Entity, subType: EntitySubTypeGroup): Entity {
	const raw: GroupRaw = { sub_type: subType, side: BLUE, alive: 1, supplies: { ammo_supply_level: 100, fuel_supply_level: 100 }, sleep: 0, assist_timer: 0, member_count: 0 };
	const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, raw);
	insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_KEYSITE_GROUP, at, undefined);
	return group;
}

function force(session: Entity, side: EntitySide): Entity {
	const raw: ForceRaw = { side, task_generation: clearedTaskGeneration() };
	const en = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, raw);
	insertLocalEntityIntoParentsChildListRaw(en, ListType.LIST_TYPE_FORCE, session, undefined);
	return en;
}

function world(numberOfEntities = 64): World {
	const replication = new RecordingEntityReplication();
	const events = new RecordingCampaignEvents();
	initialiseCampaignCore(
		{ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: replication, clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: events },
		{ numberOfEntities },
	);
	const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
	setSessionEntityRaw(session);
	setUpdateEntity(createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {}));
	const blue = force(session, BLUE);
	const airbase = keysite({ session }, blue, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, BLUE, 22000, 4);
	basedGroup(airbase, EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER);
	setEntityWorldMapSize(4, 4, 8192);
	createLocalSectorEntities();
	setGameStatus(GameStatusType.GAME_STATUS_INITIALISED);
	setGameType(GameType.GAME_TYPE_CAMPAIGN);
	return { replication, events, session, force: blue, airbase };
}

const node = (x: number, z: number) => ({ position: { x, y: 0, z }, dependent: undefined, waypoint_type: 0, formation_type: 0 });
const terminator = { position: terminator_point, dependent: undefined, waypoint_type: 38, formation_type: 0 };

describe("create_task along paths other task types take", () => {
	it("a non-primary task without an objective: NULL positions skipped, a stop timer, no start keysite, the sector of its last node", () => {
		const w = world();
		const engage = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE;
		const task = createTask(engage, BLUE, MovementType.MOVEMENT_TYPE_ALL, undefined, undefined, undefined, 0, 60, 30, undefined, 1, [
			node(1000, 1000),
			{ position: undefined, dependent: undefined, waypoint_type: 0, formation_type: 0 },
			node(9000, 1000),
			terminator,
		]);
		const raw = getLocalEntityData<TaskRaw>(task);

		expect(raw.route_length).toBe(2);
		expect(raw.stop_timer).toBe(30);
		// ENTITY_COMMS_FLOAT_VALUE of the stop timer, after the create
		expect(w.replication.events.map((e) => e.kind)).toEqual(["create", "float_value", "task_pointers"]);
		expect(getLocalEntityParent(task, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBeUndefined();
		expect(getLocalEntityParent(task, ListType.LIST_TYPE_TASK_DEPENDENT)).toBeUndefined();
		// route_nodes [route_length - 1] = (9000, 1000): sector 1, 0
		expect(getLocalEntityParent(task, ListType.LIST_TYPE_SECTOR_TASK)).toBe(getLocalRawSectorEntity(1, 0));
		// from node 0 (sector 0) to node 1 (sector 1), then the last node: 3 enemy sectors
		expect(raw.difficulty).toBe(1);
		expect(w.events.missionsCreated).toEqual([]);
	});

	it("reading past the last route argument is undefined behaviour", () => {
		world();
		expect(() => createTask(SUPPLY, BLUE, MovementType.MOVEMENT_TYPE_AIR, undefined, undefined, undefined, 1, 60, 0, undefined, 1, [node(1000, 1000)])).toThrow(EechUndefinedBehaviourError);
	});

	it("a route of only the terminator leaves assess_task_difficulty reading before its nodes", () => {
		world();
		expect(() => createTask(EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE, BLUE, MovementType.MOVEMENT_TYPE_ALL, undefined, undefined, undefined, 0, 60, 0, undefined, 1, [terminator])).toThrow(
			new EechUndefinedBehaviourError("assess_task_difficulty: route_nodes [route_length - 1] with route_length 0"),
		);
	});

	it("an enemy objective notifies the enemy force (TASK_CREATED, not ported), and is skipped without one", () => {
		const w = world();
		const redSite = keysite(w, undefined, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY, RED, 16000, 0);
		const route = [node(1000, 1000), terminator];

		// no RED force: get_local_force_entity is NULL and nothing is notified
		const task = createTask(SUPPLY, BLUE, MovementType.MOVEMENT_TYPE_AIR, w.airbase, undefined, undefined, 1, 60, 0, redSite, 1, route);
		expect(getLocalEntityParent(task, ListType.LIST_TYPE_TASK_DEPENDENT)).toBe(redSite);

		force(w.session, RED);
		expect(() => createTask(SUPPLY, BLUE, MovementType.MOVEMENT_TYPE_AIR, w.airbase, undefined, undefined, 1, 60, 0, redSite, 1, route)).toThrow(UnportedBehaviourError);
	});
});

describe("taskgen.c :: get_task_start_keysite", () => {
	it("a non-primary task needs no keysite; a given keysite is kept", () => {
		const w = world();
		const start = { value: undefined as Entity | undefined };
		expect(getTaskStartKeysite(EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE, BLUE, { x: 0, y: 0, z: 0 }, start)).toBe(true);
		expect(start.value).toBeUndefined();

		const given = { value: w.airbase as Entity | undefined };
		expect(getTaskStartKeysite(SUPPLY, BLUE, { x: 0, y: 0, z: 0 }, given)).toBe(true);
		expect(given.value).toBe(w.airbase);
	});

	it("a ground task ignores air force capacity, landing types and the groups themselves: any keysite with groups scores 12", () => {
		const w = world();
		const farp = keysite(w, w.force, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP, BLUE, 8000, 0);
		// a dead group: without the capacity check groups are not scored at all
		getLocalEntityData<GroupRaw>(basedGroup(farp, EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE)).alive = 0;
		const start = { value: undefined as Entity | undefined };
		// ADVANCE lands on the ground: the FARP (inserted at the list head) is nearer to (7000, 16000)
		expect(getTaskStartKeysite(EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ADVANCE, BLUE, { x: 7000, y: 0, z: 16000 }, start)).toBe(true);
		expect(start.value).toBe(farp);
	});

	it("unassigned tasks of other types do not count against a keysite", () => {
		const w = world();
		const recon = createTask(EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_RECON, BLUE, MovementType.MOVEMENT_TYPE_AIR, w.airbase, undefined, undefined, 0, 60, 0, undefined, 1, [node(1000, 1000), terminator]);
		expect(getLocalEntityFirstChild(w.airbase, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBe(recon);
		const start = { value: undefined as Entity | undefined };
		expect(getTaskStartKeysite(SUPPLY, BLUE, { x: 20000, y: 0, z: 16000 }, start)).toBe(true);
		expect(start.value).toBe(w.airbase);
	});
});

describe("ts_creat.c and the task's lists", () => {
	const create = (w: World, subType: EntitySubTypeTask, state: TaskStateType): Entity | undefined =>
		createClientServerEntity(EntityType.ENTITY_TYPE_TASK, ENTITY_INDEX_DONT_CARE, [
			{ kind: "int_value", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: subType },
			{ kind: "int_value", type: IntType.INT_TYPE_TASK_STATE, value: state },
			{ kind: "parent", type: ListType.LIST_TYPE_UNASSIGNED_TASK, entity: w.airbase },
		]);

	it("a task_link parent attribute puts the task on the list of its state", () => {
		const w = world();
		const task = create(w, EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_LANDING, TaskStateType.TASK_STATE_UNASSIGNED) as Entity;
		expect(getLocalEntityFirstChild(w.airbase, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBe(task);
		// not a primary task: the campaign screen is not told
		expect(w.events.missionsCreated).toEqual([]);
	});

	it("the assigned and completed lists (assignment, completion) are not ported", () => {
		const w = world();
		// a keysite's assigned and completed task list roots are not ported
		expect(() => create(w, SUPPLY, TaskStateType.TASK_STATE_ASSIGNED)).toThrow(UnportedBehaviourError);
		expect(() => create(w, SUPPLY, TaskStateType.TASK_STATE_COMPLETED)).toThrow(UnportedBehaviourError);
		// nor are the task's LINK_PARENT arms for them
		const task = create(w, EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_LANDING, TaskStateType.TASK_STATE_UNASSIGNED) as Entity;
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_PARENT, task, w.airbase, ListType.LIST_TYPE_ASSIGNED_TASK)).toThrow(
			new UnportedBehaviourError("ts_msgs.c :: response_to_link_parent (LIST_TYPE_ASSIGNED_TASK)"),
		);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_PARENT, task, w.airbase, ListType.LIST_TYPE_COMPLETED_TASK)).toThrow(
			new UnportedBehaviourError("ts_msgs.c :: response_to_link_parent (LIST_TYPE_COMPLETED_TASK)"),
		);
	});

	it("an invalid task state is fatal (get_local_task_list_type)", () => {
		const w = world();
		expect(() => create(w, SUPPLY, 3)).toThrow("TASK: Invalid Task State 3");
	});

	it("a full heap is fatal on the server", () => {
		// session, update, force, airbase, group and 16 sectors fill 21 entries
		const w = world(21);
		expect(() => create(w, SUPPLY, TaskStateType.TASK_STATE_UNASSIGNED)).toThrow("EN_CREATE: CREATE_CLIENT_SERVER_ENTITY : unable to create entity %s. Limit of %d reached");
	});
});

describe("the parent switch, transmission, notification and the sector map", () => {
	it("set_client_server_entity_parent to NULL removes and transmits NULL; the client model is not ported", () => {
		const w = world();
		// a non-primary task is on no task list, so there is nothing to unlink (a task's UNLINK_PARENT is not ported)
		const task = createTask(EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE, BLUE, MovementType.MOVEMENT_TYPE_ALL, undefined, undefined, undefined, 0, 60, 0, undefined, 1, [node(1000, 1000), terminator]);
		setClientServerEntityParent(task, ListType.LIST_TYPE_UNASSIGNED_TASK, undefined);
		expect(getLocalEntityParent(task, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBeUndefined();
		expect(w.replication.events[w.replication.events.length - 1]).toEqual({ kind: "switch_parent", entityIndex: task.index, type: ListType.LIST_TYPE_UNASSIGNED_TASK, parentIndex: -1 });

		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		expect(() => setClientServerEntityParent(task, ListType.LIST_TYPE_UNASSIGNED_TASK, w.airbase)).toThrow(UnportedBehaviourError);
	});

	it("a single player session transmits no value changes", () => {
		const w = world();
		setEntityCommsTransmission(false);
		setClientServerEntityFloatValue(w.airbase, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, 50);
		expect(getLocalEntityData<KeysiteRaw>(w.airbase).supplies.ammo_supply_level).toBe(50);
		expect(w.replication.events).toEqual([]);
	});

	it("the campaign screen hears nothing unless a campaign or skirmish is running", () => {
		const w = world();
		setGameStatus(GameStatusType.GAME_STATUS_INITIALISING);
		expect(notifyCampaignScreenMissionCreated(w.airbase)).toBe(false);
		setGameStatus(GameStatusType.GAME_STATUS_INITIALISED);
		setGameType(GameType.GAME_TYPE_DEMO);
		expect(notifyCampaignScreenMissionCreated(w.airbase)).toBe(false);
		expect(w.events.missionsCreated).toEqual([]);
	});

	it("a group gaining a member (gp_msgs.c LINK_CHILD, LIST_TYPE_MEMBER) is not ported", () => {
		const w = world();
		const group = getLocalEntityFirstChild(w.airbase, ListType.LIST_TYPE_KEYSITE_GROUP) as Entity;
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, group, undefined, ListType.LIST_TYPE_MEMBER)).toThrow(UnportedBehaviourError);
	});

	it("a sector outside the map reads past entity_sector_map", () => {
		world();
		expect(() => getLocalRawSectorEntity(4, 3)).toThrow(EechUndefinedBehaviourError);
		expect(() => getLocalRawSectorEntity(-1, 0)).toThrow(EechUndefinedBehaviourError);
	});

	it("assess_task_difficulty asserts the task", () => {
		world();
		expect(() => assessTaskDifficulty(undefined as unknown as Entity)).toThrow(EechAssertionError);
	});
});

describe("the create_supply_task observer", () => {
	it("sees each call's arguments, and initialisation removes it", () => {
		const w = world();
		const farp = keysite(w, w.force, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP, BLUE, 8000, 0);
		const seen: number[] = [];
		observeCreateSupplyTask((requester, _supplier, _cargo, movement_type) => {
			seen.push(requester.index, movement_type);
		});
		// the cargo is the airbase itself here: create_supply_task reads only its position and sub type
		createSupplyTask(farp, w.airbase, w.airbase, MovementType.MOVEMENT_TYPE_AIR, 4, undefined, undefined);
		expect(seen).toEqual([farp.index, MovementType.MOVEMENT_TYPE_AIR]);

		const again = world();
		const farp2 = keysite(again, again.force, EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP, BLUE, 8000, 0);
		createSupplyTask(farp2, again.airbase, again.airbase, MovementType.MOVEMENT_TYPE_AIR, 4, undefined, undefined);
		expect(seen).toEqual([farp.index, MovementType.MOVEMENT_TYPE_AIR]);
	});
});

describe("suitable.c: the movement stealth check never rejects with EECH's databases", () => {
	it("every group that passes movement, landing and speed for a task has its stealth (the istanbul exclusion's justification)", () => {
		for (let task = 0; task < EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS; task++) {
			for (let group = 0; group < EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS; group++) {
				const movement = TASK_DATABASE_MOVEMENT_TYPE[task] === MovementType.MOVEMENT_TYPE_ALL || TASK_DATABASE_MOVEMENT_TYPE[task] === GROUP_DATABASE_MOVEMENT_TYPE[group];
				const landing = Math.floor(TASK_DATABASE_LANDING_TYPES[task] / Math.pow(2, GROUP_DATABASE_DEFAULT_LANDING_TYPE[group])) % 2 === 1;
				const speed = TASK_DATABASE_AI_STATS_MOVEMENT_SPEED[task] <= GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[group];
				if (movement && landing && speed) {
					expect(TASK_DATABASE_AI_STATS_MOVEMENT_STEALTH[task] <= GROUP_DATABASE_AI_STATS_MOVEMENT_STEALTH[group], `task ${task} group ${group}`).toBe(true);
				}
			}
		}
		// and the table is what the function computes
		expect(calculateGroupToTaskSuitability(EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER, SUPPLY)).toBe(1);
	});
});

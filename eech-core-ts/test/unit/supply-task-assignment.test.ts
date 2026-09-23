//
// Slice 6a behaviour matrix (test/scenarios/supply-task-assignment.cases.ts)
// and the recorded random scenarios under JavaScript semantics; the
// production boundary (assign_primary_task_to_group always fails loudly,
// carrying the selection); the assign.c and group.c paths the adopted callers
// do not take; and the mechanical invariants behind the slice's coverage
// exclusions and findings. The same cases run against the original C in
// test/c-reference/supply-task-assignment.cref.test.ts and under Lua 5.1.
//

import { describe, expect, it } from "vitest";
import { SUPPLY_TASK_ASSIGNMENT_CASES, supplyTaskAssignmentExpectationFailure } from "../scenarios/supply-task-assignment.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";
import { C_REFERENCE_RANDOM_SUPPLY_TASK_ASSIGNMENT } from "../scenarios/generated/c-reference-random-supply-task-assignment.cases";
import { initialiseCampaignCore } from "../../src";
import { assignKeysiteTasks, assignPrimaryTaskToGroup, checkGroupMembersAwake, getSuitableRegisteredGroup, KEYSITE_TASK_ASSIGN_TIMER } from "../../src/ai/taskgen/assign";
import { calculateGroupToTaskSuitability } from "../../src/ai/highlevl/suitable";
import { EechAssertionError, UnportedBehaviourError, UnportedBoundaryError } from "../../src/core/assert";
import { quicksortEntityList } from "../../src/entity/en_misc/en_misc";
import type { AircraftRaw } from "../../src/entity/mobile/aircraft/ac_float";
import { clearedTaskGeneration, type ForceRaw } from "../../src/entity/special/force/force";
import { assessGroupTaskLocalityFactor, type GroupRaw } from "../../src/entity/special/group/group";
import type { KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { clearedTaskRaw, type TaskRaw } from "../../src/entity/special/task/task";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { setCommsModel } from "../../src/entity/system/comms";
import { createLocalEntityRaw } from "../../src/entity/system/en_heap";
import { getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { getLocalEntityFloatValue } from "../../src/entity/system/en_values";
import { getLocalEntityData, setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { AIRCRAFT_DATABASE_CRUISE_VELOCITY } from "../../src/generated/c-aircraft-database";
import {
	CommsModelType,
	EntitySide,
	EntitySubTypeAircraft,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntityType,
	FloatType,
	ListType,
	TaskCategoryType,
	TaskStateType,
} from "../../src/generated/c-enums";
import {
	GROUP_DATABASE_DEFAULT_BLUE_FORCE_AIRCRAFT_SUB_TYPE,
	GROUP_DATABASE_DEFAULT_ENTITY_TYPE,
	GROUP_DATABASE_DEFAULT_RED_FORCE_AIRCRAFT_SUB_TYPE,
	GROUP_DATABASE_REGISTRY_LIST_TYPE,
} from "../../src/generated/c-group-database";
import { TASK_DATABASE_MINIMUM_MEMBER_COUNT } from "../../src/generated/c-task-database";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { ScriptedClock } from "../adapters/scripted-clock";

describe("assign.c :: assign_keysite_tasks, up to assign_primary_task_to_group", () => {
	for (const c of SUPPLY_TASK_ASSIGNMENT_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			const output = runLifecycle(c.spec);

			expect(supplyTaskAssignmentExpectationFailure(c, output, firstUnmatchedLine), output.join("\n")).toBe("");
		});
	}
});

describe("recorded random supply task assignment scenarios (outputs of the original C)", () => {
	for (const c of C_REFERENCE_RANDOM_SUPPLY_TASK_ASSIGNMENT) {
		it(c.id, () => {
			expect(runLifecycle(c.spec)).toEqual(c.expected);
		});
	}
});

//
// A minimal restored world: a BLUE force; an airbase at (1000, 16000) with an
// idle, registered medium lift group led by a UH-60 at the airbase; a
// critical SUPPLY task on the airbase's unassigned list.
//

const BLUE = EntitySide.ENTITY_SIDE_BLUE_FORCE;
const MLT = EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_MEDIUM_LIFT_TRANSPORT_HELICOPTER;
const SUPPLY = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY;

interface World {
	physical: InMemoryMobilePhysicalState;
	force: Entity;
	airbase: Entity;
	group: Entity;
	leader: Entity;
	task: Entity;
}

function world(register = true): World {
	const physical = new InMemoryMobilePhysicalState();
	initialiseCampaignCore(
		{ mobilePhysicalState: physical, entityReplication: new RecordingEntityReplication(), clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: new RecordingCampaignEvents() },
		{ numberOfEntities: 64 },
	);
	const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
	setSessionEntityRaw(session);
	setUpdateEntity(createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {}));
	const forceRaw: ForceRaw = { side: BLUE, task_generation: clearedTaskGeneration() };
	const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, forceRaw);
	insertLocalEntityIntoParentsChildListRaw(force, ListType.LIST_TYPE_FORCE, session, undefined);
	const keysiteRaw: KeysiteRaw = {
		sub_type: EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE,
		side: BLUE,
		alive: 1,
		in_use: 1,
		position: { x: 1000, y: 0, z: 16000 },
		supplies: { ammo_supply_level: 100, fuel_supply_level: 100 },
		landing_types: 4,
		keysite_usable_state: 0,
	};
	const airbase = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, keysiteRaw);
	insertLocalEntityIntoParentsChildListRaw(airbase, ListType.LIST_TYPE_KEYSITE_FORCE, force, undefined);
	const groupRaw: GroupRaw = { sub_type: MLT, side: BLUE, alive: 1, supplies: { ammo_supply_level: 100, fuel_supply_level: 100 }, sleep: 0, assist_timer: 0, member_count: 1, group_list_type: 0 };
	const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, groupRaw);
	insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_KEYSITE_GROUP, airbase, undefined);
	if (register) {
		insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_AIR_REGISTRY, force, undefined);
	}
	const leaderRaw: AircraftRaw = { mob: { sub_type: EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK } };
	const leader = createLocalEntityRaw(EntityType.ENTITY_TYPE_HELICOPTER, leaderRaw);
	insertLocalEntityIntoParentsChildListRaw(leader, ListType.LIST_TYPE_MEMBER, group, undefined);
	physical.setMobilePosition(leader.index, { x: 1000, y: 0, z: 16000 });
	const taskRaw: TaskRaw = clearedTaskRaw();
	taskRaw.sub_type = SUPPLY;
	taskRaw.side = BLUE;
	taskRaw.task_state = TaskStateType.TASK_STATE_UNASSIGNED;
	taskRaw.critical_task = 1;
	taskRaw.task_priority = 4;
	taskRaw.expire_timer = 1200;
	const task = createLocalEntityRaw(EntityType.ENTITY_TYPE_TASK, taskRaw);
	insertLocalEntityIntoParentsChildListRaw(task, ListType.LIST_TYPE_UNASSIGNED_TASK, airbase, undefined);
	return { physical, force, airbase, group, leader, task };
}

describe("the slice 6a decision boundary: assign_primary_task_to_group for a task other than SUPPLY", () => {
	it("fails loudly with UnportedBoundaryError (an UnportedBehaviourError) carrying the selected group and task, before any of the transaction", () => {
		const w = world();
		getLocalEntityData<TaskRaw>(w.task).sub_type = EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ESCORT;

		let error: unknown = undefined;
		try {
			assignPrimaryTaskToGroup(w.group, w.task);
		} catch (e) {
			error = e;
		}

		expect(error).toBeInstanceOf(UnportedBoundaryError);
		expect(error).toBeInstanceOf(UnportedBehaviourError);
		expect((error as UnportedBoundaryError).boundary).toBe("assign.c :: assign_primary_task_to_group");
		expect((error as UnportedBoundaryError).args).toEqual([w.group, w.task]);
		// nothing of the transaction ran
		expect(getLocalEntityFirstChild(w.task, ListType.LIST_TYPE_WAYPOINT)).toBe(undefined);
		expect(getLocalEntityFirstChild(w.group, ListType.LIST_TYPE_GUIDE_STACK)).toBe(undefined);
		expect(getLocalEntityParent(w.task, ListType.LIST_TYPE_UNASSIGNED_TASK)).toBe(w.airbase);
	});
});

describe("assign.c and group.c paths the adopted callers do not take", () => {
	it("get_suitable_registered_group without an idle count array (msg_in.c) lets an unregistered group qualify (INT_MAX)", () => {
		const w = world(false);

		expect(getSuitableRegisteredGroup(w.task, [0, 0, 0, 0, 0, 0, 0])).toBe(undefined);
		expect(getSuitableRegisteredGroup(w.task, undefined)).toBe(w.group);
	});

	it("assess_group_task_locality_factor returns the distance when asked, and 0.0 when it rejects", () => {
		const w = world();
		const distance = { distance: -1 };

		w.physical.setMobilePosition(w.leader.index, { x: 5000, y: 0, z: 16000 });
		expect(assessGroupTaskLocalityFactor(w.group, w.task, distance)).toBe(true);
		expect(distance.distance).toBe(4000);
		expect(assessGroupTaskLocalityFactor(w.group, w.task)).toBe(true);

		w.physical.setMobilePosition(w.leader.index, { x: 60000, y: 0, z: 16000 });
		expect(assessGroupTaskLocalityFactor(w.group, w.task, distance)).toBe(false);
		expect(distance.distance).toBe(0);
		expect(assessGroupTaskLocalityFactor(w.group, w.task)).toBe(false);
	});

	it("the ASSERTs: a keysite, a task, a server, a list to sort", () => {
		const w = world();

		expect(() => assignKeysiteTasks(undefined, TaskCategoryType.TASK_CATEGORY_SUPPORT)).toThrow(new EechAssertionError("keysite"));
		expect(() => getSuitableRegisteredGroup(undefined, undefined)).toThrow(new EechAssertionError("task"));
		expect(() => quicksortEntityList(undefined, 0, [])).toThrow(new EechAssertionError("en_list"));
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		expect(() => getSuitableRegisteredGroup(w.task, undefined)).toThrow(new EechAssertionError("get_comms_model () == COMMS_MODEL_SERVER"));
		expect(() => assignKeysiteTasks(w.airbase, TaskCategoryType.TASK_CATEGORY_SUPPORT)).toThrow(new EechAssertionError("get_comms_model () == COMMS_MODEL_SERVER"));
		setCommsModel(CommsModelType.COMMS_MODEL_SERVER);
	});

	it("KEYSITE_TASK_ASSIGN_TIMER is 3.0 * ONE_MINUTE", () => {
		expect(KEYSITE_TASK_ASSIGN_TIMER).toBe(180);
	});
});

//
// Mechanical invariants over the generated (C-derived) databases. Each backs a
// coverage exclusion or a finding recorded in docs/slices/supply-task-assignment.md
// and docs/port-manifest.md.
//
describe("slice 6a invariants", () => {
	it("every nonzero group-to-task suitability is exactly 1.0, so the least suitable qualifying group is the first", () => {
		for (let g = 0; g < EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS; g++) {
			for (let t = 0; t < EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS; t++) {
				const s = calculateGroupToTaskSuitability(g, t);
				expect(s === 0 || s === 1, `${EntitySubTypeGroup[g]} ${EntitySubTypeTask[t]} ${s}`).toBe(true);
			}
		}
	});

	it("every task's minimum member count is at least 1, so a group without members never reaches its locality (group.c exclusion)", () => {
		for (let t = 0; t < EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS; t++) {
			expect(TASK_DATABASE_MINIMUM_MEMBER_COUNT[t], EntitySubTypeTask[t]).toBeGreaterThanOrEqual(1);
		}
	});

	it("every aircraft has a positive cruise velocity, so the ETA division never divides by zero", () => {
		expect(AIRCRAFT_DATABASE_CRUISE_VELOCITY.length).toBe(EntitySubTypeAircraft.NUM_ENTITY_SUB_TYPE_AIRCRAFT);
		for (let a = 0; a < EntitySubTypeAircraft.NUM_ENTITY_SUB_TYPE_AIRCRAFT; a++) {
			expect(AIRCRAFT_DATABASE_CRUISE_VELOCITY[a], EntitySubTypeAircraft[a]).toBeGreaterThan(0);
		}
	});

	it("every supply-compatible group is an air-registered aircraft group whose default aircraft have a positive cruise velocity", () => {
		let supplyGroups = 0;
		for (let g = 0; g < EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS; g++) {
			if (calculateGroupToTaskSuitability(g, SUPPLY) === 0) {
				continue;
			}
			supplyGroups++;
			expect(GROUP_DATABASE_REGISTRY_LIST_TYPE[g], EntitySubTypeGroup[g]).toBe(ListType.LIST_TYPE_AIR_REGISTRY);
			expect([EntityType.ENTITY_TYPE_HELICOPTER, EntityType.ENTITY_TYPE_FIXED_WING], EntitySubTypeGroup[g]).toContain(GROUP_DATABASE_DEFAULT_ENTITY_TYPE[g]);
			for (const aircraft of [GROUP_DATABASE_DEFAULT_BLUE_FORCE_AIRCRAFT_SUB_TYPE[g], GROUP_DATABASE_DEFAULT_RED_FORCE_AIRCRAFT_SUB_TYPE[g]]) {
				expect(AIRCRAFT_DATABASE_CRUISE_VELOCITY[aircraft], `${EntitySubTypeGroup[g]} ${EntitySubTypeAircraft[aircraft]}`).toBeGreaterThan(0);
			}
		}
		expect(supplyGroups).toBe(4);
	});

	it("only aircraft groups join the air registry, whose idle count every assigned group type needs", () => {
		for (let g = 0; g < EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS; g++) {
			if (GROUP_DATABASE_REGISTRY_LIST_TYPE[g] === ListType.LIST_TYPE_AIR_REGISTRY) {
				expect([EntityType.ENTITY_TYPE_HELICOPTER, EntityType.ENTITY_TYPE_FIXED_WING], EntitySubTypeGroup[g]).toContain(GROUP_DATABASE_DEFAULT_ENTITY_TYPE[g]);
			}
		}
	});

	it("an aircraft member's sleep is en_float.c's default 0.0, so check_group_members_awake never rejects one (assign.c exclusion)", () => {
		const w = world();
		const fixedRaw: AircraftRaw = { mob: { sub_type: EntitySubTypeAircraft.ENTITY_SUB_TYPE_AIRCRAFT_FA18_HORNET } };
		const fixed = createLocalEntityRaw(EntityType.ENTITY_TYPE_FIXED_WING, fixedRaw);
		insertLocalEntityIntoParentsChildListRaw(fixed, ListType.LIST_TYPE_MEMBER, w.group, w.leader);

		expect(getLocalEntityFloatValue(w.leader, FloatType.FLOAT_TYPE_SLEEP)).toBe(0);
		expect(getLocalEntityFloatValue(fixed, FloatType.FLOAT_TYPE_SLEEP)).toBe(0);
		expect(checkGroupMembersAwake(w.group)).toBe(true);
	});
});

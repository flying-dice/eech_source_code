//
// Task generation.
//
// C provenance: ai/taskgen/taskgen.c :: create_task, create_supply_task,
//               get_task_start_keysite, validate_task_generation, terminator_point
//
// Slice 5b (issue #14): a supply task, from create_supply_task through
// create_task's return. Assignment, waypoints, expiry, packing and destruction
// of the task are later slices. docs/slices/supply-task-construction.md maps
// every step.
//

import { ASSERT, assertNotNullDereference, EechUndefinedBehaviourError } from "../../core/assert";
import { f32Add, f32Sub, toFloat32RTZ } from "../../core/float32";
import { KILOMETRE } from "../../core/maths/miscmath";
import type { Vec3d } from "../../core/maths/vec3d";
import { normaliseAny3dVector } from "../../core/maths/vector";
import { MAX_ROUTE_NODES, NUM_TASK_ID_BITS, SECONDS_IN_A_MINUTE } from "../../generated/c-constants";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeLanding,
	EntitySubTypeTask,
	EntitySubTypeWaypoint,
	EntityType,
	FloatType,
	FormationType,
	IntType,
	ListType,
	MovementType,
	PtrType,
	Vec3dType,
} from "../../generated/c-enums";
import { TASK_DATABASE_LANDING_TYPES, TASK_DATABASE_PRIMARY_TASK } from "../../generated/c-task-database";
import { cBit, cBitAnd } from "../../core/cint";
import type { ForceRaw } from "../../entity/special/force/force";
import { getLocalForceEntity } from "../../entity/special/force/force";
import { getKeysiteSupplyPosition } from "../../entity/special/keysite/keysite";
import { getLocalSectorEntity } from "../../entity/special/sector/sector";
import { assessTaskDifficulty, findMostSuitableKeysiteForTask, type TaskRaw } from "../../entity/special/task/task";
import { getCommsModel } from "../../entity/system/comms";
import { transmitTaskPointers } from "../../entity/system/en_comms";
import { createClientServerEntity } from "../../entity/system/en_creat";
import { ENTITY_INDEX_DONT_CARE } from "../../entity/system/en_heap";
import { insertLocalEntityIntoParentsChildList, setClientServerEntityParent } from "../../entity/system/en_list";
import { notifyLocalEntity } from "../../entity/system/en_msgs";
import {
	getLocalEntityIntValue,
	getLocalEntityVec3dPtr,
	setClientServerEntityFloatValue,
	setLocalEntityFloatValue,
	setLocalEntityIntValue,
	setLocalEntityPtrValue,
} from "../../entity/system/en_values";
import { boundPositionToAdjustedMapArea } from "../../entity/system/en_world";
import { getLocalEntityData, type Entity } from "../../entity/system/entity";

// C provenance: taskgen.c :: vec3d terminator_point = {-1.0, -1.0, -1.0}
export const terminator_point: Vec3d = { x: -1.0, y: -1.0, z: -1.0 };

// C provenance: constant.h :: #define ONE_MINUTE (SECONDS_IN_A_MINUTE)
const ONE_MINUTE = SECONDS_IN_A_MINUTE;

// One route node of create_task's variable arguments: (vec3d *position, entity
// *dependent, entity_sub_types waypoint_type, entity_sub_types formation_type).
export interface RouteNodeArgument {
	position: Vec3d | undefined;
	dependent: Entity | undefined;
	waypoint_type: number;
	formation_type: number;
}

//
// C provenance: taskgen.c :: create_task
//
// The variable arguments are the route: nodes up to and including the one
// whose position equals terminator_point (compared by value, as the C loop
// does). A NULL position is skipped (cap tasks). Reading past the last
// argument is undefined behaviour, which the port refuses.
//
// validate_task_generation returns TRUE unconditionally (everything after
// its return is #if 0), so its check is always passed and not ported as a
// branch. force_local_entity_create_stack_attributes only changes on
// COMMS_DATA_FLOW_RX, which the server's TX path never is (en_creat.ts). The
// debug logging is compiled out and not ported.
//
export function createTask(
	sub_type: EntitySubTypeTask,
	side: EntitySide,
	movement_type: MovementType,
	start_keysite: Entity | undefined,
	end_keysite: Entity | undefined,
	_originator: Entity | undefined,
	critical_task: number,
	expire_timer: number,
	stop_timer: number,
	task_objective: Entity | undefined,
	task_priority: number,
	route: RouteNodeArgument[],
): Entity {
	// float parameters
	expire_timer = toFloat32RTZ(expire_timer);
	stop_timer = toFloat32RTZ(stop_timer);
	task_priority = toFloat32RTZ(task_priority);

	// debug_assert
	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	//////////////////////////////////////////////////////////////////
	// Update counter
	//////////////////////////////////////////////////////////////////

	const force_en = getLocalForceEntity(side);

	// C reads the force's raw data without a check
	assertNotNullDereference(force_en, "get_local_entity_data (force_en)");

	const force_raw = getLocalEntityData<ForceRaw>(force_en);

	force_raw.task_generation[sub_type].created++;

	//////////////////////////////////////////////////////////////////
	// Read positions off stack
	//////////////////////////////////////////////////////////////////

	let route_length = 0;

	const route_node_list: Vec3d[] = [];
	const route_dependent_list: (Entity | undefined)[] = [];
	const route_waypoint_type_list: number[] = [];
	const route_formation_type_list: number[] = [];

	let arg = 0;

	while (true) {
		if (arg >= route.length) {
			throw new EechUndefinedBehaviourError("create_task: va_arg past the last route argument (no terminator_point)");
		}

		const position = route[arg].position;
		const dependent = route[arg].dependent;
		const waypoint_type = route[arg].waypoint_type;
		const formation_type = route[arg].formation_type;

		arg++;

		// a NULL position is skipped: "added so that cap tasks do not require all 4 waypoints to be valid"
		if (position) {
			// route_node_list [route_length].x = ceil (position->x): the double ceiling of a float, stored to a float (exact)
			route_node_list[route_length] = { x: Math.ceil(position.x), y: Math.ceil(position.y), z: Math.ceil(position.z) };
			route_dependent_list[route_length] = dependent;
			route_waypoint_type_list[route_length] = waypoint_type;
			route_formation_type_list[route_length] = formation_type;

			route_length++;

			ASSERT(route_length < MAX_ROUTE_NODES, "route_length < MAX_ROUTE_NODES");

			if (position.x === terminator_point.x && position.y === terminator_point.y && position.z === terminator_point.z) {
				break;
			}
		}
	}

	//////////////////////////////////////////////////////////////////
	// Allocate route data (malloc_heap_mem + memcpy of route_length entries)
	//////////////////////////////////////////////////////////////////

	const route_nodes: Vec3d[] = [];
	const route_dependents: (Entity | undefined)[] = [];
	const route_waypoint_types: number[] = [];
	const route_formation_types: number[] = [];

	for (let i = 0; i < route_length; i++) {
		route_nodes[i] = route_node_list[i];
		route_dependents[i] = route_dependent_list[i];
		route_waypoint_types[i] = route_waypoint_type_list[i];
		route_formation_types[i] = route_formation_type_list[i];
	}

	// decreased by 1 so as not to include the terminator.
	route_length--;

	//////////////////////////////////////////////////////////////////
	// determine task id number
	//////////////////////////////////////////////////////////////////

	let id = force_raw.task_generation[sub_type].created;

	const id_max = cBit(NUM_TASK_ID_BITS) - 1;

	while (id > id_max) {
		id -= id_max;
	}

	//////////////////////////////////////////////////////////////////
	// Create task
	//////////////////////////////////////////////////////////////////

	const new_task = createClientServerEntity(EntityType.ENTITY_TYPE_TASK, ENTITY_INDEX_DONT_CARE, [
		{ kind: "parent", type: ListType.LIST_TYPE_TASK_DEPENDENT, entity: task_objective },
		{ kind: "int_value", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: sub_type },
		{ kind: "int_value", type: IntType.INT_TYPE_TASK_ID, value: id },
		{ kind: "float_value", type: FloatType.FLOAT_TYPE_EXPIRE_TIMER, value: expire_timer },
		{ kind: "float_value", type: FloatType.FLOAT_TYPE_TASK_PRIORITY, value: task_priority },
		{ kind: "int_value", type: IntType.INT_TYPE_CRITICAL_TASK, value: critical_task },
		{ kind: "int_value", type: IntType.INT_TYPE_MOVEMENT_TYPE, value: movement_type },
		{ kind: "int_value", type: IntType.INT_TYPE_ROUTE_LENGTH, value: route_length },
		{ kind: "int_value", type: IntType.INT_TYPE_SIDE, value: side },
	]);

	ASSERT(new_task !== undefined, "new_task");

	//////////////////////////////////////////////////////////////////
	// Set stop time (if needed)
	//////////////////////////////////////////////////////////////////

	if (stop_timer > 0.0) {
		setClientServerEntityFloatValue(new_task, FloatType.FLOAT_TYPE_STOP_TIMER, stop_timer);
	}

	//////////////////////////////////////////////////////////////////
	// set local pointers (and comms message)
	//////////////////////////////////////////////////////////////////

	setLocalEntityPtrValue(new_task, PtrType.PTR_TYPE_ROUTE_DEPENDENTS, route_dependents);
	setLocalEntityPtrValue(new_task, PtrType.PTR_TYPE_ROUTE_NODE, route_nodes);
	setLocalEntityPtrValue(new_task, PtrType.PTR_TYPE_ROUTE_WAYPOINT_TYPES, route_waypoint_types);
	setLocalEntityPtrValue(new_task, PtrType.PTR_TYPE_ROUTE_FORMATION_TYPES, route_formation_types);
	setLocalEntityPtrValue(new_task, PtrType.PTR_TYPE_RETURN_KEYSITE, end_keysite);

	transmitTaskPointers(new_task, {
		route_length: getLocalEntityData<TaskRaw>(new_task).route_length,
		route_nodes,
		route_formation_types,
		route_waypoint_types,
		route_dependents,
		return_keysite: end_keysite,
	});

	//////////////////////////////////////////////////////////////////
	// Attach task to keysite (only if primary task)
	//////////////////////////////////////////////////////////////////

	if (TASK_DATABASE_PRIMARY_TASK[sub_type] !== 0) {
		ASSERT(start_keysite !== undefined, "start_keysite");

		setClientServerEntityParent(new_task, ListType.LIST_TYPE_UNASSIGNED_TASK, start_keysite);
	}

	setLocalEntityIntValue(new_task, IntType.INT_TYPE_TASK_DIFFICULTY, assessTaskDifficulty(new_task).difficulty);

	//////////////////////////////////////////////////////////////////
	// notify enemy force
	//////////////////////////////////////////////////////////////////

	if (task_objective) {
		if (getLocalEntityIntValue(task_objective, IntType.INT_TYPE_SIDE) !== side) {
			const enemy_force = getLocalForceEntity(getLocalEntityIntValue(task_objective, IntType.INT_TYPE_SIDE));

			if (enemy_force) {
				notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_TASK_CREATED, enemy_force, new_task, task_objective);
			}
		}
	}

	//////////////////////////////////////////////////////////////////
	// add task to sector task list
	//////////////////////////////////////////////////////////////////

	let position: Vec3d | undefined = undefined;

	if (task_objective) {
		position = getLocalEntityVec3dPtr(task_objective, Vec3dType.VEC3D_TYPE_POSITION);
	}

	if (!position) {
		// &route_nodes [route_length - 1]: route_length is at least 1 here, since
		// assess_task_difficulty has already read the same node (and refused 0)
		position = route_nodes[route_length - 1];
	}

	const sec = getLocalSectorEntity(position);

	insertLocalEntityIntoParentsChildList(new_task, ListType.LIST_TYPE_SECTOR_TASK, sec, undefined);

	return new_task;
}

//
// C provenance: taskgen.c :: get_task_start_keysite
//
// `start_keysite` (entity **) is an in/out object. A primary task without a
// given start keysite takes the most suitable one: ground tasks ignore the
// keysites' air force capacity.
//
export function getTaskStartKeysite(sub_type: EntitySubTypeTask, side: EntitySide, start_pos: Vec3d, start_keysite: { value: Entity | undefined }): boolean {
	if (TASK_DATABASE_PRIMARY_TASK[sub_type] !== 0) {
		if (!start_keysite.value) {
			// Keysite not specified - so find best suited
			if (cBitAnd(TASK_DATABASE_LANDING_TYPES[sub_type], cBit(EntitySubTypeLanding.ENTITY_SUB_TYPE_LANDING_GROUND)) !== 0) {
				start_keysite.value = findMostSuitableKeysiteForTask(sub_type, side, start_pos, false);
			} else {
				start_keysite.value = findMostSuitableKeysiteForTask(sub_type, side, start_pos, true);
			}

			if (!start_keysite.value) {
				return false;
			}
		}
	}

	return true;
}

// The arguments of taskgen.c :: create_supply_task, in C order.
export type CreateSupplyTaskObserver = (
	requester: Entity,
	supplier: Entity,
	cargo: Entity,
	movement_type: MovementType,
	priority: number,
	start_keysite: Entity | undefined,
	end_keysite: Entity | undefined,
) => void;

let observer: CreateSupplyTaskObserver | undefined = undefined;

//
// Test seam for this one boundary (Slice 5a's): an observer sees each call's
// arguments before create_supply_task runs. It cannot change what the
// function does or returns. initialiseCampaignCore removes it.
//
export function observeCreateSupplyTask(fn: CreateSupplyTaskObserver): void {
	observer = fn;
}

export function resetCreateSupplyTaskObserver(): void {
	observer = undefined;
}

//
// C provenance: taskgen.c :: create_supply_task
//
// A route from the cargo (pick up) to 4 km short of the requester (prepare
// for drop off), the requester (drop off) and 2 km past it (finish), the two
// derived points kept off the map's 5 km perimeter.
//
// F1, a compatibility decision (docs/slices/supply-task-construction.md,
// docs/port-manifest.md): the C never sets prepare.y or finish.y. They are
// automatic variables, so create_task's ceil (position->y) reads an
// indeterminate value, which EECH's route then keeps, transmits and saves.
// That value is not defined by the source and differs between builds and
// calls. The port sets both to 0.0, and the C reference compiles this path
// with zero initialisation of automatic variables to match. This is chosen
// behaviour for undefined source behaviour, not a translation of a value
// EECH defined.
//
// Floats follow the RTZ contract (src/core/float32.ts): stop - start is a
// float subtraction; stop - direction * 4.0 * KILOMETRE multiplies a float by
// double 4.0 and float 1000 in double (exact: 24 + 12 bits) and stores the
// double difference to a float (one truncation). 20 * ONE_MINUTE is int 1200.
// The debug logging is compiled out and not ported.
//
export function createSupplyTask(
	requester: Entity,
	supplier: Entity,
	cargo: Entity,
	movement_type: MovementType,
	priority: number,
	start_keysite: Entity | undefined,
	end_keysite: Entity | undefined,
): Entity | undefined {
	// float parameter
	priority = toFloat32RTZ(priority);

	if (observer !== undefined) {
		observer(requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite);
	}

	// Neither is NULL here: response_to_force_low_on_supplies has already
	// ranged the requester's position and found the cargo on the supplier.
	const stop = getKeysiteSupplyPosition(requester) as Vec3d;

	const start = getLocalEntityVec3dPtr(cargo, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

	const side = getLocalEntityIntValue(requester, IntType.INT_TYPE_SIDE);

	const start_ks = { value: start_keysite };

	const end_ks = end_keysite;

	if (!getTaskStartKeysite(EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY, side, start, start_ks) || start_ks.value === requester) {
		return undefined;
	}

	const direction: Vec3d = {
		x: f32Sub(stop.x, start.x),
		y: f32Sub(stop.y, start.y),
		z: f32Sub(stop.z, start.z),
	};

	normaliseAny3dVector(direction);

	// F1: prepare.y and finish.y are never set in C (see above); 0.0 by decision
	const prepare: Vec3d = {
		x: f32Add(stop.x, -(direction.x * 4.0 * KILOMETRE)),
		y: 0.0,
		z: f32Add(stop.z, -(direction.z * 4.0 * KILOMETRE)),
	};

	boundPositionToAdjustedMapArea(prepare);

	const finish: Vec3d = {
		x: f32Add(stop.x, direction.x * 2.0 * KILOMETRE),
		y: 0.0,
		z: f32Add(stop.z, direction.z * 2.0 * KILOMETRE),
	};

	boundPositionToAdjustedMapArea(finish);

	const expire_time = 20 * ONE_MINUTE;

	const new_task = createTask(
		EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY,
		side,
		movement_type,
		start_ks.value,
		end_ks,
		undefined,
		1,
		expire_time,
		0.0,
		requester,
		priority,
		[
			{ position: start, dependent: supplier, waypoint_type: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, formation_type: FormationType.FORMATION_ROW_LEFT },
			{ position: prepare, dependent: undefined, waypoint_type: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF, formation_type: FormationType.FORMATION_ROW_LEFT },
			{ position: stop, dependent: requester, waypoint_type: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, formation_type: FormationType.FORMATION_ROW_LEFT },
			{ position: finish, dependent: undefined, waypoint_type: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF, formation_type: FormationType.FORMATION_ROW_LEFT },
			{ position: terminator_point, dependent: undefined, waypoint_type: EntitySubTypeWaypoint.NUM_ENTITY_SUB_TYPE_WAYPOINTS, formation_type: FormationType.FORMATION_NONE },
		],
	);

	// (float) of the cargo's int sub type
	setLocalEntityFloatValue(new_task, FloatType.FLOAT_TYPE_TASK_USER_DATA, getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE));

	return new_task;
}

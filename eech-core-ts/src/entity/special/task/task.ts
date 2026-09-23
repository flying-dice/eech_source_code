//
// Task entity.
//
// C provenance: entity/special/task/task.c, ts_int.c, ts_float.c, ts_ptr.c,
//               ts_list.c, ts_msgs.c (response_to_link_parent)
//
// Slice 5a (issue #12) read restored tasks: the duplicate-task decision of
// fc_msgs.c :: response_to_force_low_on_supplies. Slice 5b (issue #14)
// constructs them: the task's values and route pointers, its lists, the
// keysite a primary task starts from (find_most_suitable_keysite_for_task),
// its difficulty (assess_task_difficulty) and its LINK_PARENT response.
// Creation itself is ts_creat.ts; taskgen.ts builds tasks.
//
// Not ported: assignment, waypoints, expiry, completion, destruction and
// packing of tasks, and the int, float and pointer values nothing ported
// reads or writes.
//

import { ASSERT, EechFatalError, EechUndefinedBehaviourError, UnportedBehaviourError } from "../../../core/assert";
import { cBit, cBitAnd, cIntDivide, storeUnsignedBitfield } from "../../../core/cint";
import { f32Add, f32Div, f32Mul, f32Sub, toFloat32 } from "../../../core/float32";
import { getApprox2dRange } from "../../../core/maths/range";
import { KILOMETRE, max, min } from "../../../core/maths/miscmath";
import type { Vec3d } from "../../../core/maths/vec3d";
import {
	NUM_CRITICAL_TASK_BITS,
	NUM_MOVEMENT_TYPE_BITS,
	NUM_ROUTE_LENGTH_BITS,
	NUM_SIDE_BITS,
	NUM_TASK_DIFFICULTY_BITS,
	NUM_TASK_ID_BITS,
} from "../../../generated/c-constants";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeLanding,
	EntitySubTypeTask,
	EntityType,
	FloatType,
	GroupModeType,
	IntType,
	KeysiteUsableState,
	ListType,
	PtrType,
	TaskStateType,
	Vec3dType,
} from "../../../generated/c-enums";
import { KEYSITE_DATABASE_AIR_FORCE_CAPACITY } from "../../../generated/c-keysite-database";
import { TASK_DATABASE_KEYSITE_AIR_FORCE_CAPACITY, TASK_DATABASE_LANDING_TYPES, TASK_DATABASE_PRIMARY_TASK } from "../../../generated/c-task-database";
import { getGroupToTaskSuitability } from "../../../ai/highlevl/suitable";
import { notifyCampaignScreenMissionCreated } from "../../../ui_menu/campaign/ca_msgs";
import { getCommsModel } from "../../system/comms";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent, overloadEntityListLink, overloadEntityListRoot } from "../../system/en_list";
import { messageResponses, type MessageResponseFn } from "../../system/en_msgs";
import {
	fnGetLocalEntityFloatValue,
	fnGetLocalEntityIntValue,
	fnSetClientServerEntityFloatValue,
	fnSetLocalEntityFloatValue,
	fnSetLocalEntityIntValue,
	fnSetLocalEntityPtrValue,
	fnSetLocalEntityRawFloatValue,
	fnSetLocalEntityRawIntValue,
	getLocalEntityIntValue,
	getLocalEntityVec3dPtr,
	serverFloatValueSetter,
	setLocalEntityIntValue,
	type SetFloatValueFn,
	type SetIntValueFn,
} from "../../system/en_values";
import { getXSector, getZSector } from "../../system/en_world";
import { getLocalEntityData, getLocalEntityType, type Entity } from "../../system/entity";
import { getLocalForceEntity } from "../force/force";
import { getLocalRawSectorEntity, getLocalSectorEntityEnemySurfaceToAirDefenceLevel } from "../sector/sector";

// C provenance: task.h :: struct TASK (ported fields only; memset to 0 on creation)
export interface TaskRaw {
	sub_type: EntitySubTypeTask;
	task_state: TaskStateType;
	task_user_data: number;
	task_priority: number;
	stop_timer: number;
	expire_timer: number;
	// vec3d *route_nodes, entity **route_dependents, formation_types *route_formation_types,
	// entity_sub_types *route_waypoint_types: route_length + 1 entries (the terminator's included)
	route_nodes: Vec3d[] | undefined;
	route_dependents: (Entity | undefined)[] | undefined;
	route_formation_types: number[] | undefined;
	route_waypoint_types: number[] | undefined;
	return_keysite: Entity | undefined;
	// unsigned int bit-fields
	task_id: number;
	movement_type: number;
	difficulty: number;
	critical_task: number;
	route_length: number;
	side: EntitySide;
}

// A task's raw data as memset (raw, 0, sizeof (task)) leaves it.
export function clearedTaskRaw(): TaskRaw {
	return {
		sub_type: 0,
		task_state: TaskStateType.TASK_STATE_UNASSIGNED,
		task_user_data: 0.0,
		task_priority: 0.0,
		stop_timer: 0.0,
		expire_timer: 0.0,
		route_nodes: undefined,
		route_dependents: undefined,
		route_formation_types: undefined,
		route_waypoint_types: undefined,
		return_keysite: undefined,
		task_id: 0,
		movement_type: 0,
		difficulty: 0,
		critical_task: 0,
		route_length: 0,
		side: 0,
	};
}

// C provenance: task.c :: entity_is_object_of_task
export function entityIsObjectOfTask(en: Entity, task_type: EntitySubTypeTask, side: EntitySide): number {
	ASSERT(en !== undefined, "en");

	let count = 0;

	let this_task = getLocalEntityFirstChild(en, ListType.LIST_TYPE_TASK_DEPENDENT);

	while (this_task) {
		if (getLocalEntityType(this_task) === EntityType.ENTITY_TYPE_TASK) {
			if (getLocalEntityIntValue(this_task, IntType.INT_TYPE_ENTITY_SUB_TYPE) === task_type) {
				if (getLocalEntityIntValue(this_task, IntType.INT_TYPE_SIDE) === side) {
					if (getLocalEntityIntValue(this_task, IntType.INT_TYPE_TASK_STATE) !== TaskStateType.TASK_STATE_COMPLETED) {
						count++;
					}
				}
			}
		}

		this_task = getLocalEntityChildSucc(this_task, ListType.LIST_TYPE_TASK_DEPENDENT);
	}

	return count;
}

// C provenance: task.c :: get_local_task_list_type
export function getLocalTaskListType(task_en: Entity): ListType {
	const raw = getLocalEntityData<TaskRaw>(task_en);

	if (raw.task_state === TaskStateType.TASK_STATE_UNASSIGNED) {
		return ListType.LIST_TYPE_UNASSIGNED_TASK;
	}

	if (raw.task_state === TaskStateType.TASK_STATE_ASSIGNED) {
		return ListType.LIST_TYPE_ASSIGNED_TASK;
	}

	if (raw.task_state === TaskStateType.TASK_STATE_COMPLETED) {
		return ListType.LIST_TYPE_COMPLETED_TASK;
	}

	throw new EechFatalError("TASK: Invalid Task State %d", `TASK: Invalid Task State ${raw.task_state}`);
}

//
// C provenance: task.c :: assess_task_sector_difficulty
//
// get_local_raw_sector_entity has no range check: a sector outside the map
// reads past entity_sector_map, which the port refuses.
//
export function assessTaskSectorDifficulty(task_en: Entity, x: number, z: number, counts: { air_threats: number; enemy_sectors: number }): void {
	const sector_en = getLocalRawSectorEntity(x, z);

	const side = getLocalEntityIntValue(task_en, IntType.INT_TYPE_SIDE);

	ASSERT(sector_en !== undefined, "sector_en");

	if (getLocalSectorEntityEnemySurfaceToAirDefenceLevel(sector_en, side) > 0.0) {
		counts.air_threats++;
	}

	if (getLocalEntityIntValue(sector_en, IntType.INT_TYPE_SECTOR_SIDE) !== side) {
		counts.enemy_sectors++;
	}
}

//
// C provenance: task.c :: assess_task_difficulty
//
// Walks the route's sectors with Bresenham's algorithm, from the task's start
// keysite (its task_link parent) or else from the first route node, counting
// sectors with enemy air defences and sectors the task's side does not hold.
// The C out-parameters (route_air_threat, route_enemy_sectors) are returned
// alongside; the task type's difficulty_rating is read into an unused local
// and is not ported.
//
export function assessTaskDifficulty(task_en: Entity): { difficulty: number; air_threats: number; enemy_sectors: number } {
	ASSERT(task_en !== undefined, "task_en");

	const raw = getLocalEntityData<TaskRaw>(task_en);

	const route_nodes = raw.route_nodes as Vec3d[];

	const counts = { air_threats: 0, enemy_sectors: 0 };

	// raw->task_link.parent (the link of the unassigned, assigned and completed task lists)
	const start_keysite = getLocalEntityParent(task_en, ListType.LIST_TYPE_UNASSIGNED_TASK);

	let node: number;

	let last_pos: Vec3d;

	if (start_keysite) {
		//
		// start at the assigned keysite
		//

		ASSERT(getLocalEntityType(start_keysite) === EntityType.ENTITY_TYPE_KEYSITE, "get_local_entity_type (start_keysite) == ENTITY_TYPE_KEYSITE");

		node = 0;

		last_pos = getLocalEntityVec3dPtr(start_keysite, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;
	} else {
		//
		// start at the first point on the route
		//

		node = 1;

		last_pos = route_nodes[0];
	}

	// unsigned int node < unsigned route_length bit-field
	for (; node < raw.route_length; node++) {
		const this_pos = route_nodes[node];

		//
		// convert coordinates into sector coordinates
		//

		const x1 = getXSector(last_pos.x);
		const z1 = getZSector(last_pos.z);

		const x2 = getXSector(this_pos.x);
		const z2 = getZSector(this_pos.z);

		//
		// traverse sectors using bresenham's algorithm
		//

		const dx = x2 - x1;
		const dz = z2 - z1;

		const ax = Math.abs(dx) * 2;
		const az = Math.abs(dz) * 2;

		const sx = dx > 0 ? 1 : -1;
		const sz = dz > 0 ? 1 : -1;

		let x = x1;
		let z = z1;

		if (ax > az) {
			let d = az - cIntDivide(ax, 2);

			while (true) {
				assessTaskSectorDifficulty(task_en, x, z, counts);

				if (x === x2) {
					break;
				}

				if (d >= 0) {
					z += sz;
					d -= ax;
				}

				x += sx;
				d += az;
			}
		} else {
			let d = ax - cIntDivide(az, 2);

			while (true) {
				assessTaskSectorDifficulty(task_en, x, z, counts);

				if (z === z2) {
					break;
				}

				if (d >= 0) {
					x += sx;
					d -= az;
				}

				z += sz;
				d += ax;
			}
		}

		last_pos = this_pos;
	}

	//
	// do last point on the route: &raw->route_nodes [raw->route_length - 1], an
	// unsigned index (route_length 0 reads before the array)
	//

	if (raw.route_length === 0) {
		throw new EechUndefinedBehaviourError("assess_task_difficulty: route_nodes [route_length - 1] with route_length 0");
	}

	last_pos = route_nodes[raw.route_length - 1];

	assessTaskSectorDifficulty(task_en, getXSector(last_pos.x), getZSector(last_pos.z), counts);

	const air_threats = counts.air_threats;

	const enemy_sectors = counts.enemy_sectors;

	//
	// +1 difficulty point for every sector with enemy air defences ( maximum of 5 ),
	// +1 difficulty point for every 2 enemy sectors passing through ( maximum of 5 ):
	// min ((n >> 1), 5) of a non-negative count
	//

	const difficulty = min(cIntDivide(air_threats, 2), 5) + min(cIntDivide(enemy_sectors, 2), 5);

	return { difficulty, air_threats, enemy_sectors };
}

// C provenance: task.c :: #define KEYSITE_TASK_* (float literals, rounded to nearest when compiled)
const KEYSITE_TASK_IDLE_GROUP_COUNT_BIAS = toFloat32(5.0);
const KEYSITE_TASK_BUSY_GROUP_COUNT_BIAS = toFloat32(0.5);
const KEYSITE_TASK_MAX_GROUP_COUNT_BIAS = toFloat32(12.0);
// (100.0f * KILOMETRE): float * float
const KEYSITE_TASK_MAX_RANGE_BIAS = f32Mul(toFloat32(100.0), KILOMETRE);
const KEYSITE_TASK_COUNT_BIAS = toFloat32(0.2);

//
// C provenance: task.c :: find_most_suitable_keysite_for_task
//
// Scores each in-use keysite of the side that has groups and suits the task's
// landing types (and, with check_capacity, its air force capacity) by the
// groups there able to do the task, its range from pos, the unassigned tasks
// of the type already waiting there, and whether it is usable. Floats follow
// the RTZ contract at their declared types (src/core/float32.ts):
//   score += bias, min (score, 12.0f), range / max_range - 1, x * x * x * x,
//   score *= task_count, 1.0 - task_count * 0.2f are float operations;
//   score *= 2.0 * (...), task_count += 1.0 and score *= 0.5 are double
//   operations stored to a float, each exact before the one truncation.
// DEBUG_MODULE logging is compiled out and not ported.
//
export function findMostSuitableKeysiteForTask(task_type: EntitySubTypeTask, side: EntitySide, pos: Vec3d, check_capacity: boolean): Entity | undefined {
	ASSERT(pos !== undefined, "pos");

	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	//
	// Find Entity types suitable for task
	//

	const task_landing_types = TASK_DATABASE_LANDING_TYPES[task_type];

	ASSERT(task_landing_types > 0, "task_landing_types > 0");

	//
	// Find closest keysite with lowest task count and most suitable groups
	//

	const max_range = KEYSITE_TASK_MAX_RANGE_BIAS;

	const force = getLocalForceEntity(side);

	ASSERT(force !== undefined, "force");

	let best_keysite: Entity | undefined = undefined;

	let best_score = 0.0;

	let keysite = getLocalEntityFirstChild(force, ListType.LIST_TYPE_KEYSITE_FORCE);

	while (keysite) {
		if (
			getLocalEntityIntValue(keysite, IntType.INT_TYPE_IN_USE) !== 0 &&
			getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_KEYSITE_GROUP) &&
			(cBitAnd(task_landing_types, cBit(EntitySubTypeLanding.ENTITY_SUB_TYPE_LANDING_GROUND)) !== 0 ||
				cBitAnd(getLocalEntityIntValue(keysite, IntType.INT_TYPE_LANDING_TYPES), task_landing_types) !== 0)
		) {
			const keysite_type = getLocalEntityIntValue(keysite, IntType.INT_TYPE_ENTITY_SUB_TYPE);

			if (!check_capacity || KEYSITE_DATABASE_AIR_FORCE_CAPACITY[keysite_type] >= TASK_DATABASE_KEYSITE_AIR_FORCE_CAPACITY[task_type]) {
				let score = 0.0;

				//
				// bias keysites with more groups suitable for this type
				//

				if (check_capacity) {
					let group = getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_KEYSITE_GROUP);

					while (group) {
						const group_type = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

						if (getGroupToTaskSuitability(group_type, task_type) > 0.0) {
							if (getLocalEntityIntValue(group, IntType.INT_TYPE_ALIVE) !== 0) {
								if (getLocalEntityIntValue(group, IntType.INT_TYPE_GROUP_MODE) === GroupModeType.GROUP_MODE_IDLE) {
									score = f32Add(score, KEYSITE_TASK_IDLE_GROUP_COUNT_BIAS);
								} else {
									score = f32Add(score, KEYSITE_TASK_BUSY_GROUP_COUNT_BIAS);
								}
							}
						}

						group = getLocalEntityChildSucc(group, ListType.LIST_TYPE_KEYSITE_GROUP);
					}
				} else {
					score = KEYSITE_TASK_MAX_GROUP_COUNT_BIAS;
				}

				if (score !== 0.0) {
					score = min(score, KEYSITE_TASK_MAX_GROUP_COUNT_BIAS);

					//
					// find distance between keysite and task origin
					//

					const keysite_pos = getLocalEntityVec3dPtr(keysite, Vec3dType.VEC3D_TYPE_POSITION);

					const range = getApprox2dRange(keysite_pos, pos);

					if (range < max_range) {
						const x = f32Sub(f32Div(range, max_range), 1);

						// score *= 2.0 * (x * x * x * x)
						score = f32Mul(score, 2.0 * f32Mul(f32Mul(f32Mul(x, x), x), x));

						//
						// bias keysites with less tasks of this type
						//

						let task_count = 0.0;

						let ts = getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_UNASSIGNED_TASK);

						while (ts) {
							if (getLocalEntityIntValue(ts, IntType.INT_TYPE_ENTITY_SUB_TYPE) === task_type) {
								task_count = f32Add(task_count, 1.0);
							}

							ts = getLocalEntityChildSucc(ts, ListType.LIST_TYPE_UNASSIGNED_TASK);
						}

						task_count = f32Add(1.0, -f32Mul(task_count, KEYSITE_TASK_COUNT_BIAS));

						task_count = max(task_count, KEYSITE_TASK_COUNT_BIAS);

						score = f32Mul(score, task_count);

						//
						// Less suitable if out-of-action / repairing
						//

						if (getLocalEntityIntValue(keysite, IntType.INT_TYPE_KEYSITE_USABLE_STATE) !== KeysiteUsableState.KEYSITE_STATE_USABLE) {
							score = f32Mul(score, 0.5);
						}

						//
						// Check if best keysite
						//

						if (score > best_score) {
							best_keysite = keysite;

							best_score = score;
						}
					}
				}
			}
		}

		keysite = getLocalEntityChildSucc(keysite, ListType.LIST_TYPE_KEYSITE_FORCE);
	}

	return best_keysite;
}

//
// C provenance: ts_msgs.c :: response_to_link_parent
//
// Joining the unassigned list makes the task unassigned and, for a primary
// task, tells the campaign screen a mission was created. The assigned and
// completed arms (assignment and completion) are not ported. Other lists
// (update, task dependent, sector task) have no arm.
//
const responseToLinkParent: MessageResponseFn = (_message, receiver, _sender, args) => {
	const list_type = args[0] as ListType;

	const sub_type = getLocalEntityIntValue(receiver, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	if (list_type === ListType.LIST_TYPE_UNASSIGNED_TASK) {
		setLocalEntityIntValue(receiver, IntType.INT_TYPE_TASK_STATE, TaskStateType.TASK_STATE_UNASSIGNED);

		if (TASK_DATABASE_PRIMARY_TASK[sub_type] !== 0) {
			notifyCampaignScreenMissionCreated(receiver);
		}
	} else if (list_type === ListType.LIST_TYPE_ASSIGNED_TASK || list_type === ListType.LIST_TYPE_COMPLETED_TASK) {
		throw new UnportedBehaviourError(`ts_msgs.c :: response_to_link_parent (${ListType[list_type]})`);
	}

	return 1;
};

// ts_int.c :: set_local_int_value, for one field: `unsigned int field : bits`,
// or a whole int when bits is 0
function intFieldSetter(store: (raw: TaskRaw, value: number) => void, bits: number): SetIntValueFn {
	return (en, _type, value) => {
		store(getLocalEntityData<TaskRaw>(en), bits === 0 ? value : storeUnsignedBitfield(value, bits));
	};
}

// C provenance: ts_float.c :: set_local_float_value, for one field
function floatFieldSetter(store: (raw: TaskRaw, value: number) => void): SetFloatValueFn {
	return (en, _type, value) => {
		store(getLocalEntityData<TaskRaw>(en), value);
	};
}

// C provenance: ts_funcs.c :: overload_task_functions (ported subset)
export function overloadTaskFunctions(): void {
	const TASK = EntityType.ENTITY_TYPE_TASK;

	// C provenance: ts_list.c :: LIST_TYPE_*_ROOT, LIST_TYPE_*_LINK;
	//   en_list/get_prnt.h: the unassigned, assigned and completed task lists share task_link
	overloadEntityListRoot(TASK, "guide_root", [ListType.LIST_TYPE_GUIDE]);
	overloadEntityListRoot(TASK, "player_task_root", [ListType.LIST_TYPE_PLAYER_TASK]);
	overloadEntityListRoot(TASK, "task_dependent_root", [ListType.LIST_TYPE_TASK_DEPENDENT]);
	overloadEntityListRoot(TASK, "waypoint_root", [ListType.LIST_TYPE_WAYPOINT]);
	overloadEntityListLink(TASK, "pilot_lock_link", [ListType.LIST_TYPE_PILOT_LOCK]);
	overloadEntityListLink(TASK, "sector_task_link", [ListType.LIST_TYPE_SECTOR_TASK]);
	overloadEntityListLink(TASK, "task_dependent_link", [ListType.LIST_TYPE_TASK_DEPENDENT]);
	overloadEntityListLink(TASK, "task_link", [ListType.LIST_TYPE_UNASSIGNED_TASK, ListType.LIST_TYPE_ASSIGNED_TASK, ListType.LIST_TYPE_COMPLETED_TASK]);
	overloadEntityListLink(TASK, "update_link", [ListType.LIST_TYPE_UPDATE]);

	// C provenance: ts_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(TASK, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<TaskRaw>(en).sub_type);
	fnGetLocalEntityIntValue.overload(TASK, IntType.INT_TYPE_SIDE, (en) => getLocalEntityData<TaskRaw>(en).side);
	fnGetLocalEntityIntValue.overload(TASK, IntType.INT_TYPE_TASK_STATE, (en) => getLocalEntityData<TaskRaw>(en).task_state);

	// C provenance: ts_int.c :: set_local_int_value, installed as both the raw and the local setter
	const intSetters: [IntType, SetIntValueFn][] = [
		[IntType.INT_TYPE_ENTITY_SUB_TYPE, intFieldSetter((raw, value) => (raw.sub_type = value), 0)],
		[IntType.INT_TYPE_TASK_STATE, intFieldSetter((raw, value) => (raw.task_state = value), 0)],
		[IntType.INT_TYPE_TASK_ID, intFieldSetter((raw, value) => (raw.task_id = value), NUM_TASK_ID_BITS)],
		[IntType.INT_TYPE_CRITICAL_TASK, intFieldSetter((raw, value) => (raw.critical_task = value), NUM_CRITICAL_TASK_BITS)],
		[IntType.INT_TYPE_MOVEMENT_TYPE, intFieldSetter((raw, value) => (raw.movement_type = value), NUM_MOVEMENT_TYPE_BITS)],
		[IntType.INT_TYPE_ROUTE_LENGTH, intFieldSetter((raw, value) => (raw.route_length = value), NUM_ROUTE_LENGTH_BITS)],
		[IntType.INT_TYPE_SIDE, intFieldSetter((raw, value) => (raw.side = value), NUM_SIDE_BITS)],
		[IntType.INT_TYPE_TASK_DIFFICULTY, intFieldSetter((raw, value) => (raw.difficulty = value), NUM_TASK_DIFFICULTY_BITS)],
	];

	for (const [type, setter] of intSetters) {
		fnSetLocalEntityRawIntValue.overload(TASK, type, setter);
		fnSetLocalEntityIntValue.overload(TASK, type, setter);
	}

	// C provenance: ts_float.c :: get_local_float_value
	fnGetLocalEntityFloatValue.overload(TASK, FloatType.FLOAT_TYPE_TASK_USER_DATA, (en) => getLocalEntityData<TaskRaw>(en).task_user_data);

	// C provenance: ts_float.c :: set_local_float_value (raw and local), set_server_float_value
	const floatSetters: [FloatType, SetFloatValueFn][] = [
		[FloatType.FLOAT_TYPE_EXPIRE_TIMER, floatFieldSetter((raw, value) => (raw.expire_timer = value))],
		[FloatType.FLOAT_TYPE_STOP_TIMER, floatFieldSetter((raw, value) => (raw.stop_timer = value))],
		[FloatType.FLOAT_TYPE_TASK_PRIORITY, floatFieldSetter((raw, value) => (raw.task_priority = value))],
		[FloatType.FLOAT_TYPE_TASK_USER_DATA, floatFieldSetter((raw, value) => (raw.task_user_data = value))],
	];

	for (const [type, setter] of floatSetters) {
		fnSetLocalEntityRawFloatValue.overload(TASK, type, setter);
		fnSetLocalEntityFloatValue.overload(TASK, type, setter);
		fnSetClientServerEntityFloatValue[CommsModelType.COMMS_MODEL_SERVER].overload(TASK, type, serverFloatValueSetter(setter));
	}

	// C provenance: ts_ptr.c :: set_local_ptr_value
	fnSetLocalEntityPtrValue.overload(TASK, PtrType.PTR_TYPE_RETURN_KEYSITE, (en, _type, ptr) => {
		getLocalEntityData<TaskRaw>(en).return_keysite = ptr as Entity | undefined;
	});
	fnSetLocalEntityPtrValue.overload(TASK, PtrType.PTR_TYPE_ROUTE_DEPENDENTS, (en, _type, ptr) => {
		getLocalEntityData<TaskRaw>(en).route_dependents = ptr as (Entity | undefined)[];
	});
	fnSetLocalEntityPtrValue.overload(TASK, PtrType.PTR_TYPE_ROUTE_FORMATION_TYPES, (en, _type, ptr) => {
		getLocalEntityData<TaskRaw>(en).route_formation_types = ptr as number[];
	});
	fnSetLocalEntityPtrValue.overload(TASK, PtrType.PTR_TYPE_ROUTE_NODE, (en, _type, ptr) => {
		getLocalEntityData<TaskRaw>(en).route_nodes = ptr as Vec3d[];
	});
	fnSetLocalEntityPtrValue.overload(TASK, PtrType.PTR_TYPE_ROUTE_WAYPOINT_TYPES, (en, _type, ptr) => {
		getLocalEntityData<TaskRaw>(en).route_waypoint_types = ptr as number[];
	});

	// C provenance: ts_msgs.c :: overload_task_message_responses (LINK_PARENT)
	messageResponses.overload(TASK, EntityMessage.ENTITY_MESSAGE_LINK_PARENT, responseToLinkParent);
}

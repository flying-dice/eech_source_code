//
// Keysite task assignment: the decision.
//
// C provenance: ai/taskgen/assign.c :: assign_keysite_tasks,
//               suitable_group_task_specific_checks (static),
//               get_suitable_registered_group, check_group_members_awake,
//               assign_primary_task_to_group, assign_task_to_group,
//               push_task_onto_group_task_stack, assign_task_to_group_members (boundary only)
//
// Slice 6a (issue #16) ports which task a keysite assigns and to which group:
// the category's unassigned tasks in en_misc.c's quicksort order, the
// keysite's assign and reserve counts, the pilot lock, and the least suitable
// qualifying group. Slice 6b (issue #18) ports the assignment transaction for
// SUPPLY tasks (route, guide, the task becoming ASSIGNED) up to
// assign_task_to_group_members, which throws UnportedBoundaryError with the
// group, guide and valid members; for other task types
// assign_primary_task_to_group stays 6a's boundary (the selected group and task).
//
// ai_log is the release macro (highlevl.h: do { } while (0)), which evaluates
// none of its arguments; debug_log calls are guarded by DEBUG_MODULE (0).
// Neither is ported.
//

import { ASSERT, UnportedBehaviourError, UnportedBoundaryError } from "../../core/assert";
import { FLT_MAX, f32Mul } from "../../core/float32";
import { max } from "../../core/maths/miscmath";
import { SECONDS_IN_A_MINUTE, TASK_ASSIGN_ALL_MEMBERS } from "../../generated/c-constants";
import {
	CommsModelType,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntityType,
	FloatType,
	GroupModeType,
	IntType,
	ListType,
	PtrType,
	type TaskCategoryType,
	Vec3dType,
} from "../../generated/c-enums";
import { GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED, GROUP_DATABASE_DEFAULT_LANDING_TYPE, GROUP_DATABASE_MINIMUM_IDLE_COUNT } from "../../generated/c-group-database";
import { TASK_DATABASE_PRIMARY_TASK } from "../../generated/c-task-database";
import { KEYSITE_DATABASE_ASSIGN_TASK_COUNT, KEYSITE_DATABASE_RESERVE_TASK_COUNT } from "../../generated/c-keysite-database";
import { quicksortEntityList } from "../../entity/en_misc/en_misc";
import { getLocalForceEntity } from "../../entity/special/force/force";
import { assessGroupTaskLocalityFactor, getLocalGroupMemberCount, type GroupRaw } from "../../entity/special/group/group";
import { attachGroupToGuideEntity, createClientServerGuideEntity } from "../../entity/special/guide/guide";
import { getLocalEntityLandingEntity } from "../../entity/special/landing/landing";
import { getLocalGroupPrimaryTask, getLocalTaskListType, type TaskRaw } from "../../entity/special/task/task";
import { transmitSwitchList } from "../../entity/system/en_comms";
import { getCommsModel } from "../../entity/system/comms";
import { deleteLocalEntityFromParentsChildList, getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildList } from "../../entity/system/en_list";
import { getLocalEntityFloatValue, getLocalEntityIntValue, getLocalEntityPtrValue, getLocalEntityVec3dPtr, setLocalEntityPtrValue } from "../../entity/system/en_values";
import { getLocalEntityData, getLocalEntityType, type Entity } from "../../entity/system/entity";
import { getGroupToTaskSuitability } from "../highlevl/suitable";
import { createGenericWaypointRoute } from "./croute";

// C provenance: keysite.h :: #define KEYSITE_TASK_ASSIGN_TIMER (3.0 * ONE_MINUTE); constant.h ::
//               #define ONE_MINUTE (SECONDS_IN_A_MINUTE). A double.
export const KEYSITE_TASK_ASSIGN_TIMER = 3.0 * SECONDS_IN_A_MINUTE;

// C provenance: limits.h :: INT_MAX (get_suitable_registered_group's idle count without a count array)
const INT_MAX = 2147483647;

// C provenance: assign.c :: void assign_keysite_tasks (entity *keysite, task_category_types category)
export function assignKeysiteTasks(keysite: Entity | undefined, category: TaskCategoryType): void {
	ASSERT(keysite !== undefined, "keysite");

	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	let task = getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_UNASSIGNED_TASK);

	if (!task) {
		return;
	}

	const keysite_type: EntitySubTypeKeysite = getLocalEntityIntValue(keysite, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const force = getLocalEntityParent(keysite, ListType.LIST_TYPE_KEYSITE_FORCE);

	ASSERT(force !== undefined, "force");

	//
	// Count tasks at keysite
	//

	let task_count = 0;

	task = getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_UNASSIGNED_TASK);

	while (task) {
		if (getLocalEntityIntValue(task, IntType.INT_TYPE_TASK_CATEGORY) === category) {
			task_count++;
		}

		task = getLocalEntityChildSucc(task, ListType.LIST_TYPE_UNASSIGNED_TASK);
	}

	if (task_count === 0) {
		return;
	}

	//
	// Count up number of idle groups across the map (air registry only)
	//

	// C: static int idle_group_count [NUM_ENTITY_SUB_TYPE_GROUPS], memset to 0 on every call
	const idle_group_count: number[] = [];

	for (let i = 0; i < EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS; i++) {
		idle_group_count.push(0);
	}

	let group = getLocalEntityFirstChild(force, ListType.LIST_TYPE_AIR_REGISTRY);

	while (group) {
		if (getLocalEntityIntValue(group, IntType.INT_TYPE_GROUP_MODE) === GroupModeType.GROUP_MODE_IDLE) {
			const group_type = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

			idle_group_count[group_type]++;
		}

		group = getLocalEntityChildSucc(group, ListType.LIST_TYPE_AIR_REGISTRY);
	}

	//
	// Sort tasks
	//

	ASSERT(task_count > 0, "task_count > 0");

	const task_list: Entity[] = [];

	const sort_order: number[] = [];

	task_count = 0;

	task = getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_UNASSIGNED_TASK);

	while (task) {
		if (getLocalEntityIntValue(task, IntType.INT_TYPE_TASK_CATEGORY) === category) {
			task_list[task_count] = task;

			sort_order[task_count] = getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_PRIORITY);

			if (getLocalEntityIntValue(task, IntType.INT_TYPE_CRITICAL_TASK) !== 0) {
				// sort_order [task_count] *= 2.0: a double product stored as float (exact)
				sort_order[task_count] = f32Mul(sort_order[task_count], 2.0);
			}

			task_count++;
		}

		task = getLocalEntityChildSucc(task, ListType.LIST_TYPE_UNASSIGNED_TASK);
	}

	quicksortEntityList(task_list, task_count, sort_order);

	//
	// Assign tasks
	//

	// max (keysite_database [keysite_type].assign_task_count, 1u)
	let assign_count = max(KEYSITE_DATABASE_ASSIGN_TASK_COUNT[keysite_type], 1);

	let non_critical_task_count = KEYSITE_DATABASE_RESERVE_TASK_COUNT[keysite_type];

	for (let loop = 0; loop < task_count; loop++) {
		// assign_count only falls when assign_primary_task_to_group succeeds, which is past the slice 6b boundary
		/* istanbul ignore if */
		if (assign_count === 0) {
			break;
		}

		task = task_list[loop];

		//
		// Check for player lock
		//

		if (getLocalEntityParent(task, ListType.LIST_TYPE_PILOT_LOCK)) {
			continue;
		}

		//
		// Reserve non-critical tasks for player
		//

		if (getLocalEntityIntValue(task, IntType.INT_TYPE_CRITICAL_TASK) === 0) {
			if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_EXPIRE_TIMER) > KEYSITE_TASK_ASSIGN_TIMER) {
				if (non_critical_task_count > 0) {
					non_critical_task_count--;

					continue;
				}
			}
		}

		group = getSuitableRegisteredGroup(task, idle_group_count);

		if (group) {
			// assign_primary_task_to_group returns TRUE only past the slice 6b boundary
			// (assign_task_to_group_members throws), and FALSE only for states 6a never
			// selects (a group without members, an assault ship, a landing or takeoff task)
			/* istanbul ignore next */
			if (assignPrimaryTaskToGroup(group, task) !== 0) {
				//
				// Only Assign n tasks per keysite
				//

				assign_count--;
			}
		}
	}
}

//
// C provenance: assign.c :: static int suitable_group_task_specific_checks (entity *task, entity *group)
//
// Only the ESCORT and TROOP_INSERTION arms check anything; every other task
// type (SUPPLY included) passes.
//
function suitableGroupTaskSpecificChecks(task: Entity, group: Entity): boolean {
	const task_type: EntitySubTypeTask = getLocalEntityIntValue(task, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const group_type: EntitySubTypeGroup = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	switch (task_type) {
		case EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ESCORT: {
			//
			// Special check for ESCORT task :- Jets escort Jets, Helis escort anything but jets
			//

			const objective = getLocalEntityParent(task, ListType.LIST_TYPE_TASK_DEPENDENT);

			ASSERT(objective !== undefined, "objective");

			ASSERT(getLocalEntityType(objective) === EntityType.ENTITY_TYPE_GROUP, "get_local_entity_type (objective) == ENTITY_TYPE_GROUP");

			const objective_group_type: EntitySubTypeGroup = getLocalEntityIntValue(objective, IntType.INT_TYPE_ENTITY_SUB_TYPE);

			if (GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[objective_group_type] >= 5) {
				// Fast objective group
				if (GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[group_type] < 5) {
					// Slow escort group
					return false;
				}
			} else {
				// Slow objective group
				if (GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[group_type] >= 5) {
					// Fast escort group
					return false;
				}

				// magitek: avoid picking slower units than objective group
				if (GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[objective_group_type] > GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[group_type]) {
					return false;
				}
			}

			break;
		}

		case EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_TROOP_INSERTION: {
			//
			// Special check for T.I. task :- Assaults against airbases require large assault groups
			//

			const objective = getLocalEntityParent(task, ListType.LIST_TYPE_TASK_DEPENDENT);

			ASSERT(objective !== undefined, "objective");

			ASSERT(getLocalEntityType(objective) === EntityType.ENTITY_TYPE_KEYSITE, "get_local_entity_type (objective) == ENTITY_TYPE_KEYSITE");

			const objective_type = getLocalEntityIntValue(objective, IntType.INT_TYPE_ENTITY_SUB_TYPE);

			if (
				objective_type === EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE &&
				getLocalEntityIntValue(group, IntType.INT_TYPE_MEMBER_COUNT) < 2 &&
				getLocalEntityIntValue(objective, IntType.INT_TYPE_SIDE) !== getLocalEntityIntValue(group, IntType.INT_TYPE_SIDE)
			)
				return false;

			break;
		}
	}

	return true;
}

//
// C provenance: assign.c :: entity *get_suitable_registered_group (entity *task, int *idle_group_count)
//
// The least suitable qualifying group wins (result < best_result), the first
// of equals in the keysite's LIST_TYPE_KEYSITE_GROUP order; the group's alive
// flag is not checked. idle_group_count is NULL from msg_in.c (not ported),
// which lets every group past the minimum idle count.
//
export function getSuitableRegisteredGroup(task: Entity | undefined, idle_group_count: readonly number[] | undefined): Entity | undefined {
	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	ASSERT(task !== undefined, "task");

	const task_type: EntitySubTypeTask = getLocalEntityIntValue(task, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const keysite = getLocalEntityParent(task, ListType.LIST_TYPE_UNASSIGNED_TASK);

	ASSERT(keysite !== undefined, "keysite");

	let best_group: Entity | undefined = undefined;

	let best_result = FLT_MAX;

	let current_group = getLocalEntityFirstChild(keysite, ListType.LIST_TYPE_KEYSITE_GROUP);

	const distance = { distance: 0.0 };

	while (current_group) {
		//
		// Check for player lock
		//

		if (!getLocalEntityParent(current_group, ListType.LIST_TYPE_PILOT_LOCK)) {
			const group_raw = getLocalEntityData<GroupRaw>(current_group);

			//////////////////////////////////////////////////////////////////
			// stop carriers being assigned
			if (group_raw.sub_type === EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP) {
				current_group = getLocalEntityChildSucc(current_group, ListType.LIST_TYPE_KEYSITE_GROUP);

				continue;
			}
			//////////////////////////////////////////////////////////////////

			if (getLocalEntityIntValue(current_group, IntType.INT_TYPE_GROUP_MODE) === GroupModeType.GROUP_MODE_IDLE) {
				if (group_raw.sleep === 0.0) {
					if (group_raw.side === getLocalEntityIntValue(task, IntType.INT_TYPE_SIDE)) {
						// unsigned int idle_count
						let idle_count: number;

						if (idle_group_count) {
							idle_count = idle_group_count[group_raw.sub_type];
						} else {
							idle_count = INT_MAX;
						}

						if (idle_count > GROUP_DATABASE_MINIMUM_IDLE_COUNT[group_raw.sub_type]) {
							if (getLocalEntityIntValue(current_group, IntType.INT_TYPE_MEMBER_COUNT) >= getLocalEntityIntValue(task, IntType.INT_TYPE_MINIMUM_MEMBER_COUNT)) {
								// its FALSE arm is unreachable: aircraft members sleep 0.0 (check_group_members_awake below)
								/* istanbul ignore else */
								if (checkGroupMembersAwake(current_group)) {
									const result = getGroupToTaskSuitability(group_raw.sub_type, task_type);

									if (result > 0.0) {
										if (suitableGroupTaskSpecificChecks(task, current_group)) {
											//
											// locality
											// assess if group could get to end position quick enough, using first members cruise speed.
											//

											if (assessGroupTaskLocalityFactor(current_group, task, distance)) {
												//
												// best group
												//

												if (result < best_result) {
													best_result = result;

													best_group = current_group;
												}
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}

		current_group = getLocalEntityChildSucc(current_group, ListType.LIST_TYPE_KEYSITE_GROUP);
	}

	return best_group;
}

//
// C provenance: assign.c :: int assign_primary_task_to_group (entity *group_en, entity *task_en)
//
// Slice 6b (issue #18) adopts the assignment transaction for SUPPLY tasks, the
// supply chain: the route, the guide, the task becoming ASSIGNED, up to
// assign_task_to_group_members (the new boundary). For every other task type
// the transaction is not ported, so the call stays Slice 6a's decision
// boundary: it fails loudly with the selected group and task before anything
// of the transaction runs.
//
// Everything after assign_task_to_group returns TRUE (default formation, the
// return keysite re-parent, the force's TASK_ASSIGNED, escort assessment, the
// start time and the expiry reset) follows assign_task_to_group_members and is
// not reached.
//
export function assignPrimaryTaskToGroup(group_en: Entity, task_en: Entity): 0 {
	if (getLocalEntityIntValue(task_en, IntType.INT_TYPE_ENTITY_SUB_TYPE) !== EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY) {
		throw new UnportedBoundaryError("assign.c :: assign_primary_task_to_group", [group_en, task_en]);
	}

	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	const task_type: EntitySubTypeTask = getLocalEntityIntValue(task_en, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	ASSERT(TASK_DATABASE_PRIMARY_TASK[task_type] !== 0, "task_database [task_type].primary_task");

	// group_type: read, not used before the boundary
	getLocalEntityIntValue(group_en, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	ASSERT(getLocalGroupPrimaryTask(group_en) === undefined, "!get_local_group_primary_task (group_en)");

	// if (assign_task_to_group (...)) { ... return TRUE; }: assign_task_to_group
	// returns TRUE only after assign_task_to_group_members, the boundary
	assignTaskToGroup(group_en, task_en, TASK_ASSIGN_ALL_MEMBERS);

	return 0;
}

//
// C provenance: assign.c :: entity *push_task_onto_group_task_stack (entity *group, entity *task, unsigned int valid_members)
//
// The task's guide, on the group's guide stack; then (after the guide is
// created and attached, as the source requires) the task moves from its
// keysite's unassigned list to the head of its assigned list, which makes it
// ASSIGNED (ts_msgs.c), and the switch is transmitted. The #ifdef DEBUG
// duplicate check is not in release builds.
//
export function pushTaskOntoGroupTaskStack(group: Entity | undefined, task: Entity, valid_members: number): Entity {
	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	ASSERT(group !== undefined, "group");

	//
	// create guide entity for task
	//

	const guide = createClientServerGuideEntity(task, undefined, valid_members) as Entity;

	attachGroupToGuideEntity(group, guide);

	//
	// remove task and group from lists (must be done AFTER guide is created and attached)
	//

	const list_type = getLocalTaskListType(task);

	if (list_type === ListType.LIST_TYPE_UNASSIGNED_TASK) {
		const task_parent = getLocalEntityParent(task, list_type);

		if (task_parent) {
			deleteLocalEntityFromParentsChildList(task, list_type);

			//
			// add task to assigned task list, if not already on it.
			//

			insertLocalEntityIntoParentsChildList(task, ListType.LIST_TYPE_ASSIGNED_TASK, task_parent, undefined);

			transmitSwitchList(task, ListType.LIST_TYPE_UNASSIGNED_TASK, task_parent, ListType.LIST_TYPE_ASSIGNED_TASK);
		}
	}

	return guide;
}

//
// C provenance: assign.c :: int assign_task_to_group (entity *group, entity *task_en, unsigned int valid_members)
//
// FALSE for a group without members, an assault ship given anything but
// ENGAGE, and the landing and takeoff tasks. A task that assesses landing
// returns to its return keysite, or (none given) to the group's keysite, which
// is then stored as the return keysite. The route is created, then the task is
// pushed onto the group's guide stack, then its members are assigned: the
// slice 6b boundary.
//
export function assignTaskToGroup(group: Entity | undefined, task_en: Entity | undefined, valid_members: number): false {
	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	ASSERT(task_en !== undefined, "task_en");

	ASSERT(group !== undefined, "group");

	ASSERT(
		!(getLocalGroupPrimaryTask(group) !== undefined && getLocalEntityIntValue(task_en, IntType.INT_TYPE_PRIMARY_TASK) !== 0),
		"!(get_local_group_primary_task (group) && (get_local_entity_int_value (task_en, INT_TYPE_PRIMARY_TASK)))",
	);

	const task_raw = getLocalEntityData<TaskRaw>(task_en);

	const group_type: EntitySubTypeGroup = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const member = getLocalEntityFirstChild(group, ListType.LIST_TYPE_MEMBER);

	// don't if no members or if the group is a CARRIER
	if (!member) {
		return false;
	}

	if (group_type === EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ASSAULT_SHIP) {
		if (task_raw.sub_type !== EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_ENGAGE) {
			return false;
		}
	}

	//
	// check for invalid tasks (debug_fatal only #ifdef DEBUG)
	//

	switch (task_raw.sub_type) {
		case EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_LANDING:
		case EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
		case EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_TAKEOFF:
		case EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING: {
			return false;
		}
	}

	//
	// Create route
	//

	let start_keysite: Entity | undefined;

	if (getLocalEntityIntValue(group, IntType.INT_TYPE_GROUP_LIST_TYPE) === ListType.LIST_TYPE_KEYSITE_GROUP) {
		start_keysite = getLocalEntityParent(group, ListType.LIST_TYPE_KEYSITE_GROUP);
	} else {
		start_keysite = undefined;
	}

	// pos and force: read, not used
	getLocalEntityVec3dPtr(task_en, Vec3dType.VEC3D_TYPE_STOP_POSITION);

	getLocalForceEntity(getLocalEntityIntValue(task_en, IntType.INT_TYPE_SIDE));

	const sub_type = GROUP_DATABASE_DEFAULT_LANDING_TYPE[group_type];

	let end_keysite: Entity | undefined = undefined;

	// SUPPLY assesses landing (the only task type the port assigns)
	/* istanbul ignore else */
	if (getLocalEntityIntValue(task_en, IntType.INT_TYPE_ASSESS_LANDING) !== 0) {
		end_keysite = getLocalEntityPtrValue(task_en, PtrType.PTR_TYPE_RETURN_KEYSITE);

		if (end_keysite) {
			//
			// check end keysite has suitble free landing sites
			//

			if (start_keysite !== end_keysite) {
				//
				// if end keysite == start keysite then keysite MUST have enough sites because the aircraft are already there
				//

				// sites_required
				getLocalGroupMemberCount(group);

				// get_keysite_landing_sites_available (end_keysite, sub_type) < sites_required -> return FALSE
				// (END keysite was specified - but no free landing sites for this group): landing sites are
				// not ported. No unassigned supply task has a return keysite (create_supply_task leaves it NULL).
				throw new UnportedBehaviourError(`landing.c :: get_keysite_landing_sites_available (landing type ${sub_type})`);
			}
		} else {
			if (!start_keysite) {
				// get_closest_keysite (NUM_ENTITY_SUB_TYPE_KEYSITES, side, leader position, 1.0 * KILOMETRE, NULL, TRUE, NULL)
				// and the new keysite's landing sites: not ported
				throw new UnportedBehaviourError("assign.c :: assign_task_to_group: no start keysite (get_closest_keysite)");
			}

			//
			// No END keysite specified so return to start keysite
			//

			end_keysite = start_keysite;

			setLocalEntityPtrValue(task_en, PtrType.PTR_TYPE_RETURN_KEYSITE, end_keysite);
		}

		ASSERT(end_keysite !== undefined, "end_keysite");

		// landing: read, not used before the boundary
		getLocalEntityLandingEntity(end_keysite, GROUP_DATABASE_DEFAULT_LANDING_TYPE[getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE)]);
	}

	// always TRUE (croute.ts)
	createGenericWaypointRoute(group, task_en, end_keysite);

	//
	// Assign task
	//

	const guide = pushTaskOntoGroupTaskStack(group, task_en, valid_members);

	return assignTaskToGroupMembers(group, guide, valid_members);
}

//
// C provenance: assign.c :: int assign_task_to_group_members (entity *group, entity *guide, unsigned int valid_members)
//
// The boundary of slice 6b: each member's attachment to the guide, its
// TASK_ASSIGNED message (takeoff, landing reservation, weapons) and helicopter
// preparation are not ported. Reaching it fails loudly with the group, the
// guide and the valid member mask.
//
export function assignTaskToGroupMembers(group: Entity, guide: Entity, valid_members: number): never {
	throw new UnportedBoundaryError("assign.c :: assign_task_to_group_members", [group, guide, valid_members]);
}

//
// C provenance: assign.c :: int check_group_members_awake (entity *group)
//
// Every member's FLOAT_TYPE_SLEEP. The adopted slices' members are
// helicopters, whose sleep is en_float.c's default 0.0 (ac_float.ts), so the
// FALSE arm is unreachable (test/unit/supply-task-assignment.test.ts pins the
// default).
//
export function checkGroupMembersAwake(group: Entity): boolean {
	let mb = getLocalEntityFirstChild(group, ListType.LIST_TYPE_MEMBER);

	while (mb) {
		/* istanbul ignore if */
		if (getLocalEntityFloatValue(mb, FloatType.FLOAT_TYPE_SLEEP) > 0.0) {
			return false;
		}

		mb = getLocalEntityChildSucc(mb, ListType.LIST_TYPE_MEMBER);
	}

	return true;
}

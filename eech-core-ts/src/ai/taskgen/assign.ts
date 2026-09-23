//
// Keysite task assignment: the decision.
//
// C provenance: ai/taskgen/assign.c :: assign_keysite_tasks,
//               suitable_group_task_specific_checks (static),
//               get_suitable_registered_group, check_group_members_awake,
//               assign_primary_task_to_group (boundary only)
//
// Slice 6a (issue #16) ports which task a keysite assigns and to which group:
// the category's unassigned tasks in en_misc.c's quicksort order, the
// keysite's assign and reserve counts, the pilot lock, and the least suitable
// qualifying group. assign_primary_task_to_group (the assignment transaction:
// route, guide, the task becoming ASSIGNED, the members) is slice 6b / 6c;
// reaching it throws UnportedBoundaryError with the selected group and task.
//
// ai_log is the release macro (highlevl.h: do { } while (0)), which evaluates
// none of its arguments; debug_log calls are guarded by DEBUG_MODULE (0).
// Neither is ported.
//

import { ASSERT, UnportedBoundaryError } from "../../core/assert";
import { FLT_MAX, f32Mul } from "../../core/float32";
import { max } from "../../core/maths/miscmath";
import { SECONDS_IN_A_MINUTE } from "../../generated/c-constants";
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
	type TaskCategoryType,
} from "../../generated/c-enums";
import { GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED, GROUP_DATABASE_MINIMUM_IDLE_COUNT } from "../../generated/c-group-database";
import { KEYSITE_DATABASE_ASSIGN_TASK_COUNT, KEYSITE_DATABASE_RESERVE_TASK_COUNT } from "../../generated/c-keysite-database";
import { quicksortEntityList } from "../../entity/en_misc/en_misc";
import { assessGroupTaskLocalityFactor, type GroupRaw } from "../../entity/special/group/group";
import { getCommsModel } from "../../entity/system/comms";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent } from "../../entity/system/en_list";
import { getLocalEntityFloatValue, getLocalEntityIntValue } from "../../entity/system/en_values";
import { getLocalEntityData, getLocalEntityType, type Entity } from "../../entity/system/entity";
import { getGroupToTaskSuitability } from "../highlevl/suitable";

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
		// assign_count only falls when assign_primary_task_to_group succeeds, which is past the boundary
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
			// assign_primary_task_to_group is the boundary: it throws, so neither arm is reached
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
// C provenance: assign.c :: assign_primary_task_to_group (entity *group_en, entity *task_en)
//
// The boundary of slice 6a. The assignment transaction is not ported
// (slice 6b / 6c), so reaching it always fails loudly, carrying the group and
// task the ported decision selected.
//
export function assignPrimaryTaskToGroup(group_en: Entity, task_en: Entity): number {
	throw new UnportedBoundaryError("assign.c :: assign_primary_task_to_group", [group_en, task_en]);
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

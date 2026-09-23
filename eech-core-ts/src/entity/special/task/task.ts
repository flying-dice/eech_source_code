//
// Task entity: the state a restored task holds, as far as Slice 5a reads it.
//
// C provenance: entity/special/task/task.c, ts_int.c, ts_float.c, ts_list.c
//
// Slice 5a (issue #12) reads existing tasks only: the duplicate-task decision
// of fc_msgs.c :: response_to_force_low_on_supplies walks the requester's
// LIST_TYPE_TASK_DEPENDENT list. Task creation (ts_creat.c, taskgen.c ::
// create_task), route pointers and the other task lists are Slice 5b.
//

import { EntitySide, EntitySubTypeTask, EntityType, FloatType, IntType, ListType, TaskStateType } from "../../../generated/c-enums";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, overloadEntityListLink } from "../../system/en_list";
import { fnGetLocalEntityFloatValue, fnGetLocalEntityIntValue, getLocalEntityIntValue } from "../../system/en_values";
import { ASSERT } from "../../../core/assert";
import { getLocalEntityData, getLocalEntityType, type Entity } from "../../system/entity";

// C provenance: task.h :: struct TASK (ported fields only)
export interface TaskRaw {
	sub_type: EntitySubTypeTask;
	task_state: TaskStateType;
	task_user_data: number;
	// unsigned int side : NUM_SIDE_BITS (2)
	side: EntitySide;
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

// C provenance: ts_funcs.c :: overload_task_functions (ported subset)
export function overloadTaskFunctions(): void {
	const TASK = EntityType.ENTITY_TYPE_TASK;

	// C provenance: ts_list.c :: LIST_TYPE_TASK_DEPENDENT_LINK
	overloadEntityListLink(TASK, "task_dependent_link", [ListType.LIST_TYPE_TASK_DEPENDENT]);

	// C provenance: ts_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(TASK, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<TaskRaw>(en).sub_type);
	fnGetLocalEntityIntValue.overload(TASK, IntType.INT_TYPE_SIDE, (en) => getLocalEntityData<TaskRaw>(en).side);
	fnGetLocalEntityIntValue.overload(TASK, IntType.INT_TYPE_TASK_STATE, (en) => getLocalEntityData<TaskRaw>(en).task_state);

	// C provenance: ts_float.c :: get_local_float_value
	fnGetLocalEntityFloatValue.overload(TASK, FloatType.FLOAT_TYPE_TASK_USER_DATA, (en) => getLocalEntityData<TaskRaw>(en).task_user_data);
}

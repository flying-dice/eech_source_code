//
// Force message responses.
//
// C provenance: entity/special/force/fc_msgs.c
//
// Only ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES is ported (Slice 5a, issue #12).
// It lives apart from force.ts because it reaches into keysites, cargo, tasks
// and task generation, which themselves depend on the force entity.
//

import { EechUndefinedBehaviourError } from "../../../core/assert";
import { FLT_MAX, toFloat32 } from "../../../core/float32";
import { getGameStatus } from "../../../core/game-status";
import { KILOMETRE } from "../../../core/maths/miscmath";
import { createSupplyTask } from "../../../ai/taskgen/taskgen";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeCargo,
	EntitySubTypeKeysite,
	EntitySubTypeTask,
	EntityType,
	FloatType,
	GameStatusType,
	IntType,
	ListType,
	MovementType,
} from "../../../generated/c-enums";
import { TASK_DATABASE_TASK_PRIORITY } from "../../../generated/c-task-database";
import { getCommsModel } from "../../system/comms";
import { getLocalEntityChildSucc, getLocalEntityFirstChild } from "../../system/en_list";
import { messageResponses } from "../../system/en_msgs";
import { getLocalEntityFloatValue, getLocalEntityIntValue } from "../../system/en_values";
import type { Entity, MessageArg } from "../../system/entity";
import { getClosestKeysite, getKeysiteSupplyPosition } from "../keysite/keysite";
import { entityIsObjectOfTask } from "../task/task";

// 10 * KILOMETRE, passed to get_closest_keysite's float min_range
const SUPPLIER_EARLY_OUT_RANGE = toFloat32(10 * KILOMETRE);

//
// C provenance: fc_msgs.c :: response_to_force_low_on_supplies
//
// Decides whether a supply task should exist and identifies its supplier and
// cargo, then calls create_supply_task, the boundary of this slice.
//
// Transcribed as written, including behaviour that looks unintended (see
// docs/slices/supply-task-investigation.md and docs/slices/force-low-on-supplies.md):
// - the supplier search starts at the requester's own position and excludes
//   nothing, so an in-use airbase requester finds itself at range 0 and
//   becomes its own supplier (F2);
// - the duplicate-task guard counts non-completed supply tasks of the sender's
//   side, but the walk below it matches any supply task by TASK_USER_DATA;
// - get_closest_keysite reports the approximate range on its early exit and
//   the exact range otherwise, and the two are compared here as they come.
//
// The switch (sub_type) has no default: both senders pass AMMO or FUEL, so
// `factory` is always assigned; any other sub type would read it
// uninitialised, which the port refuses (EechUndefinedBehaviourError).
// The debug_log calls (DEBUG_MODULE, DEBUG_SUPPLY, both 0) are compiled out
// and not ported.
//
function responseToForceLowOnSupplies(message: EntityMessage, receiver: Entity, sender: Entity | undefined, args: MessageArg[]): number {
	if (getGameStatus() !== GameStatusType.GAME_STATUS_INITIALISED || getCommsModel() === CommsModelType.COMMS_MODEL_CLIENT) {
		return 0;
	}

	const sub_type = args[0] as EntitySubTypeCargo;

	const en = sender as Entity;

	// ensure no other task is already taking care of this supply requirement

	if (entityIsObjectOfTask(en, EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY, getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE) as EntitySide) !== 0) {
		let task = getLocalEntityFirstChild(en, ListType.LIST_TYPE_TASK_DEPENDENT);

		while (task) {
			if (getLocalEntityIntValue(task, IntType.INT_TYPE_ENTITY_SUB_TYPE) === EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY) {
				if (getLocalEntityFloatValue(task, FloatType.FLOAT_TYPE_TASK_USER_DATA) === sub_type) {
					// already got a supply task for this type.

					return 0;
				}
			}

			task = getLocalEntityChildSucc(task, ListType.LIST_TYPE_TASK_DEPENDENT);
		}
	}

	//
	// Locate correct supplier
	//

	const side = getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE) as EntitySide;

	const pos = getKeysiteSupplyPosition(en);

	const factory_actual_range = { value: FLT_MAX };

	const airbase_actual_range = { value: FLT_MAX };

	let factory: Entity | undefined = undefined;

	if (sub_type === EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO) {
		factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);

		if (!factory) {
			factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);
		}

		const airbase = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, SUPPLIER_EARLY_OUT_RANGE, airbase_actual_range, true, undefined);

		if (airbase_actual_range.value < factory_actual_range.value) {
			factory = airbase;

			factory_actual_range.value = airbase_actual_range.value;
		}
	} else if (sub_type === EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_FUEL) {
		factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);

		if (!factory) {
			factory = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FACTORY, side, pos, SUPPLIER_EARLY_OUT_RANGE, factory_actual_range, true, undefined);
		}

		const airbase = getClosestKeysite(EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, SUPPLIER_EARLY_OUT_RANGE, airbase_actual_range, true, undefined);

		if (airbase_actual_range.value < factory_actual_range.value) {
			factory = airbase;

			factory_actual_range.value = airbase_actual_range.value;
		}
	} else {
		// no case: C reads `factory` uninitialised. Neither sender reaches this.
		throw new EechUndefinedBehaviourError(`response_to_force_low_on_supplies reads factory uninitialised (switch has no case for cargo sub type ${sub_type})`);
	}

	if (factory) {
		//
		// Get exact cargo to be picked up
		//

		let cargo = getLocalEntityFirstChild(factory, ListType.LIST_TYPE_CARGO);

		while (cargo) {
			if (getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE) === sub_type) {
				break;
			}

			cargo = getLocalEntityChildSucc(cargo, ListType.LIST_TYPE_CARGO);
		}

		if (cargo) {
			//
			// create task
			//
			// (the "too far to re-supply" test above 75 km is commented out in C)
			//

			const movement_type = MovementType.MOVEMENT_TYPE_AIR;

			createSupplyTask(en, factory, cargo, movement_type, TASK_DATABASE_TASK_PRIORITY[EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY], undefined, undefined);
		}
	}

	return 1;
}

// C provenance: fc_msgs.c :: overload_force_message_responses (ported rows only)
export function overloadForceMessageResponses(): void {
	messageResponses.overload(EntityType.ENTITY_TYPE_FORCE, EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, responseToForceLowOnSupplies);
}

//
// The update entity and the entity update loop.
//
// C provenance: entity/special/update/up_update.c, up_update.h, up_msgs.c, up_list.c
//
// EECH has no callback scheduler. Each frame the host calls
// update_client_server_entities () once per time-acceleration step; it walks
// LIST_TYPE_UPDATE under the update entity and calls each entity's update
// function, subdividing the frame's delta time into equal sub-steps.
//
// Not ported: tacview frame logging (excluded), and the mobiles'
// moved / rotated / updated bitsets (no ported mobile update).
//

import { ASSERT } from "../../../core/assert";
import { toCInt } from "../../../core/cint";
import { getCommandLineEntityUpdateFrameRate } from "../../../core/cmndline";
import { f32Div, f32Mul } from "../../../core/float32";
import { getDeltaTime, isFrameRateLocked, setManualDeltaTime } from "../../../core/time";
import { EntityMessage, EntityType, IntType, ListType } from "../../../generated/c-enums";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, overloadEntityListRoot } from "../../system/en_list";
import { defaultMessageResponse, messageResponses, type MessageResponseFn } from "../../system/en_msgs";
import { updateClientServerEntity } from "../../system/en_updt";
import { setLocalEntityIntValue } from "../../system/en_values";
import type { Entity } from "../../system/entity";

// C provenance: up_update.h :: struct UPDATE
export interface UpdateRaw {}

// C provenance: up_update.c :: entity *update_entity = NULL, *update_succ = NULL;
let update_entity: Entity | undefined = undefined;

let update_succ: Entity | undefined = undefined;

export function resetUpdateEntity(): void {
	update_entity = undefined;
	update_succ = undefined;
}

// C provenance: up_update.h :: #define get_update_entity() (update_entity)
export function getUpdateEntity(): Entity | undefined {
	return update_entity;
}

// C provenance: up_update.c :: set_update_entity (debug_log not ported)
export function setUpdateEntity(en: Entity | undefined): void {
	update_entity = en;
}

// C provenance: up_update.h :: get_update_succ () / set_update_succ ()
export function getUpdateSucc(): Entity | undefined {
	return update_succ;
}

export function setUpdateSucc(en: Entity | undefined): void {
	update_succ = en;
}

// C provenance: up_update.c :: set_entity_update_frame_rate
export function setEntityUpdateFrameRate(frame_rate: number): number {
	ASSERT(frame_rate >= 1 && frame_rate <= 100, "(frame_rate >= 1) && (frame_rate <= 100)");

	// (int) (get_delta_time () * frame_rate + 1.0): float * int is float, + 1.0 is
	// double. The double sum is left to round to nearest: truncating it toward
	// zero first cannot change the (int), because a float f < 1 has f + 1.0 exact
	// or below 2 - 2^-24, and a larger f makes the sum exact.
	const entity_update_iterations = toCInt(f32Mul(getDeltaTime(), frame_rate) + 1.0);

	// get_delta_time () / entity_update_iterations: float / int is float
	const entity_update_delta_time = f32Div(getDeltaTime(), entity_update_iterations);

	setManualDeltaTime(entity_update_delta_time);

	return entity_update_iterations;
}

// C provenance: up_update.c :: update_client_server_entities
export function updateClientServerEntities(): void {
	ASSERT(getUpdateEntity() !== undefined, "get_update_entity ()");

	let iterations = 1;

	let delta_time = 0;

	if (!isFrameRateLocked()) {
		delta_time = getDeltaTime();

		iterations = setEntityUpdateFrameRate(getCommandLineEntityUpdateFrameRate());
	}

	// tacview frame logging: excluded (never logging)

	for (let loop = 0; loop < iterations; loop++) {
		let en = getLocalEntityFirstChild(update_entity as Entity, ListType.LIST_TYPE_UPDATE);

		while (en) {
			setUpdateSucc(getLocalEntityChildSucc(en, ListType.LIST_TYPE_UPDATE));

			updateClientServerEntity(en);

			setLocalEntityIntValue(en, IntType.INT_TYPE_UPDATED, 1);

			en = getUpdateSucc();
		}

		setUpdateSucc(undefined);
	}

	if (!isFrameRateLocked()) {
		setManualDeltaTime(delta_time);
	}
}

// C provenance: up_msgs.c :: response_to_unlink_child
const responseToUnlinkChild: MessageResponseFn = (_message, _receiver, sender, args) => {
	const type = args[0] as ListType;

	if (type === ListType.LIST_TYPE_UPDATE) {
		if (sender === getUpdateSucc()) {
			setUpdateSucc(getLocalEntityChildSucc(sender as Entity, ListType.LIST_TYPE_UPDATE));
		}
	}

	return 1;
};

export function overloadUpdateFunctions(): void {
	const UPDATE = EntityType.ENTITY_TYPE_UPDATE;

	// C provenance: up_list.c :: #define LIST_TYPE_UPDATE_ROOT
	overloadEntityListRoot(UPDATE, "update_root", [ListType.LIST_TYPE_UPDATE]);

	// C provenance: up_msgs.c :: overload_update_message_responses. With
	// DEBUG_MODULE 0 only UNLINK_CHILD is overloaded; the others keep
	// en_msgs.c :: default_message_response.
	messageResponses.overload(UPDATE, EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, responseToUnlinkChild);
	messageResponses.overload(UPDATE, EntityMessage.ENTITY_MESSAGE_LINK_CHILD, defaultMessageResponse);
	messageResponses.overload(UPDATE, EntityMessage.ENTITY_MESSAGE_LINK_PARENT, defaultMessageResponse);
	messageResponses.overload(UPDATE, EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, defaultMessageResponse);
}

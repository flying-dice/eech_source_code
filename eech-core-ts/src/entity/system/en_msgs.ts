//
// Entity messages.
//
// C provenance: entity/system/en_msgs/en_msgs.c :: notify_local_entity,
//               message_responses, default_message_response
//
// C delivers `va_list` arguments; the port passes the trailing arguments as an
// array in the same order.
//

import { ASSERT, UnportedBehaviourError } from "../../core/assert";
import { EntityMessage, EntityType } from "../../generated/c-enums";
import { getUnportedMessagePolicy, recordUnportedMessage, type Entity, type MessageArg } from "./entity";
import { EntityFunctionTable } from "./function-table";

export type MessageResponseFn = (message: EntityMessage, receiver: Entity, sender: Entity | undefined, args: MessageArg[]) => number;

export const messageResponses = new EntityFunctionTable<MessageResponseFn>("message_responses");

// C provenance: en_msgs.c :: notify_local_entity
export function notifyLocalEntity(message: EntityMessage, receiver: Entity | undefined, sender: Entity | undefined, ...args: MessageArg[]): number {
	ASSERT(message >= 0 && message < EntityMessage.NUM_ENTITY_MESSAGES, "(message >= 0) && (message < NUM_ENTITY_MESSAGES)");

	ASSERT(receiver !== undefined, "receiver");

	return messageResponses.lookup(receiver.type, message, EntityMessage[message])(message, receiver, sender, args);
}

// C provenance: en_msgs.c :: default_message_response. Only installed where the
// C message table is known to keep this default.
export function defaultMessageResponse(): number {
	return 0;
}

//
// Marks a message response that EECH overloads but the port has not ported
// yet. With the "throw" policy (production) delivery fails loudly. With the
// "record" policy (tests) the delivery is recorded and acknowledged with FALSE,
// the value of default_message_response, so callers that ignore the
// acknowledgement (as assess_group_supplies does) can be tested up to the
// message boundary.
//
export function overloadUnportedMessageResponse(entityType: EntityType, message: EntityMessage, provenance: string): void {
	messageResponses.overload(entityType, message, (msg, receiver, sender, args) => {
		if (getUnportedMessagePolicy() === "throw") {
			throw new UnportedBehaviourError(`message_responses [${EntityType[entityType]}] [${EntityMessage[msg]}] (${provenance})`);
		}

		recordUnportedMessage({ message: msg, receiver, sender, args, provenance });

		return 0;
	});
}

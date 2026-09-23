//
// Entity messages.
//
// C provenance: entity/system/en_msgs/en_msgs.c :: notify_local_entity,
//               message_responses, default_message_response
//
// C delivers `va_list` arguments; the port passes the trailing arguments as an
// array in the same order.
//

import { ASSERT } from "../../core/assert";
import { EntityMessage } from "../../generated/c-enums";
import type { Entity, MessageArg } from "./entity";
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

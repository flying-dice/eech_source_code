//
// Aircraft link / unlink parent responses.
//
// C provenance: entity/mobile/aircraft/ac_msgs.c :: response_to_link_parent,
//               response_to_unlink_parent, overload_aircraft_message_responses
//
// Cargo uses them too: cg_msgs.c :: overload_cargo_message_responses calls
// overload_aircraft_message_responses (ENTITY_TYPE_CARGO).
//
// Only the list types without an arm (return TRUE) are ported. The
// LIST_TYPE_GUNSHIP_TARGET, LIST_TYPE_TARGET and LIST_TYPE_UPDATE arms (radar
// contact, group targeting notification, view interest) fail loudly.
//

import { UnportedBehaviourError } from "../../../core/assert";
import { EntityMessage, ListType, type EntityType } from "../../../generated/c-enums";
import { messageResponses, type MessageResponseFn } from "../../system/en_msgs";

// C provenance: ac_msgs.c :: response_to_link_parent
const responseToLinkParent: MessageResponseFn = (_message, _receiver, _sender, args) => {
	const list_type = args[0] as ListType;

	if (list_type === ListType.LIST_TYPE_GUNSHIP_TARGET || list_type === ListType.LIST_TYPE_TARGET) {
		throw new UnportedBehaviourError(`ac_msgs.c :: response_to_link_parent (${ListType[list_type]})`);
	}

	return 1;
};

// C provenance: ac_msgs.c :: response_to_unlink_parent
const responseToUnlinkParent: MessageResponseFn = (_message, _receiver, _sender, args) => {
	const list_type = args[0] as ListType;

	if (list_type === ListType.LIST_TYPE_GUNSHIP_TARGET || list_type === ListType.LIST_TYPE_UPDATE) {
		throw new UnportedBehaviourError(`ac_msgs.c :: response_to_unlink_parent (${ListType[list_type]})`);
	}

	return 1;
};

// C provenance: ac_msgs.c :: overload_aircraft_message_responses (LINK_PARENT, UNLINK_PARENT rows only)
export function overloadAircraftLinkParentResponses(type: EntityType): void {
	messageResponses.overload(type, EntityMessage.ENTITY_MESSAGE_LINK_PARENT, responseToLinkParent);
	messageResponses.overload(type, EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, responseToUnlinkParent);
}

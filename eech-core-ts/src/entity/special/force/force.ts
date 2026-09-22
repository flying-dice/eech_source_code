//
// Force entity.
//
// C provenance: entity/special/force/force.c, fc_int.c, fc_list.c, fc_msgs.c
//

import { EntityMessage, EntitySide, EntityType, IntType, ListType } from "../../../generated/c-enums";
import { ASSERT } from "../../../core/assert";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, overloadEntityListLink, overloadEntityListRoot } from "../../system/en_list";
import { overloadUnportedMessageResponse } from "../../system/en_msgs";
import { fnGetLocalEntityIntValue, getLocalEntityIntValue } from "../../system/en_values";
import { getLocalEntityData, getLocalEntityType, getSessionEntity, type Entity } from "../../system/entity";

// C provenance: force.h :: struct FORCE (ported fields only)
export interface ForceRaw {
	side: EntitySide;
}

// C provenance: force.c :: get_local_force_entity
export function getLocalForceEntity(side: EntitySide): Entity | undefined {
	const session = getSessionEntity();

	if (!session) {
		return undefined;
	}

	let force = getLocalEntityFirstChild(session, ListType.LIST_TYPE_FORCE);

	while (force) {
		// C: debug_assert
		ASSERT(getLocalEntityType(force) === EntityType.ENTITY_TYPE_FORCE, "get_local_entity_type (force) == ENTITY_TYPE_FORCE");

		if (getLocalEntityIntValue(force, IntType.INT_TYPE_SIDE) === side) {
			return force;
		}

		force = getLocalEntityChildSucc(force, ListType.LIST_TYPE_FORCE);
	}

	return undefined;
}

export function overloadForceFunctions(): void {
	// C provenance: fc_list.c :: LIST_TYPE_KEYSITE_FORCE_ROOT, LIST_TYPE_INDEPENDENT_GROUP_ROOT, LIST_TYPE_FORCE_LINK
	overloadEntityListRoot(EntityType.ENTITY_TYPE_FORCE, "keysite_force_root", [ListType.LIST_TYPE_KEYSITE_FORCE]);
	overloadEntityListRoot(EntityType.ENTITY_TYPE_FORCE, "independent_group_root", [ListType.LIST_TYPE_INDEPENDENT_GROUP]);
	overloadEntityListLink(EntityType.ENTITY_TYPE_FORCE, "force_link", [ListType.LIST_TYPE_FORCE]);

	// C provenance: fc_int.c :: overload_force_int_value_functions, get_local_int_value
	fnGetLocalEntityIntValue.overload(EntityType.ENTITY_TYPE_FORCE, IntType.INT_TYPE_SIDE, (en) => getLocalEntityData<ForceRaw>(en).side);

	// C provenance: fc_msgs.c :: overload_force_message_responses
	overloadUnportedMessageResponse(
		EntityType.ENTITY_TYPE_FORCE,
		EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES,
		"fc_msgs.c :: response_to_force_low_on_supplies",
	);
}

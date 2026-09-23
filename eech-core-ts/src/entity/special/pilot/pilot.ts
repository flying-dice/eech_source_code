//
// Pilot entity: the pilot lock only.
//
// C provenance: entity/special/pilot/pi_list.c (LIST_TYPE_PILOT_LOCK_ROOT)
//
// Slice 6a (issue #16): a task or group a player's pilot has locked hangs on
// that pilot's LIST_TYPE_PILOT_LOCK list, and assign.c skips it. Nothing else
// of the pilot entity is ported.
//

import { EntityType, ListType } from "../../../generated/c-enums";
import { overloadEntityListRoot } from "../../system/en_list";

export function overloadPilotFunctions(): void {
	// C provenance: pi_list.c :: LIST_TYPE_PILOT_LOCK_ROOT
	overloadEntityListRoot(EntityType.ENTITY_TYPE_PILOT, "pilot_lock_root", [ListType.LIST_TYPE_PILOT_LOCK]);
}

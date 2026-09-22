//
// C provenance: entity/special/session/ss_list.c (list layout only)
//

import { EntityType, ListType } from "../../../generated/c-enums";
import { overloadEntityListRoot } from "../../system/en_list";

export { getSessionEntity } from "../../system/entity";

// C provenance: ss_list.c :: #define LIST_TYPE_FORCE_ROOT
export function overloadSessionListFunctions(): void {
	overloadEntityListRoot(EntityType.ENTITY_TYPE_SESSION, "force_root", [ListType.LIST_TYPE_FORCE]);
}

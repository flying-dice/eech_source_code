//
// Guide entity (list layout only).
//
// C provenance: entity/special/guide/gd_list.c :: #define LIST_TYPE_GUIDE_STACK_LINK
//

import { EntityType, ListType } from "../../../generated/c-enums";
import { overloadEntityListLink } from "../../system/en_list";

export function overloadGuideFunctions(): void {
	overloadEntityListLink(EntityType.ENTITY_TYPE_GUIDE, "guide_stack_link", [ListType.LIST_TYPE_GUIDE_STACK]);
}

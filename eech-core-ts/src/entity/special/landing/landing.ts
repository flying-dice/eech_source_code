//
// A keysite's landing entities.
//
// C provenance: entity/special/landing/landing.c :: get_local_entity_landing_entity
//
// assign.c :: assign_task_to_group looks up the return keysite's landing
// entity for the group's landing type and does not use it on the paths the
// port reaches. Landing entities are not ported: a restored keysite has no
// LIST_TYPE_LANDING_SITE children, and one with any fails loudly rather than
// comparing sub types of an unported entity type.
//

import { ASSERT, UnportedBehaviourError } from "../../../core/assert";
import { EntityType, ListType } from "../../../generated/c-enums";
import { getLocalEntityFirstChild, overloadEntityListLink } from "../../system/en_list";
import { getLocalEntityType, type Entity } from "../../system/entity";

// C provenance: landing.c :: get_local_entity_landing_entity (en, landing_type)
export function getLocalEntityLandingEntity(en: Entity, _landing_type: number): Entity | undefined {
	ASSERT(getLocalEntityType(en) === EntityType.ENTITY_TYPE_KEYSITE, "get_local_entity_type (en) == ENTITY_TYPE_KEYSITE");

	const landing_entity = getLocalEntityFirstChild(en, ListType.LIST_TYPE_LANDING_SITE);

	if (landing_entity) {
		throw new UnportedBehaviourError("landing.c :: get_local_entity_landing_entity (a keysite with landing entities)");
	}

	return undefined;
}

// C provenance: ld_list.c :: LIST_TYPE_LANDING_SITE_LINK (the landing entity's place on its keysite's list)
export function overloadLandingFunctions(): void {
	overloadEntityListLink(EntityType.ENTITY_TYPE_LANDING, "landing_site_link", [ListType.LIST_TYPE_LANDING_SITE]);
}

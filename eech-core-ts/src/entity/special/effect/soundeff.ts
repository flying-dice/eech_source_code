//
// Sound effects attached to an entity.
//
// C provenance: entity/special/effect/soundeff/soundeff.c :: destroy_client_server_sound_effects
//
// Destroying an entity family first destroys the SOUND_EFFECT children on its
// LIST_TYPE_SPECIAL_EFFECT list. Sound effects themselves are not ported:
// destroying one fails loudly through the destroy tables.
//

import { EntityType, ListType } from "../../../generated/c-enums";
import { destroyClientServerEntityFamily } from "../../system/en_dstry";
import { getLocalEntityChildSucc, getLocalEntityFirstChild } from "../../system/en_list";
import { getLocalEntityType, type Entity } from "../../system/entity";

export function destroyClientServerSoundEffects(en: Entity): void {
	let spec = getLocalEntityFirstChild(en, ListType.LIST_TYPE_SPECIAL_EFFECT);

	while (spec) {
		const next = getLocalEntityChildSucc(spec, ListType.LIST_TYPE_SPECIAL_EFFECT);

		if (getLocalEntityType(spec) === EntityType.ENTITY_TYPE_SOUND_EFFECT) {
			destroyClientServerEntityFamily(spec);
		}

		spec = next;
	}
}

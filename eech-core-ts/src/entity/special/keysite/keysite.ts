//
// Keysite entity.
//
// C provenance: entity/special/keysite/keysite.c, ks_int.c, ks_float.c, ks_vec3d.c, ks_list.c
//

import { ASSERT, assertNotNullDereference } from "../../../core/assert";
import { FLT_MAX } from "../../../core/float32";
import { get2dRange, getApprox2dRange } from "../../../core/maths/range";
import type { Vec3d } from "../../../core/maths/vec3d";
import { CommsModelType, EntityMessage, EntitySide, EntitySubTypeKeysite, EntityType, FloatType, IntType, ListType, Vec3dType } from "../../../generated/c-enums";
import { messageResponses, type MessageResponseFn } from "../../system/en_msgs";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, overloadEntityListLink, overloadEntityListRoot } from "../../system/en_list";
import {
	fnGetLocalEntityFloatValue,
	fnGetLocalEntityIntValue,
	fnGetLocalEntityVec3dPtr,
	fnSetClientServerEntityFloatValue,
	getLocalEntityIntValue,
	getLocalEntityVec3dPtr,
	serverFloatValueSetter,
} from "../../system/en_values";
import { getLocalEntityData, type Entity } from "../../system/entity";
import { getLocalForceEntity } from "../force/force";

// C provenance: en_types/en_suply.h :: struct SUPPLY_TYPE (ported fields only)
export interface SupplyRaw {
	ammo_supply_level: number;
	fuel_supply_level: number;
}

// C provenance: keysite.h :: struct KEYSITE (ported fields only)
export interface KeysiteRaw {
	sub_type: EntitySubTypeKeysite;
	side: EntitySide;
	in_use: number;
	position: Vec3d;
	supplies: SupplyRaw;
}

//
// C provenance: keysite.c :: get_closest_keysite
//
// `actual_range` (float *) becomes an optional out-parameter object.
// LANDING_DEBUG logging is compiled out in EECH and not ported.
//
export function getClosestKeysite(
	type: number,
	side: EntitySide,
	pos: Vec3d | undefined,
	min_range: number,
	actual_range: { value: number } | undefined,
	outside_of_range: boolean,
	exclude_keysite: Entity | undefined,
): Entity | undefined {
	ASSERT(min_range > 0.0 || outside_of_range, "min_range > 0.0 || outside_of_range");

	let best_range = FLT_MAX;

	let closest_keysite: Entity | undefined = undefined;

	const force = getLocalForceEntity(side);

	// C passes a NULL force straight to get_local_entity_first_child, which dereferences it
	assertNotNullDereference(force, "get_local_entity_first_child (force, LIST_TYPE_KEYSITE_FORCE)");

	let current_keysite = getLocalEntityFirstChild(force, ListType.LIST_TYPE_KEYSITE_FORCE);

	while (current_keysite) {
		if (current_keysite !== exclude_keysite) {
			if (getLocalEntityIntValue(current_keysite, IntType.INT_TYPE_ENTITY_SUB_TYPE) === type || type === EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES) {
				// C truthiness of an int: compare with 0 explicitly (0 is true in Lua)
				if (getLocalEntityIntValue(current_keysite, IntType.INT_TYPE_IN_USE) !== 0) {
					const keysite_pos = getLocalEntityVec3dPtr(current_keysite, Vec3dType.VEC3D_TYPE_POSITION);

					const range = getApprox2dRange(keysite_pos, pos);

					if (range <= min_range) {
						if (actual_range) {
							actual_range.value = range;
						}

						return current_keysite;
					}

					if (range < best_range && outside_of_range) {
						best_range = range;

						closest_keysite = current_keysite;
					}
				}
			}
		}

		current_keysite = getLocalEntityChildSucc(current_keysite, ListType.LIST_TYPE_KEYSITE_FORCE);
	}

	if (closest_keysite) {
		const keysite_pos = getLocalEntityVec3dPtr(closest_keysite, Vec3dType.VEC3D_TYPE_POSITION);

		best_range = get2dRange(keysite_pos, pos);
	}

	if (actual_range) {
		actual_range.value = best_range;
	}

	return closest_keysite;
}

// C provenance: ks_msgs.c :: response_to_link_child, response_to_unlink_child
// (their bodies only log under DEBUG_MODULE)
const responseToLinkOrUnlinkChild: MessageResponseFn = () => 1;

export function overloadKeysiteFunctions(): void {
	const KEYSITE = EntityType.ENTITY_TYPE_KEYSITE;

	// C provenance: ks_list.c :: LIST_TYPE_KEYSITE_GROUP_ROOT, LIST_TYPE_BUILDING_GROUP_ROOT, LIST_TYPE_CARGO_ROOT, LIST_TYPE_KEYSITE_FORCE_LINK
	overloadEntityListRoot(KEYSITE, "keysite_group_root", [ListType.LIST_TYPE_KEYSITE_GROUP]);
	overloadEntityListRoot(KEYSITE, "building_group_root", [ListType.LIST_TYPE_BUILDING_GROUP]);
	overloadEntityListRoot(KEYSITE, "cargo_root", [ListType.LIST_TYPE_CARGO]);
	overloadEntityListLink(KEYSITE, "keysite_force_link", [ListType.LIST_TYPE_KEYSITE_FORCE]);

	// C provenance: ks_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(KEYSITE, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<KeysiteRaw>(en).sub_type);
	fnGetLocalEntityIntValue.overload(KEYSITE, IntType.INT_TYPE_IN_USE, (en) => getLocalEntityData<KeysiteRaw>(en).in_use);

	// C provenance: ks_float.c :: get_local_float_value
	fnGetLocalEntityFloatValue.overload(KEYSITE, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, (en) => getLocalEntityData<KeysiteRaw>(en).supplies.ammo_supply_level);
	fnGetLocalEntityFloatValue.overload(KEYSITE, FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL, (en) => getLocalEntityData<KeysiteRaw>(en).supplies.fuel_supply_level);

	// C provenance: ks_float.c :: set_local_float_value, set_server_float_value
	const server = fnSetClientServerEntityFloatValue[CommsModelType.COMMS_MODEL_SERVER];

	server.overload(
		KEYSITE,
		FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL,
		serverFloatValueSetter((en, _type, value) => {
			getLocalEntityData<KeysiteRaw>(en).supplies.ammo_supply_level = value;
		}),
	);

	server.overload(
		KEYSITE,
		FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL,
		serverFloatValueSetter((en, _type, value) => {
			getLocalEntityData<KeysiteRaw>(en).supplies.fuel_supply_level = value;
		}),
	);

	// C provenance: ks_msgs.c :: overload_keysite_message_responses (LINK_CHILD, UNLINK_CHILD rows)
	messageResponses.overload(KEYSITE, EntityMessage.ENTITY_MESSAGE_LINK_CHILD, responseToLinkOrUnlinkChild);
	messageResponses.overload(KEYSITE, EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, responseToLinkOrUnlinkChild);

	// C provenance: ks_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION) -> &raw->position
	fnGetLocalEntityVec3dPtr.overload(KEYSITE, Vec3dType.VEC3D_TYPE_POSITION, (en) => getLocalEntityData<KeysiteRaw>(en).position);
}

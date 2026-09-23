//
// Keysite entity.
//
// C provenance: entity/special/keysite/keysite.c, ks_int.c, ks_float.c, ks_vec3d.c, ks_list.c
//

import { ASSERT, assertNotNullDereference } from "../../../core/assert";
import { f32Add, f32Mul, f32Sub, f64AddRTZ, FLT_MAX, toFloat32RTZ } from "../../../core/float32";
import { getGameStatus } from "../../../core/game-status";
import { get2dRange, getApprox2dRange } from "../../../core/maths/range";
import type { Vec3d } from "../../../core/maths/vec3d";
import { KEYSITE_SUPPLY_REQUEST_THRESHOLD, OBJECT_3D_SINGLE_CRATE } from "../../../generated/c-constants";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeCargo,
	EntitySubTypeKeysite,
	EntityType,
	FloatType,
	GameStatusType,
	IntType,
	ListType,
	Vec3dType,
} from "../../../generated/c-enums";
import { KEYSITE_DATABASE_AMMO_SUPPLY_USAGE, KEYSITE_DATABASE_FUEL_SUPPLY_USAGE } from "../../../generated/c-keysite-database";
import { createClientServerEntity } from "../../system/en_creat";
import { destroyClientServerEntityFamily } from "../../system/en_dstry";
import { ENTITY_INDEX_DONT_CARE } from "../../system/en_heap";
import { messageResponses, notifyLocalEntity, type MessageResponseFn } from "../../system/en_msgs";
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
import { getCampaignPorts, getLocalEntityData, type Entity } from "../../system/entity";
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
	// unsigned int alive : NUM_ALIVE_BITS (1)
	alive: number;
	// unsigned int in_use : NUM_IN_USE_BITS
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

//
// C provenance: keysite.c :: update_keysite_cargo
//
// Keeps the keysite's crates of one supply (ammo or fuel) in step with its
// supply level: surplus crates are destroyed, missing ones created in a row
// beside the keysite, and when none is missing and the level is low, the
// keysite's force is told (ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES). Crate
// geometry comes from the Object3DMetadata port. Floats follow the RTZ
// contract (src/core/float32.ts); docs/slices/keysite-cargo.md maps every step.
// DEBUG_MODULE / DEBUG_SUPPLY logging is compiled out in EECH and not ported.
//
export function updateKeysiteCargo(en: Entity, cargo_level: number, sub_type: EntitySubTypeCargo, cargo_size: number): void {
	// float parameters
	cargo_level = toFloat32RTZ(cargo_level);
	cargo_size = toFloat32RTZ(cargo_size);

	//
	// Get cargo position
	//

	if (getGameStatus() === GameStatusType.GAME_STATUS_INITIALISING) {
		return;
	}

	const raw = getLocalEntityData<KeysiteRaw>(en);

	if (raw.alive === 0 || raw.in_use === 0) {
		return;
	}

	// memcpy (&position, get_keysite_supply_position (en), sizeof (vec3d))
	const supply_position = getLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

	const position: Vec3d = { x: supply_position.x, y: supply_position.y, z: supply_position.z };

	// get_object_3d_bounding_box (OBJECT_3D_SINGLE_CRATE): struct OBJECT_3D_BOUNDS holds floats
	const box = getCampaignPorts().object3DMetadata.getBoundingBox(OBJECT_3D_SINGLE_CRATE);

	const xmin = toFloat32RTZ(box.xmin);
	const xmax = toFloat32RTZ(box.xmax);
	const ymin = toFloat32RTZ(box.ymin);
	const zmin = toFloat32RTZ(box.zmin);
	const zmax = toFloat32RTZ(box.zmax);

	//
	// work out start position
	//

	position.y = f32Sub(position.y, ymin);

	// sub_type * ((zmax - zmin) + 1): float + int 1, then int * float
	position.z = f32Add(position.z, f32Mul(sub_type, f32Add(f32Sub(zmax, zmin), 1)));

	// (xmax - xmin) + 1.0: a double
	const crate_spacing = f64AddRTZ(f32Sub(xmax, xmin), 1.0);

	//
	// check for existing cargo
	//

	let temp_cargo_level = cargo_level;

	// cargo = raw->cargo_root.first_child
	let cargo = getLocalEntityFirstChild(en, ListType.LIST_TYPE_CARGO);

	while (cargo) {
		if (getLocalEntityIntValue(cargo, IntType.INT_TYPE_ENTITY_SUB_TYPE) === sub_type) {
			temp_cargo_level = f32Sub(temp_cargo_level, cargo_size);

			if (temp_cargo_level < 0.0) {
				//
				// destroy surplus cargo
				//

				const destroy_cargo = cargo;

				cargo = getLocalEntityChildSucc(cargo, ListType.LIST_TYPE_CARGO);

				destroyClientServerEntityFamily(destroy_cargo);

				continue;
			}

			// position.x += (xmax - xmin) + 1.0: a double sum stored as float
			position.x = f32Add(position.x, crate_spacing);
		}

		cargo = getLocalEntityChildSucc(cargo, ListType.LIST_TYPE_CARGO);
	}

	//
	// Create new cargo as required
	//

	if (temp_cargo_level > cargo_size) {
		while (temp_cargo_level > cargo_size) {
			createClientServerEntity(EntityType.ENTITY_TYPE_CARGO, ENTITY_INDEX_DONT_CARE, [
				{ kind: "parent", type: ListType.LIST_TYPE_CARGO, entity: en },
				{ kind: "int_value", type: IntType.INT_TYPE_SIDE, value: getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE) },
				{ kind: "int_value", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: sub_type },
				{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x: position.x, y: position.y, z: position.z },
			]);

			position.x = f32Add(position.x, crate_spacing);

			temp_cargo_level = f32Sub(temp_cargo_level, cargo_size);
		}
	} else if (cargo_level <= KEYSITE_SUPPLY_REQUEST_THRESHOLD) {
		//
		// low on cargo supplies so request some more
		//

		if (sub_type === EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO) {
			if (KEYSITE_DATABASE_AMMO_SUPPLY_USAGE[raw.sub_type] < 0.0) {
				const force = getLocalForceEntity(raw.side);

				notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, force, en, sub_type);
			}
		} else if (sub_type === EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_FUEL) {
			if (KEYSITE_DATABASE_FUEL_SUPPLY_USAGE[raw.sub_type] < 0.0) {
				const force = getLocalForceEntity(raw.side);

				notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, force, en, sub_type);
			}
		}

		// switch (sub_type) has no default: other cargo sub types do nothing
	}
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
	fnGetLocalEntityIntValue.overload(KEYSITE, IntType.INT_TYPE_SIDE, (en) => getLocalEntityData<KeysiteRaw>(en).side);

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

//
// Mobile entities (aircraft and vehicles) as seen by the campaign.
//
// C provenance: entity/mobile/aircraft/ac_list.c, ac_vec3d.c, ac_funcs.c (overload_aircraft_functions)
//               entity/mobile/vehicle/vh_list.c,  vh_vec3d.c, vh_funcs.c  (overload_vehicle_functions)
//
// Only the campaign-visible surface is ported: membership of a group
// (LIST_TYPE_MEMBER_LINK) and the current position. The position is physical
// state produced by the flight model / vehicle simulation, so it is read
// through the MobilePhysicalState port. Flight models, movement, weapons and
// rendering are excluded (physical simulation).
//

import { storeUnsignedBitfield } from "../../core/cint";
import { toFloat32RTZ } from "../../core/float32";
import type { Vec3d } from "../../core/maths/vec3d";
import { EntityType, IntType, ListType, Vec3dType } from "../../generated/c-enums";
import { overloadEntityListLink, overloadEntityListRoot } from "../system/en_list";
import { defaultGetEntityIntValue, fnGetLocalEntityIntValue, fnGetLocalEntityVec3d, fnGetLocalEntityVec3dPtr, fnSetLocalEntityRawIntValue, fnSetLocalEntityRawVec3d, getLocalEntityVec3dPtr } from "../system/en_values";
import { getCampaignPorts, getLocalEntityData, type Entity } from "../system/entity";
import { overloadAircraftFloatValueFunctions } from "./aircraft/ac_float";

// C provenance: en_int.h :: NUM_ALIVE_BITS, NUM_SIDE_BITS
export const NUM_ALIVE_BITS = 1;

export const NUM_SIDE_BITS = 2;

// C provenance: en_sbtyp.h :: ENTITY_SUB_TYPE_UNINITIALISED; en_side.h :: ENTITY_SIDE_UNINITIALISED (NUM_ENTITY_SIDES)
export const ENTITY_SUB_TYPE_UNINITIALISED = -1;

export const ENTITY_SIDE_UNINITIALISED = 3;

//
// C provenance: mobile.h :: struct MOBILE (ported fields only)
//
// Every mobile entity's raw struct starts with `mobile mob`; the mobile
// overloads (mb_*.c) cast the raw data to `mobile *`. Here they read `.mob`.
// `alive` and `side` are `unsigned int` bit-fields. The attitude matrix is not
// ported (no ported reader); creation sets it to the identity.
//
export interface MobileRaw {
	sub_type: number;
	position: Vec3d;
	alive: number;
	side: number;
}

function mob(en: Entity): MobileRaw {
	return getLocalEntityData<{ mob: MobileRaw }>(en).mob;
}

//
// The mobile overloads of mb_int.c, mb_vec3d.c and mb_list.c for an entity
// type whose position is campaign state held in raw->mob.position (cargo),
// not physical state produced by a simulation. Only the rows the adopted
// slices reach are ported.
//
export function overloadMobileRawStateFunctions(type: EntityType): void {
	// C provenance: mb_int.c :: set_local_int_value (raw), get_local_int_value
	fnSetLocalEntityRawIntValue.overload(type, IntType.INT_TYPE_ALIVE, (en, _type, value) => {
		mob(en).alive = storeUnsignedBitfield(value, NUM_ALIVE_BITS);
	});
	fnSetLocalEntityRawIntValue.overload(type, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en, _type, value) => {
		mob(en).sub_type = value;
	});
	fnSetLocalEntityRawIntValue.overload(type, IntType.INT_TYPE_SIDE, (en, _type, value) => {
		mob(en).side = storeUnsignedBitfield(value, NUM_SIDE_BITS);
	});
	fnGetLocalEntityIntValue.overload(type, IntType.INT_TYPE_ALIVE, (en) => mob(en).alive);
	fnGetLocalEntityIntValue.overload(type, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => mob(en).sub_type);
	fnGetLocalEntityIntValue.overload(type, IntType.INT_TYPE_SIDE, (en) => mob(en).side);

	// C provenance: en_int.c :: default_get_entity_int_value (0): mb_int.c does not overload these
	fnGetLocalEntityIntValue.overload(type, IntType.INT_TYPE_IDENTIFY_AIRCRAFT, defaultGetEntityIntValue);
	fnGetLocalEntityIntValue.overload(type, IntType.INT_TYPE_IDENTIFY_FIXED, defaultGetEntityIntValue);
	fnGetLocalEntityIntValue.overload(type, IntType.INT_TYPE_IDENTIFY_VEHICLE, defaultGetEntityIntValue);

	// C provenance: mb_vec3d.c :: set_local_raw_vec3d, get_local_vec3d_ptr (VEC3D_TYPE_POSITION)
	fnSetLocalEntityRawVec3d.overload(type, Vec3dType.VEC3D_TYPE_POSITION, (en, _type, v) => {
		mob(en).position = { x: v.x, y: v.y, z: v.z };
	});
	fnGetLocalEntityVec3dPtr.overload(type, Vec3dType.VEC3D_TYPE_POSITION, (en) => mob(en).position);

	// C provenance: mb_list.c :: LIST_TYPE_SPECIAL_EFFECT_ROOT, LIST_TYPE_TARGET_ROOT,
	//               LIST_TYPE_PADLOCK_LINK, LIST_TYPE_SECTOR_LINK, LIST_TYPE_TARGET_LINK, LIST_TYPE_UPDATE_LINK
	overloadEntityListRoot(type, "special_effect_root", [ListType.LIST_TYPE_SPECIAL_EFFECT]);
	overloadEntityListRoot(type, "target_root", [ListType.LIST_TYPE_TARGET]);
	overloadEntityListLink(type, "padlock_link", [ListType.LIST_TYPE_PADLOCK]);
	overloadEntityListLink(type, "sector_link", [ListType.LIST_TYPE_SECTOR]);
	overloadEntityListLink(type, "target_link", [ListType.LIST_TYPE_TARGET]);
	overloadEntityListLink(type, "update_link", [ListType.LIST_TYPE_UPDATE]);
}

// C provenance: hc_funcs.c, fw_funcs.c (overload_aircraft_functions), rv_funcs.c, sh_funcs.c,
//               aa_funcs.c, ps_funcs.c (overload_vehicle_functions)
export const MOBILE_GROUP_MEMBER_ENTITY_TYPES: EntityType[] = [
	EntityType.ENTITY_TYPE_HELICOPTER,
	EntityType.ENTITY_TYPE_FIXED_WING,
	EntityType.ENTITY_TYPE_ROUTED_VEHICLE,
	EntityType.ENTITY_TYPE_SHIP_VEHICLE,
	EntityType.ENTITY_TYPE_ANTI_AIRCRAFT,
	EntityType.ENTITY_TYPE_PERSON,
];

export function overloadMobileFunctions(): void {
	for (const type of MOBILE_GROUP_MEMBER_ENTITY_TYPES) {
		// C provenance: ac_list.c / vh_list.c :: #define LIST_TYPE_MEMBER_LINK
		overloadEntityListLink(type, "member_link", [ListType.LIST_TYPE_MEMBER]);

		// C provenance: ac_vec3d.c / vh_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION) -> &raw->mob.position
		// raw->mob.position is a float vec3d, hence the narrowing.
		fnGetLocalEntityVec3dPtr.overload(type, Vec3dType.VEC3D_TYPE_POSITION, (en) => {
			const position = getCampaignPorts().mobilePhysicalState.getMobilePosition(en.index);

			return {
				x: toFloat32RTZ(position.x),
				y: toFloat32RTZ(position.y),
				z: toFloat32RTZ(position.z),
			};
		});

		// C provenance: mb_vec3d.c :: get_local_vec3d (VEC3D_TYPE_POSITION): *v = the position (slice 6b)
		fnGetLocalEntityVec3d.overload(type, Vec3dType.VEC3D_TYPE_POSITION, (en) => {
			const position = getLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

			return { x: position.x, y: position.y, z: position.z };
		});
	}

	// C provenance: hc_funcs.c, fw_funcs.c -> ac_funcs.c :: overload_aircraft_functions
	//               -> overload_aircraft_float_value_functions (the rows task assignment reads; slice 6a)
	overloadAircraftFloatValueFunctions(EntityType.ENTITY_TYPE_HELICOPTER);
	overloadAircraftFloatValueFunctions(EntityType.ENTITY_TYPE_FIXED_WING);
}

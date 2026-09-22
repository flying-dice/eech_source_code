//
// Group entity.
//
// C provenance: entity/special/group/group.c, gp_int.c, gp_float.c, gp_vec3d.c,
//               gp_ptr.c, gp_list.c, gp_dbase.c
//

import { ASSERT } from "../../../core/assert";
import { toFloat32 } from "../../../core/float32";
import { bound, KILOMETRE } from "../../../core/maths/miscmath";
import { GROUP_DATABASE_RESUPPLY_SOURCE } from "../../../generated/c-group-database";
import {
	CommsModelType,
	EntityMessage,
	EntitySide,
	EntitySubTypeCargo,
	EntitySubTypeGroup,
	EntitySubTypeKeysite,
	EntityType,
	FloatType,
	GroupModeType,
	IntType,
	ListType,
	PtrType,
	ResupplySourceType,
	Vec3dType,
} from "../../../generated/c-enums";
import { getLocalEntityFirstChild, getLocalEntityParent, overloadEntityListLink, overloadEntityListRoot } from "../../system/en_list";
import { notifyLocalEntity } from "../../system/en_msgs";
import {
	fnGetLocalEntityIntValue,
	fnGetLocalEntityPtrValue,
	fnGetLocalEntityVec3dPtr,
	fnSetClientServerEntityFloatValue,
	getLocalEntityFloatValue,
	getLocalEntityIntValue,
	getLocalEntityPtrValue,
	getLocalEntityVec3dPtr,
	serverFloatValueSetter,
	setClientServerEntityFloatValue,
} from "../../system/en_values";
import { getLocalEntityData, getLocalEntityType, type Entity } from "../../system/entity";
import { getLocalForceEntity } from "../force/force";
import { getClosestKeysite, type SupplyRaw } from "../keysite/keysite";

// C provenance: en_types/en_suply.h
export const FUEL_USAGE_ACCELERATOR = 1.0;

export const AMMO_USAGE_ACCELERATOR = 1.0;

// C provenance: group.h :: struct GROUP (ported fields only)
export interface GroupRaw {
	sub_type: EntitySubTypeGroup;
	side: EntitySide;
	supplies: SupplyRaw;
}

//
// C provenance: group.c :: assess_group_supplies
//
// debug_log calls are guarded by DEBUG_MODULE || DEBUG_SUPPLY, both 0 in
// EECH, and are not ported.
//
export function assessGroupSupplies(en: Entity): void {
	const raw = getLocalEntityData<GroupRaw>(en);

	if (getLocalEntityIntValue(en, IntType.INT_TYPE_RESUPPLY_SOURCE) === ResupplySourceType.RESUPPLY_SOURCE_GROUP) {
		//
		// Check if Group needs to request Supplies via Mission...
		//

		if (raw.supplies.ammo_supply_level < 100.0) {
			const force = getLocalForceEntity(getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE));

			notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, force, en, EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO);
		} else if (raw.supplies.fuel_supply_level < 100.0) {
			const force = getLocalForceEntity(getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE));

			notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, force, en, EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_FUEL);
		}
	} else if (getLocalEntityIntValue(en, IntType.INT_TYPE_RESUPPLY_SOURCE) === ResupplySourceType.RESUPPLY_SOURCE_KEYSITE) {
		//
		// check all group is landed. If so refuel/rearm group via keysite supplies
		//

		if (getLocalEntityIntValue(en, IntType.INT_TYPE_GROUP_MODE) === GroupModeType.GROUP_MODE_IDLE) {
			if (raw.supplies.ammo_supply_level < 100.0) {
				//
				// re-arm
				//

				let keysite = getLocalEntityParent(en, ListType.LIST_TYPE_KEYSITE_GROUP);

				if (!keysite || getLocalEntityType(keysite) !== EntityType.ENTITY_TYPE_KEYSITE) {
					keysite = getClosestKeysite(
						EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES,
						raw.side,
						getLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION),
						1.0 * KILOMETRE,
						undefined,
						true,
						undefined,
					);
				}

				ASSERT(keysite !== undefined, "keysite");

				let level = getLocalEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL);

				let required = toFloat32(100.0 - raw.supplies.ammo_supply_level * AMMO_USAGE_ACCELERATOR);

				required = toFloat32(bound(required, 0.0, level));

				level = toFloat32(level - required);

				setClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, level);

				setClientServerEntityFloatValue(en, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, raw.supplies.ammo_supply_level + required);
			}

			if (raw.supplies.fuel_supply_level < 100.0) {
				//
				// re-fuel
				//

				let keysite = getLocalEntityParent(en, ListType.LIST_TYPE_KEYSITE_GROUP);

				if (!keysite || getLocalEntityType(keysite) !== EntityType.ENTITY_TYPE_KEYSITE) {
					keysite = getClosestKeysite(
						EntitySubTypeKeysite.NUM_ENTITY_SUB_TYPE_KEYSITES,
						raw.side,
						getLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION),
						1.0 * KILOMETRE,
						undefined,
						true,
						undefined,
					);
				}

				ASSERT(keysite !== undefined, "keysite");

				let level = getLocalEntityFloatValue(keysite, FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL);

				let required = toFloat32(100.0 - raw.supplies.fuel_supply_level * FUEL_USAGE_ACCELERATOR);

				required = toFloat32(bound(required, 0.0, level));

				level = toFloat32(level - required);

				setClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL, level);

				setClientServerEntityFloatValue(en, FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL, raw.supplies.fuel_supply_level + required);
			}
		}
	}
}

export function overloadGroupFunctions(): void {
	const GROUP = EntityType.ENTITY_TYPE_GROUP;

	// C provenance: gp_list.c :: LIST_TYPE_MEMBER_ROOT, LIST_TYPE_GUIDE_STACK_ROOT, LIST_TYPE_GROUP_LINK
	//               en_list/get_prnt.h :: LIST_TYPE_GROUP_LINK serves BUILDING_GROUP, INDEPENDENT_GROUP and KEYSITE_GROUP
	overloadEntityListRoot(GROUP, "member_root", [ListType.LIST_TYPE_MEMBER]);
	overloadEntityListRoot(GROUP, "guide_stack_root", [ListType.LIST_TYPE_GUIDE_STACK]);
	overloadEntityListLink(GROUP, "group_link", [ListType.LIST_TYPE_BUILDING_GROUP, ListType.LIST_TYPE_INDEPENDENT_GROUP, ListType.LIST_TYPE_KEYSITE_GROUP]);

	// C provenance: gp_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_GROUP_MODE, (en) =>
		getLocalEntityFirstChild(en, ListType.LIST_TYPE_GUIDE_STACK) ? GroupModeType.GROUP_MODE_BUSY : GroupModeType.GROUP_MODE_IDLE,
	);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_RESUPPLY_SOURCE, (en) => GROUP_DATABASE_RESUPPLY_SOURCE[getLocalEntityData<GroupRaw>(en).sub_type]);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_SIDE, (en) => getLocalEntityData<GroupRaw>(en).side);

	// C provenance: gp_float.c :: set_local_float_value, set_server_float_value
	const server = fnSetClientServerEntityFloatValue[CommsModelType.COMMS_MODEL_SERVER];

	server.overload(
		GROUP,
		FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL,
		serverFloatValueSetter((en, _type, value) => {
			getLocalEntityData<GroupRaw>(en).supplies.ammo_supply_level = value;
		}),
	);

	server.overload(
		GROUP,
		FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL,
		serverFloatValueSetter((en, _type, value) => {
			getLocalEntityData<GroupRaw>(en).supplies.fuel_supply_level = value;
		}),
	);

	// C provenance: gp_ptr.c :: get_local_ptr_value (PTR_TYPE_GROUP_LEADER)
	fnGetLocalEntityPtrValue.overload(GROUP, PtrType.PTR_TYPE_GROUP_LEADER, (en) => getLocalEntityFirstChild(en, ListType.LIST_TYPE_MEMBER));

	// C provenance: gp_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)
	fnGetLocalEntityVec3dPtr.overload(GROUP, Vec3dType.VEC3D_TYPE_POSITION, (en, type) => {
		const member = getLocalEntityPtrValue(en, PtrType.PTR_TYPE_GROUP_LEADER);

		return member ? getLocalEntityVec3dPtr(member, type) : undefined;
	});
}

//
// Group entity.
//
// C provenance: entity/special/group/group.c, gp_int.c, gp_float.c, gp_vec3d.c,
//               gp_ptr.c, gp_list.c, gp_dbase.c, gp_updt.c, gp_msgs.c
//

import { ASSERT } from "../../../core/assert";
import { f32Add, f32Div, f32Sub, toFloat32RTZ } from "../../../core/float32";
import { getApprox2dRange } from "../../../core/maths/range";
import { UnportedBehaviourError } from "../../../core/assert";
import { bound, KILOMETRE, max } from "../../../core/maths/miscmath";
import { getDeltaTime } from "../../../core/time";
import { AMMO_USAGE_ACCELERATOR, FUEL_USAGE_ACCELERATOR } from "../../../generated/c-constants";
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
import {
	deleteLocalEntityFromParentsChildList,
	getLocalEntityFirstChild,
	getLocalEntityParent,
	insertLocalEntityIntoParentsChildList,
	overloadEntityListLink,
	overloadEntityListRoot,
} from "../../system/en_list";
import { messageResponses, notifyLocalEntity, type MessageResponseFn } from "../../system/en_msgs";
import { fnUpdateClientServerEntity } from "../../system/en_updt";
import {
	defaultSetEntityIntValue,
	fnGetLocalEntityFloatValue,
	fnGetLocalEntityIntValue,
	fnSetLocalEntityIntValue,
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
import { getUpdateEntity } from "../update/update";

// C provenance: group.h :: struct GROUP (ported fields only)
export interface GroupRaw {
	sub_type: EntitySubTypeGroup;
	side: EntitySide;
	// unsigned int alive : NUM_ALIVE_BITS (1)
	alive: number;
	supplies: SupplyRaw;
	sleep: number;
	assist_timer: number;
	// unsigned int member_count : NUM_MEMBER_COUNT_BITS (6). Slice 6a reads it; its live
	// maintenance (gp_msgs.c :: response_to_link_child, LIST_TYPE_MEMBER) is not ported.
	member_count: number;
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

				// 100.0 - (level * AMMO_USAGE_ACCELERATOR): double arithmetic stored as float
				let required = f32Add(100.0, -(raw.supplies.ammo_supply_level * AMMO_USAGE_ACCELERATOR));

				required = toFloat32RTZ(bound(required, 0.0, level));

				level = f32Sub(level, required);

				setClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, level);

				setClientServerEntityFloatValue(en, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, f32Add(raw.supplies.ammo_supply_level, required));
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

				// 100.0 - (level * FUEL_USAGE_ACCELERATOR): double arithmetic stored as float
				let required = f32Add(100.0, -(raw.supplies.fuel_supply_level * FUEL_USAGE_ACCELERATOR));

				required = toFloat32RTZ(bound(required, 0.0, level));

				level = f32Sub(level, required);

				setClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL, level);

				setClientServerEntityFloatValue(en, FloatType.FLOAT_TYPE_FUEL_SUPPLY_LEVEL, f32Add(raw.supplies.fuel_supply_level, required));
			}
		}
	}
}

//
// C provenance: group.c :: assess_group_task_locality_factor (group_en, task_en, return_distance)
//
// Slice 6a (issue #16): whether the group's first member could reach the
// task's keysite before the task expires, at its cruise velocity. The
// distance is returned through `result.distance` (C: *return_distance).
// debug_log calls are guarded by DEBUG_MODULE (0) and are not ported.
//
export function assessGroupTaskLocalityFactor(group_en: Entity, task_en: Entity, return_distance?: { distance: number }): boolean {
	//
	// locality
	// assess if group could get to end position quick enough, using first members cruise speed.
	//

	if (return_distance) {
		return_distance.distance = 0.0;
	}

	const member = getLocalEntityFirstChild(group_en, ListType.LIST_TYPE_MEMBER);

	// Unreachable through the only caller ported (assign.c :: get_suitable_registered_group):
	// a group's member_count counts its LIST_TYPE_MEMBER list, and every task's minimum member
	// count is at least 1 (test/unit/supply-task-assignment.test.ts), so a group without members
	// is rejected before its locality is assessed.
	/* istanbul ignore if */
	if (!member) {
		return false;
	}

	const pos1 = getLocalEntityVec3dPtr(member, Vec3dType.VEC3D_TYPE_POSITION);

	// get task start position, either keysite it is assigned to or the tasks start pos (NB. might actually be its end point)

	const keysite = getLocalEntityParent(task_en, ListType.LIST_TYPE_UNASSIGNED_TASK);

	let pos2;

	// The else arm is unreachable through the only caller ported (get_suitable_registered_group),
	// which takes the task from a keysite's LIST_TYPE_UNASSIGNED_TASK list. Its START_POSITION
	// row is not ported and would fail loudly.
	/* istanbul ignore else */
	if (keysite && getLocalEntityType(keysite) === EntityType.ENTITY_TYPE_KEYSITE) {
		pos2 = getLocalEntityVec3dPtr(keysite, Vec3dType.VEC3D_TYPE_POSITION);
	} else {
		pos2 = getLocalEntityVec3dPtr(task_en, Vec3dType.VEC3D_TYPE_START_POSITION);
	}

	const distance = getApprox2dRange(pos1, pos2);

	const cruise_speed = getLocalEntityFloatValue(member, FloatType.FLOAT_TYPE_CRUISE_VELOCITY);

	const eta = f32Div(distance, cruise_speed);

	if (eta > getLocalEntityFloatValue(task_en, FloatType.FLOAT_TYPE_EXPIRE_TIMER)) {
		return false;
	}

	if (return_distance) {
		return_distance.distance = distance;
	}

	return true;
}

// C provenance: gp_updt.c :: update_server
function updateServer(en: Entity): void {
	const raw = getLocalEntityData<GroupRaw>(en);

	//
	// Group is only on update list when its sleeping or under attack
	//

	if (raw.sleep > 0.0) {
		raw.sleep = f32Sub(raw.sleep, getDeltaTime());

		raw.sleep = max(raw.sleep, 0.0);
	}

	if (raw.assist_timer > 0.0) {
		raw.assist_timer = f32Sub(raw.assist_timer, getDeltaTime());

		raw.assist_timer = max(raw.assist_timer, 0.0);
	}

	if (raw.sleep === 0.0 && raw.assist_timer === 0.0) {
		deleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_UPDATE);
	}
}

// C provenance: gp_float.c :: set_local_float_value (FLOAT_TYPE_SLEEP, FLOAT_TYPE_ASSIST_TIMER)
function setLocalTimerValue(en: Entity, type: FloatType, value: number): void {
	const raw = getLocalEntityData<GroupRaw>(en);

	if (type === FloatType.FLOAT_TYPE_SLEEP) {
		raw.sleep = value;
	} else {
		raw.assist_timer = value;
	}

	if (value !== 0.0 && !getLocalEntityParent(en, ListType.LIST_TYPE_UPDATE)) {
		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_UPDATE, getUpdateEntity(), undefined);
	}
}

// C provenance: gp_msgs.c :: response_to_link_parent
const responseToLinkParent: MessageResponseFn = (_message, _receiver, _sender, args) => {
	if ((args[0] as ListType) === ListType.LIST_TYPE_DIVISION) {
		throw new UnportedBehaviourError("gp_msgs.c :: response_to_link_parent (LIST_TYPE_DIVISION) -> set_local_division_name");
	}

	return 1;
};

//
// C provenance: gp_msgs.c :: response_to_link_child
//
// Only LIST_TYPE_MEMBER has an arm (member counts, keysite importance, the
// campaign screen); every other list, such as a task joining the group's
// LIST_TYPE_TASK_DEPENDENT list, returns TRUE.
//
const responseToLinkChild: MessageResponseFn = (_message, _receiver, _sender, args) => {
	if ((args[0] as ListType) === ListType.LIST_TYPE_MEMBER) {
		throw new UnportedBehaviourError("gp_msgs.c :: response_to_link_child (LIST_TYPE_MEMBER)");
	}

	return 1;
};

// C provenance: gp_msgs.c :: response_to_unlink_parent (only debug_log_entity_message)
const responseToUnlinkParent: MessageResponseFn = () => 1;

export function overloadGroupFunctions(): void {
	const GROUP = EntityType.ENTITY_TYPE_GROUP;

	// C provenance: gp_list.c :: LIST_TYPE_MEMBER_ROOT, LIST_TYPE_GUIDE_STACK_ROOT, LIST_TYPE_GROUP_LINK
	//               en_list/get_prnt.h :: LIST_TYPE_GROUP_LINK serves BUILDING_GROUP, INDEPENDENT_GROUP and KEYSITE_GROUP
	overloadEntityListRoot(GROUP, "member_root", [ListType.LIST_TYPE_MEMBER]);
	overloadEntityListRoot(GROUP, "guide_stack_root", [ListType.LIST_TYPE_GUIDE_STACK]);
	overloadEntityListRoot(GROUP, "task_dependent_root", [ListType.LIST_TYPE_TASK_DEPENDENT]);
	overloadEntityListLink(GROUP, "group_link", [ListType.LIST_TYPE_BUILDING_GROUP, ListType.LIST_TYPE_INDEPENDENT_GROUP, ListType.LIST_TYPE_KEYSITE_GROUP]);
	overloadEntityListLink(GROUP, "update_link", [ListType.LIST_TYPE_UPDATE]);
	// C provenance: gp_list.c :: LIST_TYPE_PILOT_LOCK_LINK, LIST_TYPE_REGISTRY_LINK
	//               (fn_list.h: the registry link serves the air, ground and sea registries)
	overloadEntityListLink(GROUP, "pilot_lock_link", [ListType.LIST_TYPE_PILOT_LOCK]);
	overloadEntityListLink(GROUP, "registry_link", [ListType.LIST_TYPE_AIR_REGISTRY, ListType.LIST_TYPE_GROUND_REGISTRY, ListType.LIST_TYPE_SEA_REGISTRY]);

	// C provenance: gp_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_GROUP_MODE, (en) =>
		getLocalEntityFirstChild(en, ListType.LIST_TYPE_GUIDE_STACK) ? GroupModeType.GROUP_MODE_BUSY : GroupModeType.GROUP_MODE_IDLE,
	);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_RESUPPLY_SOURCE, (en) => GROUP_DATABASE_RESUPPLY_SOURCE[getLocalEntityData<GroupRaw>(en).sub_type]);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_SIDE, (en) => getLocalEntityData<GroupRaw>(en).side);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<GroupRaw>(en).sub_type);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_ALIVE, (en) => getLocalEntityData<GroupRaw>(en).alive);
	fnGetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_MEMBER_COUNT, (en) => getLocalEntityData<GroupRaw>(en).member_count);

	// C provenance: gp_int.c does not overload INT_TYPE_UPDATED; en_int.c's default setter applies
	fnSetLocalEntityIntValue.overload(GROUP, IntType.INT_TYPE_UPDATED, defaultSetEntityIntValue);

	// C provenance: gp_float.c :: get_local_float_value
	fnGetLocalEntityFloatValue.overload(GROUP, FloatType.FLOAT_TYPE_SLEEP, (en) => getLocalEntityData<GroupRaw>(en).sleep);
	fnGetLocalEntityFloatValue.overload(GROUP, FloatType.FLOAT_TYPE_ASSIST_TIMER, (en) => getLocalEntityData<GroupRaw>(en).assist_timer);

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

	server.overload(GROUP, FloatType.FLOAT_TYPE_SLEEP, serverFloatValueSetter(setLocalTimerValue));

	server.overload(GROUP, FloatType.FLOAT_TYPE_ASSIST_TIMER, serverFloatValueSetter(setLocalTimerValue));

	// C provenance: gp_updt.c :: overload_group_update_functions
	fnUpdateClientServerEntity.overload(GROUP, CommsModelType.COMMS_MODEL_SERVER, updateServer);

	// C provenance: gp_msgs.c :: overload_group_message_responses (LINK_CHILD, LINK_PARENT, UNLINK_PARENT only)
	messageResponses.overload(GROUP, EntityMessage.ENTITY_MESSAGE_LINK_CHILD, responseToLinkChild);
	messageResponses.overload(GROUP, EntityMessage.ENTITY_MESSAGE_LINK_PARENT, responseToLinkParent);
	messageResponses.overload(GROUP, EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, responseToUnlinkParent);

	// C provenance: gp_ptr.c :: get_local_ptr_value (PTR_TYPE_GROUP_LEADER)
	fnGetLocalEntityPtrValue.overload(GROUP, PtrType.PTR_TYPE_GROUP_LEADER, (en) => getLocalEntityFirstChild(en, ListType.LIST_TYPE_MEMBER));

	// C provenance: gp_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)
	fnGetLocalEntityVec3dPtr.overload(GROUP, Vec3dType.VEC3D_TYPE_POSITION, (en, type) => {
		const member = getLocalEntityPtrValue(en, PtrType.PTR_TYPE_GROUP_LEADER);

		return member ? getLocalEntityVec3dPtr(member, type) : undefined;
	});
}

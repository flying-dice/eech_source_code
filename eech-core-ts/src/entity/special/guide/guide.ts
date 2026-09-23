//
// Guide entity: a task on a group's guide stack, steering the group's
// members from waypoint to waypoint.
//
// C provenance: entity/special/guide/guide.h (struct GUIDE), guide.c ::
//               create_client_server_guide_entity, attach_group_to_guide_entity,
//               initialise_guide_criteria, set_client_server_guide_criteria_valid,
//               set_local_guide_criteria_valid, get_guide_criteria_valid;
//               gd_creat.c, gd_list.c, gd_int.c, gd_float.c, gd_vec3d.c, gd_msgs.c
//
// Slice 6b (issue #18) ports what assign.c :: push_task_onto_group_task_stack
// does with a guide: its creation (task LIST_TYPE_GUIDE, the task's first
// waypoint's LIST_TYPE_CURRENT_WAYPOINT, the update list; then
// ENTITY_COMMS_CREATE), its place at the head of the group's guide stack
// (ENTITY_COMMS_SWITCH_LIST), its criteria from the guide and waypoint
// databases (ENTITY_COMMS_SET_GUIDE_CRITERIA for each change) and its
// velocity (ENTITY_COMMS_FLOAT_VALUE). Guide execution (gd_updt.c, gd_nav.c,
// the attack guides, waypoint reached and action handling) runs later from the
// update list and is not ported; neither is the follower list, which the
// member assignment (the slice 6b boundary) fills.
//
// The guide's link responses are overloaded only under DEBUG_MODULE, so
// en_msgs.c's default applies to every one of them.
//

import { ASSERT } from "../../../core/assert";
import { toCUnsignedInt, unsignedToCInt } from "../../../core/cint";
import { f32Mul } from "../../../core/float32";
import type { Vec3d } from "../../../core/maths/vec3d";
import { GUIDE_DATABASE_CRITERIA } from "../../../generated/c-guide-database";
import { GROUP_DATABASE_DEFAULT_ENTITY_TYPE } from "../../../generated/c-group-database";
import {
	CommsModelType,
	EntityMessage,
	EntitySubTypeGuide,
	EntityType,
	FloatType,
	GuideCriteriaType,
	IntType,
	ListType,
	Vec3dType,
} from "../../../generated/c-enums";
import { getCommsModel } from "../../system/comms";
import { replicatedEntityAttributes, setLocalEntityAttributes, type EntityAttribute } from "../../system/en_attrs";
import { transmitEntityCreate, transmitSetGuideCriteria, transmitSwitchList } from "../../system/en_comms";
import { createClientServerEntity, fnCreateClientServerEntity, fnCreateLocalEntity, validateLocalCreateEntityIndex, validateRemoteCreateEntityIndex } from "../../system/en_creat";
import { ENTITY_INDEX_DONT_CARE, getFreeEntity } from "../../system/en_heap";
import { getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildList, overloadEntityListLink, overloadEntityListRoot } from "../../system/en_list";
import { defaultMessageResponse, messageResponses } from "../../system/en_msgs";
import {
	fnGetLocalEntityFloatValue,
	fnGetLocalEntityIntValue,
	fnGetLocalEntityVec3dPtr,
	fnSetClientServerEntityFloatValue,
	fnSetLocalEntityFloatValue,
	fnSetLocalEntityIntValue,
	fnSetLocalEntityRawFloatValue,
	fnSetLocalEntityRawIntValue,
	fnSetLocalEntityRawVec3d,
	getLocalEntityFloatValue,
	getLocalEntityIntValue,
	getLocalEntityVec3dPtr,
	serverFloatValueSetter,
	setClientServerEntityFloatValue,
	type SetFloatValueFn,
	type SetIntValueFn,
} from "../../system/en_values";
import { getLocalEntityData, setLocalEntityData, setLocalEntityType, type Entity } from "../../system/entity";
import { getUpdateEntity } from "../update/update";
import { getWaypointDatabaseLastToReachFlag, getWaypointDatabaseReachedRadiusValue, getWaypointDatabaseTransmitReconFlag, getWaypointDatabaseVelocityValue } from "../waypoint/wp_dbase";

// C provenance: guide.h :: struct GUIDE_CRITERIA_ELEMENT
export interface GuideCriteriaElement {
	valid: number;
	value: number;
}

// C provenance: guide.h :: struct GUIDE (the fields the port reads or writes)
export interface GuideRaw {
	sub_type: EntitySubTypeGuide;
	position: Vec3d;
	valid_guide_members: number;
	velocity: number;
	criteria: GuideCriteriaElement[];
}

// A guide's raw data as memset (raw, 0, sizeof (guide)) leaves it.
export function clearedGuideRaw(): GuideRaw {
	const criteria: GuideCriteriaElement[] = [];

	for (let loop = 0; loop < GuideCriteriaType.NUM_GUIDE_CRITERIA_TYPES; loop++) {
		criteria.push({ valid: 0, value: 0.0 });
	}

	return { sub_type: 0, position: { x: 0.0, y: 0.0, z: 0.0 }, valid_guide_members: 0, velocity: 0.0, criteria };
}

// C provenance: gd_creat.c :: create_local
function createLocal(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateLocalCreateEntityIndex(index);

	const en = getFreeEntity(index);

	if (en) {
		setLocalEntityType(en, type);

		// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES (after memset (raw, 0, sizeof (guide)))
		const raw = clearedGuideRaw();

		raw.sub_type = EntitySubTypeGuide.ENTITY_SUB_TYPE_GUIDE_NAVIGATION_DIRECT;

		setLocalEntityData(en, raw);

		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		setLocalEntityAttributes(en, attributes);

		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		const task = getLocalEntityParent(en, ListType.LIST_TYPE_GUIDE);

		ASSERT(task !== undefined, "raw->guide_link.parent");

		const wp = getLocalEntityParent(en, ListType.LIST_TYPE_CURRENT_WAYPOINT);

		ASSERT(wp !== undefined, "raw->current_waypoint_link.parent");

		// LINK INTO SYSTEM
		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_GUIDE, task, undefined);

		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_CURRENT_WAYPOINT, wp, undefined);

		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_UPDATE, getUpdateEntity(), undefined);
	}

	return en;
}

// C provenance: gd_creat.c :: create_remote
function createRemote(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateRemoteCreateEntityIndex(index);

	transmitEntityCreate(type, index, replicatedEntityAttributes(attributes));

	return undefined;
}

// C provenance: gd_creat.c :: create_server
function createServer(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	const en = createLocal(type, index, attributes);

	if (en) {
		createRemote(type, en.index, attributes);
	}

	return en;
}

//
// C provenance: guide.c :: create_client_server_guide_entity (task_en, first_waypoint, valid_members)
//
// valid_members is an unsigned int; ENTITY_ATTR_INT_VALUE passes it as an int.
//
export function createClientServerGuideEntity(task_en: Entity | undefined, first_waypoint: Entity | undefined, valid_members: number): Entity | undefined {
	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	ASSERT(task_en !== undefined, "task_en");

	let wp: Entity | undefined;

	if (first_waypoint) {
		// first waypoint has been specified - but check it actually belongs to the task...
		wp = first_waypoint;

		ASSERT(getLocalEntityParent(wp, ListType.LIST_TYPE_WAYPOINT) === task_en, "get_local_entity_parent (wp, LIST_TYPE_WAYPOINT) == task_en");
	} else {
		// get first waypoint of the task
		wp = getLocalEntityFirstChild(task_en, ListType.LIST_TYPE_WAYPOINT);
	}

	ASSERT(wp !== undefined, "wp");

	// set sub type (must be done before set position)
	const guide_type = getLocalEntityIntValue(wp, IntType.INT_TYPE_WAYPOINT_GUIDE_TYPE);

	// get waypoint position
	const pos = getLocalEntityVec3dPtr(wp, Vec3dType.VEC3D_TYPE_POSITION);

	ASSERT(pos !== undefined, "pos");

	return createClientServerEntity(EntityType.ENTITY_TYPE_GUIDE, ENTITY_INDEX_DONT_CARE, [
		{ kind: "parent", type: ListType.LIST_TYPE_GUIDE, entity: task_en },
		{ kind: "parent", type: ListType.LIST_TYPE_CURRENT_WAYPOINT, entity: wp },
		{ kind: "int_value", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: guide_type },
		{ kind: "int_value", type: IntType.INT_TYPE_VALID_GUIDE_MEMBERS, value: unsignedToCInt(valid_members) },
		{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x: pos.x, y: pos.y, z: pos.z },
	]);
}

// C provenance: guide.c :: attach_group_to_guide_entity
export function attachGroupToGuideEntity(group: Entity | undefined, guide: Entity | undefined): void {
	ASSERT(group !== undefined, "group");
	ASSERT(guide !== undefined, "guide");

	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	// guide stack
	ASSERT(getLocalEntityParent(guide, ListType.LIST_TYPE_GUIDE_STACK) === undefined, "get_local_entity_parent (guide, LIST_TYPE_GUIDE_STACK) == NULL");

	insertLocalEntityIntoParentsChildList(guide, ListType.LIST_TYPE_GUIDE_STACK, group, undefined);

	transmitSwitchList(guide, ListType.LIST_TYPE_GUIDE_STACK, group, ListType.LIST_TYPE_GUIDE_STACK);

	// set-up guide criteria relevant to the group type
	initialiseGuideCriteria(guide);

	// Set guide velocity with respect to the group type
	const wp = getLocalEntityParent(guide, ListType.LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT(wp !== undefined, "wp");

	const member = getLocalEntityFirstChild(group, ListType.LIST_TYPE_MEMBER);

	ASSERT(member !== undefined, "member");

	const group_type = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const waypoint_type = getLocalEntityIntValue(wp, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	let velocity = getWaypointDatabaseVelocityValue(waypoint_type, GROUP_DATABASE_DEFAULT_ENTITY_TYPE[group_type]);

	// velocity *= cruise velocity: a float product
	velocity = f32Mul(velocity, getLocalEntityFloatValue(member, FloatType.FLOAT_TYPE_CRUISE_VELOCITY));

	setClientServerEntityFloatValue(guide, FloatType.FLOAT_TYPE_VELOCITY, velocity);
}

// C provenance: guide.c :: set_local_guide_criteria_valid
function setLocalGuideCriteriaValid(en: Entity, type: GuideCriteriaType, valid: number, value: number): void {
	ASSERT(type < GuideCriteriaType.NUM_GUIDE_CRITERIA_TYPES, "type < NUM_GUIDE_CRITERIA_TYPES");

	const raw = getLocalEntityData<GuideRaw>(en);

	raw.criteria[type].valid = valid;
	raw.criteria[type].value = value;
}

// C provenance: guide.c :: set_client_server_guide_criteria_valid (value: a float parameter)
export function setClientServerGuideCriteriaValid(en: Entity, type: GuideCriteriaType, valid: number, value: number): void {
	ASSERT(type < GuideCriteriaType.NUM_GUIDE_CRITERIA_TYPES, "type < NUM_GUIDE_CRITERIA_TYPES");

	const raw = getLocalEntityData<GuideRaw>(en);

	if (raw.criteria[type].valid !== valid || raw.criteria[type].value !== value) {
		setLocalGuideCriteriaValid(en, type, valid, value);

		// transmit comms message
		transmitSetGuideCriteria(en, type, valid, value);
	}
}

// C provenance: guide.c :: get_guide_criteria_valid
export function getGuideCriteriaValid(en: Entity, type: GuideCriteriaType): number {
	ASSERT(type < GuideCriteriaType.NUM_GUIDE_CRITERIA_TYPES, "type < NUM_GUIDE_CRITERIA_TYPES");

	return getLocalEntityData<GuideRaw>(en).criteria[type].valid;
}

// C provenance: guide.c :: initialise_guide_criteria
export function initialiseGuideCriteria(en: Entity): void {
	ASSERT(getCommsModel() === CommsModelType.COMMS_MODEL_SERVER, "get_comms_model () == COMMS_MODEL_SERVER");

	const guide_type = getLocalEntityIntValue(en, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	// get current waypoint
	const current_wp = getLocalEntityParent(en, ListType.LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT(current_wp !== undefined, "current_wp");

	const waypoint_type = getLocalEntityIntValue(current_wp, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	// get follower type
	const group = getLocalEntityParent(en, ListType.LIST_TYPE_GUIDE_STACK);

	ASSERT(group !== undefined, "group");

	const group_type = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const mobile_type = GROUP_DATABASE_DEFAULT_ENTITY_TYPE[group_type];

	// get values from GUIDE DATABASE
	for (let loop = 0; loop < GuideCriteriaType.NUM_GUIDE_CRITERIA_TYPES; loop++) {
		const criterion = GUIDE_DATABASE_CRITERIA[guide_type][loop];

		if (criterion[0] !== 0) {
			setClientServerGuideCriteriaValid(en, loop, 1, criterion[1]);
		} else {
			if (getGuideCriteriaValid(en, loop) !== 0) {
				setClientServerGuideCriteriaValid(en, loop, 0, 0.0);
			}
		}
	}

	// get values from WAYPOINT DATABASE
	setClientServerGuideCriteriaValid(en, GuideCriteriaType.GUIDE_CRITERIA_RADIUS, 1, getWaypointDatabaseReachedRadiusValue(waypoint_type, mobile_type));

	setClientServerGuideCriteriaValid(en, GuideCriteriaType.GUIDE_CRITERIA_TRANSMIT_DATA, getWaypointDatabaseTransmitReconFlag(waypoint_type, mobile_type), 0.0);

	setClientServerGuideCriteriaValid(en, GuideCriteriaType.GUIDE_CRITERIA_LAST_TO_REACH, getWaypointDatabaseLastToReachFlag(waypoint_type, mobile_type), 0.0);
}

// C provenance: gd_int.c :: set_local_int_value (INT_TYPE_ENTITY_SUB_TYPE, INT_TYPE_VALID_GUIDE_MEMBERS)
const setLocalIntValue: SetIntValueFn = (en, type, value) => {
	const raw = getLocalEntityData<GuideRaw>(en);

	if (type === IntType.INT_TYPE_ENTITY_SUB_TYPE) {
		raw.sub_type = value;
	} else {
		// raw->valid_guide_members = value: an unsigned int
		raw.valid_guide_members = toCUnsignedInt(value);
	}
};

// C provenance: gd_float.c :: set_local_float_value (FLOAT_TYPE_VELOCITY)
const setLocalFloatValue: SetFloatValueFn = (en, _type, value) => {
	getLocalEntityData<GuideRaw>(en).velocity = value;
};

// C provenance: gd_funcs.c :: overload_guide_functions (ported subset)
export function overloadGuideFunctions(): void {
	const GUIDE = EntityType.ENTITY_TYPE_GUIDE;

	// C provenance: gd_list.c :: LIST_TYPE_FOLLOWER_ROOT, LIST_TYPE_CURRENT_WAYPOINT_LINK, LIST_TYPE_GUIDE_LINK,
	//               LIST_TYPE_GUIDE_STACK_LINK, LIST_TYPE_UPDATE_LINK
	overloadEntityListRoot(GUIDE, "follower_root", [ListType.LIST_TYPE_FOLLOWER]);
	overloadEntityListLink(GUIDE, "current_waypoint_link", [ListType.LIST_TYPE_CURRENT_WAYPOINT]);
	overloadEntityListLink(GUIDE, "guide_link", [ListType.LIST_TYPE_GUIDE]);
	overloadEntityListLink(GUIDE, "guide_stack_link", [ListType.LIST_TYPE_GUIDE_STACK]);
	overloadEntityListLink(GUIDE, "update_link", [ListType.LIST_TYPE_UPDATE]);

	// C provenance: gd_creat.c :: overload_guide_create_functions (server model)
	fnCreateLocalEntity.overload(GUIDE, 0, createLocal);
	fnCreateClientServerEntity.overload(GUIDE, CommsModelType.COMMS_MODEL_SERVER, createServer);

	// C provenance: gd_int.c :: get_local_int_value, set_local_int_value (raw and local)
	fnGetLocalEntityIntValue.overload(GUIDE, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<GuideRaw>(en).sub_type);
	fnGetLocalEntityIntValue.overload(GUIDE, IntType.INT_TYPE_VALID_GUIDE_MEMBERS, (en) => getLocalEntityData<GuideRaw>(en).valid_guide_members);

	for (const type of [IntType.INT_TYPE_ENTITY_SUB_TYPE, IntType.INT_TYPE_VALID_GUIDE_MEMBERS]) {
		fnSetLocalEntityRawIntValue.overload(GUIDE, type, setLocalIntValue);
		fnSetLocalEntityIntValue.overload(GUIDE, type, setLocalIntValue);
	}

	// C provenance: gd_float.c :: get_local_float_value, set_local_float_value, set_server_float_value (FLOAT_TYPE_VELOCITY)
	fnGetLocalEntityFloatValue.overload(GUIDE, FloatType.FLOAT_TYPE_VELOCITY, (en) => getLocalEntityData<GuideRaw>(en).velocity);
	fnSetLocalEntityRawFloatValue.overload(GUIDE, FloatType.FLOAT_TYPE_VELOCITY, setLocalFloatValue);
	fnSetLocalEntityFloatValue.overload(GUIDE, FloatType.FLOAT_TYPE_VELOCITY, setLocalFloatValue);
	fnSetClientServerEntityFloatValue[CommsModelType.COMMS_MODEL_SERVER].overload(GUIDE, FloatType.FLOAT_TYPE_VELOCITY, serverFloatValueSetter(setLocalFloatValue));

	// C provenance: gd_vec3d.c :: set_local_vec3d (raw), get_local_vec3d_ptr (VEC3D_TYPE_POSITION)
	fnSetLocalEntityRawVec3d.overload(GUIDE, Vec3dType.VEC3D_TYPE_POSITION, (en, _type, v) => {
		const position = getLocalEntityData<GuideRaw>(en).position;

		position.x = v.x;
		position.y = v.y;
		position.z = v.z;
	});
	fnGetLocalEntityVec3dPtr.overload(GUIDE, Vec3dType.VEC3D_TYPE_POSITION, (en) => getLocalEntityData<GuideRaw>(en).position);

	// C provenance: gd_msgs.c :: the link responses are overloaded only under DEBUG_MODULE: en_msgs.c's default
	for (const message of [
		EntityMessage.ENTITY_MESSAGE_LINK_CHILD,
		EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD,
		EntityMessage.ENTITY_MESSAGE_LINK_PARENT,
		EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT,
	]) {
		messageResponses.overload(GUIDE, message, defaultMessageResponse);
	}
}

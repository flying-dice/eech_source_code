//
// Waypoint entity: a task's route point.
//
// C provenance: entity/special/waypoint/waypoint.h (struct WAYPOINT), wp_list.c,
//               wp_int.c, wp_float.c, wp_vec3d.c, wp_char.c, wp_msgs.c (and en_float.c's
//               default getter, which wp_float.c leaves in place for FLOAT_TYPE_TASK_USER_DATA)
//
// croute.c links each route waypoint into its route dependent's
// LIST_TYPE_TASK_DEPENDENT list, beside the tasks whose objective that entity
// is. response_to_force_low_on_supplies walks that list; entity_is_object_of_task
// skips anything that is not a task, but the walk after it does not.
//
// Slice 6b (issue #18): create_generic_waypoint_route creates waypoints
// (wp_creat.ts), sets their types, formations and positions, and reads them
// back. Joining or leaving a task's LIST_TYPE_WAYPOINT list, and the local
// sub type setter, re-tag the whole list (wp_char.ts); the raw setter does
// not. Waypoints join no sector list: wp_creat.c does that only under
// DEBUG_MODULE.
//
// A VIRTUAL waypoint's position is its dependent's (wp_vec3d.c); no waypoint
// type a supply route uses is VIRTUAL for any mobile, so that arm is not
// ported and fails loudly.
//

import { ASSERT, UnportedBehaviourError } from "../../../core/assert";
import { storeUnsignedBitfield } from "../../../core/cint";
import type { Vec3d } from "../../../core/maths/vec3d";
import { NUM_POSITION_TYPE_BITS, NUM_ROUTE_NODE_BITS, NUM_TAG_BITS, NUM_WAYPOINT_FORMATION_BITS } from "../../../generated/c-constants";
import { WAYPOINT_DATABASE_GUIDE_SUB_TYPE } from "../../../generated/c-waypoint-database";
import {
	CharType,
	EntityMessage,
	EntitySubTypeWaypoint,
	EntityType,
	FloatType,
	FormationType,
	IntType,
	ListType,
	PositionType,
	Vec3dType,
} from "../../../generated/c-enums";
import { getLocalEntityParent, overloadEntityListLink, overloadEntityListRoot } from "../../system/en_list";
import { defaultMessageResponse, messageResponses, type MessageResponseFn } from "../../system/en_msgs";
import {
	defaultGetEntityFloatValue,
	fnGetLocalEntityCharValue,
	fnGetLocalEntityFloatValue,
	fnGetLocalEntityIntValue,
	fnGetLocalEntityVec3d,
	fnGetLocalEntityVec3dPtr,
	fnSetLocalEntityCharValue,
	fnSetLocalEntityFloatValue,
	fnSetLocalEntityIntValue,
	fnSetLocalEntityRawFloatValue,
	fnSetLocalEntityRawIntValue,
	fnSetLocalEntityRawVec3d,
	fnSetLocalEntityVec3d,
	type SetFloatValueFn,
	type SetIntValueFn,
} from "../../system/en_values";
import { getLocalEntityData, type Entity } from "../../system/entity";
import { updateLocalEntityWaypointListTags } from "./wp_char";


// C provenance: waypoint.h :: struct WAYPOINT (the fields the port reads or writes)
export interface WaypointRaw {
	sub_type: EntitySubTypeWaypoint;
	flight_time: number;
	altitude: number;
	heading: number;
	// unsigned int bit-fields
	position_type: number;
	route_node: number;
	tag: number;
	waypoint_formation: number;
	position: Vec3d;
}

// A waypoint's raw data as memset (raw, 0, sizeof (waypoint)) leaves it.
export function clearedWaypointRaw(): WaypointRaw {
	return {
		sub_type: 0,
		flight_time: 0.0,
		altitude: 0.0,
		heading: 0.0,
		position_type: 0,
		route_node: 0,
		tag: 0,
		waypoint_formation: 0,
		position: { x: 0.0, y: 0.0, z: 0.0 },
	};
}

// en_forms.c :: get_formation_database_count () returns NUM_FORMATION_TYPES
function getFormationDatabaseCount(): number {
	return FormationType.NUM_FORMATION_TYPES;
}

// C provenance: wp_int.c :: set_local_raw_int_value
const setLocalRawIntValue: SetIntValueFn = (en, type, value) => {
	const raw = getLocalEntityData<WaypointRaw>(en);

	switch (type) {
		case IntType.INT_TYPE_ENTITY_SUB_TYPE: {
			raw.sub_type = value;

			break;
		}
		case IntType.INT_TYPE_POSITION_TYPE: {
			raw.position_type = storeUnsignedBitfield(value, NUM_POSITION_TYPE_BITS);

			break;
		}
		case IntType.INT_TYPE_ROUTE_NODE: {
			raw.route_node = storeUnsignedBitfield(value, NUM_ROUTE_NODE_BITS);

			break;
		}
		default: {
			// INT_TYPE_WAYPOINT_FORMATION (the only other row installed)
			ASSERT(value <= getFormationDatabaseCount(), "value <= get_formation_database_count ()");

			raw.waypoint_formation = storeUnsignedBitfield(value, NUM_WAYPOINT_FORMATION_BITS);

			break;
		}
	}
};

// C provenance: wp_int.c :: set_local_int_value: the sub type re-tags the waypoint's list
const setLocalIntValue: SetIntValueFn = (en, type, value) => {
	const raw = getLocalEntityData<WaypointRaw>(en);

	switch (type) {
		case IntType.INT_TYPE_ENTITY_SUB_TYPE: {
			raw.sub_type = value;

			updateLocalEntityWaypointListTags(getLocalEntityParent(en, ListType.LIST_TYPE_WAYPOINT));

			break;
		}
		case IntType.INT_TYPE_POSITION_TYPE: {
			raw.position_type = storeUnsignedBitfield(value, NUM_POSITION_TYPE_BITS);

			break;
		}
		case IntType.INT_TYPE_ROUTE_NODE: {
			raw.route_node = storeUnsignedBitfield(value, NUM_ROUTE_NODE_BITS);

			break;
		}
		default: {
			// INT_TYPE_WAYPOINT_FORMATION: no ASSERT in the local setter
			raw.waypoint_formation = storeUnsignedBitfield(value, NUM_WAYPOINT_FORMATION_BITS);

			break;
		}
	}
};

// C provenance: wp_float.c :: set_local_float_value (raw and local)
const setLocalFloatValue: SetFloatValueFn = (en, type, value) => {
	const raw = getLocalEntityData<WaypointRaw>(en);

	if (type === FloatType.FLOAT_TYPE_ALTITUDE) {
		raw.altitude = value;
	} else if (type === FloatType.FLOAT_TYPE_FLIGHT_TIME) {
		raw.flight_time = value;
	} else {
		// FLOAT_TYPE_HEADING
		raw.heading = value;
	}
};

// C provenance: wp_vec3d.c :: get_local_vec3d_ptr / get_local_vec3d (VEC3D_TYPE_POSITION)
function waypointPosition(en: Entity): Vec3d {
	const raw = getLocalEntityData<WaypointRaw>(en);

	if (raw.position_type === PositionType.POSITION_TYPE_VIRTUAL) {
		throw new UnportedBehaviourError("wp_vec3d.c :: VEC3D_TYPE_POSITION of a POSITION_TYPE_VIRTUAL waypoint (its dependent's position)");
	}

	return raw.position;
}

//
// C provenance: wp_msgs.c :: response_to_link_parent, response_to_unlink_parent
//
// Joining or leaving a task's waypoint list re-tags that list; the sender is
// the list's parent.
//
const responseToLinkOrUnlinkParent: MessageResponseFn = (_message, _receiver, sender, args) => {
	const type = args[0] as ListType;

	if (type === ListType.LIST_TYPE_WAYPOINT) {
		if (sender) {
			updateLocalEntityWaypointListTags(sender);
		}
	}

	return 1;
};

// C provenance: wp_funcs.c :: overload_waypoint_functions (ported subset)
export function overloadWaypointFunctions(): void {
	const WAYPOINT = EntityType.ENTITY_TYPE_WAYPOINT;

	// C provenance: wp_list.c :: LIST_TYPE_CURRENT_WAYPOINT_ROOT, LIST_TYPE_TASK_DEPENDENT_LINK, LIST_TYPE_WAYPOINT_LINK
	overloadEntityListRoot(WAYPOINT, "current_waypoint_root", [ListType.LIST_TYPE_CURRENT_WAYPOINT]);
	overloadEntityListLink(WAYPOINT, "task_dependent_link", [ListType.LIST_TYPE_TASK_DEPENDENT]);
	overloadEntityListLink(WAYPOINT, "waypoint_link", [ListType.LIST_TYPE_WAYPOINT]);

	// C provenance: wp_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(WAYPOINT, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<WaypointRaw>(en).sub_type);
	fnGetLocalEntityIntValue.overload(WAYPOINT, IntType.INT_TYPE_POSITION_TYPE, (en) => getLocalEntityData<WaypointRaw>(en).position_type);
	fnGetLocalEntityIntValue.overload(WAYPOINT, IntType.INT_TYPE_ROUTE_NODE, (en) => getLocalEntityData<WaypointRaw>(en).route_node);
	fnGetLocalEntityIntValue.overload(WAYPOINT, IntType.INT_TYPE_WAYPOINT_FORMATION, (en) => getLocalEntityData<WaypointRaw>(en).waypoint_formation);
	fnGetLocalEntityIntValue.overload(WAYPOINT, IntType.INT_TYPE_WAYPOINT_GUIDE_TYPE, (en) => WAYPOINT_DATABASE_GUIDE_SUB_TYPE[getLocalEntityData<WaypointRaw>(en).sub_type]);

	// C provenance: wp_int.c :: overload_waypoint_int_value_functions (raw and local setters)
	for (const type of [IntType.INT_TYPE_ENTITY_SUB_TYPE, IntType.INT_TYPE_POSITION_TYPE, IntType.INT_TYPE_ROUTE_NODE, IntType.INT_TYPE_WAYPOINT_FORMATION]) {
		fnSetLocalEntityRawIntValue.overload(WAYPOINT, type, setLocalRawIntValue);
		fnSetLocalEntityIntValue.overload(WAYPOINT, type, setLocalIntValue);
	}

	// C provenance: wp_float.c :: get_local_float_value, set_local_float_value
	fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_ALTITUDE, (en) => getLocalEntityData<WaypointRaw>(en).altitude);
	fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_FLIGHT_TIME, (en) => getLocalEntityData<WaypointRaw>(en).flight_time);
	fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_HEADING, (en) => getLocalEntityData<WaypointRaw>(en).heading);

	for (const type of [FloatType.FLOAT_TYPE_ALTITUDE, FloatType.FLOAT_TYPE_FLIGHT_TIME, FloatType.FLOAT_TYPE_HEADING]) {
		fnSetLocalEntityRawFloatValue.overload(WAYPOINT, type, setLocalFloatValue);
		fnSetLocalEntityFloatValue.overload(WAYPOINT, type, setLocalFloatValue);
	}

	// C provenance: en_float.c :: initialise_entity_float_value_default_functions;
	// wp_float.c overloads only altitude, flight time and heading
	fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_TASK_USER_DATA, defaultGetEntityFloatValue);

	// C provenance: wp_vec3d.c :: set_local_raw_vec3d, set_local_vec3d (raw->position = *v: a copy into
	// the same storage, so a position pointer taken earlier sees the new value)
	const setPosition = (en: Entity, _type: Vec3dType, v: Vec3d): void => {
		const position = getLocalEntityData<WaypointRaw>(en).position;

		position.x = v.x;
		position.y = v.y;
		position.z = v.z;
	};

	fnSetLocalEntityRawVec3d.overload(WAYPOINT, Vec3dType.VEC3D_TYPE_POSITION, setPosition);
	fnSetLocalEntityVec3d.overload(WAYPOINT, Vec3dType.VEC3D_TYPE_POSITION, setPosition);

	// C provenance: wp_vec3d.c :: get_local_vec3d_ptr, get_local_vec3d
	fnGetLocalEntityVec3dPtr.overload(WAYPOINT, Vec3dType.VEC3D_TYPE_POSITION, (en) => waypointPosition(en));
	fnGetLocalEntityVec3d.overload(WAYPOINT, Vec3dType.VEC3D_TYPE_POSITION, (en) => {
		const v = waypointPosition(en);

		return { x: v.x, y: v.y, z: v.z };
	});

	// C provenance: wp_char.c :: set_local_char_value, get_local_char_value (raw->tag: unsigned int : NUM_TAG_BITS)
	fnSetLocalEntityCharValue.overload(WAYPOINT, CharType.CHAR_TYPE_TAG, (en, _type, value) => {
		getLocalEntityData<WaypointRaw>(en).tag = storeUnsignedBitfield(value, NUM_TAG_BITS);
	});
	fnGetLocalEntityCharValue.overload(WAYPOINT, CharType.CHAR_TYPE_TAG, (en) => getLocalEntityData<WaypointRaw>(en).tag);

	// C provenance: wp_msgs.c :: overload_waypoint_message_responses. LINK_CHILD and
	// UNLINK_CHILD are overloaded only under DEBUG_MODULE >= 3, so en_msgs.c's default applies.
	messageResponses.overload(WAYPOINT, EntityMessage.ENTITY_MESSAGE_LINK_PARENT, responseToLinkOrUnlinkParent);
	messageResponses.overload(WAYPOINT, EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, responseToLinkOrUnlinkParent);
	messageResponses.overload(WAYPOINT, EntityMessage.ENTITY_MESSAGE_LINK_CHILD, defaultMessageResponse);
	messageResponses.overload(WAYPOINT, EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, defaultMessageResponse);
}

//
// Port: outbound replication of authoritative entity changes.
//
// C provenance: entity/system/en_comms/en_comms.c :: transmit_entity_comms_message
//               (called by every set_server_*_value, create_remote and
//               destroy_remote in the entity overloads)
//
// In EECH the server applies a change locally and then transmits it to
// clients (ENTITY_COMMS_FLOAT_VALUE, ENTITY_COMMS_CREATE, ENTITY_COMMS_DESTROY,
// ...). The network transport is not part of the campaign core; the semantic
// contract "this authoritative change happened" is. An adapter may forward it
// to a transport, mirror it into DCS, or drop it.
//

import type { EntityType, FloatType, IntType, ListType, Vec3dType } from "../generated/c-enums";

// One creation attribute as ENTITY_COMMS_CREATE packs it (en_attrs.c ::
// pack_entity_attributes): floats narrowed, entities by index (-1: NULL).
export type ReplicatedEntityAttribute =
	| { kind: "int_value"; type: IntType; value: number }
	| { kind: "float_value"; type: FloatType; value: number }
	| { kind: "vec3d"; type: Vec3dType; x: number; y: number; z: number }
	| { kind: "parent"; type: ListType; entityIndex: number }
	| { kind: "child_pred"; type: ListType; entityIndex: number };

export interface EntityReplication {
	// C: transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value)
	transmitEntityFloatValue(entityIndex: number, type: FloatType, value: number): void;

	// C: transmit_entity_comms_message (ENTITY_COMMS_CREATE, NULL, type, index, pargs)
	transmitEntityCreate(type: EntityType, entityIndex: number, attributes: ReplicatedEntityAttribute[]): void;

	// C: transmit_entity_comms_message (ENTITY_COMMS_DESTROY, en)
	transmitEntityDestroy(entityIndex: number): void;

	// C: transmit_entity_comms_message (ENTITY_COMMS_SET_TASK_POINTERS, task):
	// a new task's route (route_length nodes, the terminator not included) and
	// return keysite. Nodes are floats; entities by index (-1: NULL).
	transmitTaskPointers(taskIndex: number, route: ReplicatedTaskRoute): void;

	// C: transmit_entity_comms_message (ENTITY_COMMS_SWITCH_PARENT, en, type, parent)
	transmitSwitchParent(entityIndex: number, type: ListType, parentIndex: number): void;

	// slice 6b
	// C: transmit_entity_comms_message (ENTITY_COMMS_INT_VALUE, en, type, value)
	transmitEntityIntValue(entityIndex: number, type: IntType, value: number): void;
	// C: transmit_entity_comms_message (ENTITY_COMMS_CREATE_WAYPOINT_ROUTE, task, group, return_keysite,
	// start, stop, check_sum, node_count): clients rebuild the route from it and compare checksums
	transmitCreateWaypointRoute(taskIndex: number, route: ReplicatedWaypointRoute): void;
	// C: transmit_entity_comms_message (ENTITY_COMMS_SWITCH_LIST, en, from_type, parent, to_type)
	transmitSwitchList(entityIndex: number, fromType: ListType, parentIndex: number, toType: ListType): void;
	// C: transmit_entity_comms_message (ENTITY_COMMS_SET_GUIDE_CRITERIA, guide, type, valid, value)
	transmitSetGuideCriteria(guideIndex: number, type: number, valid: number, value: number): void;
}

// What ENTITY_COMMS_CREATE_WAYPOINT_ROUTE packs (en_comms.c), in its order:
// group and return keysite by index (-1: NULL), the start and stop positions
// when given, the checksum, and every waypoint of the task's route by index.
export interface ReplicatedWaypointRoute {
	groupIndex: number;
	returnKeysiteIndex: number;
	start: { x: number; y: number; z: number } | undefined;
	stop: { x: number; y: number; z: number } | undefined;
	checkSum: number;
	waypointIndices: number[];
}

// What ENTITY_COMMS_SET_TASK_POINTERS packs (en_comms.c), in its order.
export interface ReplicatedTaskRoute {
	nodes: { x: number; y: number; z: number }[];
	formationTypes: number[];
	waypointTypes: number[];
	dependentIndices: number[];
	returnKeysiteIndex: number;
}

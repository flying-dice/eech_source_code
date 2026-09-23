//
// Outbound entity comms: the one place the campaign core transmits.
//
// C provenance: entity/system/en_comms/en_comms.c :: transmit_entity_comms_message
//
// Every transmission goes through transmit_entity_comms_message, which first
// traps single player:
//
//   if ((direct_play_get_comms_mode () == DIRECT_PLAY_COMMS_MODE_NONE) || (!comms_messages_enabled))
//   {
//     return;
//   }
//
// and then packs the message. Whether a session has other players is
// campaign state the host sets (like the comms model and the game status), so
// it is core state here: setEntityCommsTransmission. The core starts with
// transmission on, as every slice before 5b assumed. What is transmitted goes
// to the EntityReplication port.
//
// Packing is not only encoding. ENTITY_COMMS_SET_TASK_POINTERS packs each
// route node with pack_vec3d (VEC3D_TYPE_POSITION), which checks it
// (ASSERT (point_inside_map_volume (v) || v->y == -10000)) and, where the
// ASSERT is compiled out, bounds the task's own node in place. The check runs
// only when something is transmitted, so a single player session never makes
// it. The ASSERT keeps its debug-build meaning, as every ASSERT in the port
// does, so the in-place bound is not reachable (docs/slices/supply-task-construction.md, F1).
//

import { ASSERT } from "../../core/assert";
import type { Vec3d } from "../../core/maths/vec3d";
import type { EntityType, FloatType, IntType, ListType } from "../../generated/c-enums";
import type { ReplicatedEntityAttribute } from "../../ports";
import { pointInsideMapVolume } from "./en_world";
import { getCampaignPorts, type Entity } from "./entity";

let transmission = true;

// The host's session set-up: false for a single player session
// (direct_play_get_comms_mode () == DIRECT_PLAY_COMMS_MODE_NONE).
export function setEntityCommsTransmission(active: boolean): void {
	transmission = active;
}

export function resetEntityCommsTransmission(): void {
	transmission = true;
}

// C: ENTITY_COMMS_FLOAT_VALUE
export function transmitEntityFloatValue(en: Entity, type: FloatType, value: number): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitEntityFloatValue(en.index, type, value);
}

// C: ENTITY_COMMS_CREATE
export function transmitEntityCreate(type: EntityType, index: number, attributes: ReplicatedEntityAttribute[]): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitEntityCreate(type, index, attributes);
}

// C: ENTITY_COMMS_DESTROY
export function transmitEntityDestroy(en: Entity): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitEntityDestroy(en.index);
}

// The route a task holds (ts_ptr.c's pointers and the task's route_length).
export interface TaskRoutePointers {
	route_length: number;
	route_nodes: Vec3d[];
	route_formation_types: number[];
	route_waypoint_types: number[];
	route_dependents: (Entity | undefined)[];
	return_keysite: Entity | undefined;
}

// C provenance: en_vec3d.c :: pack_vec3d, VEC3D_PACK_TYPE_POSITION: the check before packing
function packPosition(v: Vec3d): void {
	ASSERT(pointInsideMapVolume(v) || v.y === -10000, "point_inside_map_volume (v) || v->y == -10000");
}

function indexOf(en: Entity | undefined): number {
	return en === undefined ? -1 : en.index;
}

// C: ENTITY_COMMS_SET_TASK_POINTERS
export function transmitTaskPointers(task: Entity, route: TaskRoutePointers): void {
	if (!transmission) {
		return;
	}

	const nodes: Vec3d[] = [];
	const formationTypes: number[] = [];
	const waypointTypes: number[] = [];
	const dependentIndices: number[] = [];

	// positions (unsigned int loop < raw->route_length)
	for (let loop = 0; loop < route.route_length; loop++) {
		const node = route.route_nodes[loop];

		packPosition(node);

		nodes.push({ x: node.x, y: node.y, z: node.z });
	}

	for (let loop = 0; loop < route.route_length; loop++) {
		formationTypes.push(route.route_formation_types[loop]);
	}

	for (let loop = 0; loop < route.route_length; loop++) {
		waypointTypes.push(route.route_waypoint_types[loop]);
	}

	for (let loop = 0; loop < route.route_length; loop++) {
		dependentIndices.push(indexOf(route.route_dependents[loop]));
	}

	getCampaignPorts().entityReplication.transmitTaskPointers(task.index, {
		nodes,
		formationTypes,
		waypointTypes,
		dependentIndices,
		returnKeysiteIndex: indexOf(route.return_keysite),
	});
}

// C: ENTITY_COMMS_INT_VALUE (slice 6b: the task's route checksum)
export function transmitEntityIntValue(en: Entity, type: IntType, value: number): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitEntityIntValue(en.index, type, value);
}

//
// C: ENTITY_COMMS_CREATE_WAYPOINT_ROUTE (task, group, return_keysite, start, stop, check_sum, node_count).
// The start and stop positions are packed with pack_vec3d (VEC3D_TYPE_POSITION),
// the same check as the task pointers'; the waypoints are the task's
// LIST_TYPE_WAYPOINT list at transmission (node_count is not packed: the list
// is counted again).
//
export function transmitCreateWaypointRoute(
	task: Entity,
	group: Entity,
	returnKeysite: Entity | undefined,
	start: Vec3d | undefined,
	stop: Vec3d | undefined,
	checkSum: number,
	waypoints: Entity[],
): void {
	if (!transmission) {
		return;
	}

	if (start !== undefined) {
		packPosition(start);
	}

	if (stop !== undefined) {
		packPosition(stop);
	}

	const waypointIndices: number[] = [];

	for (const wp of waypoints) {
		waypointIndices.push(wp.index);
	}

	getCampaignPorts().entityReplication.transmitCreateWaypointRoute(task.index, {
		groupIndex: group.index,
		returnKeysiteIndex: indexOf(returnKeysite),
		start: start === undefined ? undefined : { x: start.x, y: start.y, z: start.z },
		stop: stop === undefined ? undefined : { x: stop.x, y: stop.y, z: stop.z },
		checkSum,
		waypointIndices,
	});
}

// C: ENTITY_COMMS_SWITCH_LIST (en, from list type, parent, to list type)
export function transmitSwitchList(en: Entity, fromType: ListType, parent: Entity | undefined, toType: ListType): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitSwitchList(en.index, fromType, indexOf(parent), toType);
}

// C: ENTITY_COMMS_SET_GUIDE_CRITERIA (guide, criteria type, valid, value)
export function transmitSetGuideCriteria(guide: Entity, type: number, valid: number, value: number): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitSetGuideCriteria(guide.index, type, valid, value);
}

// C: ENTITY_COMMS_SWITCH_PARENT
export function transmitSwitchParent(en: Entity, type: ListType, parent: Entity | undefined): void {
	if (!transmission) {
		return;
	}

	getCampaignPorts().entityReplication.transmitSwitchParent(en.index, type, indexOf(parent));
}

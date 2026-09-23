//
// Task waypoint routes.
//
// C provenance: ai/taskgen/croute.c :: create_generic_waypoint_route,
//               parser_task_waypoint_route, generate_route_check_sum,
//               generate_biased_vec3d_route, create_route, generate_best_mid_point,
//               get_best_point, second_past_route, get_route_point_rating,
//               optimise_route, route_biasing_database
//
// Slice 6b (issue #18): assign.c :: assign_task_to_group builds the group's
// waypoint route for its task before the guide exists. The task's specified
// route (Slice 5b), a start waypoint at the first member and a LAND waypoint
// at the return keysite are joined by a biased route search over the terrain
// (the TerrainElevation port) and the sectors' sides; the result's checksum
// is transmitted, each point becomes a waypoint (closest road node from the
// RoadNetwork port), the parser spaces them, and ENTITY_COMMS_CREATE_WAYPOINT_ROUTE
// lets clients rebuild the same route and compare checksums.
//
// The server builds the route (the client rebuild from the message is not
// ported). No randomness and no clock. Floating point follows the declared
// types with every operation truncated (docs/fidelity/fpu-semantics.md,
// decision D1): float operations are f32*, double sums and products stored to
// float are f32Add / f32Mul of the exact double operands.
//
// Undefined behaviour in the C is surfaced, not repaired:
//   - specified_route is read uninitialised when the task has no route nodes
//     (EechUndefinedBehaviourError);
//   - second_past_route reads best_point uninitialised when get_best_point
//     returns FALSE on its first iteration (a later FALSE reuses the previous
//     point, which is defined);
//   - parser_task_waypoint_route reads the position of a NULL waypoint when the
//     last waypoint is a NAVIGATION one (decision D5: EechNullDereferenceError);
//     supply routes end with LAND, so the SUPPLY path never reaches it.
//
// The route search recurses until every leg is at most min_route_range long.
// For a start outside the map (an aircraft off the map: invalid campaign
// state) that never happens; EECH has no guard and neither does the port.
//

import { ASSERT, assertNotNullDereference, EechUndefinedBehaviourError, UnportedBehaviourError } from "../../core/assert";
import { storeUnsignedBitfield, toCInt } from "../../core/cint";
import { f32Add, f32Div, f32Mul, f32Sub, toFloat32 } from "../../core/float32";
import { getInverseSquareRoot } from "../../core/maths/invsqrt";
import { bound, max, min } from "../../core/maths/miscmath";
import { get2dRange, getApprox3dRange, getSqr2dRange } from "../../core/maths/range";
import type { Vec3d } from "../../core/maths/vec3d";
import { checkZero3dVector, normalise3dVector } from "../../core/maths/vector";
import { GROUP_DATABASE_DEFAULT_ENTITY_TYPE, GROUP_DATABASE_MOVEMENT_TYPE } from "../../generated/c-group-database";
import {
	ROUTE_BIASING_ELEVATION_BIAS,
	ROUTE_BIASING_MIN_ROUTE_RANGE,
	ROUTE_BIASING_NUM_ROUTE_SAMPLES,
	ROUTE_BIASING_OPTIMISE_TOLERANCE,
	ROUTE_BIASING_RANGE_BIAS,
	ROUTE_BIASING_ROUTE_DEVIATION_SIZE,
	ROUTE_BIASING_SIDE_BIAS,
} from "../../generated/c-route-biasing-database";
import { TASK_DATABASE_ADD_START_WAYPOINT, TASK_DATABASE_TASK_ROUTE_SEARCH } from "../../generated/c-task-database";
import { CommsModelType, EntitySubTypeWaypoint, EntityType, FloatType, FormationType, IntType, ListType, PtrType, Vec3dType } from "../../generated/c-enums";
import { getLocalSectorEntity } from "../../entity/special/sector/sector";
import { getWaypointDatabaseMinimumPreviousWaypointDistance, getWaypointDatabasePositionTypeValue } from "../../entity/special/waypoint/wp_dbase";
import { getCommsModel } from "../../entity/system/comms";
import { transmitCreateWaypointRoute } from "../../entity/system/en_comms";
import { createLocalEntity } from "../../entity/system/en_creat";
import { ENTITY_INDEX_DONT_CARE } from "../../entity/system/en_heap";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, insertLocalEntityIntoParentsChildList } from "../../entity/system/en_list";
import {
	getLocalEntityFloatValue,
	getLocalEntityIntValue,
	getLocalEntityPtrValue,
	getLocalEntityVec3d,
	getLocalEntityVec3dPtr,
	setClientServerEntityIntValue,
	setLocalEntityIntValue,
	setLocalEntityVec3d,
} from "../../entity/system/en_values";
import { boundPositionToAdjustedMapVolume, getWorldMap, pointInsideMapArea } from "../../entity/system/en_world";
import { getCampaignPorts, type Entity } from "../../entity/system/entity";
import { getClosestRoadNode } from "../ai_misc/ai_misc";

// C provenance: croute.h :: struct ROUTE_NODE
export interface RouteNode {
	type: number;
	formation: number;
	dependent: Entity | undefined;
	position: Vec3d;
	next: RouteNode | undefined;
	prev: RouteNode | undefined;
}

// malloc_fast_mem + memset (node, 0, sizeof (route_node)), or a node whose every field the C sets
function routeNode(type: number, x: number, y: number, z: number): RouteNode {
	return { type, formation: 0, dependent: undefined, position: { x, y, z }, next: undefined, prev: undefined };
}

// C: float *best_point_terrain_elevations (malloc_fast_mem, num_route_samples + 1 entries; [0] is never used)
let best_point_terrain_elevations: number[] = [];

//
// terrelev.h :: get_3d_terrain_elevation (X, Z): get_3d_terrain_point_data (x, z, NULL),
// a float. The TerrainElevation port supplies the map's value; it is environment
// input, narrowed to float as the C function returns it.
//
function get3dTerrainElevation(x: number, z: number): number {
	return toFloat32(getCampaignPorts().terrainElevation.getTerrainElevation(x, z));
}

//
// C provenance: croute.c :: create_generic_waypoint_route (group, task_en, return_keysite,
//               client_start = NULL, client_stop = NULL, waypoint_indices = NULL, indices_count = 0)
//
// Returns TRUE on every path: its FALSE arm follows a FALSE from
// generate_biased_vec3d_route, which always returns TRUE.
//
export function createGenericWaypointRoute(group: Entity | undefined, task_en: Entity | undefined, return_keysite: Entity | undefined): true {
	ASSERT(group !== undefined, "group");

	ASSERT(task_en !== undefined, "task_en");

	if (getLocalEntityFirstChild(task_en, ListType.LIST_TYPE_WAYPOINT)) {
		return true;
	}

	// the client rebuilds the route from ENTITY_COMMS_CREATE_WAYPOINT_ROUTE (client_start,
	// client_stop, waypoint_indices; the checksum comparison): not ported. The server's
	// force_local_entity_create_stack_attributes is untouched (COMMS_DATA_FLOW_TX).
	if (getCommsModel() !== CommsModelType.COMMS_MODEL_SERVER) {
		throw new UnportedBehaviourError("croute.c :: create_generic_waypoint_route (COMMS_MODEL_CLIENT)");
	}

	//////////////////////////////////////////////////////////////////
	//
	// Sort route generation type
	//
	//////////////////////////////////////////////////////////////////

	const member = getLocalEntityFirstChild(group, ListType.LIST_TYPE_MEMBER);

	ASSERT(member !== undefined, "member");

	// the group has no INT_TYPE_SECTOR_SIDE row: en_int.c's default, 0 (decision D2)
	const side = getLocalEntityIntValue(group, IntType.INT_TYPE_SECTOR_SIDE);

	// mb_vel: read, not used (the flight times read the cruise velocity again)
	getLocalEntityFloatValue(member, FloatType.FLOAT_TYPE_CRUISE_VELOCITY);

	const height = getLocalEntityFloatValue(member, FloatType.FLOAT_TYPE_CRUISE_ALTITUDE);

	const group_type = getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const movement_type = GROUP_DATABASE_MOVEMENT_TYPE[group_type];

	const task_route_type: EntityType = GROUP_DATABASE_DEFAULT_ENTITY_TYPE[group_type];

	const task_type = getLocalEntityIntValue(task_en, IntType.INT_TYPE_ENTITY_SUB_TYPE);

	const generate_route = TASK_DATABASE_TASK_ROUTE_SEARCH[task_type];

	const start_point_count = TASK_DATABASE_ADD_START_WAYPOINT[task_type];

	//////////////////////////////////////////////////////////////////
	//
	// Generate list of Specified route
	//
	//////////////////////////////////////////////////////////////////

	const route_points_ptr = getLocalEntityPtrValue<Vec3d[] | undefined>(task_en, PtrType.PTR_TYPE_ROUTE_NODE);

	const route_dependents_ptr = getLocalEntityPtrValue<(Entity | undefined)[] | undefined>(task_en, PtrType.PTR_TYPE_ROUTE_DEPENDENTS);

	const route_waypoint_types_ptr = getLocalEntityPtrValue<number[] | undefined>(task_en, PtrType.PTR_TYPE_ROUTE_WAYPOINT_TYPES);

	const route_formation_types_ptr = getLocalEntityPtrValue<number[] | undefined>(task_en, PtrType.PTR_TYPE_ROUTE_FORMATION_TYPES);

	const route_length = getLocalEntityIntValue(task_en, IntType.INT_TYPE_ROUTE_LENGTH);

	// C: route_node *specified_route (uninitialised until the loop runs)
	let specified_route: RouteNode | undefined = undefined;

	let last_specified_node: RouteNode | undefined = undefined;

	for (let loop = route_length - 1; loop >= 0; loop--) {
		assertNotNullDereference(route_points_ptr, "route_points_ptr");
		assertNotNullDereference(route_dependents_ptr, "route_dependents_ptr");
		assertNotNullDereference(route_waypoint_types_ptr, "route_waypoint_types_ptr");
		assertNotNullDereference(route_formation_types_ptr, "route_formation_types_ptr");

		// position: ceil of each float, stored as float (whole numbers)
		const node = routeNode(route_waypoint_types_ptr[loop], Math.ceil(route_points_ptr[loop].x), Math.ceil(route_points_ptr[loop].y), Math.ceil(route_points_ptr[loop].z));

		node.dependent = route_dependents_ptr[loop];

		node.formation = route_formation_types_ptr[loop];

		node.next = last_specified_node;
		node.prev = undefined;

		if (last_specified_node) {
			last_specified_node.prev = node;
		}

		last_specified_node = node;

		specified_route = node;
	}

	// every path below reads specified_route
	if (specified_route === undefined) {
		throw new EechUndefinedBehaviourError("croute.c :: create_generic_waypoint_route: specified_route read uninitialised (the task has no route nodes)");
	}

	let start_ptr: Vec3d | undefined = undefined;

	let stop_ptr: Vec3d | undefined = undefined;

	// SUPPLY adds a start waypoint (the only task type the port assigns: assign.ts)
	/* istanbul ignore else */
	if (start_point_count > 0) {
		//
		// Add start waypoint
		//

		const start = getLocalEntityVec3d(member, Vec3dType.VEC3D_TYPE_POSITION);

		start.x = Math.ceil(start.x);
		start.y = Math.ceil(start.y);
		start.z = Math.ceil(start.z);

		start_ptr = start;

		const start_node = routeNode(EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, start.x, start.y, start.z);

		start_node.dependent = undefined;

		start_node.formation = FormationType.FORMATION_ROW_LEFT;

		start_node.next = specified_route;
		start_node.prev = undefined;

		specified_route.prev = start_node;

		specified_route = start_node;

		// assign_task_to_group always passes the return keysite for SUPPLY (assess_landing)
		/* istanbul ignore else */
		if (return_keysite) {
			const stop = getLocalEntityVec3d(return_keysite, Vec3dType.VEC3D_TYPE_POSITION);

			stop.x = Math.ceil(stop.x);
			stop.y = Math.ceil(stop.y);
			stop.z = Math.ceil(stop.z);

			stop_ptr = stop;

			//
			// add land waypoint
			//

			const land_node = routeNode(EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_LAND, stop.x, stop.y, stop.z);

			land_node.dependent = return_keysite;

			land_node.formation = FormationType.FORMATION_ROW_LEFT;

			let last: RouteNode = specified_route;

			while (last.next) {
				last = last.next;
			}

			land_node.prev = last;
			land_node.next = undefined;

			last.next = land_node;
		}
	}

	//////////////////////////////////////////////////////////////////
	//
	// Generate route
	//
	//////////////////////////////////////////////////////////////////

	let new_route: RouteNode | undefined;

	// SUPPLY searches its route (task_route_search)
	/* istanbul ignore else */
	if (generate_route !== 0) {
		new_route = generateBiasedVec3dRoute(specified_route, side, movement_type);
	} else {
		new_route = specified_route;
	}

	//////////////////////////////////////////////////////////////////
	//
	// Generate checksum
	//
	//////////////////////////////////////////////////////////////////

	const check_sum = generateRouteCheckSum(new_route);

	// the server's arm (the client compares with the transmitted checksum)
	setClientServerEntityIntValue(task_en, IntType.INT_TYPE_ROUTE_CHECK_SUM, check_sum);

	//////////////////////////////////////////////////////////////////
	//
	// Create waypoint route
	//
	//////////////////////////////////////////////////////////////////

	let node_count = 0;

	let last_wp: Entity | undefined = undefined;

	let specified_route_ptr: RouteNode | undefined = specified_route;

	let flight_time = 0.0;

	const map = getWorldMap();

	let route: RouteNode | undefined = new_route;

	while (route) {
		const waypoint_pos = route.position;

		// bound (VALUE, 1, MAX_MAP_? - 1): MAX_MAP_? - 1 is a float subtraction
		waypoint_pos.x = bound(waypoint_pos.x, 1, f32Sub(map.max_map_x, 1));
		waypoint_pos.y = bound(waypoint_pos.y, 1, f32Sub(map.max_map_y, 1));
		waypoint_pos.z = bound(waypoint_pos.z, 1, f32Sub(map.max_map_z, 1));

		ASSERT(pointInsideMapArea(waypoint_pos), "point_inside_map_area(waypoint_pos)");

		// terrain_elevation: read and never used (decision D4: the port is still asked)
		get3dTerrainElevation(waypoint_pos.x, waypoint_pos.z);

		const road_node = getClosestRoadNode(waypoint_pos, 5.0);

		// flight time between waypoints

		if (last_wp) {
			const last_waypoint_pos = getLocalEntityVec3dPtr(last_wp, Vec3dType.VEC3D_TYPE_POSITION);

			const range = get2dRange(waypoint_pos, last_waypoint_pos);

			flight_time = 999999.0;

			if (getLocalEntityFloatValue(member, FloatType.FLOAT_TYPE_CRUISE_VELOCITY) > 0.0) {
				flight_time = f32Div(range, getLocalEntityFloatValue(member, FloatType.FLOAT_TYPE_CRUISE_VELOCITY));
			}
		}

		// waypoint index: ENTITY_INDEX_DONT_CARE (no waypoint_indices on the server)

		const wp = createLocalEntity(EntityType.ENTITY_TYPE_WAYPOINT, ENTITY_INDEX_DONT_CARE, [
			{ kind: "parent", type: ListType.LIST_TYPE_WAYPOINT, entity: task_en },
			{ kind: "child_pred", type: ListType.LIST_TYPE_WAYPOINT, entity: last_wp },
			{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x: waypoint_pos.x, y: waypoint_pos.y, z: waypoint_pos.z },
			{ kind: "int_value", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION },
			{ kind: "int_value", type: IntType.INT_TYPE_ROUTE_NODE, value: road_node },
			{ kind: "int_value", type: IntType.INT_TYPE_WAYPOINT_FORMATION, value: FormationType.FORMATION_ROW_LEFT },
			{ kind: "float_value", type: FloatType.FLOAT_TYPE_ALTITUDE, value: height },
			{ kind: "float_value", type: FloatType.FLOAT_TYPE_FLIGHT_TIME, value: flight_time },
		]);

		//
		// Assign specified waypoint stuff
		//

		// the C reads specified_route_ptr->position whether or not a specified node is left
		assertNotNullDereference(specified_route_ptr, "specified_route_ptr");

		if (waypoint_pos.x === specified_route_ptr.position.x && waypoint_pos.z === specified_route_ptr.position.z) {
			setLocalEntityIntValue(wp, IntType.INT_TYPE_ENTITY_SUB_TYPE, specified_route_ptr.type);

			setLocalEntityIntValue(wp, IntType.INT_TYPE_WAYPOINT_FORMATION, specified_route_ptr.formation);

			setLocalEntityIntValue(wp, IntType.INT_TYPE_POSITION_TYPE, getWaypointDatabasePositionTypeValue(specified_route_ptr.type, task_route_type));

			if (specified_route_ptr.dependent) {
				insertLocalEntityIntoParentsChildList(wp, ListType.LIST_TYPE_TASK_DEPENDENT, specified_route_ptr.dependent, undefined);
			}

			specified_route_ptr = specified_route_ptr.next;
		}

		node_count++;

		last_wp = wp;

		route = route.next;
	}

	//////////////////////////////////////////////////////////////////
	//
	// Post process route
	//
	//////////////////////////////////////////////////////////////////

	parserTaskWaypointRoute(group, task_en);

	//////////////////////////////////////////////////////////////////
	//
	// Create on Clients
	//
	//////////////////////////////////////////////////////////////////

	const waypoints: Entity[] = [];

	for (let wp = getLocalEntityFirstChild(task_en, ListType.LIST_TYPE_WAYPOINT); wp !== undefined; wp = getLocalEntityChildSucc(wp, ListType.LIST_TYPE_WAYPOINT)) {
		waypoints.push(wp);
	}

	transmitCreateWaypointRoute(task_en, group, return_keysite, start_ptr, stop_ptr, check_sum, waypoints);

	return true;
}

//
// C provenance: croute.c :: parser_task_waypoint_route
//
// Spaces the waypoints: where a waypoint is closer to the next than the next
// type's minimum previous-waypoint distance (for the group's mobile type), a
// NAVIGATION waypoint moves to the midpoint between its neighbours (and, if
// still too close, the minimum distance back from the next along the half
// leg); otherwise a NAVIGATION next waypoint moves towards the one after it.
// Moves write the raw position only.
//
// Only ATTACK and RECON waypoints have a nonzero minimum distance (wp_dbase.c),
// so a supply route (NAVIGATION, PICK_UP, PREPARE_FOR_DROP_OFF, DROP_OFF,
// FINISH_DROP_OFF, LAND) is never moved, and the second arm (a NAVIGATION
// next waypoint, minimum distance 0) is never taken by any route.
//
export function parserTaskWaypointRoute(group: Entity, task: Entity): void {
	ASSERT(task.type === EntityType.ENTITY_TYPE_TASK, "task->type == ENTITY_TYPE_TASK");

	let last_wp = getLocalEntityFirstChild(task, ListType.LIST_TYPE_WAYPOINT);

	if (!last_wp) {
		return;
	}

	let this_wp = getLocalEntityChildSucc(last_wp, ListType.LIST_TYPE_WAYPOINT);

	if (!this_wp) {
		return;
	}

	let next_wp = getLocalEntityChildSucc(this_wp, ListType.LIST_TYPE_WAYPOINT);

	if (!next_wp) {
		return;
	}

	const mobile_type: EntityType = GROUP_DATABASE_DEFAULT_ENTITY_TYPE[getLocalEntityIntValue(group, IntType.INT_TYPE_ENTITY_SUB_TYPE)];

	const new_pos: Vec3d = { x: 0.0, y: 0.0, z: 0.0 };

	while (next_wp) {
		const this_pos = getLocalEntityVec3dPtr(this_wp, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

		const next_pos = getLocalEntityVec3dPtr(next_wp, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

		//
		// check if this_wp is on top off next_wp
		//

		let range = getApprox3dRange(next_pos, this_pos);

		let min_range = getWaypointDatabaseMinimumPreviousWaypointDistance(getLocalEntityIntValue(next_wp, IntType.INT_TYPE_ENTITY_SUB_TYPE), mobile_type);

		if (range < min_range) {
			const last_pos = getLocalEntityVec3dPtr(last_wp as Entity, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

			//
			// move this waypoint if its a navigation one
			//

			if (getLocalEntityIntValue(this_wp, IntType.INT_TYPE_ENTITY_SUB_TYPE) === EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION) {
				// (float - float) / 2.0: exact halving of the float difference
				const delta_position: Vec3d = { x: f32Sub(next_pos.x, last_pos.x) / 2.0, y: 0.0, z: f32Sub(next_pos.z, last_pos.z) / 2.0 };

				// ceil (float + float)
				new_pos.x = Math.ceil(f32Add(last_pos.x, delta_position.x));
				new_pos.y = Math.ceil(f32Add(this_pos.y, delta_position.y));
				new_pos.z = Math.ceil(f32Add(last_pos.z, delta_position.z));

				range = getApprox3dRange(next_pos, new_pos);

				min_range = getWaypointDatabaseMinimumPreviousWaypointDistance(getLocalEntityIntValue(next_wp, IntType.INT_TYPE_ENTITY_SUB_TYPE), mobile_type);

				if (range < min_range) {
					normalise3dVector(delta_position);

					delta_position.x = f32Mul(delta_position.x, min_range);
					delta_position.y = f32Mul(delta_position.y, min_range);
					delta_position.z = f32Mul(delta_position.z, min_range);

					new_pos.x = Math.ceil(f32Sub(next_pos.x, delta_position.x));
					new_pos.y = Math.ceil(f32Sub(next_pos.y, delta_position.y));
					new_pos.z = Math.ceil(f32Sub(next_pos.z, delta_position.z));
				}

				setLocalEntityVec3d(this_wp, Vec3dType.VEC3D_TYPE_POSITION, new_pos);
			} else if (
				// Unreachable under the compiled databases: it needs range < the minimum previous
				// waypoint distance of a NAVIGATION next_wp, which is 0 in every mobile column of
				// wp_dbase.c (test/unit/supply-task-transaction.test.ts), and no range is below 0.
				// Decision D5's NULL dereference is inside it.
				/* istanbul ignore next */
				getLocalEntityIntValue(next_wp, IntType.INT_TYPE_ENTITY_SUB_TYPE) === EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION
			) {
				const next_next_wp = getLocalEntityChildSucc(next_wp, ListType.LIST_TYPE_WAYPOINT);

				// next_next_pos = get_local_entity_vec3d_ptr (next_next_wp, ...) before the NULL check
				// (decision D5): a route whose last waypoint is NAVIGATION dereferences NULL here
				assertNotNullDereference(next_next_wp, "next_next_wp");

				const next_next_pos = getLocalEntityVec3dPtr(next_next_wp, Vec3dType.VEC3D_TYPE_POSITION) as Vec3d;

				// if (next_next_wp): always true once its position has been read

				//
				// move the next one towards the next_next
				//

				const delta_position: Vec3d = { x: f32Sub(next_next_pos.x, this_pos.x) / 2.0, y: 0.0, z: f32Sub(next_next_pos.z, this_pos.z) / 2.0 };

				new_pos.x = Math.ceil(f32Add(this_pos.x, delta_position.x));
				new_pos.y = Math.ceil(f32Add(next_pos.y, delta_position.y));
				new_pos.z = Math.ceil(f32Add(this_pos.z, delta_position.z));

				const inner_range = getApprox3dRange(next_next_pos, new_pos);

				const inner_min_range = getWaypointDatabaseMinimumPreviousWaypointDistance(getLocalEntityIntValue(next_next_wp, IntType.INT_TYPE_ENTITY_SUB_TYPE), mobile_type);

				if (inner_range < inner_min_range) {
					normalise3dVector(delta_position);

					delta_position.x = f32Mul(delta_position.x, inner_min_range);
					delta_position.y = f32Mul(delta_position.y, inner_min_range);
					delta_position.z = f32Mul(delta_position.z, inner_min_range);

					new_pos.x = Math.ceil(f32Sub(next_next_pos.x, delta_position.x));
					new_pos.y = Math.ceil(f32Sub(next_next_pos.y, delta_position.y));
					new_pos.z = Math.ceil(f32Sub(next_next_pos.z, delta_position.z));
				}

				boundPositionToAdjustedMapVolume(new_pos);

				setLocalEntityVec3d(next_wp, Vec3dType.VEC3D_TYPE_POSITION, new_pos);
			}
		}

		last_wp = this_wp;

		this_wp = next_wp;

		next_wp = getLocalEntityChildSucc(next_wp, ListType.LIST_TYPE_WAYPOINT);
	}
}

//
// C provenance: croute.c :: generate_route_check_sum
//
// An unsigned char sum of (int) x + (int) y + (int) z over every node but the
// first and the last (wrapping modulo 256).
//
export function generateRouteCheckSum(first: RouteNode | undefined): number {
	let check_sum = 0;

	// dont checksum start and end points because entities may be in slightly different positions due to pack/unpack

	// route = route->next: a NULL route (a single specified point) is dereferenced
	assertNotNullDereference(first, "route");

	let route = first.next;

	if (route) {
		while (route.next) {
			check_sum = storeUnsignedBitfield(check_sum + toCInt(route.position.x), 8);

			check_sum = storeUnsignedBitfield(check_sum + toCInt(route.position.y), 8);

			check_sum = storeUnsignedBitfield(check_sum + toCInt(route.position.z), 8);

			route = route.next;
		}
	}

	return check_sum;
}

//
// C provenance: croute.c :: generate_biased_vec3d_route (points, &route, side, movement_type)
//
// A searched route between each pair of consecutive points, joined; the first
// node of every sub-route after the first is dropped, and each sub-route's end
// takes the type and formation of the point it ends at (the first node, the
// first point's). Always returns TRUE in the C; the port returns the route.
//
function generateBiasedVec3dRoute(points: RouteNode, side: number, movement_type: number): RouteNode | undefined {
	let route_start: RouteNode | undefined = undefined;

	let node_count = 0;

	best_point_terrain_elevations = [];

	let this_node = points;

	let next_node = this_node.next;

	while (next_node) {
		let fast_route = createRoute(this_node.position, next_node.position, side, movement_type);

		secondPastRoute(fast_route, side, movement_type);

		optimiseRoute(fast_route, movement_type);

		// skip start of route if its not the first iteration
		// this avoids putting 2 waypoints on top of each other...
		// ie. end of one route and start of next
		if (node_count > 0) {
			fast_route = fast_route.next as RouteNode;
			fast_route.prev = undefined;

			ASSERT(route_start !== undefined, "route_start");

			// link into list

			let route_end: RouteNode = route_start;

			while (route_end.next) {
				route_end = route_end.next;
			}

			route_end.next = fast_route;

			// set last waypoint type (of sub_route)

			route_end = fast_route;

			while (route_end.next) {
				route_end = route_end.next;
			}

			route_end.type = next_node.type;
			route_end.formation = next_node.formation;
		} else {
			// set 1st waypoint type
			fast_route.type = this_node.type;
			fast_route.formation = this_node.formation;

			route_start = fast_route;

			// set last waypoint type (of this sub_route)
			let route_end: RouteNode = route_start;

			while (route_end.next) {
				route_end = route_end.next;
			}

			route_end.type = next_node.type;
			route_end.formation = next_node.formation;
		}

		node_count++;

		this_node = next_node;
		next_node = next_node.next;
	}

	// a single point: *route = route_start = NULL (generate_route_check_sum then dereferences it)
	return route_start;
}

// C provenance: croute.c :: create_route: start and end (ceil), split until every leg is short enough
function createRoute(start: Vec3d, end: Vec3d, side: number, movement_type: number): RouteNode {
	const start_node = routeNode(EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, Math.ceil(start.x), Math.ceil(start.y), Math.ceil(start.z));

	const end_node = routeNode(EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, Math.ceil(end.x), Math.ceil(end.y), Math.ceil(end.z));

	start_node.next = end_node;
	end_node.prev = start_node;

	generateBestMidPoint(start_node, end_node, side, movement_type);

	return start_node;
}

// C provenance: croute.c :: generate_best_mid_point (recursive)
function generateBestMidPoint(start_node: RouteNode, end_node: RouteNode, side: number, movement_type: number): boolean {
	const best_point: Vec3d = { x: 0.0, y: 0.0, z: 0.0 };

	if (getBestPoint(start_node.position, end_node.position, best_point, side, movement_type)) {
		const new_node = routeNode(EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, Math.ceil(best_point.x), Math.ceil(best_point.y), Math.ceil(best_point.z));

		new_node.next = end_node;
		new_node.prev = start_node;

		start_node.next = new_node;
		end_node.prev = new_node;

		generateBestMidPoint(start_node, new_node, side, movement_type);

		generateBestMidPoint(new_node, end_node, side, movement_type);

		return true;
	}

	return false;
}

//
// C provenance: croute.c :: get_best_point
//
// For a leg longer than min_route_range (squared 2D range), the lowest rated
// of num_route_samples points on the perpendicular through its midpoint
// (spacing: the perpendicular / (samples * deviation)), bounded to the map;
// the first of equal ratings is kept. Writes *best_point only then.
//
function getBestPoint(start: Vec3d, end: Vec3d, best_point: Vec3d, side: number, movement_type: number): boolean {
	const map = getWorldMap();

	const num_route_samples = ROUTE_BIASING_NUM_ROUTE_SAMPLES[movement_type];

	// Quantize points

	const start_point: Vec3d = { x: Math.ceil(start.x), y: Math.ceil(start.y), z: Math.ceil(start.z) };

	const end_point: Vec3d = { x: Math.ceil(end.x), y: Math.ceil(end.y), z: Math.ceil(end.z) };

	const range = getSqr2dRange(start_point, end_point);

	const min_route_range = ROUTE_BIASING_MIN_ROUTE_RANGE[movement_type];

	if (range > f32Mul(min_route_range, min_route_range)) {
		const route_direction_x = f32Sub(end_point.x, start_point.x);
		const route_direction_z = f32Sub(end_point.z, start_point.z);

		const test_direction_x = route_direction_z;
		const test_direction_z = -route_direction_x;

		// test increments: float / (float * float)
		const divisor = f32Mul(num_route_samples, ROUTE_BIASING_ROUTE_DEVIATION_SIZE[movement_type]);

		const test_direction_x_inc = f32Div(test_direction_x, divisor);
		const test_direction_z_inc = f32Div(test_direction_z, divisor);

		// route mid point: float + (float * 0.5), a double sum stored to float
		const mid_point_x = f32Add(start_point.x, route_direction_x * 0.5);
		const mid_point_z = f32Add(start_point.z, route_direction_z * 0.5);

		// test point start: float - float * (float * 0.5), a double difference stored to float
		start_point.x = f32Add(mid_point_x, -(test_direction_x_inc * (num_route_samples * 0.5)));
		start_point.z = f32Add(mid_point_z, -(test_direction_z_inc * (num_route_samples * 0.5)));

		start_point.x = bound(start_point.x, map.min_map_x, map.max_map_x);
		start_point.y = 0.0;
		start_point.z = bound(start_point.z, map.min_map_z, map.max_map_z);

		// get terrain elevations across sample area
		const test_point: Vec3d = { x: Math.ceil(start_point.x), y: 0.0, z: Math.ceil(start_point.z) };

		for (let loop = 1; loop <= num_route_samples; loop++) {
			ASSERT(pointInsideMapArea(test_point), "point_inside_map_area(&test_point)");

			best_point_terrain_elevations[loop] = Math.ceil(get3dTerrainElevation(test_point.x, test_point.z));

			// next test point
			test_point.x = f32Add(test_point.x, test_direction_x_inc);
			test_point.z = f32Add(test_point.z, test_direction_z_inc);

			test_point.x = bound(Math.ceil(test_point.x), map.min_map_x, map.max_map_x);
			test_point.z = bound(Math.ceil(test_point.z), map.min_map_z, map.max_map_z);
		}

		// test point start
		test_point.x = Math.ceil(start_point.x);
		test_point.y = 0.0;
		test_point.z = Math.ceil(start_point.z);

		// best so far
		let best_point_rating = getRoutePointRating(1.0, test_point, side, movement_type);

		best_point.x = test_point.x;
		best_point.y = test_point.y;
		best_point.z = test_point.z;

		// test points along perpendicular line (float loop)
		for (let loop = 1.0; loop < num_route_samples; loop++) {
			// next test point
			test_point.x = f32Add(test_point.x, test_direction_x_inc);
			test_point.z = f32Add(test_point.z, test_direction_z_inc);

			test_point.x = bound(Math.ceil(test_point.x), map.min_map_x, map.max_map_x);
			test_point.z = bound(Math.ceil(test_point.z), map.min_map_z, map.max_map_z);

			// best so far (loop + 1.0: a whole number, exact)
			const point_rating = getRoutePointRating(loop + 1.0, test_point, side, movement_type);

			if (point_rating < best_point_rating) {
				best_point_rating = point_rating;

				best_point.x = test_point.x;
				best_point.y = test_point.y;
				best_point.z = test_point.z;
			}
		}

		return true;
	}

	return false;
}

//
// C provenance: croute.c :: second_past_route
//
// Re-places every interior node between its (already moved) predecessor and
// its successor. best_point is one local for the whole walk: a FALSE from
// get_best_point leaves the previous iteration's point, and on the first
// iteration leaves it uninitialised. Exported (as generate_route_check_sum and
// the parser are) so that a hand-built route can pin that undefined behaviour:
// the search itself reaches it only where map bounds break the perpendicular
// bisector's symmetry.
//
export function secondPastRoute(route: RouteNode, side: number, movement_type: number): void {
	const best_point: Vec3d = { x: 0.0, y: 0.0, z: 0.0 };

	let best_point_initialised = false;

	let prev_node = route;

	let node = prev_node.next as RouteNode;

	let next_node = node.next;

	while (next_node) {
		if (getBestPoint(prev_node.position, next_node.position, best_point, side, movement_type)) {
			best_point_initialised = true;
		}

		if (!best_point_initialised) {
			throw new EechUndefinedBehaviourError("croute.c :: second_past_route: best_point read uninitialised (get_best_point FALSE on the first iteration)");
		}

		node.position.x = Math.ceil(best_point.x);
		node.position.y = Math.ceil(best_point.y);
		node.position.z = Math.ceil(best_point.z);

		prev_node = node;

		node = prev_node.next as RouteNode;

		next_node = node.next;
	}
}

//
// C provenance: croute.c :: get_route_point_rating
//
// elevation_bias x the 3-sample average elevation (at least 0), plus
// range_bias x |2s - n| / 2n and side_bias x (sector side != side), each
// scaled by max (average, 1.0).
//
function getRoutePointRating(sample_number: number, test_point: Vec3d, side: number, movement_type: number): number {
	const num_route_samples = ROUTE_BIASING_NUM_ROUTE_SAMPLES[movement_type];

	// minimum height of 1.0m to allow biasing (float sums, left to right)
	let average_terrain_elevation = f32Add(
		f32Add(best_point_terrain_elevations[toCInt(sample_number)], best_point_terrain_elevations[toCInt(max(sample_number - 1.0, 1.0))]),
		best_point_terrain_elevations[toCInt(min(sample_number + 1.0, num_route_samples))],
	);

	// average /= 3.0: a double quotient stored to float
	average_terrain_elevation = f32Div(average_terrain_elevation, 3.0);

	average_terrain_elevation = max(average_terrain_elevation, 0.0);

	const elevation = f32Mul(ROUTE_BIASING_ELEVATION_BIAS[movement_type], average_terrain_elevation);

	// range_bias * fabs ((2.0 * s) - n) / (n * 2.0) in double, stored to float: the
	// product of the float bias and a whole number up to n is exact, and so is n * 2.0
	const range_bias = f32Div(ROUTE_BIASING_RANGE_BIAS[movement_type] * Math.abs(2.0 * sample_number - num_route_samples), num_route_samples * 2.0);

	const sector_en = getLocalSectorEntity(test_point);

	const sector_side = getLocalEntityIntValue(sector_en, IntType.INT_TYPE_SECTOR_SIDE);

	const side_bias = f32Mul(ROUTE_BIASING_SIDE_BIAS[movement_type], sector_side !== side ? 1 : 0);

	const scale = max(average_terrain_elevation, 1.0);

	return f32Add(f32Add(elevation, f32Mul(range_bias, scale)), f32Mul(side_bias, scale));
}

//
// C provenance: croute.c :: optimise_route
//
// Removes every interior node with a zero-length leg on either side, or whose
// unit legs (get_inverse_square_root) have |dot product| > optimise_tolerance.
//
function optimiseRoute(first: RouteNode, movement_type: number): void {
	let node = first.next as RouteNode;

	let prev_node = node.prev as RouteNode;

	let next_node = node.next;

	while (next_node) {
		let remove_node = false;

		const vec1: Vec3d = { x: f32Sub(node.position.x, prev_node.position.x), y: 0.0, z: f32Sub(node.position.z, prev_node.position.z) };

		if (checkZero3dVector(vec1)) {
			remove_node = true;
		} else {
			const vec2: Vec3d = { x: f32Sub(next_node.position.x, node.position.x), y: 0.0, z: f32Sub(next_node.position.z, node.position.z) };

			if (checkZero3dVector(vec2)) {
				remove_node = true;
			} else {
				let square = getInverseSquareRoot(f32Add(f32Mul(vec1.x, vec1.x), f32Mul(vec1.z, vec1.z)));
				vec1.x = f32Mul(vec1.x, square);
				vec1.z = f32Mul(vec1.z, square);

				square = getInverseSquareRoot(f32Add(f32Mul(vec2.x, vec2.x), f32Mul(vec2.z, vec2.z)));
				vec2.x = f32Mul(vec2.x, square);
				vec2.z = f32Mul(vec2.z, square);

				const angle = f32Add(f32Mul(vec1.x, vec2.x), f32Mul(vec1.z, vec2.z));

				if (Math.abs(angle) > ROUTE_BIASING_OPTIMISE_TOLERANCE[movement_type]) {
					remove_node = true;
				}
			}
		}

		if (remove_node) {
			// delete node

			prev_node.next = next_node;

			next_node.prev = prev_node;

			node = next_node;

			next_node = next_node.next;
		} else {
			// advance links

			prev_node = prev_node.next as RouteNode;

			node = prev_node.next as RouteNode;

			next_node = node.next;
		}
	}
}

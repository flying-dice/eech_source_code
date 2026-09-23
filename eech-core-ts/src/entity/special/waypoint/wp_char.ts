//
// Waypoint tags: the letters the map and planner show for a route.
//
// C provenance: entity/special/waypoint/wp_char.c :: reset_waypoint_tags,
//               update_local_entity_waypoint_list_tags, get_waypoint_sub_type_tag
//
// Navigation waypoints take A to V (wrapping to A), target and action
// waypoints X to Z (wrapping to X). The landing, taxi and similar types set
// the target counter to 'W' and take 'W' themselves, so a target waypoint
// after one of them is tagged 'W' too. The counters are file statics that
// every list update resets. Tags are C chars; the port holds their codes.
//

import { ASSERT, EechFatalError } from "../../../core/assert";
import { CharType, EntitySubTypeWaypoint, IntType, ListType } from "../../../generated/c-enums";
import { getLocalEntityChildSucc, getLocalEntityFirstChild } from "../../system/en_list";
import { getLocalEntityIntValue, setLocalEntityCharValue } from "../../system/en_values";
import type { Entity } from "../../system/entity";

const CHAR_A = 65;
const CHAR_V = 86;
const CHAR_W = 87;
const CHAR_X = 88;
const CHAR_Z = 90;

// C: static char nav_tag, target_tag (static storage: 0)
let nav_tag = 0;
let target_tag = 0;

// C provenance: wp_char.c :: reset_waypoint_tags
export function resetWaypointTags(): void {
	nav_tag = CHAR_A;
	target_tag = CHAR_X;
}

// C provenance: wp_char.c :: update_local_entity_waypoint_list_tags
export function updateLocalEntityWaypointListTags(parent: Entity | undefined): void {
	ASSERT(parent !== undefined, "parent");

	resetWaypointTags();

	let en = getLocalEntityFirstChild(parent, ListType.LIST_TYPE_WAYPOINT);

	while (en) {
		const sub_type = getLocalEntityIntValue(en, IntType.INT_TYPE_ENTITY_SUB_TYPE);

		const letter = getWaypointSubTypeTag(sub_type);

		setLocalEntityCharValue(en, CharType.CHAR_TYPE_TAG, letter);

		en = getLocalEntityChildSucc(en, ListType.LIST_TYPE_WAYPOINT);
	}
}

// C provenance: wp_char.c :: get_waypoint_sub_type_tag
export function getWaypointSubTypeTag(sub_type: EntitySubTypeWaypoint): number {
	switch (sub_type) {
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_CAP_START:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_END:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION: {
			nav_tag = nav_tag <= CHAR_V ? nav_tag : CHAR_A;

			// return nav_tag ++
			const tag = nav_tag;

			nav_tag++;

			return tag;
		}
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_ATTACK:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_CAP_LOOP:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_DEFEND:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_ESCORT:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_IMPOSSIBLE:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_RECON:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_REPAIR:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TARGET:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TROOP_CAPTURE:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TROOP_DEFEND:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TROOP_INSERT:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TROOP_PICKUP_POINT_END:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TROOP_PICKUP_POINT_START:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TROOP_PUTDOWN_POINT:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_WAIT:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_INSERTION: {
			target_tag = target_tag <= CHAR_Z ? target_tag : CHAR_X;

			// return target_tag ++
			const tag = target_tag;

			target_tag++;

			return tag;
		}
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_APPROACH:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_CONVOY:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_HOLDING:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_HOLDING_LOOP:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_LAND:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_LANDED:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_LIFT_OFF:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_LOWER_UNDERCARRIAGE:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_REVERSE_CONVOY:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_RAISE_UNDERCARRIAGE:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_START_UP:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_SUB_ROUTE_NAVIGATION:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TAXI:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TOUCH_DOWN:
		case EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_TAKEN_OFF: {
			target_tag = CHAR_W;

			return target_tag;
		}
	}

	throw new EechFatalError("Invalid waypoint entity sub-type = %d", `Invalid waypoint entity sub-type = ${sub_type}`);
}

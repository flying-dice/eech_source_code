//
// Waypoint entity creation.
//
// C provenance: entity/special/waypoint/wp_creat.c :: create_local,
//               overload_waypoint_create_functions
//
// croute.c creates route waypoints locally (create_local_entity), never
// through the client-server create: clients rebuild the route from
// ENTITY_COMMS_CREATE_WAYPOINT_ROUTE. A new waypoint joins its dependent's
// LIST_TYPE_TASK_DEPENDENT list (when that attribute is given) and its task's
// LIST_TYPE_WAYPOINT list after the given predecessor. The sector list is
// joined only under DEBUG_MODULE.
//
// A position outside the map volume is replaced from the task's position and
// heading in the C; route waypoints are bounded into the map before they are
// created, so that repair is not ported and fails loudly.
//

import { ASSERT, UnportedBehaviourError } from "../../../core/assert";
import { EntitySubTypeWaypoint, EntityType, FormationType, ListType, PositionType, Vec3dType } from "../../../generated/c-enums";
import { setLocalEntityAttributes, type EntityAttribute } from "../../system/en_attrs";
import { fnCreateLocalEntity, validateLocalCreateEntityIndex } from "../../system/en_creat";
import { getFreeEntity } from "../../system/en_heap";
import { getLocalEntityChildPred, getLocalEntityParent, insertLocalEntityIntoParentsChildList } from "../../system/en_list";
import { getLocalEntityVec3d } from "../../system/en_values";
import { getWorldMap, pointInsideMapVolume } from "../../system/en_world";
import { setLocalEntityData, setLocalEntityType, type Entity } from "../../system/entity";
import { clearedWaypointRaw } from "./waypoint";

// C provenance: wp_creat.c :: create_local
function createLocal(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateLocalCreateEntityIndex(index);

	const en = getFreeEntity(index);

	if (en) {
		setLocalEntityType(en, type);

		// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES (after memset (raw, 0, sizeof (waypoint)))
		const raw = clearedWaypointRaw();

		const map = getWorldMap();

		raw.position.x = map.mid_map_x;
		raw.position.z = map.mid_map_z;
		raw.position.y = map.mid_map_y;

		raw.sub_type = EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION;

		raw.waypoint_formation = FormationType.FORMATION_ROW_LEFT;

		raw.position_type = PositionType.POSITION_TYPE_ABSOLUTE;

		raw.heading = 0.0;

		setLocalEntityData(en, raw);

		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		setLocalEntityAttributes(en, attributes);

		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		const task = getLocalEntityParent(en, ListType.LIST_TYPE_WAYPOINT);

		ASSERT(task !== undefined, "raw->waypoint_link.parent");

		const v = getLocalEntityVec3d(en, Vec3dType.VEC3D_TYPE_POSITION);

		if (!pointInsideMapVolume(v)) {
			throw new UnportedBehaviourError("wp_creat.c :: create_local: a waypoint outside the map volume (repositioned from its task's position and heading)");
		}

		// LINK INTO SYSTEM
		const dependent = getLocalEntityParent(en, ListType.LIST_TYPE_TASK_DEPENDENT);

		if (dependent) {
			insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_TASK_DEPENDENT, dependent, undefined);
		}

		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_WAYPOINT, task, getLocalEntityChildPred(en, ListType.LIST_TYPE_WAYPOINT));
	}

	return en;
}

// C provenance: wp_creat.c :: overload_waypoint_create_functions (local create)
export function overloadWaypointCreateFunctions(): void {
	fnCreateLocalEntity.overload(EntityType.ENTITY_TYPE_WAYPOINT, 0, createLocal);
}

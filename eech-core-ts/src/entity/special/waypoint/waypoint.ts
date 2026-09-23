//
// Waypoint entity: the state a restored route waypoint holds, as far as
// Slice 5a reads it.
//
// C provenance: entity/special/waypoint/wp_int.c, wp_list.c (and en_float.c's
// default getter, which wp_float.c leaves in place for FLOAT_TYPE_TASK_USER_DATA)
//
// croute.c links each route waypoint into its route dependent's
// LIST_TYPE_TASK_DEPENDENT list, beside the tasks whose objective that entity
// is. response_to_force_low_on_supplies walks that list; entity_is_object_of_task
// skips anything that is not a task, but the walk after it does not.
//

import { EntitySubTypeWaypoint, EntityType, FloatType, IntType, ListType } from "../../../generated/c-enums";
import { overloadEntityListLink } from "../../system/en_list";
import { defaultGetEntityFloatValue, fnGetLocalEntityFloatValue, fnGetLocalEntityIntValue } from "../../system/en_values";
import { getLocalEntityData } from "../../system/entity";

// C provenance: waypoint.h :: struct WAYPOINT (ported fields only)
export interface WaypointRaw {
	sub_type: EntitySubTypeWaypoint;
}

// C provenance: wp_funcs.c :: overload_waypoint_functions (ported subset)
export function overloadWaypointFunctions(): void {
	const WAYPOINT = EntityType.ENTITY_TYPE_WAYPOINT;

	// C provenance: wp_list.c :: LIST_TYPE_TASK_DEPENDENT_LINK
	overloadEntityListLink(WAYPOINT, "task_dependent_link", [ListType.LIST_TYPE_TASK_DEPENDENT]);

	// C provenance: wp_int.c :: get_local_int_value
	fnGetLocalEntityIntValue.overload(WAYPOINT, IntType.INT_TYPE_ENTITY_SUB_TYPE, (en) => getLocalEntityData<WaypointRaw>(en).sub_type);

	// C provenance: en_float.c :: initialise_entity_float_value_default_functions;
	// wp_float.c overloads only altitude, flight time and heading
	fnGetLocalEntityFloatValue.overload(WAYPOINT, FloatType.FLOAT_TYPE_TASK_USER_DATA, defaultGetEntityFloatValue);
}

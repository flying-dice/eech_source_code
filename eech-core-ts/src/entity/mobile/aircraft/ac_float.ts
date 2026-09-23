//
// Aircraft float values read by task assignment.
//
// C provenance: entity/mobile/aircraft/ac_float.c :: get_local_float_value,
//               overload_aircraft_float_value_functions (ported rows only)
//
// Slice 6a (issue #16): assign.c reads a group member's cruise velocity
// (group.c :: assess_group_task_locality_factor) and sleep timer (assign.c ::
// check_group_members_awake). No aircraft file (ac_float.c, hc_float.c,
// fw_float.c) overloads FLOAT_TYPE_SLEEP, so en_float.c's default (0.0)
// answers it: an aircraft member is always awake.
//

import { AIRCRAFT_DATABASE_CRUISE_VELOCITY } from "../../../generated/c-aircraft-database";
import { FloatType, type EntitySubTypeAircraft, type EntityType } from "../../../generated/c-enums";
import { defaultGetEntityFloatValue, fnGetLocalEntityFloatValue } from "../../system/en_values";
import { getLocalEntityData } from "../../system/entity";

// C provenance: aircraft.h :: struct AIRCRAFT (ported fields only: mob.sub_type). The
// position is physical state, read through the MobilePhysicalState port (mobile.ts).
export interface AircraftRaw {
	mob: { sub_type: EntitySubTypeAircraft };
}

export function overloadAircraftFloatValueFunctions(type: EntityType): void {
	// C provenance: ac_float.c :: get_local_float_value (FLOAT_TYPE_CRUISE_VELOCITY)
	fnGetLocalEntityFloatValue.overload(type, FloatType.FLOAT_TYPE_CRUISE_VELOCITY, (en) => AIRCRAFT_DATABASE_CRUISE_VELOCITY[getLocalEntityData<AircraftRaw>(en).mob.sub_type]);

	// C provenance: en_float.c :: default_get_entity_float_value (0.0): no aircraft file overloads FLOAT_TYPE_SLEEP
	fnGetLocalEntityFloatValue.overload(type, FloatType.FLOAT_TYPE_SLEEP, defaultGetEntityFloatValue);
}

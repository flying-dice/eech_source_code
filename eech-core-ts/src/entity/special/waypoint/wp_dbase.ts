//
// Waypoint database accessors.
//
// C provenance: entity/special/waypoint/wp_dbase.c :: get_waypoint_database_*_value,
//               get_waypoint_database_*_flag, get_waypoint_database_minimum_previous_waypoint_distance
//
// Each accessor picks the fixed wing, helicopter, ground (routed vehicle,
// anti-aircraft, person) or ship column by the mobile's entity type; any other
// entity type is debug_fatal ("WP_DBASE : Invalid entity type (%s)").
//

import { EechFatalError } from "../../../core/assert";
import { EntityType } from "../../../generated/c-enums";
import {
	WAYPOINT_DATABASE_CRITERIA_LAST_TO_REACH,
	WAYPOINT_DATABASE_CRITERIA_TRANSMIT_RECON,
	WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE,
	WAYPOINT_DATABASE_POSITION_TYPE,
	WAYPOINT_DATABASE_REACHED_RADIUS,
	WAYPOINT_DATABASE_VELOCITY,
} from "../../../generated/c-waypoint-database";

// the switch (mobile_type) of every accessor: the column block of c-waypoint-database.ts
function mobileBlock(mobile_type: EntityType): number {
	switch (mobile_type) {
		case EntityType.ENTITY_TYPE_FIXED_WING: {
			return 0;
		}
		case EntityType.ENTITY_TYPE_HELICOPTER: {
			return 1;
		}
		case EntityType.ENTITY_TYPE_ROUTED_VEHICLE:
		case EntityType.ENTITY_TYPE_ANTI_AIRCRAFT:
		case EntityType.ENTITY_TYPE_PERSON: {
			return 2;
		}
		case EntityType.ENTITY_TYPE_SHIP_VEHICLE: {
			return 3;
		}
	}

	throw new EechFatalError("WP_DBASE : Invalid entity type (%s)", `WP_DBASE : Invalid entity type (${EntityType[mobile_type]})`);
}

// C provenance: wp_dbase.c :: get_waypoint_database_reached_radius_value
export function getWaypointDatabaseReachedRadiusValue(waypoint_type: number, mobile_type: EntityType): number {
	return WAYPOINT_DATABASE_REACHED_RADIUS[waypoint_type][mobileBlock(mobile_type)];
}

// C provenance: wp_dbase.c :: get_waypoint_database_velocity_value
export function getWaypointDatabaseVelocityValue(waypoint_type: number, mobile_type: EntityType): number {
	return WAYPOINT_DATABASE_VELOCITY[waypoint_type][mobileBlock(mobile_type)];
}

// C provenance: wp_dbase.c :: get_waypoint_database_transmit_recon_flag
export function getWaypointDatabaseTransmitReconFlag(waypoint_type: number, mobile_type: EntityType): number {
	return WAYPOINT_DATABASE_CRITERIA_TRANSMIT_RECON[waypoint_type][mobileBlock(mobile_type)];
}

// C provenance: wp_dbase.c :: get_waypoint_database_last_to_reach_flag
export function getWaypointDatabaseLastToReachFlag(waypoint_type: number, mobile_type: EntityType): number {
	return WAYPOINT_DATABASE_CRITERIA_LAST_TO_REACH[waypoint_type][mobileBlock(mobile_type)];
}

// C provenance: wp_dbase.c :: get_waypoint_database_position_type_value
export function getWaypointDatabasePositionTypeValue(waypoint_type: number, mobile_type: EntityType): number {
	return WAYPOINT_DATABASE_POSITION_TYPE[waypoint_type][mobileBlock(mobile_type)];
}

// C provenance: wp_dbase.c :: get_waypoint_database_minimum_previous_waypoint_distance
export function getWaypointDatabaseMinimumPreviousWaypointDistance(waypoint_type: number, mobile_type: EntityType): number {
	return WAYPOINT_DATABASE_MINIMUM_PREVIOUS_WAYPOINT_DISTANCE[waypoint_type][mobileBlock(mobile_type)];
}



































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	vehicle
		*raw;







	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_DEATH_TIMER:

		{
			raw->death_timer = value;

			break;
		}

		case FLOAT_TYPE_DISTANCE:

		{
			raw->distance = value;

			break;
		}

		case FLOAT_TYPE_WEAPON_TO_INTERCEPT_POINT_RANGE:

		{
			raw->weapon_to_intercept_point_range = value;

			break;
		}

		case FLOAT_TYPE_LOADING_DOOR_STATE:

		{
			raw->loading_door_state = value;

			break;
		}

		case FLOAT_TYPE_RADAR_ROTATION_STATE:

		{
			raw->radar_rotation_state = value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			raw->sleep = value;

			break;
		}

		case FLOAT_TYPE_TARGET_FIRE_TIMER:

		{
			raw->target_fire_timer = value;

			break;
		}

		case FLOAT_TYPE_TARGET_SCAN_TIMER:

		{
			raw->target_scan_timer = value;

			break;
		}

		case FLOAT_TYPE_VIEW_INTEREST_LEVEL:

		{
			raw->view_interest_level = value;

			break;
		}

		case FLOAT_TYPE_WEAPON_BURST_TIMER:

		{
			raw->weapon_burst_timer = value;

			break;
		}

		case FLOAT_TYPE_WEAPON_SYSTEM_READY_STATE:

		{
			raw->weapon_system_ready_state = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}

#endif



































































#if GET_LOCAL_REQUIRED

static float get_local_float_value (entity *en, float_types type)
{
	vehicle
		*raw;

	float
		value;

	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AIR_SCAN_CEILING:

		{
			value = vehicle_database[raw->mob.sub_type].air_scan_ceiling;

			break;
		}

		case FLOAT_TYPE_AIR_SCAN_FLOOR:

		{
			value = vehicle_database[raw->mob.sub_type].air_scan_floor;

			break;
		}

		case FLOAT_TYPE_AIR_SCAN_RANGE:

		{
			value = vehicle_database[raw->mob.sub_type].air_scan_range;

			break;
		}

		case FLOAT_TYPE_AMMO_ECONOMY:

		{
			value = vehicle_database[raw->mob.sub_type].ammo_economy;

			break;
		}

		case FLOAT_TYPE_AXLE_LENGTH:

		{
			value = vehicle_database[raw->mob.sub_type].axle_length;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MAX_DISTANCE:

		{
			value = vehicle_database[raw->mob.sub_type].chase_view_max_distance;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MAX_DISTANCE_TEST:

		{
			value = vehicle_database[raw->mob.sub_type].chase_view_max_distance_test;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MIN_DISTANCE:

		{
			value = vehicle_database[raw->mob.sub_type].chase_view_min_distance;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MIN_DISTANCE_TEST:

		{
			value = vehicle_database[raw->mob.sub_type].chase_view_min_distance_test;

			break;
		}

		case FLOAT_TYPE_CRUISE_VELOCITY:

		{
			value = vehicle_database[raw->mob.sub_type].cruise_velocity;

			break;
		}

		case FLOAT_TYPE_DEATH_TIMER:

		{
			value = raw->death_timer;

			break;
		}

		case FLOAT_TYPE_DISTANCE:

		{
			value = raw->distance;

			break;
		}

		case FLOAT_TYPE_WEAPON_TO_INTERCEPT_POINT_RANGE:

		{
			value = raw->weapon_to_intercept_point_range;

			break;
		}

		case FLOAT_TYPE_FUEL_ECONOMY:

		{
			value = vehicle_database[raw->mob.sub_type].fuel_economy;

			break;
		}

		case FLOAT_TYPE_LOADING_DOOR_STATE:

		{
			value = raw->loading_door_state;

			break;
		}

		case FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE:

		{
			value = object_3d_information_database[raw->object_3d_shape].maximum_distance;

			break;
		}

		case FLOAT_TYPE_MAX_ACCELERATION:

		{
			value = vehicle_database[raw->mob.sub_type].max_acceleration;

			break;
		}

		case FLOAT_TYPE_MAX_TURN_RATE:

		{
			value = vehicle_database[raw->mob.sub_type].max_turn_rate;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_AIR_THREAT:

		{
			value = vehicle_database[raw->mob.sub_type].potential_surface_to_air_threat;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_SURFACE_THREAT:

		{
			value = vehicle_database[raw->mob.sub_type].potential_surface_to_surface_threat;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_SURFACE_TO_AIR_THREAT:

		{
			value = vehicle_database[raw->mob.sub_type].potential_surface_to_air_threat;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_SURFACE_TO_SURFACE_THREAT:

		{
			value = vehicle_database[raw->mob.sub_type].potential_surface_to_surface_threat;

			break;
		}

		case FLOAT_TYPE_POWER_OUTPUT:

		{
			value = vehicle_database[raw->mob.sub_type].power_output;

			break;
		}

		case FLOAT_TYPE_RADAR_ROTATION_STATE:

		{
			value = raw->radar_rotation_state;

			break;
		}

		case FLOAT_TYPE_RECON_RADIUS:

		{
			value = vehicle_database[raw->mob.sub_type].recon_radius;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			value = raw->sleep;

			break;
		}

		case FLOAT_TYPE_SURFACE_SCAN_RANGE:

		{
			value = vehicle_database[raw->mob.sub_type].surface_scan_range;

			break;
		}

		case FLOAT_TYPE_TARGET_FIRE_TIMER:

		{
			value = raw->target_fire_timer;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK:

		{
			value = vehicle_database[raw->mob.sub_type].potential_surface_to_air_threat;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK:

		{
			value = vehicle_database[raw->mob.sub_type].potential_surface_to_surface_threat;

			break;
		}

		case FLOAT_TYPE_TARGET_SCAN_DELAY:

		{
			value = vehicle_database[raw->mob.sub_type].target_scan_delay;

			break;
		}

		case FLOAT_TYPE_TARGET_SCAN_TIMER:

		{
			value = raw->target_scan_timer;

			break;
		}

		case FLOAT_TYPE_TERRAIN_ELEVATION:

		{
			value = get_3d_terrain_point_data_elevation (&raw->terrain_info);

			break;
		}

		case FLOAT_TYPE_VIEW_INTEREST_LEVEL:

		{
			value = raw->view_interest_level;

			break;
		}

		case FLOAT_TYPE_WEAPON_BURST_TIMER:

		{
			value = raw->weapon_burst_timer;

			break;
		}

		case FLOAT_TYPE_WEAPON_SYSTEM_READY_STATE:

		{
			value = raw->weapon_system_ready_state;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}

#endif
























































































































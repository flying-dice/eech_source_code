

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	aircraft
		*raw;







	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AIR_RADAR_CONTACT_TIMEOUT:

		{
			raw->air_radar_contact_timeout = value;

			break;
		}

		case FLOAT_TYPE_CARGO_DOOR_STATE:

		{
			raw->cargo_door_state = value;

			break;
		}

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

		case FLOAT_TYPE_RUDDER_STATE:

		{
			raw->rudder_state = value;

			break;
		}

		case FLOAT_TYPE_UNDERCARRIAGE_STATE:

		{
			raw->undercarriage_state = value;

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

		case FLOAT_TYPE_WEAPON_LAUNCH_DELAY:

		{
			raw->weapon_launch_delay = value;

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
	aircraft
		*raw;

	float
		value;

	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AIR_RADAR_CONTACT_TIMEOUT:

		{
			value = raw->air_radar_contact_timeout;

			break;
		}

		case FLOAT_TYPE_AIR_SCAN_RANGE:

		{
			value = aircraft_database[raw->mob.sub_type].air_scan_range;

			break;
		}

		case FLOAT_TYPE_AMMO_ECONOMY:

		{
			value = aircraft_database[raw->mob.sub_type].ammo_economy;

			break;
		}

		case FLOAT_TYPE_ATTACK_ALTITUDE:

		{
			value = aircraft_database[raw->mob.sub_type].attack_altitude;

			break;
		}

		case FLOAT_TYPE_AVOIDANCE_RADIUS:

		{
			value = aircraft_database[raw->mob.sub_type].avoidance_radius;

			break;
		}

		case FLOAT_TYPE_CARGO_DOOR_STATE:

		{
			value = raw->cargo_door_state;

			break;
		}

		case FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE:

		{
			if (raw->mob.alive)
			{
				value = aircraft_database[raw->mob.sub_type].centre_of_gravity_to_ground_distance;
			}
			else
			{
				value = aircraft_database[raw->mob.sub_type].centre_of_gravity_to_ground_distance_destroyed;
			}

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MAX_DISTANCE:

		{
			value = aircraft_database[raw->mob.sub_type].chase_view_max_distance;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MAX_DISTANCE_TEST:

		{
			value = aircraft_database[raw->mob.sub_type].chase_view_max_distance_test;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MIN_DISTANCE:

		{
			value = aircraft_database[raw->mob.sub_type].chase_view_min_distance;

			break;
		}

		case FLOAT_TYPE_CHASE_VIEW_MIN_DISTANCE_TEST:

		{
			value = aircraft_database[raw->mob.sub_type].chase_view_min_distance_test;

			break;
		}

		case FLOAT_TYPE_CRUISE_ALTITUDE:

		{
			value = aircraft_database[raw->mob.sub_type].cruise_altitude;

			break;
		}

		case FLOAT_TYPE_CRUISE_VELOCITY:

		{
			value = aircraft_database[raw->mob.sub_type].cruise_velocity;

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

		case FLOAT_TYPE_FUEL_DEFAULT_WEIGHT:

		{
			value = aircraft_database[raw->mob.sub_type].fuel_default_weight;

			break;
		}

		case FLOAT_TYPE_FUEL_ECONOMY:

		{
			value = aircraft_database[raw->mob.sub_type].fuel_economy;

			break;
		}

		case FLOAT_TYPE_FUSELAGE_ANGLE:

		{
			value = aircraft_database[raw->mob.sub_type].fuselage_angle;

			break;
		}

		case FLOAT_TYPE_LOADING_DOOR_STATE:

		{
			value = raw->loading_door_state;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_DIRECTION:

		{
			value = aircraft_database[raw->mob.sub_type].main_rotor_direction;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_BLADE_DROOP_ANGLE:

		{
			value = aircraft_database[raw->mob.sub_type].main_rotor_blade_droop_angle;

			break;
		}

		// Xhit: added FLOAT_TYPE_MAIN_ROTOR_RADIUS for downwash effect (030328)
		case FLOAT_TYPE_MAIN_ROTOR_RADIUS:

		{
			value = aircraft_database[raw->mob.sub_type].main_rotor_radius;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_SHAFT_ANGLE:

		{
			value = aircraft_database[raw->mob.sub_type].main_rotor_shaft_angle;

			break;
		}

		case FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE:

		{
			value = object_3d_information_database[raw->object_3d_shape].maximum_distance;

			break;
		}

		case FLOAT_TYPE_MAX_TURN_RATE:

		{
			value = aircraft_database[raw->mob.sub_type].max_turn_rate;

			break;
		}

		case FLOAT_TYPE_MAX_ROLL:

		{
			value = aircraft_database[raw->mob.sub_type].max_roll;

			break;
		}
		// Xhit: added FLOAT_TYPE_NUMBER_OF_MAIN_ROTORS for downwash effect (030328)
		case FLOAT_TYPE_NUMBER_OF_MAIN_ROTORS:

		{
			value = aircraft_database[raw->mob.sub_type].number_of_main_rotors;

			break;
		}

		case FLOAT_TYPE_MAX_VERTICAL_TURN_ALLOWANCE:

		{
			value = aircraft_database[raw->mob.sub_type].max_vertical_turn_allowance;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_AIR_THREAT:

		{
			value = aircraft_database[raw->mob.sub_type].potential_air_to_air_threat;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_AIR_TO_AIR_THREAT:

		{
			value = aircraft_database[raw->mob.sub_type].potential_air_to_air_threat;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_AIR_TO_SURFACE_THREAT:

		{
			value = aircraft_database[raw->mob.sub_type].potential_air_to_surface_threat;

			break;
		}

		case FLOAT_TYPE_POTENTIAL_SURFACE_THREAT:

		{
			value = aircraft_database[raw->mob.sub_type].potential_air_to_surface_threat;

			break;
		}

		case FLOAT_TYPE_PROPELLOR_DIRECTION:

		{
			value = aircraft_database[raw->mob.sub_type].propellor_direction;

			break;
		}

		case FLOAT_TYPE_RADAR_ALTITUDE:

		{
			value = max (raw->mob.position.y - get_3d_terrain_point_data_elevation (&raw->terrain_info), 0.0f);

			break;
		}

		case FLOAT_TYPE_RECON_RADIUS:

		{
			value = aircraft_database[raw->mob.sub_type].recon_radius;

			break;
		}

		case FLOAT_TYPE_RUDDER_STATE:

		{
			value = raw->rudder_state;

			break;
		}

		case FLOAT_TYPE_SURFACE_SCAN_RANGE:

		{
			value = aircraft_database[raw->mob.sub_type].surface_scan_range;

			break;
		}

		case FLOAT_TYPE_TAIL_ROTOR_DIRECTION:

		{
			value = aircraft_database[raw->mob.sub_type].tail_rotor_direction;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK:

		{
			value = aircraft_database[raw->mob.sub_type].potential_air_to_air_threat;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK:

		{
			value = aircraft_database[raw->mob.sub_type].potential_air_to_surface_threat;

			break;
		}

		case FLOAT_TYPE_TARGET_SCAN_DELAY:

		{
			value = aircraft_database[raw->mob.sub_type].target_scan_delay;

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

		case FLOAT_TYPE_UNDERCARRIAGE_STATE:

		{
			value = raw->undercarriage_state;

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

		case FLOAT_TYPE_WEAPON_LAUNCH_DELAY:

		{
			value = raw->weapon_launch_delay;

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


























































































































































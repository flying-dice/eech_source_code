

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	helicopter
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{

			raw->fuel_supply_level = value;

			break;
		}

		case FLOAT_TYPE_INVULNERABLE_TIMER:

		{
			raw->invulnerable_timer = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE:

		{
			raw->main_rotor_blade_coning_angle = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_PITCH:

		{
			raw->main_rotor_pitch = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_ROLL:

		{
			raw->main_rotor_roll = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_RPM:

		{
			raw->main_rotor_rpm = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER:

		{
			raw->main_rotor_spin_up_timer = value;

			break;
		}

		case FLOAT_TYPE_TAIL_ROTOR_RPM:

		{
			raw->tail_rotor_rpm = value;

			break;
		}

		
		// Jabberwock 050310 MP Cannon bug
		case FLOAT_TYPE_PLAYER_WEAPON_HEADING:

		{
			raw->player_weapon_heading = value;

			break;
		}

		case FLOAT_TYPE_PLAYER_WEAPON_PITCH:

		{
			raw->player_weapon_pitch = value;

			break;
		}
		// Jabberwock 050310 ends

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}





static void set_remote_float_value (entity *en, float_types type, float value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}





static void set_server_float_value (entity *en, float_types type, float value)
{
	validate_client_server_local_fn ();

	set_local_float_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_float_value (en, type, value);
}





static void set_client_float_value (entity *en, float_types type, float value)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_float_value (en, type, value);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_float_value (en, type, value);
	}
}





static float get_local_float_value (entity *en, float_types type)
{
	helicopter
		*raw;

	float
		value;

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			value = raw->fuel_supply_level;

			break;
		}

		case FLOAT_TYPE_INVULNERABLE_TIMER:

		{
			value = raw->invulnerable_timer;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE:

		{
			value = raw->main_rotor_blade_coning_angle;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_PITCH:

		{
			value = raw->main_rotor_pitch;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_ROLL:

		{
			value = raw->main_rotor_roll;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_RPM:

		{
			value = raw->main_rotor_rpm;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER:

		{
			value = raw->main_rotor_spin_up_timer;

			break;
		}

		case FLOAT_TYPE_RADAR_STEALTH_FACTOR:

		{
			value = NO_RADAR_STEALTH_FACTOR;

			if (raw->ac.mob.sub_type == ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE)
				if (get_local_entity_weapon_system_ready_state (en) == WEAPON_SYSTEM_READY_CLOSED)
					value -= NO_RADAR_STEALTH_FACTOR * 0.4;

			if (!get_comanche_stub_wings_attached (en))
				value -= NO_RADAR_STEALTH_FACTOR * 0.35;
				
			if (get_local_entity_undercarriage_state (en) == AIRCRAFT_UNDERCARRIAGE_UP)
				value -= NO_RADAR_STEALTH_FACTOR * 0.1;

			ASSERT (value > 0.0);

			break;
		}

		case FLOAT_TYPE_TAIL_ROTOR_RPM:

		{
			value = raw->tail_rotor_rpm;

			break;
		}

		
		// Jabberwock 050310 MP Cannon bug
		case FLOAT_TYPE_PLAYER_WEAPON_HEADING:

		{
			value = raw->player_weapon_heading;

			break;
		}

		case FLOAT_TYPE_PLAYER_WEAPON_PITCH:

		{
			value = raw->player_weapon_pitch;

			break;
		}
		// Jabberwock 050310 ends

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}





void overload_helicopter_float_value_functions (void)
{
	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]					 		= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]						  	= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_INVULNERABLE_TIMER]					 		= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_INVULNERABLE_TIMER]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_INVULNERABLE_TIMER][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_INVULNERABLE_TIMER][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_INVULNERABLE_TIMER]						  	= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE]					 		= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE]						  	= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_PITCH]					 			= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_PITCH]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_PITCH][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_PITCH][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_PITCH]						  		= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_ROLL]					 			= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_ROLL]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_ROLL][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_ROLL][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_ROLL]						  		= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_RPM]				 				= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_RPM]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_RPM][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_RPM][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_RPM]					  			= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER]				 				= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER]					  			= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_RADAR_STEALTH_FACTOR]  						= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_TAIL_ROTOR_RPM]					 			= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_TAIL_ROTOR_RPM]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_TAIL_ROTOR_RPM][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_TAIL_ROTOR_RPM][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_TAIL_ROTOR_RPM]					  			= get_local_float_value;
	
	// Jabberwock 050310 MP Cannon bug
	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_HEADING]					 			= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_HEADING]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_HEADING][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_HEADING][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_HEADING]					  			= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_PITCH]					 			= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_PITCH]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_PITCH][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_PITCH][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_HELICOPTER][FLOAT_TYPE_PLAYER_WEAPON_PITCH]					  			= get_local_float_value;
	// Jabberwock 050310 ends
}





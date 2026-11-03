

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	mobile
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_HEADING:

		{

			matrix3x3
				attitude;

			float
				pitch,
				roll;

			pitch = get_pitch_from_attitude_matrix (raw->attitude);

			roll = get_roll_from_attitude_matrix (raw->attitude);

			get_3d_transformation_matrix (attitude, value, pitch, roll);

			set_local_entity_attitude_matrix (en, attitude);

			break;
		}

		case FLOAT_TYPE_HIGH_VELOCITY:
		case FLOAT_TYPE_LOW_VELOCITY:
		case FLOAT_TYPE_MEDIUM_VELOCITY:
		case FLOAT_TYPE_VELOCITY:
		case FLOAT_TYPE_VERY_HIGH_VELOCITY:

		{
			int update_smoke = FALSE;
			
			if(get_local_entity_int_value (en, INT_TYPE_IDENTIFY_AIRCRAFT))
				if (raw->velocity > AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD && value <= AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD ||
						raw->velocity <= AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD && value > AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD)
					update_smoke = TRUE;
			
			raw->velocity = value;

			if (update_smoke)
				switch_aircraft_damaged_smoke(en);

			break;
		}

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
	mobile
		*raw;

	float
		value;

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			value = raw->position.y;

			break;
		}

		case FLOAT_TYPE_HEADING:

		{
			value = get_heading_from_attitude_matrix (raw->attitude);

			break;
		}

		case FLOAT_TYPE_HIGH_VELOCITY:
		case FLOAT_TYPE_LOW_VELOCITY:
		case FLOAT_TYPE_MEDIUM_VELOCITY:
		case FLOAT_TYPE_VELOCITY:
		case FLOAT_TYPE_VERY_HIGH_VELOCITY:

		{
			value = raw->velocity;

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





void overload_mobile_float_value_functions (entity_types type)
{
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_ALTITUDE]											= get_local_float_value;

	fn_set_local_entity_float_value				[type][FLOAT_TYPE_HEADING]												= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_HEADING][COMMS_MODEL_SERVER]					= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_HEADING][COMMS_MODEL_CLIENT]					= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_HEADING]												= get_local_float_value;

	fn_set_local_entity_raw_float_value			[type][FLOAT_TYPE_HIGH_VELOCITY]											= set_local_float_value;
	fn_set_local_entity_float_value				[type][FLOAT_TYPE_HIGH_VELOCITY]											= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_HIGH_VELOCITY][COMMS_MODEL_SERVER]					= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_HIGH_VELOCITY][COMMS_MODEL_CLIENT]					= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_HIGH_VELOCITY]											= get_local_float_value;

	fn_set_local_entity_raw_float_value			[type][FLOAT_TYPE_LOW_VELOCITY]											= set_local_float_value;
	fn_set_local_entity_float_value				[type][FLOAT_TYPE_LOW_VELOCITY]											= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_LOW_VELOCITY][COMMS_MODEL_SERVER]					= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_LOW_VELOCITY][COMMS_MODEL_CLIENT]					= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_LOW_VELOCITY]											= get_local_float_value;

	fn_set_local_entity_raw_float_value			[type][FLOAT_TYPE_MEDIUM_VELOCITY]											= set_local_float_value;
	fn_set_local_entity_float_value				[type][FLOAT_TYPE_MEDIUM_VELOCITY]											= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_MEDIUM_VELOCITY][COMMS_MODEL_SERVER]					= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_MEDIUM_VELOCITY][COMMS_MODEL_CLIENT]					= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_MEDIUM_VELOCITY]											= get_local_float_value;

	fn_set_local_entity_raw_float_value			[type][FLOAT_TYPE_VELOCITY]											= set_local_float_value;
	fn_set_local_entity_float_value				[type][FLOAT_TYPE_VELOCITY]											= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_VELOCITY][COMMS_MODEL_SERVER]					= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_VELOCITY][COMMS_MODEL_CLIENT]					= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_VELOCITY]											= get_local_float_value;

	fn_set_local_entity_raw_float_value			[type][FLOAT_TYPE_VERY_HIGH_VELOCITY]											= set_local_float_value;
	fn_set_local_entity_float_value				[type][FLOAT_TYPE_VERY_HIGH_VELOCITY]											= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_VERY_HIGH_VELOCITY][COMMS_MODEL_SERVER]					= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_VERY_HIGH_VELOCITY][COMMS_MODEL_CLIENT]					= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_VERY_HIGH_VELOCITY]											= get_local_float_value;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	waypoint
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			raw->altitude = value;

			break;
		}

		case FLOAT_TYPE_FLIGHT_TIME:

		{
			raw->flight_time = value;

			break;
		}

		case FLOAT_TYPE_HEADING:

		{
			raw->heading = value;

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
	waypoint
		*raw;

	float
		value;

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			value = raw->altitude;

			break;
		}

		case FLOAT_TYPE_FLIGHT_TIME:

		{
			value = raw->flight_time;

			break;
		}

		case FLOAT_TYPE_HEADING:

		{
			value = raw->heading;

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





void overload_waypoint_float_value_functions (void)
{

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_ALTITUDE]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_ALTITUDE]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_ALTITUDE][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_ALTITUDE][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_ALTITUDE]								= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_FLIGHT_TIME]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_FLIGHT_TIME]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_FLIGHT_TIME][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_FLIGHT_TIME][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_FLIGHT_TIME]								= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_HEADING]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_HEADING]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_HEADING][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_HEADING][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_WAYPOINT][FLOAT_TYPE_HEADING]								= get_local_float_value;
}





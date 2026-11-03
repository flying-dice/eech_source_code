

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	routed_vehicle
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (routed_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_SUB_WAYPOINT_COUNT:

		{
			raw->sub_waypoint_count = value;

			break;
		}
		/*

		case INT_TYPE_TIMER:

		{
			raw->timer = value;

			break;
		}
		*/

		case INT_TYPE_WAYPOINT_NEXT_INDEX:

		{

			raw->waypoint_next_index = value;

			break;
		}

		case INT_TYPE_WAYPOINT_THIS_INDEX:

		{
			raw->waypoint_this_index = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}





static void set_remote_int_value (entity *en, int_types type, int value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_INT_VALUE, en, type, value);
}





static void set_server_int_value (entity *en, int_types type, int value)
{
	validate_client_server_local_fn ();

	set_local_int_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_int_value (en, type, value);
}





static void set_client_int_value (entity *en, int_types type, int value)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_int_value (en, type, value);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_int_value (en, type, value);
	}
}





static int get_local_int_value (entity *en, int_types type)
{
	routed_vehicle
		*raw;

	int
		value;

	raw = (routed_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_IDENTIFY_ROUTED:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_MOVEMENT_TYPE:

		{
			value = MOVEMENT_TYPE_GROUND;

			break;
		}

		case INT_TYPE_SUB_WAYPOINT_COUNT:

		{
			value = raw->sub_waypoint_count;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_VEHICLE | TASK_TARGET_TYPE_ROUTED);

			break;
		}
		/*

		case INT_TYPE_TIMER:

		{
			value = raw->timer;

			break;
		}
		*/

		case INT_TYPE_WAYPOINT_NEXT_INDEX:

		{
			value = raw->waypoint_next_index;

			break;
		}

		case INT_TYPE_WAYPOINT_THIS_INDEX:

		{
			value = raw->waypoint_this_index;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}

	return (value);
}





void overload_routed_vehicle_int_value_functions (void)
{
	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_IDENTIFY_ROUTED]								= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_MOVEMENT_TYPE]									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_SUB_WAYPOINT_COUNT]							= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_SUB_WAYPOINT_COUNT]							= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_SUB_WAYPOINT_COUNT][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_SUB_WAYPOINT_COUNT][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_SUB_WAYPOINT_COUNT]							= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_TASK_TARGET_TYPE]								= get_local_int_value;

/*
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_TIMER]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_TIMER]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_TIMER][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_TIMER][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_TIMER]									= get_local_int_value;
*/
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_NEXT_INDEX]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_NEXT_INDEX]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_NEXT_INDEX][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_NEXT_INDEX][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_NEXT_INDEX]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_THIS_INDEX]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_THIS_INDEX]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_THIS_INDEX][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_THIS_INDEX][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_ROUTED_VEHICLE][INT_TYPE_WAYPOINT_THIS_INDEX]								= get_local_int_value;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	guide
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CHECK_WAYPOINT_ACTION:

		{
			raw->check_waypoint_action = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_GUIDE_POSITION_TYPE:

		{
			raw->guide_position_type = (guide_position_types) value;

			break;
		}

		case INT_TYPE_VALID_GUIDE_MEMBERS:

		{
			raw->valid_guide_members = value;

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
	guide
		*raw;

	int
		value;

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CHECK_WAYPOINT_ACTION:

		{
			value = raw->check_waypoint_action;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_EVADE_FIRE:

		{
			value = guide_database [raw->sub_type].evade_fire;

			break;
		}

		case INT_TYPE_FOLLOW_FORMATION:

		{
			value = guide_database [raw->sub_type].follow_formation;

			break;
		}

		case INT_TYPE_GUIDE_POSITION_TYPE:

		{
			value = raw->guide_position_type;

			break;
		}

		case INT_TYPE_TERRAIN_FOLLOW_MODE:

		{
			value = guide_database [raw->sub_type].terrain_follow_mode;

			break;
		}

		case INT_TYPE_VALID_GUIDE_MEMBERS:

		{
			value = raw->valid_guide_members;

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





void overload_guide_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_GUIDE][INT_TYPE_CHECK_WAYPOINT_ACTION]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_CHECK_WAYPOINT_ACTION]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_CHECK_WAYPOINT_ACTION][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_CHECK_WAYPOINT_ACTION][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_CHECK_WAYPOINT_ACTION]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_GUIDE][INT_TYPE_ENTITY_SUB_TYPE]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_ENTITY_SUB_TYPE]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_ENTITY_SUB_TYPE]									= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_EVADE_FIRE]											= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_FOLLOW_FORMATION]									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_GUIDE][INT_TYPE_GUIDE_POSITION_TYPE]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_GUIDE_POSITION_TYPE]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_GUIDE_POSITION_TYPE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_GUIDE_POSITION_TYPE][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_GUIDE_POSITION_TYPE]								= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_TERRAIN_FOLLOW_MODE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_GUIDE][INT_TYPE_VALID_GUIDE_MEMBERS]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_VALID_GUIDE_MEMBERS]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_VALID_GUIDE_MEMBERS][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_GUIDE][INT_TYPE_VALID_GUIDE_MEMBERS][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_GUIDE][INT_TYPE_VALID_GUIDE_MEMBERS]								= get_local_int_value;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	pilot
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (pilot *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CREW_ROLE:

		{
			raw->crew_role = (crew_roles) value;

			break;
		}

		case INT_TYPE_DIFFICULTY_LEVEL:

		{
			raw->difficulty_level = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_KILLS:

		{
			raw->kills = value;

			//
			// update high score table if deathmatch, and victim was a player gunship
			//

			update_pilot_high_score_table ();

			break;
		}

		case INT_TYPE_PILOT_RANK:

		{
			raw->rank = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

			break;
		}

		case INT_TYPE_UNIQUE_ID:

		{
			raw->unique_id = value;

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
	pilot
		*raw;

	int
		value;

	raw = (pilot *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CREW_ROLE:

		{
			value = raw->crew_role;

			break;
		}

		case INT_TYPE_DIFFICULTY_LEVEL:

		{
			value = raw->difficulty_level;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_KILLS:

		{
			value = raw->kills;

			break;
		}

		case INT_TYPE_PILOT_RANK:

		{
			value = raw->rank;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

			break;
		}

		case INT_TYPE_UNIQUE_ID:

		{
			value = raw->unique_id;

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





void overload_pilot_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_CREW_ROLE]										= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_CREW_ROLE]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_CREW_ROLE][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_CREW_ROLE][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_CREW_ROLE]										= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_DIFFICULTY_LEVEL]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_DIFFICULTY_LEVEL]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_DIFFICULTY_LEVEL][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_DIFFICULTY_LEVEL][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_DIFFICULTY_LEVEL]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_ENTITY_SUB_TYPE]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_ENTITY_SUB_TYPE]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_ENTITY_SUB_TYPE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_KILLS]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_KILLS]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_KILLS][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_KILLS][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_KILLS]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_PILOT_RANK]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_PILOT_RANK]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_PILOT_RANK][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_PILOT_RANK][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_PILOT_RANK]									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_SIDE]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_SIDE]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_SIDE][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_SIDE][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_SIDE]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_PILOT][INT_TYPE_UNIQUE_ID]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_UNIQUE_ID]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_UNIQUE_ID][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_PILOT][INT_TYPE_UNIQUE_ID][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_PILOT][INT_TYPE_UNIQUE_ID]									= get_local_int_value;
}





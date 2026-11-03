

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	sector
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_KEYSITE_COUNT:

		{
			raw->keysite_count = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

			#ifdef DEBUG

			if (value != ENTITY_SIDE_NEUTRAL)
			{
				ASSERT (get_game_status () == GAME_STATUS_INITIALISING);
			}

			#endif
			
			break;
		}

		case INT_TYPE_X_SECTOR:

		{
			raw->x_sector = value;

			break;
		}

		case INT_TYPE_Z_SECTOR:

		{
			raw->z_sector = value;

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
	sector
		*raw;

	int
		value;

	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_KEYSITE_COUNT:

		{
			value = raw->keysite_count;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

			break;
		}

		case INT_TYPE_SECTOR_SIDE:

		{
			if (raw->sector_side [ENTITY_SIDE_BLUE_FORCE] > raw->sector_side [ENTITY_SIDE_RED_FORCE])
			{
				value = ENTITY_SIDE_BLUE_FORCE;
			}
			else
			{
				value = ENTITY_SIDE_RED_FORCE;
			}

			break;
		}

		case INT_TYPE_X_SECTOR:

		{
			value = raw->x_sector;

			break;
		}

		case INT_TYPE_Z_SECTOR:

		{
			value = raw->z_sector;

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





void overload_sector_int_value_functions (void)
{

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SECTOR][INT_TYPE_KEYSITE_COUNT]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_KEYSITE_COUNT]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_KEYSITE_COUNT][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_KEYSITE_COUNT][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_KEYSITE_COUNT]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SECTOR][INT_TYPE_SECTOR_SIDE]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_SECTOR_SIDE]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_SECTOR_SIDE][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_SECTOR_SIDE][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_SECTOR_SIDE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SECTOR][INT_TYPE_SIDE]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_SIDE]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_SIDE][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_SIDE][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_SIDE]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SECTOR][INT_TYPE_X_SECTOR]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_X_SECTOR]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_X_SECTOR][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_X_SECTOR][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_X_SECTOR]									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SECTOR][INT_TYPE_Z_SECTOR]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_Z_SECTOR]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_Z_SECTOR][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SECTOR][INT_TYPE_Z_SECTOR][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SECTOR][INT_TYPE_Z_SECTOR]									= get_local_int_value;
}





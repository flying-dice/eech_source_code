

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	regen
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (regen *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ALIVE:

		{
			raw->alive = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_REGEN_CREATION_SUB_TYPE:

		{
			raw->regen_creation_sub_type = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

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
	regen
		*raw;

	int
		value;

	raw = (regen *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ALIVE:

		{
			value = raw->alive;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_REGEN_CREATION_SUB_TYPE:

		{
			value = raw->regen_creation_sub_type;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

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





void overload_regen_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_REGEN][INT_TYPE_ALIVE]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_ALIVE]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_ALIVE][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_ALIVE][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_ALIVE]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_REGEN][INT_TYPE_ENTITY_SUB_TYPE]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_ENTITY_SUB_TYPE]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_ENTITY_SUB_TYPE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_REGEN][INT_TYPE_REGEN_CREATION_SUB_TYPE]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_REGEN_CREATION_SUB_TYPE]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_REGEN_CREATION_SUB_TYPE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_REGEN_CREATION_SUB_TYPE][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_REGEN_CREATION_SUB_TYPE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_REGEN][INT_TYPE_SIDE]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_SIDE]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_SIDE][COMMS_MODEL_SERVER]					= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_REGEN][INT_TYPE_SIDE][COMMS_MODEL_CLIENT]					= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_REGEN][INT_TYPE_SIDE]											= get_local_int_value;
}





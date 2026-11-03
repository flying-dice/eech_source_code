

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	division
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (division *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DIVISION_ID:

		{
			raw->division_id = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

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
	division
		*raw;

	int
		value;

	raw = (division *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DIVISION_ID:

		{
			value = raw->division_id;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

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





void overload_division_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_DIVISION][INT_TYPE_DIVISION_ID]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_DIVISION][INT_TYPE_DIVISION_ID]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_DIVISION][INT_TYPE_DIVISION_ID][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_DIVISION][INT_TYPE_DIVISION_ID][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_DIVISION][INT_TYPE_DIVISION_ID]									= get_local_int_value;
	
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_DIVISION][INT_TYPE_ENTITY_SUB_TYPE]							= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_DIVISION][INT_TYPE_ENTITY_SUB_TYPE]							= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_DIVISION][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_DIVISION][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_DIVISION][INT_TYPE_ENTITY_SUB_TYPE]							= get_local_int_value;
}





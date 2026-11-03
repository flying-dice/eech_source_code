

































































#include "project.h"





#define DEBUG_MODULE 0





/*
static void set_local_int_value (entity *en, int_types type, int value)
{
	object
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DAMAGE_LEVEL:

		{
			raw->damage_level = value;

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
	object
		*raw;

	int
		value;

	raw = get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DAMAGE_LEVEL:

		{
			value = raw->damage_level;

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





void overload_object_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_OBJECT][INT_TYPE_DAMAGE_LEVEL]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_OBJECT][INT_TYPE_DAMAGE_LEVEL]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_OBJECT][INT_TYPE_DAMAGE_LEVEL][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_OBJECT][INT_TYPE_DAMAGE_LEVEL][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_OBJECT][INT_TYPE_DAMAGE_LEVEL]									= get_local_int_value;
}
*/




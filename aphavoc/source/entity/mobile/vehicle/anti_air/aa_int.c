

































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void set_local_int_value (entity *en, int_types type, int value)
{
	anti_aircraft
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (anti_aircraft *) get_local_entity_data (en);

	switch (type)
	{

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
*/




static int get_local_int_value (entity *en, int_types type)
{
	anti_aircraft
		*raw;

	int
		value;

	raw = (anti_aircraft *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_IDENTIFY_ANTI_AIRCRAFT:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_MOVEMENT_TYPE:

		{
			value = MOVEMENT_TYPE_GROUND;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_VEHICLE | TASK_TARGET_TYPE_ANTI_AIRCRAFT);

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





void overload_anti_aircraft_int_value_functions (void)
{

	fn_get_local_entity_int_value				[ENTITY_TYPE_ANTI_AIRCRAFT][INT_TYPE_IDENTIFY_ANTI_AIRCRAFT]						= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_ANTI_AIRCRAFT][INT_TYPE_MOVEMENT_TYPE]									= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_ANTI_AIRCRAFT][INT_TYPE_TASK_TARGET_TYPE]								= get_local_int_value;
}





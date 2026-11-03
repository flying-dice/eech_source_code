

































































#include "project.h"





#define DEBUG_MODULE 0





#if DEBUG_MODULE

static void set_local_float_value (entity *en, float_types type, float value)
{
}

#endif





#if DEBUG_MODULE

static void set_remote_float_value (entity *en, float_types type, float value)
{

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}

#endif





#if DEBUG_MODULE

static void set_server_float_value (entity *en, float_types type, float value)
{
	validate_client_server_local_fn ();

	set_local_float_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_float_value (en, type, value);
}

#endif





#if DEBUG_MODULE

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

#endif





#if DEBUG_MODULE

static float get_local_float_value (entity *en, float_types type)
{
	return (FALSE);
}

#endif





void overload_landing_float_value_functions (void)
{
}





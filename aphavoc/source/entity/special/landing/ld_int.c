

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	landing
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (landing *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_FREE_LANDING_SITES:

		{
			
			raw->free_landing_sites = value;

			break;
		}

		case INT_TYPE_INSIDE_HANGAR:

		{
			raw->inside_hangar = value;

			break;
		}

		case INT_TYPE_LANDED_LOCK:

		{
			raw->landed_lock = value;

			break;
		}

		case INT_TYPE_LANDING_LOCK:

		{
			raw->landing_lock = value;

			break;
		}

		case INT_TYPE_RESERVED_LANDING_SITES:

		{
			raw->reserved_landing_sites = value;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			raw->route_node = value;

			break;
		}

		case INT_TYPE_TOTAL_LANDING_SITES:

		{
			raw->total_landing_sites = value;

			break;
		}

		case INT_TYPE_TAKEOFF_LOCK:

		{
			raw->takeoff_lock = value;

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
	landing
		*raw;

	int
		value;

	raw = (landing *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_FREE_LANDING_SITES:

		{
			value = raw->free_landing_sites;

			break;
		}

		case INT_TYPE_INSIDE_HANGAR:

		{
			value = raw->inside_hangar;

			break;
		}

		case INT_TYPE_LANDED_LOCK:

		{
			value = raw->landed_lock;

			break;
		}

		case INT_TYPE_LANDING_LOCK:

		{
			value = raw->landing_lock;

			break;
		}

		case INT_TYPE_RESERVED_LANDING_SITES:

		{
			value = raw->reserved_landing_sites;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			debug_fatal ("ld_int: can't use route node");

			value = raw->route_node;

			break;
		}

		case INT_TYPE_TOTAL_LANDING_SITES:

		{
			value = raw->total_landing_sites;

			break;
		}

		case INT_TYPE_TAKEOFF_LOCK:

		{
			value = raw->takeoff_lock;

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





void overload_landing_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_ENTITY_SUB_TYPE]										= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_ENTITY_SUB_TYPE]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_ENTITY_SUB_TYPE]										= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_FREE_LANDING_SITES]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_FREE_LANDING_SITES]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_FREE_LANDING_SITES][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_FREE_LANDING_SITES][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_FREE_LANDING_SITES]									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_INSIDE_HANGAR]										= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_INSIDE_HANGAR]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_INSIDE_HANGAR][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_INSIDE_HANGAR][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_INSIDE_HANGAR]										= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_LANDED_LOCK]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_LANDED_LOCK]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_LANDED_LOCK][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_LANDED_LOCK][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_LANDED_LOCK]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_LANDING_LOCK]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_LANDING_LOCK]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_LANDING_LOCK][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_LANDING_LOCK][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_LANDING_LOCK]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_RESERVED_LANDING_SITES]							= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_RESERVED_LANDING_SITES]							= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_RESERVED_LANDING_SITES][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_RESERVED_LANDING_SITES][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_RESERVED_LANDING_SITES]							= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_ROUTE_NODE]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_ROUTE_NODE]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_ROUTE_NODE][COMMS_MODEL_SERVER]					= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_ROUTE_NODE][COMMS_MODEL_CLIENT]					= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_ROUTE_NODE]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_TOTAL_LANDING_SITES]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_TOTAL_LANDING_SITES]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_TOTAL_LANDING_SITES][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_TOTAL_LANDING_SITES][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_TOTAL_LANDING_SITES]								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_LANDING][INT_TYPE_TAKEOFF_LOCK]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_TAKEOFF_LOCK]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_TAKEOFF_LOCK][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_LANDING][INT_TYPE_TAKEOFF_LOCK][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_LANDING][INT_TYPE_TAKEOFF_LOCK]											= get_local_int_value;
}





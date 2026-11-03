

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	segment
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (segment *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_BRIDGE_SEGMENT_TYPE:

		{
			raw->bridge_segment_type = value;

			break;
		}

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
	segment
		*raw;

	int
		value;

	raw = (segment *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_BRIDGE_SEGMENT_TYPE:

		{
			value = raw->bridge_segment_type;

			break;
		}

		case INT_TYPE_DAMAGE_LEVEL:

		{
			value = raw->damage_level;

			break;
		}

		case INT_TYPE_EXPLOSIVE_POWER:

		{
			value = EXPLOSIVE_POWER_LOW;

			break;
		}

		case INT_TYPE_EXPLOSIVE_QUALITY:

		{
			value = EXPLOSIVE_QUALITY_DUST;

			break;
		}

		case INT_TYPE_TARGET_SYMBOL_TYPE:

		{
			value = TARGET_SYMBOL_STRUCTURE;

			break;
		}

		case INT_TYPE_TARGET_TYPE:

		{
			value = segment_database[raw->bridge_segment_type].target_type;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = TASK_TARGET_TYPE_BRIDGE;

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{
			value = segment_database[raw->bridge_segment_type].armor_level;

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





void overload_segment_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SEGMENT][INT_TYPE_BRIDGE_SEGMENT_TYPE]							= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_BRIDGE_SEGMENT_TYPE]							= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SEGMENT][INT_TYPE_BRIDGE_SEGMENT_TYPE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SEGMENT][INT_TYPE_BRIDGE_SEGMENT_TYPE][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_BRIDGE_SEGMENT_TYPE]							= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_SEGMENT][INT_TYPE_DAMAGE_LEVEL]										= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_DAMAGE_LEVEL]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SEGMENT][INT_TYPE_DAMAGE_LEVEL][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_SEGMENT][INT_TYPE_DAMAGE_LEVEL][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_DAMAGE_LEVEL]										= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_EXPLOSIVE_POWER]									= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_EXPLOSIVE_QUALITY]								= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_TARGET_SYMBOL_TYPE]								= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_TARGET_TYPE]										= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_TASK_TARGET_TYPE]								= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_SEGMENT][INT_TYPE_ARMOR_LEVEL]						= get_local_int_value;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	group
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			raw->supplies.ammo_supply_level = value;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			raw->assist_timer = value;

			if ((value != 0.0) && (!raw->update_link.parent))
			{

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
			}

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			raw->supplies.fuel_supply_level = value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			raw->sleep = value;

			if ((value != 0.0) && (!raw->update_link.parent))
			{

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
			}

			break;
		}

		case FLOAT_TYPE_LAST_SEEN_TIME:

		{
			raw->last_seen_time = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}





static void set_remote_float_value (entity *en, float_types type, float value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}





static void set_server_float_value (entity *en, float_types type, float value)
{
	validate_client_server_local_fn ();

	set_local_float_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_float_value (en, type, value);
}





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





static float get_local_float_value (entity *en, float_types type)
{
	group
		*raw;

	float
		value;

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			value = raw->supplies.ammo_supply_level;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			value = raw->assist_timer;

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			value = raw->supplies.fuel_supply_level;

			break;
		}

		case FLOAT_TYPE_REARMING_TIME:

		{
			value = group_database [raw->sub_type].rearming_time;

			break;
		}

		case FLOAT_TYPE_RECON_DISTANCE:

		{
			value = 100.0;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			value = raw->sleep;

			break;
		}

		case FLOAT_TYPE_LAST_SEEN_TIME:

		{
			value = raw->last_seen_time;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}





void overload_group_float_value_functions (void)
{
	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_GROUP][FLOAT_TYPE_AMMO_SUPPLY_LEVEL]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_AMMO_SUPPLY_LEVEL]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_AMMO_SUPPLY_LEVEL][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_AMMO_SUPPLY_LEVEL][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_AMMO_SUPPLY_LEVEL]								= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_GROUP][FLOAT_TYPE_ASSIST_TIMER]										= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_ASSIST_TIMER]										= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_ASSIST_TIMER][COMMS_MODEL_SERVER]			= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_ASSIST_TIMER][COMMS_MODEL_CLIENT]			= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_ASSIST_TIMER]										= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_GROUP][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]								= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_REARMING_TIME]						= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_RECON_DISTANCE]						= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_GROUP][FLOAT_TYPE_SLEEP]									= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_SLEEP]									= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_SLEEP][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_SLEEP][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_SLEEP]									= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_GROUP][FLOAT_TYPE_LAST_SEEN_TIME]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_LAST_SEEN_TIME]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_LAST_SEEN_TIME][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_GROUP][FLOAT_TYPE_LAST_SEEN_TIME][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_GROUP][FLOAT_TYPE_LAST_SEEN_TIME]								= get_local_float_value;
}





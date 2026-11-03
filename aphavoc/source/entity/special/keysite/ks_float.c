

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	keysite
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			raw->supplies.ammo_supply_level = value;

			break;
		}

		case FLOAT_TYPE_ASSIGN_TIMER:

		{
			raw->assign_timer = value;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			raw->assist_timer = value;

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			raw->supplies.fuel_supply_level = value;

			break;
		}

		case FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH:

		{
			raw->keysite_maximum_strength = value;

			break;
		}

		case FLOAT_TYPE_KEYSITE_STRENGTH:

		{
			raw->keysite_strength = value;

			break;
		}

		case FLOAT_TYPE_REPAIR_TIMER:

		{
			raw->repair_timer = value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			raw->sleep = value;

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
	keysite
		*raw;

	float
		value;

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			value = raw->supplies.ammo_supply_level;

			break;
		}

		case FLOAT_TYPE_ASSIGN_TIMER:

		{
			value = raw->assign_timer;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			value = raw->assist_timer;

			break;
		}

		case FLOAT_TYPE_EFFICIENCY:

		{
			if (raw->keysite_maximum_strength > 0.0)
			{
				ASSERT (raw->keysite_strength <= raw->keysite_maximum_strength);

				value = raw->keysite_strength / raw->keysite_maximum_strength;
			}
			else
			{
				value = 1.0;
			}

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			value = raw->supplies.fuel_supply_level;

			break;
		}

		case FLOAT_TYPE_MINIMUM_EFFICIENCY:

		{
			value = keysite_database [raw->sub_type].minimum_efficiency;

			break;
		}

		case FLOAT_TYPE_KEYSITE_IMPORTANCE:

		{
			value = keysite_database [raw->sub_type].importance;

			break;
		}

		case FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH:

		{
			value = raw->keysite_maximum_strength;

			break;
		}

		case FLOAT_TYPE_KEYSITE_STRENGTH:

		{
			value = raw->keysite_strength;

			break;
		}

		case FLOAT_TYPE_RECON_DISTANCE:

		{
			value = keysite_database [raw->sub_type].recon_distance;

			break;
		}

		case FLOAT_TYPE_REPAIR_TIMER:

		{
			value = raw->repair_timer;

			break;
		}

		case FLOAT_TYPE_SECTOR_SIDE_MAX_VALUE:

		{
			value = keysite_database [raw->sub_type].sector_side_max_value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			value = raw->sleep;

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





void overload_keysite_float_value_functions (void)
{

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL]							= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_AMMO_SUPPLY_LEVEL]							= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIGN_TIMER]									= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIGN_TIMER]									= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIGN_TIMER][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIGN_TIMER][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIGN_TIMER]									= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIST_TIMER]									= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIST_TIMER]									= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIST_TIMER][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIST_TIMER][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_ASSIST_TIMER]									= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_EFFICIENCY]										= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]							= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_FUEL_SUPPLY_LEVEL]							= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_IMPORTANCE]							= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH]					= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH]					= set_local_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH]					= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_STRENGTH]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_STRENGTH]								= set_local_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_KEYSITE_STRENGTH]								= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_MINIMUM_EFFICIENCY]							= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_RECON_DISTANCE]							= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_REPAIR_TIMER]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_REPAIR_TIMER]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_REPAIR_TIMER][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_REPAIR_TIMER][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_REPAIR_TIMER]								= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_SECTOR_SIDE_MAX_VALUE]		= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_SLEEP]							= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_SLEEP]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_SLEEP][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_SLEEP][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_KEYSITE][FLOAT_TYPE_SLEEP]							= get_local_float_value;
}





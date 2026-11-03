

































































#include "project.h"





#define DEBUG_MODULE 0





/*
static void set_local_float_value (entity *en, float_types type, float value)
{
	weapon
		*raw;







	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}
*/





/*
static void set_remote_float_value (entity *en, float_types type, float value)
{






	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}
*/





/*
static void set_server_float_value (entity *en, float_types type, float value)
{
	validate_client_server_local_fn ();

	set_local_float_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_float_value (en, type, value);
}
*/





/*
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
*/





static float get_local_float_value (entity *en, float_types type)
{
	weapon
		*raw;

	float
		value;

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE:

		{
			object_3d_index_numbers
				object_3d_shape;

			object_3d_shape = weapon_database[raw->mob.sub_type].default_3d_shape;

			value = object_3d_information_database[object_3d_shape].maximum_distance;

			break;
		}

		case FLOAT_TYPE_WEIGHT:

		{
			value = get_weapon_current_weight(raw->weapon_lifetime, raw->mob.sub_type);

			break;
		}

		case FLOAT_TYPE_WEAPON_LIFETIME:

		{
			value = raw->weapon_lifetime;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK:

		{
			value = 2;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK:

		{
			value = 2;

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





















































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void set_local_float_value (entity *en, float_types type, float value)
{
	ship_vehicle
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (ship_vehicle *) get_local_entity_data (en);

	switch (type)
	{

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
*/




static float get_local_float_value (entity *en, float_types type)
{
	ship_vehicle
		*raw;

	float
		value;

	raw = (ship_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AVOIDANCE_RADIUS:

		{

			float
				length;

			object_3d_bounds
				*bounding_box;

			bounding_box = get_object_3d_bounding_box_without_lines (raw->vh.object_3d_shape);

			length = max (bounding_box->zmax, bounding_box->xmax);

			value = max (50.0, (2.0 * length));

			break;
		}

		case FLOAT_TYPE_FIXED_STRUCTURE_HEIGHT:

		{
			object_3d_bounds
				*bounding_box;

			bounding_box = get_object_3d_bounding_box_without_lines (raw->vh.object_3d_shape);

			value = bounding_box->ymax;

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





void overload_ship_vehicle_float_value_functions (void)
{

	fn_get_local_entity_float_value				[ENTITY_TYPE_SHIP_VEHICLE][FLOAT_TYPE_AVOIDANCE_RADIUS]								= get_local_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_SHIP_VEHICLE][FLOAT_TYPE_FIXED_STRUCTURE_HEIGHT]						= get_local_float_value;
}





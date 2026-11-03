

































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void set_local_float_value (entity *en, float_types type, float value)
{
	ship_vehicle
		*raw;







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
















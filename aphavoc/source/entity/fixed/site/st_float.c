

































































#include "project.h"





#define DEBUG_MODULE 0





static float get_local_float_value (entity *en, float_types type)
{
	site
		*raw;

	float
		value;

	raw = (site *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AVOIDANCE_RADIUS:

		{
			object_3d_bounds
				*bounding_box;

			bounding_box = get_object_3d_bounding_box_without_lines (raw->fix.object_3d_shape);

			value = max (50.0, (2.0 * bounding_box->ymax));

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





void overload_site_float_value_functions (void)
{
	fn_get_local_entity_float_value		[ENTITY_TYPE_SITE][FLOAT_TYPE_AVOIDANCE_RADIUS]	= get_local_float_value;
}





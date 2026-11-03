

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	city
		*raw;







	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_APPROXIMATION_OBJECT_3D_SHAPE:

		{
			raw->approximation_object_3d_shape = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}























































static int get_local_int_value (entity *en, int_types type)
{
	city
		*raw;

	int
		value;

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_APPROXIMATION_OBJECT_3D_SHAPE:

		{
			value = raw->approximation_object_3d_shape;

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


















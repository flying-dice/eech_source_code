

































































#include "project.h"





#define DEBUG_MODULE 0





static float get_local_float_value (entity *en, float_types type)
{
	pylon
		*raw;

	float
		value;

	raw = (pylon *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AVOIDANCE_RADIUS:

		{
			value = 50.0;

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





void overload_pylon_float_value_functions (void)
{
	fn_get_local_entity_float_value		[ENTITY_TYPE_PYLON][FLOAT_TYPE_AVOIDANCE_RADIUS]	= get_local_float_value;
}





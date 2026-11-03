

































































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














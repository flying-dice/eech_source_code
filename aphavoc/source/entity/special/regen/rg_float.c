

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	regen
		*raw;







	raw = (regen *) get_local_entity_data (en);

	switch (type)
	{

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























































static float get_local_float_value (entity *en, float_types type)
{
	regen
		*raw;

	float
		value;

	raw = (regen *) get_local_entity_data (en);

	switch (type)
	{

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




















































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	guide
		*raw;







	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_VELOCITY:

		{
			raw->velocity = value;

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
	guide
		*raw;

	float
		value;

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_VELOCITY:

		{
			value = raw->velocity;

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


















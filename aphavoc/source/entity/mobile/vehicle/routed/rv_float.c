

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	routed_vehicle
		*raw;







	raw = (routed_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_DESIRED_VELOCITY:

		{
			raw->desired_velocity = value;

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
	routed_vehicle
		*raw;

	float
		value;

	raw = (routed_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_DESIRED_VELOCITY:

		{
			value = raw->desired_velocity;

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





















































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	fixed_wing
		*raw;







	raw = (fixed_wing *) get_local_entity_data (en);

	switch (type)
	{

































		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}























































static float get_local_float_value (entity *en, float_types type)
{
	fixed_wing
		*raw;

	float
		value;

	raw = (fixed_wing *) get_local_entity_data (en);

	switch (type)
	{

































		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}































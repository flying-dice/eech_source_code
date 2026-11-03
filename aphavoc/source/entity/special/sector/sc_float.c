

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	sector
		*raw;







	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}

#endif



































































#if GET_LOCAL_REQUIRED

static float get_local_float_value (entity *en, float_types type)
{
	sector
		*raw;

	float
		value;

	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_TALLEST_STRUCTURE_HEIGHT:

		{
			value = raw->tallest_structure_height;

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

#endif














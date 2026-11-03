

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	waypoint
		*raw;







	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			raw->altitude = value;

			break;
		}

		case FLOAT_TYPE_FLIGHT_TIME:

		{
			raw->flight_time = value;

			break;
		}

		case FLOAT_TYPE_HEADING:

		{
			raw->heading = value;

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
	waypoint
		*raw;

	float
		value;

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			value = raw->altitude;

			break;
		}

		case FLOAT_TYPE_FLIGHT_TIME:

		{
			value = raw->flight_time;

			break;
		}

		case FLOAT_TYPE_HEADING:

		{
			value = raw->heading;

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































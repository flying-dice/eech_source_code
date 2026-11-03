

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	bridge
		*raw;







	raw = (bridge *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_PITCH:

		{
			raw->pitch = value;

			break;
		}

		case FLOAT_TYPE_SCALE:

		{
			raw->scale = value;

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
	bridge
		*raw;

	float
		value;

	raw = (bridge *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_PITCH:

		{
			value = raw->pitch;

			break;
		}

		case FLOAT_TYPE_RECON_DISTANCE:

		{
			value = 800.0;

			break;
		}

		case FLOAT_TYPE_SCALE:

		{
			value = raw->scale;

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




























































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	fixed_wing
		*raw;







	raw = (fixed_wing *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AFTERBURNER_STATE:

		{
			raw->afterburner_state = value;

			break;
		}

		case FLOAT_TYPE_AIRBRAKE_STATE:

		{
			raw->airbrake_state = value;

			break;
		}

		case FLOAT_TYPE_FLAP_STATE:

		{
			raw->flap_state = value;

			break;
		}

		case FLOAT_TYPE_PROPELLOR_RPM:

		{
			raw->propellor_rpm = value;

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
	fixed_wing
		*raw;

	float
		value;

	raw = (fixed_wing *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AFTERBURNER_STATE:

		{
			value = raw->afterburner_state;

			break;
		}

		case FLOAT_TYPE_AIRBRAKE_STATE:

		{
			value = raw->airbrake_state;

			break;
		}

		case FLOAT_TYPE_FLAP_STATE:

		{
			value = raw->flap_state;

			break;
		}

		case FLOAT_TYPE_PROPELLOR_RPM:

		{
			value = raw->propellor_rpm;

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































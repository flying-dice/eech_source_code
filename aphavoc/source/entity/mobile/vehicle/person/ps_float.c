

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	person
		*raw;







	raw = (person *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_PERSON_ANIMATION_STATE:

		{
			raw->person_animation_state = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}

#endif





#if SET_CLIENT_SERVER_REQUIRED












#endif





#if SET_CLIENT_SERVER_REQUIRED












#endif





#if SET_CLIENT_SERVER_REQUIRED

















#endif





#if GET_LOCAL_REQUIRED

static float get_local_float_value (entity *en, float_types type)
{
	person
		*raw;

	float
		value;

	raw = (person *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_PERSON_ANIMATION_STATE:

		{
			value = raw->person_animation_state;

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


















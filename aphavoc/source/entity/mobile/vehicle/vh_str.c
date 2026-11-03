

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_string (entity *en, string_types type, char *s)
{
	vehicle
		*raw;

	ASSERT (s);







	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}
}

#endif









































































#if GET_LOCAL_REQUIRED

static const char *get_local_string (entity *en, string_types type)
{
	vehicle
		*raw;

	const char
		*s;

	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{









		case STRING_TYPE_FULL_NAME:

		{
			s = vehicle_database[raw->mob.sub_type].full_name;

			break;
		}

















		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}

	return (s);
}

#endif




















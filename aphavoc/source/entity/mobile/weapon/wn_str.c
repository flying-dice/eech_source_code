

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_string (entity *en, string_types type, char *s)
{
	weapon
		*raw;

	ASSERT (s);







	raw = (weapon *) get_local_entity_data (en);

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





#if SET_CLIENT_SERVER_REQUIRED














#endif





#if SET_CLIENT_SERVER_REQUIRED














#endif





#if SET_CLIENT_SERVER_REQUIRED



















#endif





#if GET_LOCAL_REQUIRED

static const char *get_local_string (entity *en, string_types type)
{
	weapon
		*raw;

	const char
		*s;

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = weapon_database[raw->mob.sub_type].full_name;

			break;
		}

		case STRING_TYPE_HUD_NAME:

		{
			s = weapon_database[raw->mob.sub_type].hud_name;

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
















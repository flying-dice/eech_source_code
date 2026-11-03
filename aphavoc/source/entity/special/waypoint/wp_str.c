

































































#include "project.h"





#define DEBUG_MODULE 0













































































































static const char *get_local_string (entity *en, string_types type)
{
	task
		*raw;

	const char
		*s;

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = waypoint_database[raw->sub_type].full_name;

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
















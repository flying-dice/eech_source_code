

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_string (entity *en, string_types type, const char *s)
{
	group
		*raw;

	ASSERT (s);







	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_DIVISION_NAME:

		{
			strncpy (raw->division_name, s, get_string_type_max_length (type));

			break;
		}

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}
}





























































static const char *get_local_string (entity *en, string_types type)
{
	group
		*raw;

	const char
		*s;

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_DIVISION_NAME:

		{
			s = raw->division_name;

			break;
		}

		case STRING_TYPE_FULL_NAME:

		{
			s = group_database[raw->sub_type].full_name;

			break;
		}

		case STRING_TYPE_GROUP_CALLSIGN:

		{
			s = group_callsign_names [raw->group_callsign].callsign;

			break;
		}

		case STRING_TYPE_GROUP_SHORT_NAME:

		{
			s = group_database[raw->sub_type].group_short_name;

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
























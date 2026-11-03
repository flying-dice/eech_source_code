

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_string (entity *en, string_types type, const char *s)
{
	division
		*raw;

	ASSERT (s);







	raw = (division *) get_local_entity_data (en);

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
	division
		*raw;

	char
		*s;

	raw = (division *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_DIVISION_NAME:

		{
			s = raw->division_name;

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


















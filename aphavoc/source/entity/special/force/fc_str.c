

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_string (entity *en, string_types type, const char *s)
{
	force
		*raw;

	ASSERT (s);







	raw = (force *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FORCE_NAME:

		{
			strncpy (raw->force_name, s, get_string_type_max_length (type));

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
	force
		*raw;

	char
		*s;

	raw = (force *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FORCE_NAME:

		{
			s = raw->force_name;

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


















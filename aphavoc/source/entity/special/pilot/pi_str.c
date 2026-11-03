

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_string (entity *en, string_types type, const char *s)
{
	pilot
		*raw;

	ASSERT (s);







	raw = (pilot *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_PILOTS_NAME:

		{
			strncpy (raw->pilots_name, s, get_string_type_max_length (type));

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
	pilot
		*raw;

	char
		*s;

	raw = (pilot *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_PILOTS_NAME:

		{
			s = raw->pilots_name;

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


















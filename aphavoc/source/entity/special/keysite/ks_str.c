

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_string (entity *en, string_types type, const char *s)
{
	keysite
		*raw;

	ASSERT (s);







	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:
		case STRING_TYPE_KEYSITE_NAME:

		{
			strncpy (raw->keysite_name, s, get_string_type_max_length (type));

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
	keysite
		*raw;

	const char
		*s;

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = keysite_database [raw->sub_type].full_name;

			break;
		}

		case STRING_TYPE_KEYSITE_NAME:

		{
			s = raw->keysite_name;

			break;
		}

		case STRING_TYPE_SHORT_DISPLAY_NAME:

		{
			s = keysite_database [raw->sub_type].short_name;

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


























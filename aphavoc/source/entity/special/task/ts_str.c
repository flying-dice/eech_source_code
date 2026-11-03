

































































#include "project.h"





#define DEBUG_MODULE 0





/*

static void set_local_string (entity *en, string_types type, char *s)
{
	aircraft
		*raw;

	ASSERT (s);







	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}
}

*/









































































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
			s = task_database[raw->sub_type].full_name;

			break;
		}

		case STRING_TYPE_SHORT_DISPLAY_NAME:

		{
			s = task_database[raw->sub_type].short_name;

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
















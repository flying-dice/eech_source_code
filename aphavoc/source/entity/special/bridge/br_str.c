

































































#include "project.h"





#define DEBUG_MODULE 0





static const char *get_local_string (entity *en, string_types type)
{
	const char
		*s;

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = get_trans ("Bridge"); 

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














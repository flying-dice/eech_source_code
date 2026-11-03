

































































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





void overload_bridge_string_functions (void)
{
	fn_get_local_entity_string	[ENTITY_TYPE_BRIDGE][STRING_TYPE_FULL_NAME]		= get_local_string;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	cargo
		*raw;







	raw = (cargo *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}























































static int get_local_int_value (entity *en, int_types type)
{
	cargo
		*raw;

	int
		value;

	raw = (cargo *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_IDENTIFY_CARGO:

		{
			value = 1;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_CARGO);

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}

	return (value);
}























































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	division
		*raw;







	raw = (division *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DIVISION_ID:

		{
			raw->division_id = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}























































static int get_local_int_value (entity *en, int_types type)
{
	division
		*raw;

	int
		value;

	raw = (division *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DIVISION_ID:

		{
			value = raw->division_id;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

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
























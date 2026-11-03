

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	pylon
		*raw;







	raw = (pylon *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DAMAGE_LEVEL:

		{
			raw->damage_level = value;

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
	pylon
		*raw;

	int
		value;

	raw = (pylon *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DAMAGE_LEVEL:

		{
			value = raw->damage_level;

			break;
		}

		case INT_TYPE_TARGET_SYMBOL_TYPE:

		{
			value = TARGET_SYMBOL_HAZZARD;

			break;
		}

		case INT_TYPE_TARGET_TYPE:

		{
			value = TARGET_TYPE_HAZZARD;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = TASK_TARGET_TYPE_POWER;

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{
			value = 10;

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


























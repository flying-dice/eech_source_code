

































































#include "project.h"





#define DEBUG_MODULE 0





static int get_local_int_value (entity *en, int_types type)
{
	ship_vehicle
		*raw;

	int
		value;

	raw = (ship_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_IDENTIFY_SHIP_VEHICLE:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_MOVEMENT_TYPE:

		{
			value = MOVEMENT_TYPE_SEA;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_VEHICLE | TASK_TARGET_TYPE_SHIP);

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


















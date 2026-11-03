

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	routed_vehicle
		*raw;







	raw = (routed_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_SUB_WAYPOINT_COUNT:

		{
			raw->sub_waypoint_count = value;

			break;
		}
		/*

		case INT_TYPE_TIMER:

		{
			raw->timer = value;

			break;
		}
		*/

		case INT_TYPE_WAYPOINT_NEXT_INDEX:

		{

			raw->waypoint_next_index = value;

			break;
		}

		case INT_TYPE_WAYPOINT_THIS_INDEX:

		{
			raw->waypoint_this_index = value;

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
	routed_vehicle
		*raw;

	int
		value;

	raw = (routed_vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_IDENTIFY_ROUTED:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_MOVEMENT_TYPE:

		{
			value = MOVEMENT_TYPE_GROUND;

			break;
		}

		case INT_TYPE_SUB_WAYPOINT_COUNT:

		{
			value = raw->sub_waypoint_count;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_VEHICLE | TASK_TARGET_TYPE_ROUTED);

			break;
		}
		/*

		case INT_TYPE_TIMER:

		{
			value = raw->timer;

			break;
		}
		*/

		case INT_TYPE_WAYPOINT_NEXT_INDEX:

		{
			value = raw->waypoint_next_index;

			break;
		}

		case INT_TYPE_WAYPOINT_THIS_INDEX:

		{
			value = raw->waypoint_this_index;

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











































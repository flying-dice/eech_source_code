

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	guide
		*raw;







	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CHECK_WAYPOINT_ACTION:

		{
			raw->check_waypoint_action = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_GUIDE_POSITION_TYPE:

		{
			raw->guide_position_type = (guide_position_types) value;

			break;
		}

		case INT_TYPE_VALID_GUIDE_MEMBERS:

		{
			raw->valid_guide_members = value;

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
	guide
		*raw;

	int
		value;

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CHECK_WAYPOINT_ACTION:

		{
			value = raw->check_waypoint_action;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_EVADE_FIRE:

		{
			value = guide_database [raw->sub_type].evade_fire;

			break;
		}

		case INT_TYPE_FOLLOW_FORMATION:

		{
			value = guide_database [raw->sub_type].follow_formation;

			break;
		}

		case INT_TYPE_GUIDE_POSITION_TYPE:

		{
			value = raw->guide_position_type;

			break;
		}

		case INT_TYPE_TERRAIN_FOLLOW_MODE:

		{
			value = guide_database [raw->sub_type].terrain_follow_mode;

			break;
		}

		case INT_TYPE_VALID_GUIDE_MEMBERS:

		{
			value = raw->valid_guide_members;

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










































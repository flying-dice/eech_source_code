

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_int_value (entity *en, int_types type, int value)
{
	waypoint
		*raw;







	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_POSITION_TYPE:

		{

			raw->position_type = value;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			raw->route_node = value;

			break;
		}

		case INT_TYPE_WAYPOINT_FORMATION:

		{
			ASSERT (value <= get_formation_database_count ());

			raw->waypoint_formation = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}





static void set_local_int_value (entity *en, int_types type, int value)
{
	waypoint
		*raw;







	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			update_local_entity_waypoint_list_tags (raw->waypoint_link.parent);

			break;
		}

		case INT_TYPE_POSITION_TYPE:

		{
			raw->position_type = value;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			raw->route_node = value;

			break;
		}

		case INT_TYPE_WAYPOINT_FORMATION:

		{
			raw->waypoint_formation = value;

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
	waypoint
		*raw;

	int
		value;

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CHECK_WAYPOINT_ACTION:

		{
			value = waypoint_database [raw->sub_type].check_waypoint_action;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_IDENTIFY_WAYPOINT:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_MAP_ICON:

		{
			value = waypoint_database [raw->sub_type].map_icon;

			break;
		}

		case INT_TYPE_MOBILE_FOLLOW_WAYPOINT_OFFSET:

		{
			value = waypoint_database [raw->sub_type].mobile_follow_waypoint_offset;

			break;
		}

		case INT_TYPE_PLANNER_MOVEABLE:

		{
			value = waypoint_database [raw->sub_type].planner_moveable;

			break;
		}

		case INT_TYPE_POSITION_TYPE:

		{
			value = raw->position_type;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			value = raw->route_node;

			break;
		}

		case INT_TYPE_VERBOSE_OPERATIONAL_STATE:

		{
			value = waypoint_database [raw->sub_type].verbose_operational_state;

			break;
		}

		case INT_TYPE_WAYPOINT_FORMATION:

		{
			value = raw->waypoint_formation;

			break;
		}

		case INT_TYPE_WAYPOINT_GUIDE_TYPE:

		{
			value = waypoint_database [raw->sub_type].guide_sub_type;

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
























































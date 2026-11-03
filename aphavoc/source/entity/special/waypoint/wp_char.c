

































































#include "project.h"





#define DEBUG_MODULE 0





static char
	nav_tag,
	target_tag;





void reset_waypoint_tags (void)
{

	nav_tag = 'A';
	target_tag = 'X';
}





void update_local_entity_waypoint_list_tags (entity *parent)
{
	entity
		*en;

	char
		letter;

	entity_sub_types
		sub_type;

	ASSERT (parent);

	reset_waypoint_tags ();

	en = get_local_entity_first_child (parent, LIST_TYPE_WAYPOINT);

	while (en)
	{
		sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

		letter = get_waypoint_sub_type_tag (sub_type);

		set_local_entity_char_value (en, CHAR_TYPE_TAG, letter);

		en = get_local_entity_child_succ (en, LIST_TYPE_WAYPOINT);
	}
}





void update_client_server_entity_waypoint_list_tags (entity *parent)
{
	entity
		*en;

	char
		letter;

	entity_sub_types
		sub_type;

	ASSERT (parent);

	reset_waypoint_tags ();

	en = get_local_entity_first_child (parent, LIST_TYPE_WAYPOINT);

	while (en)
	{
		sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

		letter = get_waypoint_sub_type_tag (sub_type);

		set_client_server_entity_char_value (en, CHAR_TYPE_TAG, letter);

		en = get_local_entity_child_succ (en, LIST_TYPE_WAYPOINT);
	}
}





char get_waypoint_sub_type_tag (entity_sub_types sub_type)
{

	switch (sub_type)
	{
		case ENTITY_SUB_TYPE_WAYPOINT_CAP_START:
		case ENTITY_SUB_TYPE_WAYPOINT_END:
		case ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION:
		{

			nav_tag = (nav_tag <= 'V') ? (nav_tag) : ('A');

			return nav_tag ++;
		}
		case ENTITY_SUB_TYPE_WAYPOINT_ATTACK:
		case ENTITY_SUB_TYPE_WAYPOINT_CAP_LOOP:
		case ENTITY_SUB_TYPE_WAYPOINT_DEFEND:
		case ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF:
		case ENTITY_SUB_TYPE_WAYPOINT_ESCORT:
		case ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF:
		case ENTITY_SUB_TYPE_WAYPOINT_IMPOSSIBLE:
		case ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF:
		case ENTITY_SUB_TYPE_WAYPOINT_PICK_UP:
		case ENTITY_SUB_TYPE_WAYPOINT_RECON:
		case ENTITY_SUB_TYPE_WAYPOINT_REPAIR:
		case ENTITY_SUB_TYPE_WAYPOINT_TARGET:
		case ENTITY_SUB_TYPE_WAYPOINT_TROOP_CAPTURE:
		case ENTITY_SUB_TYPE_WAYPOINT_TROOP_DEFEND:
		case ENTITY_SUB_TYPE_WAYPOINT_TROOP_INSERT:
		case ENTITY_SUB_TYPE_WAYPOINT_TROOP_PICKUP_POINT_END:
		case ENTITY_SUB_TYPE_WAYPOINT_TROOP_PICKUP_POINT_START:
		case ENTITY_SUB_TYPE_WAYPOINT_TROOP_PUTDOWN_POINT:
		case ENTITY_SUB_TYPE_WAYPOINT_WAIT:
		case ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_INSERTION:
		{

			target_tag = (target_tag <= 'Z') ? (target_tag) : ('X');

			return target_tag ++;
		}
		case ENTITY_SUB_TYPE_WAYPOINT_APPROACH:
		case ENTITY_SUB_TYPE_WAYPOINT_CONVOY:
		case ENTITY_SUB_TYPE_WAYPOINT_HOLDING:
		case ENTITY_SUB_TYPE_WAYPOINT_HOLDING_LOOP:
		case ENTITY_SUB_TYPE_WAYPOINT_LAND:
		case ENTITY_SUB_TYPE_WAYPOINT_LANDED:
		case ENTITY_SUB_TYPE_WAYPOINT_LIFT_OFF:
		case ENTITY_SUB_TYPE_WAYPOINT_LOWER_UNDERCARRIAGE:
		case ENTITY_SUB_TYPE_WAYPOINT_REVERSE_CONVOY:
		case ENTITY_SUB_TYPE_WAYPOINT_RAISE_UNDERCARRIAGE:
		case ENTITY_SUB_TYPE_WAYPOINT_START_UP:
		case ENTITY_SUB_TYPE_WAYPOINT_SUB_ROUTE_NAVIGATION:
		case ENTITY_SUB_TYPE_WAYPOINT_TAXI:
		case ENTITY_SUB_TYPE_WAYPOINT_TOUCH_DOWN:
		case ENTITY_SUB_TYPE_WAYPOINT_TAKEN_OFF:
		{

			target_tag = 'W';

			return target_tag;
		}
		default:
		{
			debug_fatal ("Invalid waypoint entity sub-type = %d", sub_type);

			break;
		}
	}

	return 'A';
}





static void set_local_char_value (entity *en, char_types type, char value)
{
	waypoint
		*raw;







	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case CHAR_TYPE_TAG:

		{
			raw->tag = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_char_type (en, type);

			break;
		}
	}
}























































static char get_local_char_value (entity *en, char_types type)
{
	waypoint
		*raw;

	char
		value;

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case CHAR_TYPE_TAG:

		{
			value = raw->tag;

			break;
		}

		default:

		{
			debug_fatal_invalid_char_type (en, type);

			break;
		}
	}

	return (value);
}


















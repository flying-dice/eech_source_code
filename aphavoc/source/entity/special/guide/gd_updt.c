

































































#include "project.h"





#define DEBUG_MODULE 			0





static void update_server (entity *en)
{
	ASSERT (en);







	if (!get_local_entity_first_child (en, LIST_TYPE_FOLLOWER))
	{
		//
		// only update the guide if something is following it (may change)
		//
		
		return;
	}

	if (get_local_entity_int_value (en, INT_TYPE_CHECK_WAYPOINT_ACTION))
	{
		if (check_guide_waypoint_action_reached (en))
		{
			notify_local_entity (ENTITY_MESSAGE_WAYPOINT_ACTION, en, NULL);
		}
	}

	if (check_guide_reached_waypoint (en))
	{
		notify_local_entity (ENTITY_MESSAGE_WAYPOINT_REACHED, en, NULL);
	}
}


































































































#include "project.h"





#define DEBUG_MODULE 0





void navigation_guide_waypoint_reached (entity *en)
{
	entity
		*current_wp;

	//
	// set next waypoint (actual set function is called from within wp_msgs)
	//

	current_wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (current_wp);

	// WARNING : must be done last because it can destroy the current guide entity
	notify_local_entity (ENTITY_MESSAGE_WAYPOINT_REACHED, current_wp, en);
}





		

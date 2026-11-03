

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	guide
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (guide *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_FOLLOWER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_GUIDE);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_GUIDE_STACK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


	//
	// FREE ENTITY DATA
	//


	if (raw->terrain_polygon_list)
	{
		free_mem (raw->terrain_polygon_list);
	}

	free_mem (raw);

	set_free_entity (en);
}






















































































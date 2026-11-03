

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	waypoint
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (waypoint *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_CURRENT_WAYPOINT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_WAYPOINT);


	//
	// FREE ENTITY DATA
	//



	//
	// CHECK ANY POINTERS
	//


	free_mem (raw);

	set_free_entity (en);
}
























































































































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	landing
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (landing *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->assigned_task_root.first_child);


	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_TAKEOFF_QUEUE);







	delete_local_entity_from_parents_child_list (en, LIST_TYPE_LANDING_SITE);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{

	destroy_local_entity_children (en, LIST_TYPE_ASSIGNED_TASK);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{

	destroy_client_server_entity_children (en, LIST_TYPE_ASSIGNED_TASK);

	destroy_client_server_entity (en);
}







































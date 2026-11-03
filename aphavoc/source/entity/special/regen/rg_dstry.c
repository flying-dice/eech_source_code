

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	regen
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (regen *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->member_root.first_child);


	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	delete_local_entity_from_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_REGEN);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{

	destroy_local_entity_children (en, LIST_TYPE_MEMBER);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{

	destroy_client_server_entity_children (en, LIST_TYPE_MEMBER);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{






	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}
























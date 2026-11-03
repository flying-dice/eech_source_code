

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	division
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (division *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_DIVISION);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DIVISION);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DIVISION_HEADQUARTERS);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{
	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{
	destroy_client_server_entity (en);
}







































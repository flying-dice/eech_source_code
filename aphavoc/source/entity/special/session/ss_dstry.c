

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	session
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (session *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->force_root.first_child);


	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);

	unlink_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	set_session_entity (NULL);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{
	destroy_local_entity_children (en, LIST_TYPE_FORCE);



	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{
	destroy_client_server_entity_children (en, LIST_TYPE_FORCE);



	destroy_client_server_entity (en);
}







































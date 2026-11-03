

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	update
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (update *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_UPDATE);

	set_update_entity (NULL);

	set_update_succ (NULL);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}


















void destroy_local_update_entity (void)
{

	//
	// should be destroyed by destroy_local_entity_family
	//

	ASSERT (get_update_entity () == NULL);
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	bridge
		*raw;


	//
	// PRE-AMBLE
	//










	raw = (bridge *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	

	destroy_local_entity_children (en, LIST_TYPE_SEGMENT);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}






















































































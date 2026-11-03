

































































#include "project.h"

#include "ai/taskgen/taskgen.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	force
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (force *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->keysite_force_root.first_child);

	ASSERT (!raw->pilot_root.first_child);


	//
	// DEBUG
	//









	//
	// DESTROY COMPONENTS
	//


	if (raw->campaign_criteria)
	{

		campaign_criteria_type
			*destroy_campaign_criteria;

		while (raw->campaign_criteria)
		{
	
			destroy_campaign_criteria = raw->campaign_criteria;
	
			raw->campaign_criteria = raw->campaign_criteria->next;

			free_mem (destroy_campaign_criteria);
		}
	}


	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_DIVISION);

	unlink_local_entity_children (en, LIST_TYPE_INDEPENDENT_GROUP);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_FORCE);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{
	destroy_local_entity_children (en, LIST_TYPE_KEYSITE_FORCE);

	destroy_local_entity_children (en, LIST_TYPE_PILOT);

	destroy_local_entity_children (en, LIST_TYPE_AIR_REGISTRY);
	destroy_local_entity_children (en, LIST_TYPE_GROUND_REGISTRY);
	destroy_local_entity_children (en, LIST_TYPE_SEA_REGISTRY);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{

	destroy_client_server_entity_children (en, LIST_TYPE_KEYSITE_FORCE);

	destroy_client_server_entity_children (en, LIST_TYPE_PILOT);

	destroy_client_server_entity_children (en, LIST_TYPE_AIR_REGISTRY);
	destroy_client_server_entity_children (en, LIST_TYPE_GROUND_REGISTRY);
	destroy_client_server_entity_children (en, LIST_TYPE_SEA_REGISTRY);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{






	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}


























































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{

	group
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (group *) get_local_entity_data (en);


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


	//
	// if group in registry list. Valid list types are :
	//
	//	LIST_TYPE_AIR_REGISTRY
	//	LIST_TYPE_GROUND_REGISTRY
	//	LIST_TYPE_SEA_REGISTRY
	//	LIST_TYPE_INVALID
	//

	if (get_local_entity_int_value (en, INT_TYPE_REGISTRY_LIST_TYPE) != LIST_TYPE_INVALID)
	{
		delete_local_entity_from_parents_child_list (en, (list_types) get_local_entity_int_value (en, INT_TYPE_REGISTRY_LIST_TYPE));
	}

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DIVISION);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PILOT_LOCK);

	//
	// unlink from keysite
	//

	delete_local_entity_from_parents_child_list (en, raw->group_list_type);

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_GUIDE_STACK);

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




















static void kill_local (entity *en)
{
	int
		sub_type;

	group
		*raw;

	entity
		*force;

	list_types
		list_type;

	ASSERT (en);

	raw = (group *) get_local_entity_data (en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);

	list_type = (list_types) get_local_entity_int_value (en, INT_TYPE_REGISTRY_LIST_TYPE);

	if (list_type != LIST_TYPE_INVALID)
	{
		force = get_local_force_entity ((entity_sides) raw->side);

		ASSERT (force);

		remove_group_type_from_force_info (force, raw->sub_type);
	}

	//
	// Free callsign
	//

	free_group_callsign (en);

	//
	// Group MUST be Empty, and have NO TASKS
	//

	ASSERT (!get_local_entity_first_child (en, LIST_TYPE_MEMBER));

	ASSERT (!get_local_entity_first_child (en, LIST_TYPE_GUIDE_STACK));







	//
	// Kill
	//
	
	if (get_local_entity_int_value (en, INT_TYPE_LOCAL_ONLY_GROUP))
	{
		//
		// local only groups must not be re-used otherwise they will clash when a game is saved + restored
		//

		sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

		delete_local_entity_from_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT);

		unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

		unlink_local_entity_children (en, LIST_TYPE_GUIDE_STACK);

		delete_local_entity_from_parents_child_list (en, raw->group_list_type);

		delete_local_entity_from_parents_child_list (en, list_type);
	
		delete_local_entity_from_parents_child_list (en, LIST_TYPE_PILOT_LOCK);

		delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
	}
	else
	{
		//
		// Destroy entity
		//

		destroy_local_entity (en);
	}
}






















































































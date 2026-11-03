

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	city
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	#endif

	raw = (city *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->city_building_root.first_child);

	// city must not have task dependents
	ASSERT (!raw->fix.task_dependent_root.first_child);

	// city must not have targets
	ASSERT (!raw->fix.target_root.first_child);


	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	//
	// fixed
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DESIGNATED_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_GUNSHIP_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

	//
	// city
	//


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}





static void destroy_remote (entity *en)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_DESTROY, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_DESTROY, en);
}





static void destroy_server (entity *en)
{
	//
	// destroy remote entity first (keeping local entity valid)
	//

	validate_client_server_remote_fn ();

	destroy_remote (en);

	validate_client_server_local_fn ();

	//
	// destroy local using 'full' function
	//

	destroy_local_entity (en);
}





static void destroy_client (entity *en)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		destroy_remote (en);
	}
	else
	{
		validate_client_server_local_fn ();

		//
		// destroy local using 'full' function
		//

		destroy_local_entity (en);
	}
}





static void destroy_local_family (entity *en)
{
	destroy_local_entity_children (en, LIST_TYPE_CITY_BUILDING);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{
	destroy_client_server_entity_children (en, LIST_TYPE_CITY_BUILDING);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_DESTROY_FAMILY, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}





void overload_city_destroy_functions (void)
{
	fn_destroy_local_entity						[ENTITY_TYPE_CITY]							= destroy_local;
	fn_destroy_client_server_entity			[ENTITY_TYPE_CITY][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity			[ENTITY_TYPE_CITY][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_destroy_local_entity_family			[ENTITY_TYPE_CITY]		  		  			= destroy_local_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_CITY][COMMS_MODEL_SERVER]	= destroy_server_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_CITY][COMMS_MODEL_CLIENT]	= destroy_client_family;

	fn_kill_local_entity							[ENTITY_TYPE_CITY]		  		  			= destroy_local_family;
	fn_kill_client_server_entity				[ENTITY_TYPE_CITY][COMMS_MODEL_SERVER]	= destroy_server_family;
	fn_kill_client_server_entity				[ENTITY_TYPE_CITY][COMMS_MODEL_CLIENT]	= destroy_client_family;
}





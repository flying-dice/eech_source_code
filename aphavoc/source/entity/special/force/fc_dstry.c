

































































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


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	#endif

	raw = (force *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->keysite_force_root.first_child);

	ASSERT (!raw->pilot_root.first_child);


	//
	// DEBUG
	//


	#ifdef DEBUG

	display_task_generation_stats (en);

	#endif


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
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_DESTROY_FAMILY, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}





void overload_force_destroy_functions (void)
{
	fn_destroy_local_entity						[ENTITY_TYPE_FORCE]								= destroy_local;
	fn_destroy_client_server_entity			[ENTITY_TYPE_FORCE][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity			[ENTITY_TYPE_FORCE][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_destroy_local_entity_family			[ENTITY_TYPE_FORCE]								= destroy_local_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_FORCE][COMMS_MODEL_SERVER]	= destroy_server_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_FORCE][COMMS_MODEL_CLIENT]	= destroy_client_family;

	fn_kill_local_entity							[ENTITY_TYPE_FORCE]								= destroy_local_family;
	fn_kill_client_server_entity				[ENTITY_TYPE_FORCE][COMMS_MODEL_SERVER]	= destroy_server_family;
	fn_kill_client_server_entity				[ENTITY_TYPE_FORCE][COMMS_MODEL_CLIENT]	= destroy_client_family;
}





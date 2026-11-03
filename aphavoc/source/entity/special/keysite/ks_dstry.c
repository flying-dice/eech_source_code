

































































#include "project.h"

#include "ai/highlevl/highlevl.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	keysite
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	#endif

	raw = (keysite *) get_local_entity_data (en);


	//
	// VALIDATE
	//


	ASSERT (!raw->unassigned_task_root.first_child);

	ASSERT (!raw->assigned_task_root.first_child);

	ASSERT (!raw->completed_task_root.first_child);

	ASSERT (!raw->landing_site_root.first_child);

	ASSERT (!raw->regen_root.first_child);

	ASSERT (!raw->cargo_root.first_child);


	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_BUILDING_GROUP);

	unlink_local_entity_children (en, LIST_TYPE_KEYSITE_GROUP);

	unlink_local_entity_children (en, LIST_TYPE_DIVISION_HEADQUARTERS);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_KEYSITE_FORCE);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

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

	destroy_local_entity_children (en, LIST_TYPE_UNASSIGNED_TASK);

	destroy_local_entity_children (en, LIST_TYPE_ASSIGNED_TASK);

	destroy_local_entity_children (en, LIST_TYPE_COMPLETED_TASK);

	destroy_local_entity_children (en, LIST_TYPE_REGEN);

	destroy_local_entity_children (en, LIST_TYPE_CARGO);

	destroy_local_entity_children (en, LIST_TYPE_LANDING_SITE);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{

	destroy_client_server_entity_children (en, LIST_TYPE_UNASSIGNED_TASK);

	destroy_client_server_entity_children (en, LIST_TYPE_ASSIGNED_TASK);

	destroy_client_server_entity_children (en, LIST_TYPE_COMPLETED_TASK);

	destroy_client_server_entity_children (en, LIST_TYPE_REGEN);

	destroy_client_server_entity_children (en, LIST_TYPE_CARGO);

	destroy_client_server_entity_children (en, LIST_TYPE_LANDING_SITE);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_DESTROY_FAMILY, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}





static void kill_local (entity *en)
{

	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE >= 2

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_KILL, en);

	#endif

	if (!get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		return;
	}


	//
	// VALIDATE
	//



	//
	// KILL
	//


	//
	// Notify campaign screen
	//

	notify_campaign_screen (CAMPAIGN_SCREEN_BASE_DESTROYED, en);

	//
	// Set keysite to unusable
	//
			
	set_local_entity_int_value (en, INT_TYPE_KEYSITE_USABLE_STATE, KEYSITE_STATE_UNUSABLE);

	if (get_local_entity_int_value (en, INT_TYPE_IN_USE))
	{
		set_local_entity_int_value (en, INT_TYPE_IN_USE, FALSE);

		//
		// Update Influence maps
		//
			
		update_imap_sector_side (en, FALSE);

		if (get_comms_model () == COMMS_MODEL_SERVER)
		{
			update_imap_importance_level (en, FALSE);

			update_imap_distance_to_friendly_base ((entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE));

			//
			// Destroy anything landed on keysite - and redirect anything else belonging to keysite
			// (Must be called AFTER keysite IN_USE set to FALSE)
			//

			destroy_keysite (en, (entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE));
		}
	}

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	//
	// Notify force to check campaign criteria
	//

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		entity
			*force;

		force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

		while (force)
		{
			notify_local_entity (ENTITY_MESSAGE_CHECK_CAMPAIGN_OBJECTIVES, force, en);

			force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
		}
	}


	//
	// SPECIAL EFFECTS
	//


	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		ai_log ("(KEYSITE) %s DESTROYED", get_local_entity_string (en, STRING_TYPE_KEYSITE_NAME));

		send_text_message (en, NULL, MESSAGE_TEXT_SYSTEM_KEYSITE_DESTROYED, NULL);
	}


	//
	//

}





static void kill_remote (entity *en)
{
	#if DEBUG_MODULE >= 2

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_KILL, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_KILL, en);
}





static void kill_server (entity *en)
{
	//
	// kill remote entity first (keeping local entity valid)
	//

	validate_client_server_remote_fn ();

	kill_remote (en);

	validate_client_server_local_fn ();

	//
	// kill local using 'full' function
	//

	kill_local_entity (en);
}





static void kill_client (entity *en)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		kill_remote (en);
	}
	else
	{
		validate_client_server_local_fn ();

		//
		// kill local using 'full' function
		//

		kill_local_entity (en);
	}
}





void overload_keysite_destroy_functions (void)
{
	fn_destroy_local_entity						[ENTITY_TYPE_KEYSITE]							= destroy_local;
	fn_destroy_client_server_entity			[ENTITY_TYPE_KEYSITE][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity			[ENTITY_TYPE_KEYSITE][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_destroy_local_entity_family			[ENTITY_TYPE_KEYSITE]							= destroy_local_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_KEYSITE][COMMS_MODEL_SERVER]	= destroy_server_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_KEYSITE][COMMS_MODEL_CLIENT]	= destroy_client_family;

	fn_kill_local_entity							[ENTITY_TYPE_KEYSITE]							= kill_local;
	fn_kill_client_server_entity				[ENTITY_TYPE_KEYSITE][COMMS_MODEL_SERVER]	= kill_server;
	fn_kill_client_server_entity				[ENTITY_TYPE_KEYSITE][COMMS_MODEL_CLIENT]	= kill_client;
}





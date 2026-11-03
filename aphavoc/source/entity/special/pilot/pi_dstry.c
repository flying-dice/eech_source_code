

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	pilot
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (pilot *) get_local_entity_data (en);

	//
	// notify others that player has quit
	//

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		send_pilot_quit_message (en);
	}


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	unlink_local_entity_children (en, LIST_TYPE_PILOT_LOCK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_AIRCREW);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PILOT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PLAYER_TASK);

	//
	// Notify Campaign Screen
	//
	
	campaign_screen_update_chat_page_pilot (en);

	//
	// check if this player entity belongs to the local player
	//

	if (en == get_pilot_entity ())
	{
		//
		// just set raw value (otherwise it will destroy itself again)
		//
		
		debug_log("pilot entity destroyed");
			
		pilot_entity = NULL;
	}


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}





static void destroy_remote (entity *en)
{






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
























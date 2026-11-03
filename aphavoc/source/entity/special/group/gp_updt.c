

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	group
		*raw;

	//
	// Group is only on update list when its sleeping or under attack
	//

	raw = (group *) get_local_entity_data (en);

	if (raw->sleep > 0.0)
	{
		raw->sleep -= get_delta_time ();

		raw->sleep = max (raw->sleep, 0.0f);
	}

	if (raw->assist_timer > 0.0)
	{
		raw->assist_timer -= get_delta_time ();

		raw->assist_timer = max (raw->assist_timer, 0.0f);
	}

	if ((raw->sleep == 0.0) && (raw->assist_timer == 0.0))
	{
		delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
	}
}





void overload_group_update_functions (void)
{
	fn_update_client_server_entity[ENTITY_TYPE_GROUP][COMMS_MODEL_SERVER]  	= update_server;
}





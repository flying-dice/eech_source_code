

































































#include "project.h"





#define DEBUG_MODULE 0





void update_server (entity *en)
{

	#ifdef DEBUG

	#if (LANDING_ROUTE_CHECK)
	{
	
		entity
			*keysite;
	
		keysite = get_local_entity_parent (en, LIST_TYPE_LANDING_SITE);
	
		if (get_local_entity_int_value (keysite, INT_TYPE_IN_USE))
		{

			//if (strcmp (get_local_entity_string (keysite, STRING_TYPE_KEYSITE_NAME), "BATUMI") == 0)
			{
	
				validate_landing_entity_routes (en);
			}
		}
	}
	#endif

	#endif
}





void update_client (entity *en)
{
}





void overload_landing_update_functions (void)
{
	fn_update_client_server_entity[ENTITY_TYPE_LANDING][COMMS_MODEL_SERVER]	= update_server;

	//fn_update_client_server_entity[ENTITY_TYPE_LANDING][COMMS_MODEL_CLIENT]  = update_client;
}





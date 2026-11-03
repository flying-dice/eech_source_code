

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	crater
		*raw;

	raw = (crater *) get_local_entity_data (en);

	raw->crater_lifetime -= get_delta_time();

	if (raw->crater_lifetime <= 0.0)
	{
		raw->crater_lifetime = 0.0;
		destroy_client_server_entity_family (en);
	}
}





static void update_client (entity *en)
{
}





void overload_crater_update_functions (void)
{
	fn_update_client_server_entity[ENTITY_TYPE_CRATER][COMMS_MODEL_SERVER]	= update_server;

	fn_update_client_server_entity[ENTITY_TYPE_CRATER][COMMS_MODEL_CLIENT]	= update_client;
}





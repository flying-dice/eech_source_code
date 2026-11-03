

































































#include "project.h"





#define DEBUG_MODULE 0





void (*fn_update_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);





static void default_update_entity (entity *en)
{
}





void initialise_entity_update_default_functions (void)
{
	int
		i;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		fn_update_client_server_entity[i][COMMS_MODEL_SERVER]	= default_update_entity;

		fn_update_client_server_entity[i][COMMS_MODEL_CLIENT]	= default_update_entity;
	}
}





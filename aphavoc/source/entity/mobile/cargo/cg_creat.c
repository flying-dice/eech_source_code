

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	cargo
		*raw;


  	//
  	// VALIDATE
  	//


	validate_local_create_entity_index (index);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_CREATE, NULL, type, index);

	#endif

	en = get_free_entity (index);

	if (en)
	{

   	//
   	// MALLOC ENTITY DATA
   	//


		set_local_entity_type (en, type);

		raw = (cargo *) malloc_fast_mem (sizeof (cargo));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (cargo));

		//
		// mobile
		//

		raw->mob.sub_type = ENTITY_SUB_TYPE_UNINITIALISED;

		raw->mob.position.x = MID_MAP_X;
		raw->mob.position.y = MID_MAP_Y;
		raw->mob.position.z = MID_MAP_Z;

		get_identity_matrix3x3 (raw->mob.attitude);

		raw->mob.alive = TRUE;

		raw->mob.side = ENTITY_SIDE_UNINITIALISED;

		//
		// cargo
		//


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//



		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// BUILD COMPONENTS
		//



		//
		// LINK INTO SYSTEM
		//


		if (raw->cargo_link.parent)
		{

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_CARGO, raw->cargo_link.parent, NULL);
		}

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->mob.position), NULL);

		//insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
	}

	return (en);
}





static entity *create_remote (entity_types type, int index, char *pargs)
{
	validate_remote_create_entity_index (index);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_CREATE, NULL, type, index);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_CREATE, NULL, type, index, pargs);

	return (NULL);
}





static entity *create_server (entity_types type, int index, char *pargs)
{
	entity
		*en;

	validate_client_server_local_fn ();

	en = create_local (type, index, pargs);

	if (en)
	{
		validate_client_server_remote_fn ();

		create_remote (type, get_local_entity_index (en), pargs);
	}

	return (en);
}





static entity *create_client (entity_types type, int index, char *pargs)
{
	entity
		*en;

	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		en = create_remote (type, index, pargs);
	}
	else
	{
		validate_client_server_local_fn ();

		en = create_local (type, index, pargs);
	}

	return (en);
}





void overload_cargo_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_CARGO]		  								= create_local;

	fn_create_client_server_entity[ENTITY_TYPE_CARGO][COMMS_MODEL_SERVER]	= create_server;

	fn_create_client_server_entity[ENTITY_TYPE_CARGO][COMMS_MODEL_CLIENT]	= create_client;
}








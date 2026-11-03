

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	city_building
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

		raw = (city_building*) malloc_fast_mem (sizeof (city_building));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (city_building));

		//
		// fixed
		//

		raw->fix.position.x = MID_MAP_X;
		raw->fix.position.y = MID_MAP_Y;
		raw->fix.position.z = MID_MAP_Z;

		raw->fix.sub_type = ENTITY_SUB_TYPE_FIXED_UNKNOWN;

		raw->fix.object_3d_shape = OBJECT_3D_INVALID_OBJECT_INDEX;

		raw->fix.alive = TRUE;

		raw->fix.side = ENTITY_SIDE_NEUTRAL;

		//
		// city_building
		//


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->city_building_link.parent);


		//
		// RESOLVE DEFAULT VALUES
		//


		if (raw->fix.object_3d_shape == OBJECT_3D_INVALID_OBJECT_INDEX)
		{
			raw->fix.object_3d_shape = fixed_database[raw->fix.sub_type].default_3d_shape;
		}

		if (fixed_3d_object_database[raw->fix.object_3d_shape].valid_entry)
		{
			raw->damage_level = fixed_3d_object_database[raw->fix.object_3d_shape].initial_damage;
		}
		else
		{
			raw->damage_level = 500;		
		}


		//
		// BUILD COMPONENTS
		//



		//
		// LINK INTO SYSTEM
		//


		insert_local_entity_into_parents_child_list (en, LIST_TYPE_CITY_BUILDING, raw->city_building_link.parent, raw->city_building_link.child_pred);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->fix.position), NULL);
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





void overload_city_building_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_CITY_BUILDING]		  								= create_local;

	fn_create_client_server_entity[ENTITY_TYPE_CITY_BUILDING][COMMS_MODEL_SERVER]	= create_server;

	fn_create_client_server_entity[ENTITY_TYPE_CITY_BUILDING][COMMS_MODEL_CLIENT]	= create_client;
}





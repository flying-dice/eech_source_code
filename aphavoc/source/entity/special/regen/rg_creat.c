

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{

	entity
		*keysite,
		*wp,
		*en;

	regen
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

		raw = (regen *) malloc_fast_mem (sizeof (regen));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (regen));

		raw->position.x = MID_MAP_X;
		raw->position.y = MID_MAP_Y;
		raw->position.z = MID_MAP_Z;

		raw->alive = TRUE;

		raw->side = ENTITY_SIDE_NEUTRAL;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (entity_sub_type_regen_valid (raw->sub_type));


		//
		// RESOLVE DEFAULT VALUES
		//


		raw->sleep = frand1 () * 5.0 * ONE_MINUTE; 


		//
		// BUILD COMPONENTS
		//



		//
		// LINK INTO SYSTEM
		//


		keysite = raw->regen_link.parent;

		ASSERT (keysite);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_REGEN, keysite, NULL);

		wp = raw->current_waypoint_link.parent;

		ASSERT (wp);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT, wp, NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

		#if DEBUG_MODULE
		{
			entity
				*task,
				*landing,
				*keysite;

			task = get_local_entity_parent (wp, LIST_TYPE_WAYPOINT);
			landing = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);
			keysite = get_local_entity_parent (landing, LIST_TYPE_LANDING_SITE);

			debug_log ("RG_CREAT: %s creating regen %s, at keysite %s, time %f", entity_side_names [raw->side], entity_sub_type_regen_names [raw->sub_type],
							get_local_entity_string (keysite, STRING_TYPE_KEYSITE_NAME), regen_frequency [raw->side]);
		}
		#endif
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





void overload_regen_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_REGEN]		  								= create_local;

	fn_create_client_server_entity[ENTITY_TYPE_REGEN][COMMS_MODEL_SERVER]	= create_server;

	fn_create_client_server_entity[ENTITY_TYPE_REGEN][COMMS_MODEL_CLIENT]	= create_client;
}





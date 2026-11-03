

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	pilot
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

		raw = (pilot *) malloc_fast_mem (sizeof (pilot));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (pilot));

		strcpy (raw->pilots_name, "UNKNOWN");

		raw->sub_type = ENTITY_SUB_TYPE_PILOT_PILOT;

		raw->crew_role = CREW_ROLE_PILOT;

		raw->difficulty_level = GAME_DIFFICULTY_INVALID;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->pilot_link.parent);


		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//


		if (raw->aircrew_link.parent)
		{

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_AIRCREW, raw->aircrew_link.parent, raw->aircrew_link.child_pred);
		}

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_PILOT, raw->pilot_link.parent, raw->pilot_link.child_pred);

		#if DEBUG_MODULE

		debug_log ("PI_CREAT: creating pilot %s (%s %s) index %d in helicopter %s",
						raw->pilots_name,
						entity_sub_type_pilot_names [raw->sub_type],
						pilot_rank_names [raw->rank],
						get_local_entity_index (en),
						entity_sub_type_aircraft_names [get_local_entity_int_value (raw->aircrew_link.parent, INT_TYPE_ENTITY_SUB_TYPE)]);

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





void overload_pilot_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_PILOT]											= create_local;

	fn_create_client_server_entity[ENTITY_TYPE_PILOT][COMMS_MODEL_SERVER]	= create_server;

	fn_create_client_server_entity[ENTITY_TYPE_PILOT][COMMS_MODEL_CLIENT]	= create_client;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en,
		*force;

	group
		*raw;

	list_types
		list_type;


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

		raw = (group *) malloc_fast_mem (sizeof (group));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (group));

		raw->sub_type = ENTITY_SUB_TYPE_UNINITIALISED;

		raw->side = ENTITY_SIDE_UNINITIALISED;

		raw->member_count = 0;

		raw->engage_enemy = FALSE;
		
		raw->alive = TRUE;

		strcpy (raw->division_name, "UNKNOWN");

		raw->supplies.ammo_supply_level = 100.0;
		raw->supplies.fuel_supply_level = 100.0;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->side != ENTITY_SIDE_NEUTRAL);

		ASSERT (entity_sub_type_group_valid (raw->sub_type));

		ASSERT (raw->sleep == 0.0);


		//
		// RESOLVE DEFAULT VALUES
		//


		//
		// side
		//

		if (raw->side == ENTITY_SIDE_UNINITIALISED)
		{
			raw->side = get_local_entity_int_value (raw->group_link.parent, INT_TYPE_SIDE);
		}

		// for intermittent map updates of enemy
		raw->last_seen_time = 0.0;
		raw->last_known_position.x = MID_MAP_X;
		raw->last_known_position.y = MID_MAP_Y;
		raw->last_known_position.z = MID_MAP_Z;

		//
		// formation
		//

		raw->group_formation = group_database [raw->sub_type].default_group_formation;

		ASSERT (raw->group_formation < FORMATION_INVALID);

		//
		// Assign callsigns
		//

		raw->group_callsign = assign_group_callsign (en);
		

		//
		// LINK INTO SYSTEM
		//


		ASSERT (raw->group_link.parent);

		raw->group_list_type = group_database [raw->sub_type].group_list_type;

		if (raw->group_list_type == LIST_TYPE_INDEPENDENT_GROUP)
		{
			if (get_local_entity_type (raw->group_link.parent) == ENTITY_TYPE_KEYSITE)
			{
				raw->group_list_type = LIST_TYPE_KEYSITE_GROUP;
			}
		}
			
		insert_local_entity_into_parents_child_list (en, raw->group_list_type, raw->group_link.parent, raw->group_link.child_pred);

		#if DEBUG_MODULE

		debug_log ("GP_CREAT: Created %s (%d) at %s (%d)",
										get_local_entity_string (en, STRING_TYPE_FULL_NAME),
										get_local_entity_index (en),
										get_local_entity_string (raw->group_link.parent, STRING_TYPE_KEYSITE_NAME),
										get_local_entity_index (raw->group_link.parent));

		#endif

		//
		// Link into registry
		//

		list_type = (list_types) get_local_entity_int_value (en, INT_TYPE_REGISTRY_LIST_TYPE);

		if (list_type != LIST_TYPE_INVALID)
		{
			force = get_local_force_entity ((entity_sides) raw->side);

			ASSERT (force);

			insert_local_entity_into_parents_child_list (en, list_type, force, NULL);

			add_group_type_to_force_info (force, raw->sub_type);
		}
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





void overload_group_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_GROUP]											= create_local;

	fn_create_client_server_entity[ENTITY_TYPE_GROUP][COMMS_MODEL_SERVER]	= create_server;

	fn_create_client_server_entity[ENTITY_TYPE_GROUP][COMMS_MODEL_CLIENT]	= create_client;
}





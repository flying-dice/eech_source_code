

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{

	entity
		*en;

	waypoint
		*raw;

	vec3d
		v;


  	//
  	// VALIDATE
  	//


	validate_local_create_entity_index (index);







	en = get_free_entity (index);

	if (en)
	{

   	//
   	// MALLOC ENTITY DATA
   	//


		set_local_entity_type (en, type);

		raw = (waypoint *) malloc_fast_mem (sizeof (waypoint));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (waypoint));

		raw->position.x = MID_MAP_X;
		raw->position.z = MID_MAP_Z;
		raw->position.y = MID_MAP_Y;

		raw->sub_type = ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION;

		raw->waypoint_formation = FORMATION_ROW_LEFT;

		raw->position_type = POSITION_TYPE_ABSOLUTE;

		raw->heading = 0.0;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->waypoint_link.parent);

		get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &v);

		if (!point_inside_map_volume (&v)) // something wrong, replace position with any other
		{
			get_local_entity_vec3d (raw->waypoint_link.parent, VEC3D_TYPE_POSITION, &v);
			v.x += sin(get_local_entity_float_value(raw->waypoint_link.parent, FLOAT_TYPE_HEADING)) * 500;
			v.z += cos(get_local_entity_float_value(raw->waypoint_link.parent, FLOAT_TYPE_HEADING)) * 500;
			bound_position_to_map_area(&v);
			v.y = get_3d_terrain_elevation(v.x, v.z) + 50;
			
			set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &v);
		}

		ASSERT (point_inside_map_volume (&v));
		

		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//


		if (raw->task_dependent_link.parent)
		{

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT, raw->task_dependent_link.parent, NULL);
		}

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_WAYPOINT, raw->waypoint_link.parent, raw->waypoint_link.child_pred);






	}

	return (en);
}





static entity *create_remote (entity_types type, int index, char *pargs)
{
	validate_remote_create_entity_index (index);







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


















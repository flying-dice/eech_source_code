

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	entity
		*parent,
		*next_segment,
		*prev_segment;

	entity_sub_types
		sub_type;

	vec3d
		*pos;

	float
		terrain_height;

	terrain_3d_point_data
		terrain_info;

	//
	// notify the segments neighbours as applicable
	//

	parent = get_local_entity_parent (en, LIST_TYPE_SEGMENT);

	ASSERT (parent);

	next_segment = get_local_entity_child_succ (en, LIST_TYPE_SEGMENT);

	if (next_segment)
	{
		notify_local_entity (ENTITY_MESSAGE_COLLISION, parent, next_segment);
	}

	prev_segment = get_local_entity_child_pred (en, LIST_TYPE_SEGMENT);

	if (prev_segment)
	{
		notify_local_entity (ENTITY_MESSAGE_COLLISION, parent, prev_segment);
	}

	sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

	if (sub_type == ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION)
	{
		//
		// make the segment drop to the floor ( removing it from the update list when it hits )
		//
	
		pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	
		pos->y -= ( 10.0 * get_delta_time() );
	
		memset (&terrain_info, 0, sizeof (terrain_3d_point_data));
	
		ASSERT(point_inside_map_area(pos));

		terrain_height = get_3d_terrain_point_data (pos->x, pos->z, &terrain_info);

		if (get_terrain_type_class (terrain_info.terrain_type) == TERRAIN_CLASS_WATER)
		{
			terrain_height -= 1.0;
		}
	
		if ( pos->y <= terrain_height )
		{
			pos->y = terrain_height;
	
			delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
	
			if (get_comms_model () == COMMS_MODEL_SERVER)
				create_client_server_collision_effect (pos, DYNAMICS_COLLISION_SURFACE_GROUND, 2);
		}
	}
	else
	{
		//
		// segment is supported, and so should be instantly removed from the update list
		// ( only put there in the first place so that neighbours would be notified )
		//

		delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
	}
}





void overload_segment_update_functions (void)
{
	fn_update_client_server_entity[ENTITY_TYPE_SEGMENT][COMMS_MODEL_SERVER]	= update_server;

	fn_update_client_server_entity[ENTITY_TYPE_SEGMENT][COMMS_MODEL_CLIENT]	= update_server;
}





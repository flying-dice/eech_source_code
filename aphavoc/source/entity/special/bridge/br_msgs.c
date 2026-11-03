

































































#include "project.h"

#include "ai/taskgen/taskgen.h"
#include "ai/ai_misc/ai_route.h"





#define DEBUG_MODULE 0





static int response_to_child_killed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	int
		start_node,
		end_node;

	entity_sub_types
		bridge_type;

	entity
		*sec;

	vec3d
		*position;

	bridge_type = get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE);







	start_node = get_local_entity_int_value (receiver, INT_TYPE_START_NODE);

	end_node = get_local_entity_int_value (receiver, INT_TYPE_END_NODE);







	//
	// Break route link
	//

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_SEGMENT);
	
		if (get_local_entity_int_value (receiver, INT_TYPE_ALIVE))
		{

			int
				count;

			set_local_entity_int_value (receiver, INT_TYPE_ALIVE, FALSE);

			count = get_road_link_breaks (start_node, end_node);

			count ++;

			set_road_link_breaks (start_node, end_node, count);

			debug_log ("BR_MSGS: Breaking bridge and route between %d and %d... broken bridge count = %d", count);
	
			//
			// Stop any vehicle on route
			//
	
			stop_vehicles_on_route (start_node, end_node);

			//
			// Create Repair task
			//

			position = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);
				
			sec = get_local_sector_entity (position);

			ASSERT (sec);
			
			if (!create_repair_task ((entity_sides) get_local_entity_int_value (sec, INT_TYPE_SIDE), position, receiver, task_database [ENTITY_SUB_TYPE_TASK_REPAIR].task_priority, NULL, NULL))
			{
				debug_log ("BR_MSGS: can't create repair mission");
			}
			else
			{
				debug_log ("BR_MSGS: created repair mission to [%f, %f, %f]", position->x, position->y, position->z);
			}
		}
	}

	//
	// insert segment into the update list ( so that it can fall to the floor etc. )
	//

	insert_local_entity_into_parents_child_list (sender, LIST_TYPE_UPDATE, get_update_entity (), NULL);

	return (TRUE);
}





// N.B. This function called on both SERVER and CLIENT

static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*pred,
		*succ;

	entity_sub_types
		sub_type;

	//
	// Special case for bridge collision ( actual collision is handled by segments )
	//
	// This fn called by a segment when either of its neighbours have been killed, or it has been hit but not killed
	//

	sub_type = get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE);

	if (sub_type == ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION)
	{
		if (get_local_entity_int_value (sender, INT_TYPE_ALIVE))
		{
			//
			// Bridge section has nothing to support it, so kill it
			//
		
			if (get_comms_model() == COMMS_MODEL_SERVER)
			{
				kill_client_server_entity (sender);


			}
		}
	}

	//
	// change object shape ( depending on state of its neighbours )
	//

	pred = get_local_entity_child_pred (sender, LIST_TYPE_SEGMENT);

	succ = get_local_entity_child_succ (sender, LIST_TYPE_SEGMENT);

	set_local_entity_int_value (sender, INT_TYPE_OBJECT_3D_SHAPE, get_bridge_segment_damaged_object (sender, pred, succ));

	return (TRUE);
}
























































































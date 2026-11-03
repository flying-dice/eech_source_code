

































































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







	return (TRUE);
}





// N.B. This function called on both SERVER and CLIENT









































































































































































































#include "project.h"





#define DEBUG_MODULE 0
























































































































































































































































































































void stop_vehicles_on_route (int start_node, int end_node)
{
	entity
		*task,
		*force,
		*group,
		*member;

	int
		index1,
		index2;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (force)
	{
		group = get_local_entity_first_child (force, LIST_TYPE_GROUND_REGISTRY);

		while (group)
		{
			task = get_local_group_primary_task (group);

			if (task)
			{
				member = (entity *) get_local_entity_ptr_value (group, PTR_TYPE_GROUP_LEADER);
	
				ASSERT (member);
			
				index1 = get_local_entity_int_value (member, INT_TYPE_WAYPOINT_THIS_INDEX);
	
				index2 = get_local_entity_int_value (member, INT_TYPE_WAYPOINT_NEXT_INDEX);
	
				if (((index1 == start_node) && (index2 == end_node)) || ((index2 == start_node) && (index1 == end_node)))
				{
					//
					// destroy task
					//
	
					notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, task, group, TASK_TERMINATED_GROUP_DESTROYED);

					//
					// set group to busy, and operational state to stopped
					//
	
					set_client_server_entity_int_value (group, INT_TYPE_GROUP_MODE, GROUP_MODE_BUSY);

					{
						entity
							*mb;

						mb = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

						while (mb)
						{
							set_client_server_entity_int_value (mb, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_STOPPED);

							mb = get_local_entity_child_succ (mb, LIST_TYPE_MEMBER);
						}
					}


					//
					// SPECIAL_EFFECT_HOOK FOR BRIDGE OUT
					//

	

					//
					//

				}
			}
	
			group = get_local_entity_child_succ (group, LIST_TYPE_GROUND_REGISTRY);
		}

		force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
	}
}







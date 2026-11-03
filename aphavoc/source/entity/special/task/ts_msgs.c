

































































#include "project.h"






//
// 2 levels of debug
//


#define DEBUG_MODULE 0



































static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;

	entity_sub_types
		sub_type;







	//
	// notify campaign screen
	//

	list_type = va_arg (pargs, list_types);

	sub_type = get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE);

	switch (list_type)
	{

		case LIST_TYPE_UNASSIGNED_TASK:

		{
			set_local_entity_int_value (receiver, INT_TYPE_TASK_STATE, TASK_STATE_UNASSIGNED);

			if (task_database [sub_type].primary_task)
			{
				notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_CREATED, receiver);
			}

			break;
		}

		case LIST_TYPE_ASSIGNED_TASK:

		{
			set_local_entity_int_value (receiver, INT_TYPE_TASK_STATE, TASK_STATE_ASSIGNED);

			if (task_database [sub_type].primary_task)
			{
				notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_ASSIGNED, receiver);
			}

			break;
		}

		case LIST_TYPE_COMPLETED_TASK:

		{
			set_local_entity_int_value (receiver, INT_TYPE_TASK_STATE, TASK_STATE_COMPLETED);

			if (task_database [sub_type].primary_task)
			{
				notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_COMPLETED, receiver);
			}

			break;
		}
	}

	return (TRUE);
}















































































static int response_to_task_completed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	task
		*raw;

	entity
		*force,
		*guide,
		*list_parent;

	task_terminated_types
		task_terminated;

	task_completed_types
		task_completed;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	task_terminated = (task_terminated_types) va_arg (pargs, int);

	raw = (task *) get_local_entity_data (receiver);

	if (raw->task_completed == TASK_INCOMPLETE)
	{
		if (task_database [raw->sub_type].primary_task)
		{
			switch (raw->task_state)
			{
				case TASK_STATE_UNASSIGNED:
				{
					//
					// Terminate task
					//
	
					set_client_server_entity_int_value (receiver, INT_TYPE_TASK_COMPLETED, TASK_COMPLETED_FAILURE);

					notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, receiver, sender, TASK_TERMINATED_OBJECTIVE_MESSAGE);

					break;
				}
				case TASK_STATE_ASSIGNED:
				{
					//
					// Assess task rating
					//
					
					task_completed = assess_task_completeness (receiver, task_terminated);

					if (task_completed != TASK_INCOMPLETE)
					{
						ASSERT (task_completed != raw->task_completed);
	
						set_client_server_entity_int_value (receiver, INT_TYPE_TASK_COMPLETED, task_completed);

//						#if DEBUG_MODULE
					
						debug_colour_log (DEBUG_COLOUR_GREEN, "TS_MSGS: %s Task %s (%d) completed - Rating %.1f (%s)",
													entity_side_names [get_local_entity_int_value (receiver, INT_TYPE_SIDE)],
													get_local_entity_string (receiver, STRING_TYPE_FULL_NAME),
													get_local_entity_index (receiver),
													(get_local_entity_float_value (receiver, FLOAT_TYPE_RATING) * 100.0),
													task_terminated_names [task_terminated]
												);

//						#endif

						//
						// Award Task Points
						//

						award_points_for_task_completion (receiver);

						//
						// Notify force 
						//		
			
						force = get_local_force_entity ((entity_sides) raw->side);
	
						notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, force, receiver, task_terminated);

						//
						// move the task to the sessions completed task list
						//

						list_parent = get_local_entity_parent (receiver, LIST_TYPE_ASSIGNED_TASK);

						ASSERT (list_parent);

						set_client_server_entity_parent (receiver, LIST_TYPE_ASSIGNED_TASK, NULL);

						set_client_server_entity_parent (receiver, LIST_TYPE_COMPLETED_TASK, list_parent);

						//
						// Notify Guide which will inform group and return them to base
						//
					
						guide = get_local_entity_first_child (receiver, LIST_TYPE_GUIDE);
					
						while (guide)
						{
							notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, guide, receiver, task_terminated);
					
							guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE);
						}
					}

					break;
				}
			}
		}
		else
		{
			//
			// Terminate task
			//

			set_client_server_entity_int_value (receiver, INT_TYPE_TASK_COMPLETED, TASK_COMPLETED_FAILURE);

			notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, receiver, sender, TASK_TERMINATED_OBJECTIVE_MESSAGE);
		}
	}

	return (TRUE);
}





static int response_to_task_terminated (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	task
		*raw;

	entity
		*guide,
		*next,
		*en;

	task_terminated_types
		task_terminated;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (receiver);

	raw = (task *) get_local_entity_data (receiver);

	if (task_database [raw->sub_type].persistent_task)
	{
		//
		// Landing / Takeoff etc SHOULD NOT be terminated
		//

//		debug_fatal ("TS_MSGS: Task %s incorrectly terminated", get_local_entity_string (receiver, STRING_TYPE_FULL_NAME));

		ASSERT (!"TS_MSGS: Persistent task incorrectly terminated");
		return FALSE;
	}

	//
	// Check for cyclic conditions
	//

	if (raw->task_terminated != TASK_TERMINATED_IN_PROGRESS)
	{
		return FALSE;
	}

	task_terminated = va_arg (pargs, task_terminated_types);










	raw->task_terminated = task_terminated;

	//
	// Call completed code also
	//

	if (raw->task_completed == TASK_INCOMPLETE)
	{
		notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, receiver, sender, task_terminated);
	}

	//
	// Inform dependents task terminated
	//

	en = get_local_entity_first_child (receiver, LIST_TYPE_TASK_DEPENDENT);

	while (en)
	{
		next = get_local_entity_child_succ (en, LIST_TYPE_TASK_DEPENDENT);

		if (en != sender)
		{
			notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, en, receiver, task_terminated);
		}

		en = next;
	}

	ASSERT (get_local_entity_type (receiver) == ENTITY_TYPE_TASK);		// Check task has not been prematurely destroyed by dependents

	//
	// Inform GUIDE ENTITY task terminated (This will sever all links between the task and it's guide / group / members etc.)
	//

	do
	{
		guide = get_local_entity_first_child (receiver, LIST_TYPE_GUIDE);

		if (guide)
		{
			notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, guide, receiver, task_terminated);
		}
	}
	while (guide);

	if ((raw->task_state != TASK_STATE_UNASSIGNED) && (task_database [raw->sub_type].visible_task))
	{
		//
		// Destroy Links
		//
	
		ASSERT (task_database [raw->sub_type].primary_task);

		kill_client_server_entity (receiver);
	}
	else
	{
		//
		// Destroy task
		//
			
		destroy_local_entity_family (receiver);
	
		transmit_entity_comms_message (ENTITY_COMMS_DESTROY_LOCAL_FAMILY, receiver);
	}

	return (TRUE);
}









































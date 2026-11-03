

































































#include "project.h"





#define DEBUG_MODULE 0

































































static int response_to_waypoint_action (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*current_wp;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (get_local_entity_int_value (receiver, INT_TYPE_CHECK_WAYPOINT_ACTION));

	current_wp = get_local_entity_parent (receiver, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (current_wp);

	ASSERT (get_local_entity_int_value (current_wp, INT_TYPE_CHECK_WAYPOINT_ACTION));

	notify_local_entity (ENTITY_MESSAGE_WAYPOINT_ACTION, current_wp, receiver);

	set_local_entity_int_value (receiver, INT_TYPE_CHECK_WAYPOINT_ACTION, FALSE);

	return (TRUE);
}





static int response_to_waypoint_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	switch (get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE))
	{
		//
		// NAVIGATION GUIDES
		//
		
		case ENTITY_SUB_TYPE_GUIDE_NAVIGATION_DIRECT:
		case ENTITY_SUB_TYPE_GUIDE_NAVIGATION_VIRTUAL:
		case ENTITY_SUB_TYPE_GUIDE_NAVIGATION_ROUTED:
		case ENTITY_SUB_TYPE_GUIDE_NAVIGATION_ALTITUDE:
		case ENTITY_SUB_TYPE_GUIDE_LANDING_DIRECT:
		case ENTITY_SUB_TYPE_GUIDE_LANDED:
		{
			navigation_guide_waypoint_reached (receiver);

			break;
		}

		//
		// AIR-TO-AIR ATTACK GUIDES
		//

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_MOVE_SIX:
		{
			attack_guide_move_six_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_MOVE_CIRCLE:
		{
			attack_guide_move_circle_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_FIRE_INTERCEPT:
		{
			attack_guide_fire_intercept_reached (receiver);

			break;
		}

		//
		// AIR-TO-GROUND ATTACK GUIDES
		//

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_APPROACH:
		{
			attack_guide_approach_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_SEEK_COVER:
		{
			attack_guide_seek_cover_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FLY_TO_COVER:
		{
			attack_guide_fly_to_cover_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_TAKE_COVER:
		{
			attack_guide_take_cover_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_CLIMB:
		{
			attack_guide_climb_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DIVE:
		{
			attack_guide_dive_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE:
		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE_VIRTUAL:
		{
			attack_guide_fire_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DISENGAGE:
		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DISENGAGE_VIRTUAL:
		{
			attack_guide_disengage_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_EGRESS:
		{
			attack_guide_egress_reached (receiver);

			break;
		}

		//
		// HASTY AIR-TO-GROUND ATTACK GUIDES (HELICOPTERS ONLY)
		//

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_HASTY_FIRE:
		{
			attack_guide_hasty_fire_reached (receiver);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_HASTY_TAKE_COVER:
		{
			attack_guide_hasty_take_cover_reached (receiver);

			break;
		}

		default:
		{
			debug_fatal ("GD_MSGS: Invalid guide type %d", get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE));
		}
	}

	return (TRUE);
}





static int response_to_task_completed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*group;

	task_terminated_types
		task_terminated;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_TASK);

	task_terminated = (task_terminated_types) va_arg (pargs, int);

	group = get_local_entity_parent (receiver, LIST_TYPE_GUIDE_STACK);
		
	ASSERT (group);

	notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, group, receiver, task_terminated);

	return TRUE;
}





static int response_to_task_terminated (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*group;

	task_terminated_types
		task_terminated;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_TASK);

	task_terminated = va_arg (pargs, task_terminated_types);

	group = get_local_entity_parent (receiver, LIST_TYPE_GUIDE_STACK);
		
	ASSERT (group);

	//
	// Inform GROUP ENTITY task terminated 
	//

	notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, group, receiver, task_terminated);

	//
	// Destroy GUIDE entity
	//

	//
	// NOTE:- The following two "if" statements are only necessary because notifying the group (above) can currently
	// kill the group (thus destroying the guides itself) if it cannot return to a base.
	// This should be resolved once return tasks are generated with the primary task.
	//

	if (get_local_entity_type (receiver) == ENTITY_TYPE_GUIDE)
	{
		if (get_local_entity_parent (receiver, LIST_TYPE_GUIDE) == sender)
		{
			destroy_client_server_entity (receiver);
		}
	}

	return TRUE;
}





static int response_to_group_killed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*task;

	int
		task_type;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_GROUP);

	//
	// Inform TASK ENTITY task terminated  (N.B. This will destroy this GUIDE)
	//

	task = get_local_entity_parent (receiver, LIST_TYPE_GUIDE);
		
	ASSERT (task);
	
	task_type = get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE);

	if (task_database [task_type].persistent_task)
	{
		destroy_client_server_entity (receiver);
	}
	else
	{
		notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, task, receiver, TASK_TERMINATED_GROUP_DESTROYED);
	}

	return TRUE;
}


































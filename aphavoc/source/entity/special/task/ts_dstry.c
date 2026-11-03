

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{

	task_kill_loss_data
		*destroy_kill,
		*kill;

	task
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (task *) get_local_entity_data (en);

	if (raw->player_task)
	{
		debug_log ("TS_DSTRY : destroying player task %d", get_local_entity_safe_index (en));
	}


	//
	// VALIDATE
	//


	ASSERT (!raw->guide_root.first_child);

	ASSERT (!raw->waypoint_root.first_child);


	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	// leave on task lists for destroy

	unlink_local_entity_children (en, LIST_TYPE_PLAYER_TASK);

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, get_local_task_list_type (en));

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PILOT_LOCK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR_TASK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);











	//
	// FREE ENTITY DATA
	//


	if (raw->route_dependents)
	{

		free_mem (raw->route_dependents);
	}

	if (raw->route_formation_types)
	{

		free_mem (raw->route_formation_types);
	}

	if (raw->route_waypoint_types)
	{

		free_mem (raw->route_waypoint_types);
	}

	if (raw->route_nodes)
	{

		free_mem (raw->route_nodes);
	}

	// kills

	kill = raw->task_kills;

	while (kill)
	{

		destroy_kill = kill;

		kill = kill->next;

		free_mem (destroy_kill);
	}

	// losses

	kill = raw->task_losses;

	while (kill)
	{

		destroy_kill = kill;

		kill = kill->next;

		free_mem (destroy_kill);
	}

	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{

	destroy_local_entity_children (en, LIST_TYPE_GUIDE);

	destroy_local_entity_children (en, LIST_TYPE_WAYPOINT);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{

	destroy_client_server_entity_children (en, LIST_TYPE_GUIDE);

	destroy_client_server_entity_children (en, LIST_TYPE_WAYPOINT);

	destroy_client_server_entity (en);
}




















static void kill_local (entity *en)
{
	task
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (task *) get_local_entity_data (en);

	ASSERT (get_local_task_list_type (en) != LIST_TYPE_UNASSIGNED_TASK);

	ASSERT (raw->task_state != TASK_STATE_UNASSIGNED);

	ASSERT (task_database [raw->sub_type].primary_task);
	
	ASSERT (task_database [raw->sub_type].visible_task);
	
	// task_dependent_root

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PILOT_LOCK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR_TASK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT);

	destroy_local_entity_children (en, LIST_TYPE_GUIDE);

	//
	// Set EXPIRE timer so the the task will be DESTROYED after a certain time
	//

	set_local_entity_float_value (en, FLOAT_TYPE_EXPIRE_TIMER, get_completed_task_expire_time ());

	debug_assert (raw->expire_timer > 0.0);
}






















































































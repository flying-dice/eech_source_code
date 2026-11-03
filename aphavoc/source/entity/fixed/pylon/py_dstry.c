

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	pylon
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	#endif

	raw = (pylon *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	//
	// fixed
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DESIGNATED_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_GUNSHIP_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

	//
	// pylon
	//


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}





static void destroy_remote (entity *en)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_DESTROY, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_DESTROY, en);
}





static void destroy_server (entity *en)
{
	//
	// destroy remote entity first (keeping local entity valid)
	//

	validate_client_server_remote_fn ();

	destroy_remote (en);

	validate_client_server_local_fn ();

	//
	// destroy local using 'full' function
	//

	destroy_local_entity (en);
}





static void destroy_client (entity *en)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		destroy_remote (en);
	}
	else
	{
		validate_client_server_local_fn ();

		//
		// destroy local using 'full' function
		//

		destroy_local_entity (en);
	}
}





static void kill_local (entity *en)
{
	vec3d
		pos;

	entity
		*task,
		*destroy_task;

	pylon
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_KILL, en);

	#endif

	raw = (pylon *) get_local_entity_data (en);


	//
	// UNLINK FROM SYSTEM
	//


	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		task = get_local_entity_first_child (en, LIST_TYPE_TASK_DEPENDENT);

		while (task)
		{

			destroy_task = task;

			task = get_local_entity_child_succ (task, LIST_TYPE_TASK_DEPENDENT);

			if (destroy_task->type == ENTITY_TYPE_TASK)
			{

				#if DEBUG_MODULE

				debug_log ("PY_DSTRY: killing pylon, notifying task %s complete", entity_sub_type_task_names [get_local_entity_int_value (destroy_task, INT_TYPE_ENTITY_SUB_TYPE)]);

				#endif

				notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, destroy_task, en, TASK_TERMINATED_OBJECTIVE_MESSAGE);
			}
		}
	}

	//
	// fixed
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	// gunship_target_link

	// sector_link

	//
	// pylon
	//


	//
 	// KILL
	//


	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);


	//
	// SPECIAL EFFECTS
	//


	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &pos);

		ASSERT(point_inside_map_area(&pos));
	
		pos.y = get_3d_terrain_elevation (pos.x, pos.z);

		create_client_server_crater (CRATER_TYPE_MEDIUM_EXPLOSION, &pos);

		create_client_server_meta_smoke_list_specified_offset (META_SMOKE_LIST_TYPE_FIRE_SHORT_DURATION, NULL, &pos);

		create_client_server_object_killed_explosion_effect (en);
	}

	raw->fix.object_3d_shape = get_3d_object_destroyed_object_index (raw->fix.object_3d_shape);
}





static void kill_remote (entity *en)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_KILL, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_KILL, en);
}





static void kill_server (entity *en)
{
	//
	// kill remote entity first (keeping local entity valid)
	//

	validate_client_server_remote_fn ();

	kill_remote (en);

	validate_client_server_local_fn ();

	//
	// kill local using 'full' function
	//

	kill_local_entity (en);
}





static void kill_client (entity *en)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		kill_remote (en);
	}
	else
	{
		validate_client_server_local_fn ();

		//
		// kill local using 'full' function
		//

		kill_local_entity (en);
	}
}





void overload_pylon_destroy_functions (void)
{
	fn_destroy_local_entity						[ENTITY_TYPE_PYLON]		  						= destroy_local;
	fn_destroy_client_server_entity			[ENTITY_TYPE_PYLON][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity			[ENTITY_TYPE_PYLON][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_destroy_local_entity_family			[ENTITY_TYPE_PYLON]		  		  			  	= destroy_local;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_PYLON][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_PYLON][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_kill_local_entity							[ENTITY_TYPE_PYLON]		  						= kill_local;
	fn_kill_client_server_entity				[ENTITY_TYPE_PYLON][COMMS_MODEL_SERVER]	= kill_server;
	fn_kill_client_server_entity				[ENTITY_TYPE_PYLON][COMMS_MODEL_CLIENT]	= kill_client;
}





void destroy_local_pylon_entities (void)
{

}





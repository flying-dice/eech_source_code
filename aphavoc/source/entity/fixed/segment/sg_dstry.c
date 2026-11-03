

































































#include "project.h"





#define DEBUG_MODULE 0





static void kill_routed_vehicles_on_segment (entity *en);





static void destroy_local (entity *en)
{
	segment
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	debug_log ("SG_DSTRY: destroying segment %d", get_local_entity_index (en));

	#endif

	raw = (segment *) get_local_entity_data (en);


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
	// segment
	//

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SEGMENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


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
	entity
		*parent;

	entity
		*task,
		*destroy_task;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_KILL, en);

	#endif


	//
	// VALIDATE
	//


	ASSERT (get_local_entity_int_value (en, INT_TYPE_ALIVE));

	parent = get_local_entity_parent (en, LIST_TYPE_SEGMENT);

	ASSERT (parent);


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

				debug_log ("SG_DSTRY: killing segment, notifying task %s complete", entity_sub_type_task_names [get_local_entity_int_value (destroy_task, INT_TYPE_ENTITY_SUB_TYPE)]);

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
	// segment
	//

	// segment_link

	// update_link


	//
	// KILL
	//


	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);


	//
	// SPECIAL EFFECTS
	//


	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		create_client_server_object_killed_explosion_effect (en);

		//
		// kill any routed vehicles that are on the bridge at the time
		//

		kill_routed_vehicles_on_segment (en);
	}

	//
	// notify the parent's collision function ( primarily to change the objects shape ),
	// and the parents killed function ( to handle breaking of links, and setting off chain reactions etc. )
	//

	notify_local_entity (ENTITY_MESSAGE_COLLISION, parent, en);

	notify_local_entity (ENTITY_MESSAGE_CHILD_KILLED, parent, en);
}





static void kill_remote (entity *en)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_kill, en);

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





void overload_segment_destroy_functions (void)
{
	fn_destroy_local_entity						[ENTITY_TYPE_SEGMENT]							= destroy_local;
	fn_destroy_client_server_entity			[ENTITY_TYPE_SEGMENT][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity 			[ENTITY_TYPE_SEGMENT][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_destroy_local_entity_family			[ENTITY_TYPE_SEGMENT]							= destroy_local;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_SEGMENT][COMMS_MODEL_SERVER]	= destroy_server;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_SEGMENT][COMMS_MODEL_CLIENT]	= destroy_client;

	fn_kill_local_entity							[ENTITY_TYPE_SEGMENT]							= kill_local;
	fn_kill_client_server_entity				[ENTITY_TYPE_SEGMENT][COMMS_MODEL_SERVER]	= kill_server;
	fn_kill_client_server_entity				[ENTITY_TYPE_SEGMENT][COMMS_MODEL_CLIENT]	= kill_client;
}





void kill_routed_vehicles_on_segment (entity *en)
{
	float
		range,
		damage_radius;

	int
		sx,
		sz;

	vec3d
		*segment_position,
		*object_position;

	entity
		*sect,
		*object;

	bridge_segment_types
		bridge_segment_type;

	//
	// get damage radius ( plus a little overlap )
	//

	bridge_segment_type = (bridge_segment_types) get_local_entity_int_value (en, INT_TYPE_BRIDGE_SEGMENT_TYPE);

	damage_radius = bridge_segment_length (bridge_segment_type);

	if (damage_radius == 0.0)
	{
		return;
	}

	damage_radius *= ROOT2;

	//
	// get sector which the segment is in ( don't bother with adjacent sectors for now )
	//

	segment_position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	get_x_sector (sx, segment_position->x);
	get_z_sector (sz, segment_position->z);

	sect = get_local_raw_sector_entity (sx, sz);

	//
	// search for routed vehicles
	//

	object = get_local_entity_first_child (sect, LIST_TYPE_SECTOR);

	while (object)
	{
		if (object->type == ENTITY_TYPE_ROUTED_VEHICLE)
		{
			if (get_local_entity_int_value (object, INT_TYPE_ALIVE))
			{
				object_position = get_local_entity_vec3d_ptr (object, VEC3D_TYPE_POSITION);

				if (object_position)
				{
					range = get_approx_3d_range (segment_position, object_position);

					if (range < damage_radius)
					{
						kill_client_server_entity (object);
					}
				}
			}
		}

		object = get_local_entity_child_succ (object, LIST_TYPE_SECTOR);
	}
}





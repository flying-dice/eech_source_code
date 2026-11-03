

































































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





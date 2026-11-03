

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	site_updatable
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (site_updatable *) get_local_entity_data (en);


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
	// site_updatable
	//

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void kill_local (entity *en)
{
	site_updatable
		*raw;

	vec3d
		pos;

	entity
		*task,
		*destroy_task;


	//
	// PRE-AMBLE
	//








	raw = (site_updatable *) get_local_entity_data (en);


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
	// site
	//

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


	//
 	// KILL
	//


	// must be done BEFORE alive flag set
	subtract_local_entity_importance_from_keysite (en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);


	//
	// SPECIAL EFFECTS
	//













	//
	// must be done AFTER object explosion
	//

	raw->fix.object_3d_shape = get_3d_object_destroyed_object_index (raw->fix.object_3d_shape);
}






















































































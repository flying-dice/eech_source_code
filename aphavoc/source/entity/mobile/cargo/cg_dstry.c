

































































#include "project.h"





// 2 levels of debug

#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{

	cargo
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (cargo *) get_local_entity_data (en);


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
	// cargo
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_CARGO);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	//
	// mobile
	//

	unlink_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{
	destroy_local_sound_effects (en);

	destroy_local_entity (en);
}































static void kill_local (entity *en)
{

	entity
		*task,
		*destroy_task;

	cargo
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (cargo *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// KILL
	//


	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);


	//
	// SEND NOTIFY MESSAGES
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


		//
		// SEND NOTIFY MESSAGES
		//

	}


	//
	// UNLINK FROM SYSTEM
	//


	//
	// cargo
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_CARGO);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	// view_link

	//
	// mobile
	//

	unlink_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	// sector_link

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

	// update_link


	//
	// SPECIAL EFFECTS
	//

}






















































































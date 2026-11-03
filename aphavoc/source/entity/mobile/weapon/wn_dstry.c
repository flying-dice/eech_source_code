

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	weapon
		*raw;
	entity
		*task,
		*destroy_task;


	//
	// PRE-AMBLE
	//


	raw = (weapon *) get_local_entity_data (en);








	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



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
	// if the weapon is a decoy then any locked on weapons should re-acquire lock on the target (decoy launcher) if any
	//






	if (weapon_database[raw->mob.sub_type].decoy_type != WEAPON_DECOY_TYPE_NONE)
	{
		if (raw->launched_weapon_link.parent)
		{
			while (raw->mob.target_root.first_child)
			{
				notify_local_entity (ENTITY_MESSAGE_DECOY_EXPIRED, raw->mob.target_root.first_child, raw->launched_weapon_link.parent);
			}
		}
	}

	// notify player log of weapon fired

	if(raw->launched_weapon_link.parent == get_gunship_entity ())
	{
		inc_player_weapon_log_fired (get_global_gunship_side (), get_current_player_log (), raw->mob.sub_type);

		if (raw->kill_code == WEAPON_KILL_CODE_HIT_TARGET)
		{
			inc_player_weapon_log_hit (get_global_gunship_side (), get_current_player_log (), raw->mob.sub_type);

			debug_log("WN_DSTRY: hit target!");
		}
	}


	//
	// weapon
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_LAUNCHED_WEAPON);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_VIEW_WEAPON);

	//
	// mobile
	//

	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ENGINE_LOOPING1);

	unlink_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

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





static void destroy_server_family (entity *en)
{
	destroy_client_server_sound_effects (en);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{






	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}
























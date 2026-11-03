

































































#include "project.h"

#include "entity/tacview/tacview.h"





// 2 levels of debug

#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{

	routed_vehicle
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (routed_vehicle *) get_local_entity_data (en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//


	destruct_3d_object (raw->vh.inst3d);

	free_mem (raw->vh.weapon_package_status_array);

	if (raw->vh.sound_effect_data)
	{

		destroy_system_sound_effect (raw->vh.sound_effect_data);
	}


	//
	// UNLINK FROM SYSTEM
	//


	//
	// routed_vehicle
	//

	//
	// vehicle
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_LAUNCHED_WEAPON);

	unlink_local_entity_children (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_FOLLOWER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DESIGNATED_TARGET); // Jabberwock 031107 Designated targets

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_GUNSHIP_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_VIEW);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TAKEOFF_QUEUE);

	//
	// mobile
	//

	unlink_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);

	#if LANDING_ROUTE_CHECK

	destroy_debug_entity_landing_route_check (en);

	#endif


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}



































































static void destroy_local_family (entity *en)
{
	destroy_local_entity_children (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	destroy_local_sound_effects (en);

	destroy_local_entity (en);
}

































static void kill_local (entity *en)
{

	int
		losses;

	routed_vehicle
		*raw;

	entity
		*task,
		*destroy_task,
		*group;


	//
	// PRE-AMBLE
	//








	if (!get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		return;
	}

	raw = (routed_vehicle *) get_local_entity_data (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);




	ASSERT (group);

	//
	// update force info
	//

	remove_from_force_info (get_local_force_entity ((entity_sides) raw->vh.mob.side), en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//


	if (get_comms_model () == COMMS_MODEL_SERVER)
	{

		//
		// SEND NOTIFY MESSAGES
		//


		// notify regen of kill

		add_entity_to_regen_queue
		(
			(entity_sides) get_local_entity_int_value(en, INT_TYPE_SIDE),
			get_local_entity_type(en),
			get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE),
			get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)
		);

		//
		//
		//

		task = get_local_entity_first_child (en, LIST_TYPE_TASK_DEPENDENT);

		while (task)
		{

			destroy_task = task;

			task = get_local_entity_child_succ (task, LIST_TYPE_TASK_DEPENDENT);

			if (get_local_entity_type (destroy_task) == ENTITY_TYPE_TASK)
			{







				notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, destroy_task, en, TASK_TERMINATED_OBJECTIVE_MESSAGE);
			}
		}

		//
		// Release landing lock (if any)
		//

		release_mobile_entity_landing_locks (en);
	}


	//
	// UNLINK FROM SYSTEM
	//


	//
	// routed_vehicle
	//

	//
	// vehicle
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_MOVEMENT_DEPENDENT);







	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_FOLLOWER);

	// gunship_target_link

	// member_link

	// view_link

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TAKEOFF_QUEUE);

	//
	// mobile
	//

	//
	// kill weapon sound effects
	//

	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_CHAIN_GUN);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_GATLING_GUN);

	//
	// kill engine sound effects
	//

	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ENGINE_LOOPING1);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ENGINE_LOOPING2);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	// sector_link

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

	// update_link

	set_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_DYING);


	//
	// KILL
	//


	// must be done before alive flag set to FALSE
	remove_mobile_values_from_sector (get_local_entity_parent (en, LIST_TYPE_SECTOR), en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);

	damage_routed_vehicle_3d_object (en);

	//
	// group losses
	//

	losses = get_local_entity_int_value (group, INT_TYPE_LOSSES);

	losses ++;

	set_local_entity_int_value (group, INT_TYPE_LOSSES, losses);

	//
	// task losses
	//

	task = get_local_group_primary_task (group);

	if (task)
	{
		losses = get_local_entity_int_value (task, INT_TYPE_LOSSES);

		losses ++;

		set_local_entity_int_value (task, INT_TYPE_LOSSES, losses);
	}

	//
	// Notify Campaign Screen
	//

	notify_campaign_screen (CAMPAIGN_SCREEN_GROUP_REMOVE_MEMBER, group);

	//
	// Notify the GROUP that the mobile has been killed  (N.B. must be done AFTER mobile is unlinked from member list)
	//

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		notify_local_entity (ENTITY_MESSAGE_MOBILE_KILLED, group, en);
	}


	//
	// SPECIAL EFFECTS
	//


	set_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER, 0);

	routed_vehicle_impact_movement (en);

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{

	}

	#if LANDING_ROUTE_CHECK

	destroy_debug_entity_landing_route_check (en);

	#endif


	//
	// SPECIAL_EFFECT_HOOK FOR BEING_DESTROYED
	//



	//
	//

}






















































































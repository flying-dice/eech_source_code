

































































#include "project.h"





// 2 levels of debug

#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{

	person
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE >= 2

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	#endif

	raw = (person *) get_local_entity_data (en);

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
	// person
	//

	//
	// vehicle
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_LAUNCHED_WEAPON);

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


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}





static void destroy_remote (entity *en)
{
	#if DEBUG_MODULE >= 2

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





static void destroy_local_family (entity *en)
{
	destroy_local_entity_children (en, LIST_TYPE_MOVEMENT_DEPENDENT);

//	destroy_local_entity_children (en, LIST_TYPE_WAYPOINT);

	destroy_local_sound_effects (en);

	destroy_local_entity (en);
}





static void destroy_server_family (entity *en)
{
	destroy_client_server_entity_children (en, LIST_TYPE_MOVEMENT_DEPENDENT);

//	destroy_client_server_entity_children (en, LIST_TYPE_WAYPOINT);

	destroy_client_server_sound_effects (en);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{
	#if DEBUG_MODULE >= 2

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_DESTROY_FAMILY, en);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}





static void kill_local (entity *en)
{

	person
		*raw;

	int
		losses;

	entity
		*task,
		*destroy_task,
		*group;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE >= 2

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_KILL, en);

	#endif

	if (!get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		return;
	}

	raw = (person *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// KILL
	//


	// must be done before alive flag set to FALSE
	remove_mobile_values_from_sector (get_local_entity_parent (en, LIST_TYPE_SECTOR), en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);

	// update person object
	damage_person_3d_object (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	ASSERT (group);

	//
	// group losses
	//

	losses = get_local_entity_int_value (group, INT_TYPE_LOSSES);

	losses ++;

	set_local_entity_int_value (group, INT_TYPE_LOSSES, losses);


	//
	// DESTROY COMPONENTS
	//


	if (get_comms_model () == COMMS_MODEL_SERVER)
	{

		//
		// SEND NOTIFY MESSAGES
		//


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

				#if DEBUG_MODULE

				debug_log ("PS_DSTRY: killing person, notifying task %s complete", entity_sub_type_task_names [get_local_entity_int_value (destroy_task, INT_TYPE_ENTITY_SUB_TYPE)]);

				#endif

				notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, destroy_task, en);
			}
		}

		//
		// Release landing lock (if any)
		//

		//release_mobile_entity_landing_locks (en);
	}


	//
	// UNLINK FROM SYSTEM
	//


	//
	// person
	//

	//
	// vehicle
	//

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_LAUNCHED_WEAPON);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);

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

	//unlink_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	// sector_link

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

	set_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_DEAD);

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

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		create_client_server_object_killed_explosion_effect (en);
	}
}





static void kill_remote (entity *en)
{
	#if DEBUG_MODULE >= 2

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





void overload_person_destroy_functions (void)
{
	fn_destroy_local_entity						[ENTITY_TYPE_PERSON]									= destroy_local;
	fn_destroy_client_server_entity			[ENTITY_TYPE_PERSON][COMMS_MODEL_SERVER]		= destroy_server;
	fn_destroy_client_server_entity			[ENTITY_TYPE_PERSON][COMMS_MODEL_CLIENT]		= destroy_client;

	fn_destroy_local_entity_family			[ENTITY_TYPE_PERSON]									= destroy_local_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_PERSON][COMMS_MODEL_SERVER]		= destroy_server_family;
	fn_destroy_client_server_entity_family	[ENTITY_TYPE_PERSON][COMMS_MODEL_CLIENT]		= destroy_client_family;

	fn_kill_local_entity							[ENTITY_TYPE_PERSON]									= kill_local;
	fn_kill_client_server_entity				[ENTITY_TYPE_PERSON][COMMS_MODEL_SERVER]		= kill_server;
	fn_kill_client_server_entity				[ENTITY_TYPE_PERSON][COMMS_MODEL_CLIENT]		= kill_client;
}






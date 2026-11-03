

































































#include "project.h"

#include "entity/tacview/tacview.h"





// 2 levels of debug

#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{

	helicopter
		*raw;


	//
	// PRE-AMBLE
	//








	raw = (helicopter *) get_local_entity_data (en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//


	destruct_3d_object (raw->ac.inst3d);

	free_mem (raw->ac.weapon_package_status_array);

	if (raw->ac.sound_effect_data)
	{

		destroy_system_sound_effect (raw->ac.sound_effect_data);
	}


	//
	// UNLINK FROM SYSTEM
	//


	//
	// helicopter
	//

	unlink_local_entity_children (en, LIST_TYPE_DESIGNATED_TARGET); // Jabberwock 031107 Designated targets

	unlink_local_entity_children (en, LIST_TYPE_GUNSHIP_TARGET);

	unlink_local_entity_children (en, LIST_TYPE_PADLOCK);

	//
	// aircraft
	//

	unlink_local_entity_children (en, LIST_TYPE_AIRCREW);

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);

	unlink_local_entity_children (en, LIST_TYPE_LAUNCHED_WEAPON);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_FOLLOWER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_DESIGNATED_TARGET); // Jabberwock 031107 Designated targets

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_GUNSHIP_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

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
	//
	//

	if (en == get_gunship_entity ())
	{
		set_raw_gunship_entity (NULL);
	}

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





static void destroy_server_family (entity *en)
{
	destroy_client_server_entity_children (en, LIST_TYPE_MOVEMENT_DEPENDENT);

	destroy_client_server_sound_effects (en);

	destroy_client_server_entity (en);
}





static void destroy_client_family (entity *en)
{






	transmit_entity_comms_message (ENTITY_COMMS_DESTROY_FAMILY, en);
}





static void kill_local (entity *en)
{

	int
		losses;

	helicopter
		*raw;

	entity
		*task,
		*group,
		*member,
		*destroy_task;


	//
	// PRE-AMBLE
	//








	if (!get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		return;
	}

	raw = (helicopter *) get_local_entity_data (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	ASSERT (group);

	//
	// update force info
	//

	remove_from_force_info (get_local_force_entity ((entity_sides) raw->ac.mob.side), en);




	if (en == get_gunship_entity ())
	{
		//
		// Award Points for mission
		//

		task = get_local_entity_primary_task (en);

		if (task && get_local_entity_int_value (task, INT_TYPE_TASK_STATE) == TASK_STATE_COMPLETED)
		{
			// Only award points for COMPLETE missions (also means player can't rejoin that mission and get points again)
			notify_gunship_entity_mission_terminated (en, task);
		}
		else if (!player_mission_logged() && get_game_type() != GAME_TYPE_FREE_FLIGHT)
		{
			int side = get_global_gunship_side();
			player_log_type* log = get_current_player_log();

			debug_log("Helicopter destroyed, logging pilot mission");

			inc_player_log_missions_flown(side, log);
			update_player_log_mission_success_rate (side, log, 0.0);
		}
	}


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//








	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		// notify regen of kill

		add_entity_to_regen_queue
		(
			(entity_sides) get_local_entity_int_value(en, INT_TYPE_SIDE),
			get_local_entity_type(en),
			get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE),
			get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)
		);

		//
		// notify task dependents
		//

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
		// Release landing lock (if any)
		//

		release_mobile_entity_landing_locks (en);

		//
		// if group contains player, set all helicopters in group to "weapons free", and clear "hold position" flag
		//

		if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

			while (member)
			{
				set_client_server_entity_int_value (member, INT_TYPE_WEAPONS_HOLD, FALSE);

				set_client_server_entity_int_value (member, INT_TYPE_POSITION_HOLD, FALSE);

				member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
			}
		}
	}


	//
	// UNLINK FROM SYSTEM
	//


	//
	// helicopter
	//

	unlink_local_entity_children (en, LIST_TYPE_DESIGNATED_TARGET);

	unlink_local_entity_children (en, LIST_TYPE_GUNSHIP_TARGET);

	unlink_local_entity_children (en, LIST_TYPE_PADLOCK);

	//
	// aircraft
	//

	// aircrew_root

	unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);


	//
	// the eject sequence relies on launched weapons NOT being unlinked
	//

	// unlink_local_entity_children (en, LIST_TYPE_LAUNCHED_WEAPON);

	//


	delete_local_entity_from_parents_child_list (en, LIST_TYPE_FOLLOWER);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);

	// gunship_target_link

	// member_link

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_MOVEMENT_DEPENDENT);

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

	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_SLAP);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_APU_TURBINE);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE1);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE2);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_UP1);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_UP2);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_DOWN1);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_DOWN2);

	//
	// kill radio / warning effects
	//

	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_INCOMING_MISSILE_WARNING);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_LOCK_ON_TONE);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_MCA);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_RADAR_LOCKED);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_RADAR_TRACKED);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_CPG_MESSAGE);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE);
	kill_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_WARNING_MESSAGE);

	notify_speech_buffers_entity_killed (en);

	unlink_local_entity_children (en, LIST_TYPE_TARGET);

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_PADLOCK);

	// sector_link

	delete_local_entity_from_parents_child_list (en, LIST_TYPE_TARGET);

	// update_link

	//
	// death timer (set time after which entity is totally destroyed)
	//

	set_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER, 0);


	//
	// KILL
	//


	damage_helicopter_3d_object (en);

	// must be done before alive flag set to FALSE
	remove_mobile_values_from_sector (get_local_entity_parent (en, LIST_TYPE_SECTOR), en);

	set_local_entity_int_value (en, INT_TYPE_ALIVE, FALSE);

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
	// Notify force to check campaign criteria
	//

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		entity
			*force;

		force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

		while (force)
		{
			notify_local_entity (ENTITY_MESSAGE_CHECK_CAMPAIGN_OBJECTIVES, force, en);

			force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
		}
	}


	//
	// SPECIAL EFFECTS
	//


	helicopter_impact_movement (en);

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		vec3d
			*position;

		sound_sample_indices
			sound_sample_index;

		set_client_server_entity_int_value (en, INT_TYPE_MAIN_ROTOR_DAMAGED, TRUE);

		set_client_server_entity_int_value (en, INT_TYPE_TAIL_ROTOR_DAMAGED, TRUE);



		position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

		sound_sample_index = SOUND_SAMPLE_INDEX_SHOT_DOWN_HELICOPTER;

		create_client_server_sound_effect_entity
		(
			en,
			ENTITY_SIDE_NEUTRAL,
			ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING,
			SOUND_CHANNEL_SOUND_EFFECT,
			SOUND_LOCALITY_ALL,
			NULL,												// position
			1.0,												// amplification
			1.0,												// Werewolf pitch
			TRUE,												// valid sound effect
			TRUE,												// looping
			1,													// sample count
			&sound_sample_index							// sample index list
		);
	}

	set_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_DYING);


	//
	// VIEWS
	//


	if ((en == get_gunship_entity ()) && (!get_local_entity_int_value (en, INT_TYPE_EJECTED)))
	{
		set_gunship_entity (NULL);
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






















































































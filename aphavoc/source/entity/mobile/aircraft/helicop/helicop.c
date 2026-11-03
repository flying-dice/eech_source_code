

































































#include "project.h"
#include "ai/taskgen/assign.h"
#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





entity
	*gunship_entity = NULL;

sector
	*gunship_current_sector = NULL;





//Werewolf: Changed to accomodate new rotor sound components: turbine sound, rotorslap sound
static void create_specified_helicopter_rotor_sound_effects (entity *en, sound_locality_types locality, sound_sample_indices looping_effect, sound_sample_indices wind_up_effect, sound_sample_indices wind_down_effect, sound_sample_indices turbine_effect, sound_sample_indices rotorslap_effect);
static float kill_sound_effect(entity* en, entity_sub_types type);

static int mission_logged = FALSE;





int player_mission_logged(void)
{
	return mission_logged;
}
























































void assign_entity_to_user (entity *en)
{
   entity
		*group,
		*member,
		*previous,
		*primary_task,
		*old_group;

	float
		fuel;

	int
		input_events_flag,
		ingame_events_flag;

	comms_data_flow_types
		store_data_flow;

	debug_log("Assigning gunship to user: %s", get_sub_type_name(en));

	ASSERT (get_pilot_entity () || (en == NULL && get_gunship_entity() == NULL));

	old_group = NULL;

	//
	// Maintain Input Events
	//

	if (get_event_stack_head_function () == set_input_events)
	{
		pop_event (set_input_events);

		input_events_flag = TRUE;
	}
	else
	{
		input_events_flag = FALSE;
	}

	//
	// Maintain Planner Events
	//

	if (get_event_stack_head_function () == ingame_screen_set_events)
	{
		ASSERT (get_in_flight_game_mode () == IN_FLIGHT_GAME_MODE_PLANNER);

		pop_event (ingame_screen_set_events);

		ingame_events_flag = TRUE;
	}
	else
	{
		ingame_events_flag = FALSE;
	}

	previous = gunship_entity;

	if (previous)
	{

		if (get_current_flight_dynamics ())
		{

			fuel = get_current_flight_dynamics_fuel_weight ();

			set_client_server_entity_float_value (previous, FLOAT_TYPE_FUEL_SUPPLY_LEVEL, fuel);
		}
	}

	if (en)
	{
		if (!get_local_entity_suitable_for_player (en, get_pilot_entity ()))
		{
			en = NULL;

			notify_clear_current_selections (NULL, NULL);
		}
	}

	if (en)
	{
		// set invulnerable timer and display message

		set_local_entity_float_value (en, FLOAT_TYPE_INVULNERABLE_TIMER, command_line_user_invulnerable_time);

		//set_status_message ("Invulnerable", STATUS_MESSAGE_TYPE_ALERT);

		if (get_time_acceleration () != TIME_ACCELERATION_PAUSE)
		{
			set_min_time_acceleration (NULL);
		}

		set_global_gunship_type ((gunship_types) get_local_entity_int_value (en, INT_TYPE_GUNSHIP_TYPE));
	}
	else
	{

		set_global_gunship_type (NUM_GUNSHIP_TYPES);

		if (get_camera_entity())
		{
			if (previous)
			{
				set_external_view_entity (previous);

				notify_local_entity (ENTITY_MESSAGE_SET_CAMERA_ACTION, get_camera_entity (), NULL, CAMERA_ACTION_END_OF_MISSION);
			}
			else
			{
				notify_local_entity (ENTITY_MESSAGE_SET_CAMERA_ACTION, get_camera_entity (), NULL, CAMERA_ACTION_CINEMATIC);
			}
		}
	}

	//
	// big problems as this function can be called by the client in RX mode,
	// but it needs to do a lot of setting things c/s (i.e. it really needs to be in TX mode)
	//

	store_data_flow = get_comms_data_flow ();

	set_comms_data_flow (COMMS_DATA_FLOW_TX);

	//
	// release old gunship entity
	//

	primary_task = NULL;

	if (gunship_entity)
	{
		int
			alive = get_local_entity_int_value(gunship_entity, INT_TYPE_ALIVE),
			side = get_global_gunship_side();

		player_log_type
			*log = get_current_player_log();

		//
		// Award Points for mission
		//





		primary_task = get_local_entity_primary_task (gunship_entity);

		if (primary_task
			&& get_local_entity_int_value (primary_task, INT_TYPE_TASK_STATE) == TASK_STATE_COMPLETED)
		{
			notify_gunship_entity_mission_terminated (gunship_entity, primary_task);
		}

		if (get_game_type() != GAME_TYPE_FREE_FLIGHT)
		{
			if (alive)
			{
				// if landed somewhere other than a keysite (base), then it's a lost helicopter
				if (!get_local_entity_int_value(gunship_entity, INT_TYPE_AIRBORNE_AIRCRAFT)
					&& !get_keysite_currently_landed_at())
				{
					debug_log("lost gunship - landed outside base");
					inc_player_log_helicopters_lost(side, log);
				}
			}
			else
			{
				debug_log("pilot killed");
				inc_player_log_deaths(side, log);
				inc_player_log_helicopters_lost(side, log);
			}
		}

		//
		// Deinitialise
		//

		deinitialise_cockpits ();

		deinitialise_avionics ();

		//
		// Deinitialise flight dynamics
		//

		if (get_current_flight_dynamics ())
		{
			//
			// damage helicopter according to flight model
			//

			damage_entity_to_flight_model (gunship_entity);

			deinitialise_flight_dynamics ();
		}

		set_client_server_entity_int_value (gunship_entity, INT_TYPE_PLAYER, ENTITY_PLAYER_AI);

		// check if this is necessary...
		switch (get_local_entity_int_value (gunship_entity, INT_TYPE_OPERATIONAL_STATE))
		{
			case OPERATIONAL_STATE_LANDED:
			{
				if (primary_task)
				{
					set_client_server_entity_int_value (gunship_entity, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_NAVIGATING);
				}

				break;
			}
		}

		old_group = get_local_entity_parent (gunship_entity, LIST_TYPE_MEMBER);

		gunship_current_sector = NULL;
	}

	//
	// set new gunship entity
	//

	gunship_entity = en;

	//
	// assign pilot
	//

	if (en)
	{
		set_client_server_entity_parent (get_pilot_entity (), LIST_TYPE_AIRCREW, en);
		set_local_entity_int_value (get_pilot_entity (), INT_TYPE_CREW_ROLE, CREW_ROLE_PILOT);
	}

	//
	// Tidy up old group (must be done after gunship_entity and aircrew link set)
	//

	if (old_group)
	{
		notify_campaign_screen (CAMPAIGN_SCREEN_GROUP_REMOVE_MEMBER, old_group);

		//
		// set all helicopters in old group to "weapons free", and clear "hold position" flag
		//

		member = get_local_entity_first_child (old_group, LIST_TYPE_MEMBER);

		while (member)
		{
			set_client_server_entity_int_value (member, INT_TYPE_WEAPONS_HOLD, FALSE);

			set_client_server_entity_int_value (member, INT_TYPE_POSITION_HOLD, FALSE);

			member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
		}
	}

	//
	//
	//

	if (en)
	{
		set_client_server_entity_int_value (en, INT_TYPE_PLAYER, ENTITY_PLAYER_REMOTE);

		debug_log ("Setting gunship_entity (type = %s, index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));

		set_valid_helicopter (TRUE);

		group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

		// backup pilot log

		backup_current_player_log();
		mission_logged = FALSE;

		//
		// Initialise avionics, cockpits and flight model
		//

		initialise_flight_dynamics (en); 

		update_helicopter_damage ();

		initialise_avionics ();

		initialise_cockpits ();

		fuel = get_local_entity_float_value (en, FLOAT_TYPE_FUEL_SUPPLY_LEVEL);

		set_current_flight_dynamics_fuel_weight (fuel);
		
		current_flight_dynamics->fuel_weight.modifier = ceil(fuel / get_local_entity_float_value (en, FLOAT_TYPE_FUEL_DEFAULT_WEIGHT) * 4.0) / 4.0;

		restore_helicopter_entity (en);

		current_flight_dynamics->main_rotor_rpm.value = en->helicopter_data->main_rotor_rpm;

		//
		// set view mode
		//

		set_external_view_entity (en);

		notify_local_entity (ENTITY_MESSAGE_SET_CAMERA_ACTION, get_camera_entity (), NULL, CAMERA_ACTION_CHASE);

		reinitialise_virtual_cockpit_view ();

		set_view_mode (VIEW_MODE_COCKPIT_PANEL_LEVEL_AHEAD);

		//
		// set all helicopters in new group to "weapons hold"
		//

		if (get_global_avionics_realism_level () > AVIONICS_REALISM_LEVEL_SIMPLE)
		{
			if (group)
			{
				entity
					*member;

				member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

				while (member)
				{
					set_client_server_entity_int_value (member, INT_TYPE_WEAPONS_HOLD, TRUE);

					member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
				}
			}
		}

		//
		// setup gunship
		//

		set_client_server_entity_int_value (en, INT_TYPE_UNIQUE_ID, direct_play_get_player_id ());

		// set gunship sector
		{
			vec3d* pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
			entity* sec = NULL;

			if (pos)
			{
				sec = get_local_sector_entity(pos);
				gunship_current_sector = (sector *) get_local_entity_data(sec);
			}
		}

		if (command_line_tacview_logging && command_line_tacview_logging < 3)
			start_tacview_logging(en);

		//
		// Notify campaign screen
		//

		activate_accept_selections_button ();

	}
	else
	{
		set_valid_helicopter (FALSE);

		debug_log ("Setting gunship_entity to NULL");

		if (previous)
		{
			player_quit_current_task ();
		}
	}

	//
	//
	//

	if (ingame_events_flag)
	{
		push_event (ingame_screen_set_events, "ingame screen events");
	}

	if (input_events_flag)
	{
		push_event (set_input_events, "input events");
	}

	set_comms_data_flow (store_data_flow);
}






















































































void restore_helicopter_entity (entity *en)
{
	ASSERT (en);

	restore_helicopter_main_rotors (en);
	set_client_server_entity_int_value (en, INT_TYPE_MAIN_ROTOR_DAMAGED, FALSE);

	restore_helicopter_tail_rotors (en);
	set_client_server_entity_int_value (en, INT_TYPE_TAIL_ROTOR_DAMAGED, FALSE);
}





void notify_gunship_entity_mission_completed (entity *en, entity *task)
{
	char
		s [128];

	ASSERT (en);

	ASSERT (en == get_gunship_entity ());

	ASSERT (task);

	ASSERT (get_local_entity_int_value (task, INT_TYPE_TASK_STATE) == TASK_STATE_COMPLETED);

	ASSERT (get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED) != TASK_INCOMPLETE);

	//
	// CAMPAIGN :- Set-up debriefing page
	//

	autoselect_debriefing_page (task, FALSE);

	//
	// Show Mission Complete Message
	//

	sprintf (s, "Mission %s", task_status_names [get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED)]);

	set_status_message (get_trans (s), STATUS_MESSAGE_TYPE_ALERT);

	//
	// Play Mission Complete CD track
	//

	play_music (MUSIC_TYPE_MISSION_COMPLETE, TRUE);
}





void notify_gunship_entity_mission_terminated (entity *en, entity *task)
{
	int
		side,
		rank,
		medal,
		points;

	player_log_type
		*log;

	ASSERT (en);

	ASSERT (en == get_gunship_entity ());

	side = get_local_entity_int_value (en, INT_TYPE_SIDE);

	ASSERT (task);

	ASSERT (get_local_entity_int_value (task, INT_TYPE_TASK_STATE) == TASK_STATE_COMPLETED);

	log = get_current_player_log ();

	medal = MEDAL_TYPE_NONE;

	rank = PILOT_RANK_NONE;


	points = get_local_entity_int_value (task, INT_TYPE_TASK_SCORE);

	// increment player experience points

	inc_player_log_experience (side, log, points);

	// increment number of missions flown

	inc_player_log_missions_flown (side, log);

	switch (get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED))
	{

		case TASK_COMPLETED_SUCCESS:
		{

			//
			// SUCCESSFUL MISSION
			//

			debug_filtered_log ("HELICOP: Awarded %d Points to player for %s %d", points, get_local_entity_string (task, STRING_TYPE_FULL_NAME), get_local_entity_index (task));

			if (points > 0)
			{

				//
				// AWARD VALOUR MEDALS
				//

				medal |= 1 << award_valour_medal (side, points);

				//
				// AWARD PROMOTION
				//

				rank = award_player_rank (side);
			}

			//
			// AWARD AIR MEDAL (no. of successful missions in a row)
			//

			medal |= 1 << award_air_medal_medal (side, TRUE);

			//
			// AWARD FLIGHT TIME MEDALS
			//

			medal |= 1 << award_aviator_wings (side);

			//
			// AWARD PURPLE HEART
			//

			medal |= 1 << award_purple_heart_medal (side);

			//
			// Update Success Rate
			//

			update_player_log_mission_success_rate (side, log, 1.0);

			break;
		}

		case TASK_COMPLETED_PARTIAL:
		{

			//
			// AWARD FLIGHT TIME MEDALS
			//

			medal |= 1 << award_aviator_wings (side);

			//
			// AWARD PURPLE HEART
			//

			medal |= 1 << award_purple_heart_medal (side);

			//
			// Update Success Rate
			//

			update_player_log_mission_success_rate (side, log, 0.5);

			break;
		}

		case TASK_COMPLETED_FAILURE:
		{

			//
			// AWARD FLIGHT TIME MEDALS
			//

			medal |= 1 << award_aviator_wings (side);

			// reset air_medal counter
			award_air_medal_medal (side, FALSE);

			//
			// Update Success Rate
			//

			update_player_log_mission_success_rate (side, log, 0.0);

			break;
		}

		default:
		{

			debug_fatal ("HELICOP: unknown task completed %d", get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED));
		}
	}

	//
	// store medals awarding in task
	//

	set_local_entity_int_value (task, INT_TYPE_AWARDED_MEDALS, medal);

	set_local_entity_int_value (task, INT_TYPE_AWARDED_PROMOTION, rank);

	//
	// CAMPAIGN :- Set-up debriefing page
	//

	autoselect_debriefing_page (task, TRUE);

	mission_logged = TRUE;
}






















































































































































































































































































































































































































































































































































































































































#define HELICOPTER_ROTOR_SOUNDS_LOOPING_MAX_RPM_VALUE	45
#define HELICOPTER_ROTOR_SOUNDS_LOOPING_MIN_RPM_VALUE	30



















































































































































































































































































































































int get_local_entity_suitable_for_player (entity *en, entity *pilot)
{
	entity
		*task;

	if (!en)
	{
		return FALSE;
	}

	if (get_local_entity_type (en) != ENTITY_TYPE_HELICOPTER)
	{
		// JB 030313 Fly any aircraft
		//return FALSE;
	}

	//
	// don't allow user to set gunship to wrong side helicopter
	// Jabberwock 040202 - unless we're in free flight

	if (pilot)
	{
		if ((get_local_entity_int_value (pilot, INT_TYPE_SIDE) != get_local_entity_int_value (en, INT_TYPE_SIDE)) && (get_game_type () != GAME_TYPE_FREE_FLIGHT))
		{
			return FALSE;
		}
	}

	//
	// check that the entity is the correct sub_type, i.e apache or havoc / comanche or hokum
	//

	if (!get_local_entity_int_value (en, INT_TYPE_PLAYER_CONTROLLABLE))
	{
		return FALSE;
	}

	switch (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE))
	{
		case ENTITY_SUB_TYPE_AIRCRAFT_AH64D_APACHE_LONGBOW:
		case ENTITY_SUB_TYPE_AIRCRAFT_MI28N_HAVOC_B:
		{
			if (!get_global_apache_havoc_installed())
			{
				// JB 030311 Disable EEAH check
				//return FALSE;
			}

			break;
		}
		case ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE:
		case ENTITY_SUB_TYPE_AIRCRAFT_KA52_HOKUM_B:
		////Moje 030606 Fly the Black-Hawk, 030612 Hind 030816 AH64A and KA50
		case ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK:
		case ENTITY_SUB_TYPE_AIRCRAFT_MI24D_HIND:
		case ENTITY_SUB_TYPE_AIRCRAFT_AH64A_APACHE:
		// new entities by GCsDriver  08-12-2007
		case ENTITY_SUB_TYPE_AIRCRAFT_KA29_HELIX_B:
		case ENTITY_SUB_TYPE_AIRCRAFT_KA50_HOKUM:
		case ENTITY_SUB_TYPE_AIRCRAFT_AH1T_SEACOBRA:
		case ENTITY_SUB_TYPE_AIRCRAFT_AH1W_SUPERCOBRA:
		case ENTITY_SUB_TYPE_AIRCRAFT_AH1Z_VIPER:
		case ENTITY_SUB_TYPE_AIRCRAFT_OH58D_KIOWA_WARRIOR:
		case ENTITY_SUB_TYPE_AIRCRAFT_MI17_HIP:
		case ENTITY_SUB_TYPE_AIRCRAFT_MI6_HOOK:
		case ENTITY_SUB_TYPE_AIRCRAFT_CH3_JOLLY_GREEN_GIANT:
		case ENTITY_SUB_TYPE_AIRCRAFT_CH53E_SUPER_STALLION:
		case ENTITY_SUB_TYPE_AIRCRAFT_CH46E_SEA_KNIGHT:
		case ENTITY_SUB_TYPE_AIRCRAFT_CH47D_CHINOOK:
		case ENTITY_SUB_TYPE_AIRCRAFT_MV22_OSPREY:
		{
			break;
		}
		default:
		{
			// JB 030311 Fly any aircraft
			//debug_fatal ("HELICOP: Player Controllable Flag Set For Invalid Aircraft (%s)", get_local_entity_string (en, STRING_TYPE_FULL_NAME));
		}
	}

	//
	// check that new gunship entity is not already being flown by another human
	//

	if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
	{
		return FALSE;
	}

	//
	// check that new gunship entity is alive
	//

	if (get_local_entity_int_value (en, INT_TYPE_ALIVE) == FALSE)
	{
		return FALSE;
	}

	//
	// check that new gunship entity hasn't ejected
	//

	if (get_local_entity_int_value (en, INT_TYPE_EJECTED))
	{
		return FALSE;
	}

	//
	// check that gunship isn't landing
	//

	// JB 030313 Enable control of aircraft while landing and taxiing.
	/*
	switch (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE))
	{

		case OPERATIONAL_STATE_LANDING:
		case OPERATIONAL_STATE_LANDING_HOLDING:
		case OPERATIONAL_STATE_TAXIING:
		{

			return FALSE;
		}
	}
	*/

	//
	// If the entity is doing a task, only allow player to join it if task incomplete
	//

	task = get_local_entity_primary_task (en);

	if (task)
	{
		if (get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED) != TASK_INCOMPLETE)
		{
			return TRUE;	////Moje 030606 Changed FALSE to TRUE, makes choppers flyable after mission complete
		}
	}

	return TRUE;
}



























































































































void transmit_player_recon_data (entity *en)
{
	entity
		*guide,
		*current_waypoint;

	vec3d
		*wp_pos,
		*en_pos;

	float
		recon_radius;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

	if (guide)
	{
		if (get_guide_criteria_valid (guide, GUIDE_CRITERIA_TRANSMIT_DATA))
		{
			current_waypoint = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);

			ASSERT (current_waypoint);

			en_pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

			wp_pos = get_local_entity_vec3d_ptr (current_waypoint, VEC3D_TYPE_POSITION);

			recon_radius = get_local_entity_waypoint_database_action_radius_value (current_waypoint, en);

			if (get_sqr_2d_range (wp_pos, en_pos) < (recon_radius * recon_radius))
			{
				debug_log ("HELICOP: Transmitting recon data");

				notify_local_entity (ENTITY_MESSAGE_WAYPOINT_RECON_REACHED, en, current_waypoint);
			}
		}
	}
}





int get_helicopter_allowed_to_rearm (entity *en)
{

	if (get_local_entity_int_value (en, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
	{

		if (helicopter_within_keysite_area (en))
		{

			return TRUE;
		}
	}
	else
	{

		if ((helicopter_within_keysite_area (en)) && (!get_local_entity_int_value (en, INT_TYPE_AIRBORNE_AIRCRAFT)))
		{

			return TRUE;
		}
	}

	return FALSE;
}





int helicopter_within_keysite_area (entity *en)
{

	vec3d
		*hc_position;

	float
		actual_range;

	entity
		*keysite;

	hc_position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	keysite = get_closest_keysite (NUM_ENTITY_SUB_TYPE_KEYSITES, (entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE), hc_position, 0.0, &actual_range, TRUE, NULL);

	if (keysite)
	{

		object_3d_instance
			*inst3d;

		vec3d
			position,
			*keysite_pos;

		struct OBJECT_3D_BOUNDS
			*bounding_box;

		float
			xmin,
			xmax,
			zmin,
			zmax;

//		debug_log ("DYNAMICS: landed close to keysite %s (range %f)", get_local_entity_string (keysite, STRING_TYPE_KEYSITE_NAME), actual_range);

		xmin = 0.0;
		xmax = 0.0;
		zmin = 0.0;
		zmax = 0.0;

		switch (get_local_entity_int_value (keysite, INT_TYPE_ENTITY_SUB_TYPE))
		{

			case ENTITY_SUB_TYPE_KEYSITE_ANCHORAGE:
			{

				float
					heading = 0.0;

				entity
					*ship;

				ship = get_local_entity_parent (keysite, LIST_TYPE_MOVEMENT_DEPENDENT);

				ASSERT (ship);
				ASSERT (get_local_entity_type (ship) == ENTITY_TYPE_SHIP_VEHICLE);

				heading = get_local_entity_float_value (ship, FLOAT_TYPE_HEADING);

				//
				// rotate model position into 'keysite space'
				//

				keysite_pos = get_local_entity_vec3d_ptr (ship, VEC3D_TYPE_POSITION);

				position.x = hc_position->x - keysite_pos->x;
				position.y = hc_position->y - keysite_pos->y;
				position.z = hc_position->z - keysite_pos->z;

				position.x = position.x * cos (-heading) + position.z * sin (-heading);
				position.z = position.z * cos (-heading) - position.x * sin (-heading);

				inst3d = (object_3d_instance *) get_local_entity_ptr_value (ship, PTR_TYPE_INSTANCE_3D_OBJECT);

				bounding_box = get_object_3d_bounding_box_without_lines (inst3d->object_number);

				// make sure that the keysite is at least 200 m radius
					// why? works fine /thealx/
				if (get_global_gunship_side () == ENTITY_SIDE_BLUE_FORCE)
				{
					xmin = min (bounding_box->xmin, -22.0f);
					xmax = max (bounding_box->xmax, 22.0f);
					zmin = min (bounding_box->zmin, -126.0f);
					zmax = max (bounding_box->zmax, 128.0f);
				}
				else
				{
					xmin = min (bounding_box->xmin, -32.0f);
					xmax = max (bounding_box->xmax, 20.0f);
					zmin = min (bounding_box->zmin, -137.0f);
					zmax = max (bounding_box->zmax, 47.0f);
				}

				break;
			}
			case ENTITY_SUB_TYPE_KEYSITE_FARP:
			{

				keysite_pos = get_local_entity_vec3d_ptr (keysite, VEC3D_TYPE_POSITION);

				position.x = hc_position->x - keysite_pos->x;
				position.y = hc_position->y - keysite_pos->y;
				position.z = hc_position->z - keysite_pos->z;

				bounding_box = get_object_3d_bounding_box_without_lines (get_local_entity_int_value (keysite, INT_TYPE_OBJECT_INDEX));

				// make sure that the keysite is at least 400 m radius
				xmin = min (bounding_box->xmin, -400.0f);
				xmax = max (bounding_box->xmax, 400.0f);
				zmin = min (bounding_box->zmin, -400.0f);
				zmax = max (bounding_box->zmax, 400.0f);

				break;
			}
			default:
			{

				keysite_pos = get_local_entity_vec3d_ptr (keysite, VEC3D_TYPE_POSITION);

				position.x = hc_position->x - keysite_pos->x;
				position.y = hc_position->y - keysite_pos->y;
				position.z = hc_position->z - keysite_pos->z;

				bounding_box = get_object_3d_bounding_box_without_lines (get_local_entity_int_value (keysite, INT_TYPE_OBJECT_INDEX));

				// make sure that the airbase is at least 400 m radius
				xmin = min (bounding_box->xmin, -400.0f);
				xmax = max (bounding_box->xmax, 400.0f);
				zmin = min (bounding_box->zmin, -400.0f);
				zmax = max (bounding_box->zmax, 400.0f);

				break;
			}
		}

		if (((position.x > xmin) && (position.x < xmax)) && ((position.z > zmin) && (position.z < zmax)))
		{

			return TRUE;
		}
	}

	return FALSE;
}





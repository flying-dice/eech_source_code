

































































#include "project.h"





#define DEBUG_MODULE 0

#define DEBUG_KILL_INFO TRUE













































































































































































































static int calculate_task_points_for_kill (entity *victim, entity *aggressor)
{
	int
		side,
		points;

	float
		p;

	entity
		*target,
		*group,
		*aggressor_group;

	points = get_local_entity_int_value (victim, INT_TYPE_POINTS_VALUE);

	if (points == 0)
	{
		return 0;
	}

	//
	// No points awarded for same-side kills....
	//

	side = get_local_entity_int_value (aggressor, INT_TYPE_SIDE);

	if (get_local_entity_int_value (victim, INT_TYPE_SIDE) == side)
	{
		return 0;
	}

	aggressor_group = get_local_entity_parent (aggressor, LIST_TYPE_MEMBER);

	//
	// Extra points for helping out allies....
	//

	target = get_local_entity_parent (victim, LIST_TYPE_TARGET);

	if (target)
	{
		p = (float)points;

		if ((get_local_entity_int_value (target, INT_TYPE_IDENTIFY_AIRCRAFT)) ||
				(get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE)))
		{
			group = get_local_entity_parent (target, LIST_TYPE_MEMBER);

			if (group)
			{
				if (group == aggressor_group)
				{
					//
					// 10% bonus for threat to your group....
					//
					
					p += (p * 0.10);
				}
				else if (get_local_entity_int_value (group, INT_TYPE_SIDE) == side)
				{
					//
					// 50% bonus for threat to another friendly group....
					//
					
					p += (p * 0.50);
				}
			}
		}

		//
		// Round Up
		//
		
		convert_float_to_int (ceil (p), &points);
	}

	return points;
}





void credit_client_server_mobile_kill (entity *victim, entity *aggressor)
{
	entity
		*force,
		*task,
		*group;

	int
		side,
		kills,
		points,
		total;

	ASSERT (victim);

	ASSERT (aggressor);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	// Victim should still be "alive" at this point
	ASSERT (get_local_entity_int_value (victim, INT_TYPE_ALIVE));

	//
	// Group kills
	//

	group = get_local_entity_parent (aggressor, LIST_TYPE_MEMBER);

	if (group)
	{
		kills = get_local_entity_int_value (group, INT_TYPE_KILLS);

		kills ++;

		set_client_server_entity_int_value (group, INT_TYPE_KILLS, kills);
	}

	//
	// Task kills
	//

	task = get_local_entity_primary_task (aggressor);

	if (task)
	{
		kills = get_local_entity_int_value (task, INT_TYPE_KILLS);

		kills ++;

		set_client_server_entity_int_value (task, INT_TYPE_KILLS, kills);

		points = calculate_task_points_for_kill (victim, aggressor);

		if (points > 0)
		{
			total = get_local_entity_int_value (task, INT_TYPE_TASK_SCORE);

			total += points;

			set_client_server_entity_int_value (task, INT_TYPE_TASK_SCORE, total);
		}

		add_kill_to_task (task, victim, aggressor);
	

	}

	//
	// Task losses
	//

	task = get_local_entity_primary_task (victim);

	if (task)
	{
		add_loss_to_task (task, victim, aggressor);
	

	}

	//
	// Player kills
	//

	if (get_game_type () != GAME_TYPE_FREE_FLIGHT)
	{
		if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_LOCAL)
		{
			inc_player_log_kills (get_global_gunship_side (), get_current_player_log (), victim);
		}
		else if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_REMOTE)
		{

		}
	}
	
	//
	// Send Text Message
	//



	//
	// Add to force statistics
	//

	side = get_local_entity_int_value (aggressor, INT_TYPE_SIDE);
	
	force = get_local_force_entity ((entity_sides) side);

	add_mobile_to_force_kills_stats (force, victim);



	#if DEBUG_KILL_INFO

	display_debug_kill_info (victim, aggressor);

	#endif
}





#define MIN_DEATH_TIMER_RANGE	1000 * 1000
#define MAX_DEATH_TIMER_RANGE	10000 * 10000

#define MIN_DEATH_TIMER_VALUE	5.0 * ONE_MINUTE
#define MAX_DEATH_TIMER_VALUE	3.0 * ONE_HOUR

#define DEATH_TIMER_SPECIAL_GAME_MULTIPLIER	0.1












































































int insert_mobile_into_takeoff_route (entity *member, entity *landing_entity)
{
	entity
		*task,
		*guide,
		*group,
		*route_wp;

	unsigned int
		member_number;

	ASSERT (member);

	ASSERT (landing_entity);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	group = get_local_entity_parent (member, LIST_TYPE_MEMBER);

	ASSERT (group);

	task = get_local_landing_entity_task (landing_entity, ENTITY_SUB_TYPE_TASK_TAKEOFF);

	ASSERT (task);

	//
	// Link into new route
	//

	route_wp = get_local_landing_entity_route (landing_entity, ENTITY_SUB_TYPE_TASK_TAKEOFF);

	ASSERT (route_wp);

	member_number = get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER);

	guide = create_client_server_guide_entity (task, route_wp, (1 << member_number));

	attach_group_to_guide_entity (group, guide);

	attach_group_member_to_guide_entity (member, guide);




	//
	// Unlock landed site, do after locking takeoff route in case entity can't takeoff.
	//

	notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_SITE, landing_entity, member);

	//
	// set operational state
	//

	switch (get_local_entity_type (member))
	{
		case ENTITY_TYPE_FIXED_WING:
		case ENTITY_TYPE_PERSON:
		{
			set_client_server_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_TAXIING);

			break;
		}
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SHIP_VEHICLE:
		{
			set_client_server_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_TAKEOFF);

			break;
		}
		case ENTITY_TYPE_HELICOPTER:
		{
			//
			// Operational state remains landed until rotors spun up, and all human players in the group have taken off
			//
			
			set_client_server_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_LANDED);

			if (get_local_entity_int_value (member, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
			{
				set_client_server_entity_float_value (member, FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER, MAIN_ROTOR_SPIN_UP_TIMER);
			}

			break;
		}

		default:
		{

			debug_fatal ("MOBILE: can't set operational state for %s (%d)",
							get_local_entity_string (member, STRING_TYPE_FULL_NAME),
							get_local_entity_index (member));

			break;
		}
	}

	//
	// Remove from MOVEMENT_DEPENDENT list (if on mobile_keysite)
	//

	if (get_local_entity_type (member) != ENTITY_TYPE_FIXED_WING)
	{

		delete_local_entity_from_parents_child_list (member, LIST_TYPE_MOVEMENT_DEPENDENT);
	}


	//
	// SPECIAL_EFFECT_HOOK FOR STARTING TO TAKEOFF
	//


	close_client_server_entity_loading_doors (member);

	close_client_server_entity_cargo_doors (member);


	//
	//


	return TRUE;
}





int mobile_check_all_members_taken_off (entity *en)
{
	entity
		*group,
		*force,
		*member,
		*mb_guide,
		*mb_task,
		*new_guide,
		*task,
		*current_guide,
		*landing_en;

	entity_sides
		side;

	unsigned int
		member_number,
		takeoff_flag;

	//
	// Called when "Taken Off" and "Loop" waypoints are reached
	//
	// Check to see if all other members have taken off - if so insert the whole group onto the primary task
	// Otherwise insert the member onto the holding pattern
	//

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	landing_en = get_local_group_member_landing_entity_from_task (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	takeoff_flag = TRUE;

	member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

	while (member)
	{
		if (member != en)
		{
			mb_guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);
	
			if (mb_guide)
			{
				mb_task = get_local_entity_parent (mb_guide, LIST_TYPE_GUIDE);
	
				if (mb_task)
				{
					switch (get_local_entity_int_value (mb_task, INT_TYPE_ENTITY_SUB_TYPE))
					{
						case ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING:
						{
							break;
						}

						default:
						{
							if (get_local_entity_int_value (member, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
							{
								takeoff_flag = FALSE;
							}
							else if (get_local_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_LANDED)
							{
								takeoff_flag = FALSE;
							}
			
							break;
						}
					}
				}
			}
		}

		member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
	}

	//
	//
	//

	if (takeoff_flag)
	{
		//
		// move all members from takeoff route to group wp route
		//

		//
		// find primary task
		//

		new_guide = get_local_group_primary_guide (group);

		ASSERT (new_guide);

		//
		// Allow group to engage
		//









		
		set_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY, get_local_entity_int_value (group, INT_TYPE_DEFAULT_ENGAGE_ENEMY));

		//
		// Remove independent groups from keysite
		//

		ASSERT (get_local_entity_int_value (group, INT_TYPE_GROUP_LIST_TYPE) == LIST_TYPE_KEYSITE_GROUP);

		if (get_local_entity_int_value (group, INT_TYPE_DEFAULT_GROUP_LIST_TYPE) == LIST_TYPE_INDEPENDENT_GROUP)
		{
			side = (entity_sides) get_local_entity_int_value (group, INT_TYPE_SIDE);

			force = get_local_force_entity (side);

			ASSERT (force);

			set_client_server_entity_int_value (group, INT_TYPE_GROUP_LIST_TYPE, LIST_TYPE_INDEPENDENT_GROUP);

			set_client_server_entity_parent (group, LIST_TYPE_KEYSITE_GROUP, NULL);

			set_client_server_entity_parent (group, LIST_TYPE_INDEPENDENT_GROUP, force);
		}

		member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

		while (member)
		{
			//
			// remove from takeoff task (assumes mobile is the only guide follower) 
			//

			current_guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);

			ASSERT (current_guide);

			if (current_guide == new_guide)
			{
				member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
				continue;
			}

			ASSERT (current_guide != new_guide);

			destroy_client_server_entity (current_guide);







			attach_group_member_to_guide_entity (member, new_guide);

			set_client_server_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_NAVIGATING);


			//
			// SPECIAL_EFFECT_HOOK FOR COMPLETED TAKEOFF AND ENTERING NAVIGATING MODE
			//



			//
			//


			member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
		}
	}
	else
	{
		//
		// place member into holding route
		//







		//
		// remove from takeoff task (assumes mobile is the only guide follower) 
		//

		current_guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

		ASSERT (current_guide);

		destroy_client_server_entity (current_guide);

		//
		// find holding task
		//

		task = get_local_landing_entity_task (landing_en, ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING);

		ASSERT (task);

		member_number = get_local_entity_int_value (en, INT_TYPE_GROUP_MEMBER_NUMBER);

		new_guide = create_client_server_guide_entity (task, NULL, (1 << member_number));

		attach_group_to_guide_entity (group, new_guide);

		attach_group_member_to_guide_entity (en, new_guide);
	}

	return takeoff_flag;
}
















































































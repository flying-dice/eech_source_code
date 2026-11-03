

































































#include "project.h"

#include "ai/taskgen/taskgen.h"
#include "ai/highlevl/highlevl.h"





// 2 levels of debug

#define DEBUG_MODULE 0

































































static int response_to_check_campaign_objectives (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*en,
		*group,
		*enemy_force;

	int
		side,
		enemy_side,
		sub_type;

	campaign_completed_types
		complete;

	complete = CAMPAIGN_COMPLETED_FALSE;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	if (get_local_entity_int_value (get_session_entity (), INT_TYPE_SESSION_COMPLETE))
	{
		return TRUE;
	}

	side = get_local_entity_int_value (receiver, INT_TYPE_SIDE);

	enemy_side = get_enemy_side (side);

	enemy_force = get_local_force_entity ((entity_sides) enemy_side);

	ASSERT (enemy_force);

	//
	// Check Keysite Objectives
	//

	complete = CAMPAIGN_COMPLETED_OBJECTIVES;

	en = get_local_entity_first_child (receiver, LIST_TYPE_CAMPAIGN_OBJECTIVE);

	while (en)
	{
		if (get_local_entity_type (en) == ENTITY_TYPE_KEYSITE)
		{
			sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			if (keysite_database [sub_type].troop_insertion_target)
			{
				if (get_local_entity_int_value (en, INT_TYPE_SIDE) != side)
				{
					complete = CAMPAIGN_COMPLETED_FALSE;

					break;
				}
			}
			else
			{
				if (get_local_entity_int_value (en, INT_TYPE_ALIVE))
				{
					complete = CAMPAIGN_COMPLETED_FALSE;

					break;
				}
			}
		}
		
		en = get_local_entity_child_succ (en, LIST_TYPE_CAMPAIGN_OBJECTIVE);
	}

	if (complete != CAMPAIGN_COMPLETED_FALSE)
	{
		ai_log ("(FORCE) Side %s Completed Keysite Objectives", entity_side_short_names [side]);
	}

	//
	// Check Opposing Forces for Minimum Keysite count
	//

	if (complete == CAMPAIGN_COMPLETED_FALSE)
	{
		complete = CAMPAIGN_COMPLETED_OBJECTIVES;

		en = get_local_entity_first_child (enemy_force, LIST_TYPE_KEYSITE_FORCE);

		while (en)
		{
			sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			if (keysite_database [sub_type].air_force_capacity != KEYSITE_AIR_FORCE_CAPACITY_NONE)
			{
				if ((get_local_entity_int_value (en, INT_TYPE_ALIVE)) && (get_local_entity_int_value (en, INT_TYPE_IN_USE)))
				{
					complete = CAMPAIGN_COMPLETED_FALSE;

					break;
				}
			}

			en = get_local_entity_child_succ (en, LIST_TYPE_KEYSITE_FORCE);
		}

		if (complete != CAMPAIGN_COMPLETED_FALSE)
		{
			ai_log ("(FORCE) Side %s Has No Keysites Left", entity_side_short_names [enemy_side]);
		}
	}

	//
	// Check Opposing Forces for Player gunships
	//

	if (complete == CAMPAIGN_COMPLETED_FALSE)
	{
		complete = CAMPAIGN_COMPLETED_VALID_GUNSHIPS;

		group = get_local_entity_first_child (enemy_force, LIST_TYPE_AIR_REGISTRY);

		while (group)
		{
			sub_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

			if (group_database [sub_type].default_entity_type == ENTITY_TYPE_HELICOPTER)
			{
				en = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

				while (en)
				{
					sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);
		
					if (aircraft_database [sub_type].player_controllable)
					{
						if (aircraft_database[sub_type].view_category == VIEW_CATEGORY_COMBAT_HELICOPTERS)
						{
								complete = CAMPAIGN_COMPLETED_FALSE;
								break;
						}
					}

					en = get_local_entity_child_succ (en, LIST_TYPE_MEMBER);
				}
			}

			if (complete == CAMPAIGN_COMPLETED_FALSE) break;

			group = get_local_entity_child_succ (group, LIST_TYPE_AIR_REGISTRY);
		}

		if (complete != CAMPAIGN_COMPLETED_FALSE)
		{
			ai_log ("(FORCE) Side %s Has No Player Controllable Gunships Left", entity_side_short_names [enemy_side]);
		}
	}

	//
	//
	//
	
	if (complete != CAMPAIGN_COMPLETED_FALSE)
	{
		//
		// THIS FORCE HAS MET ALL CAMPAIGN OBJECTIVES 
		//

		campaign_completed ((entity_sides) side, complete);
	}

	return (TRUE);
}





static int response_to_entity_fired_at (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*victim,
		*group;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (sender);

	victim = va_arg (pargs, entity *);

	ASSERT (victim);

	if (get_local_entity_int_value (sender, INT_TYPE_ALIVE))
	{
		//
		// Only respond to MOBILE Targets (Keysites are handled elsewhere)
		//
		
		if (get_local_entity_int_value (victim, INT_TYPE_IDENTIFY_MOBILE))
		{
			group = get_local_entity_parent (victim, LIST_TYPE_MEMBER);
	
			if (group)
			{
				notify_local_entity (ENTITY_MESSAGE_ENTITY_FIRED_AT, group, sender, victim);
			}
		}
	}

	return (TRUE);
}





static int response_to_force_armour_advancing (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{







	return (TRUE);
}





static int response_to_force_armour_resisting (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity
		*enemy_entity;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}

	enemy_entity = va_arg (pargs, entity *);









	//
	// RETALIATION
	//





























































	//
	//
	//

	return (TRUE);
}





static int response_to_force_armour_retreating (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*enemy_entity;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}

	enemy_entity = va_arg (pargs, entity *);







	return (TRUE);
}





static int response_to_force_armour_stuck (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{







	return (TRUE);
}





static int response_to_force_destroyed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	force
		*force_raw;

	entity
		*enemy_entity;

	//campaign_criteria
		//criteria;

	//campaign_criteria_type
		//*campaign_criteria;

	int
		sub_type;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return FALSE;
		//debug_fatal ("FC_MSGS: client not allowed here");
	}

	force_raw = (force *) get_local_entity_data (receiver);







	enemy_entity = va_arg (pargs, entity *);

	sub_type = get_local_entity_int_value (sender, INT_TYPE_OBJECT_3D_SHAPE);


/*
	if (get_local_entity_int_value (receiver, INT_TYPE_SIDE) != get_local_entity_int_value (sender, INT_TYPE_SIDE))
	{

		criteria = CAMPAIGN_CRITERIA_DESTROYED_ENEMY_OBJECTS;
	}
	else
	{

		criteria = CAMPAIGN_CRITERIA_DESTROYED_ALLIED_OBJECTS;
	}

	campaign_criteria = get_force_campaign_criteria (receiver, criteria, sub_type);

	if (!campaign_criteria)
	{

		campaign_criteria = get_force_campaign_criteria (receiver, criteria, OBJECT_3D_LAST);
	}

	if (campaign_criteria)
	{
	
		campaign_criteria->count ++;
	}
*/

	return (TRUE);
}





static int response_to_force_entered_sector (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*sector_en;

	int
		side;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}

	sector_en = va_arg (pargs, entity *);

	side = get_local_entity_int_value (receiver, INT_TYPE_SIDE);






	
	return (TRUE);
}





static int response_to_force_inefficient_keysite (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	//campaign_criteria
		//criteria;

	//campaign_criteria_type
		//*campaign_criteria;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}






	

/*
	if (get_local_entity_int_value (receiver, INT_TYPE_SIDE) == get_local_entity_int_value (sender, INT_TYPE_SIDE))
	{

		criteria = CAMPAIGN_CRITERIA_INEFFICIENT_ALLIED_KEYSITES;
	}
	else
	{

		criteria = CAMPAIGN_CRITERIA_INEFFICIENT_ENEMY_KEYSITES;
	}

	campaign_criteria = get_force_campaign_criteria (receiver, criteria, -1);

	if (campaign_criteria)
	{
	
		campaign_criteria->count ++;
	}
*/


	//
	// RETALIATION
	//

	//
	//
	//

	return (TRUE);
}





static int response_to_force_low_on_supplies (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity_sides
		side;

	movement_types
		movement_type;

	entity_sub_types
		sub_type;

	float
		factory_actual_range,
		airbase_actual_range;

	vec3d
		*pos;

	entity
		*task,
		*cargo,
		*airbase,
		*factory;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}

	sub_type = va_arg (pargs, entity_sub_types);














	// ensure no other task is already taking care of this supply requirement

	// replace en with sender!

	if ((entity_is_object_of_task (sender, ENTITY_SUB_TYPE_TASK_SUPPLY, (entity_sides) get_local_entity_int_value (sender, INT_TYPE_SIDE))))
	{
		task = get_local_entity_first_child (sender, LIST_TYPE_TASK_DEPENDENT);

		while (task)
		{

			if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_SUPPLY)
			{

				if (get_local_entity_float_value (task, FLOAT_TYPE_TASK_USER_DATA) == sub_type)
				{

					// already got a supply task for this type.

					return (FALSE);
				}
			}

			task = get_local_entity_child_succ (task, LIST_TYPE_TASK_DEPENDENT);
		}
	}

	//
	// Locate correct supplier
	//

	side = (entity_sides) get_local_entity_int_value (sender, INT_TYPE_SIDE);

	pos = get_keysite_supply_position (sender);

	switch (sub_type)
	{

		case ENTITY_SUB_TYPE_CARGO_AMMO:
		{

			factory = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_FACTORY, side, pos, 10 * KILOMETRE, &factory_actual_range, TRUE, NULL);

			if (!factory)
			{

				factory = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY, side, pos, 10 * KILOMETRE, &factory_actual_range, TRUE, NULL);
			}

			airbase = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, 10 * KILOMETRE, &airbase_actual_range, TRUE, NULL);

			if (airbase_actual_range < factory_actual_range)
			{

				factory = airbase;

				factory_actual_range = airbase_actual_range;
			}

			break;
		}

		case ENTITY_SUB_TYPE_CARGO_FUEL:
		{

			factory = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY, side, pos, 10 * KILOMETRE, &factory_actual_range, TRUE, NULL);

			if (!factory)
			{

				factory = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_FACTORY, side, pos, 10 * KILOMETRE, &factory_actual_range, TRUE, NULL);
			}

			airbase = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, pos, 10 * KILOMETRE, &airbase_actual_range, TRUE, NULL);

			if (airbase_actual_range < factory_actual_range)
			{

				factory = airbase;

				factory_actual_range = airbase_actual_range;
			}

			break;
		}
	}

	if (factory)
	{

		//
		// Get exact cargo to be picked up
		//

		cargo = get_local_entity_first_child (factory, LIST_TYPE_CARGO);

		while (cargo)
		{

			if (get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE) == sub_type)
			{

				break;
			}

			cargo = get_local_entity_child_succ (cargo, LIST_TYPE_CARGO);
		}

		if (cargo)
		{
	
			//
			// create task
			//
/*
			if (factory_actual_range > 75 * KILOMETRE)
			{






		
				return (FALSE);
			}
			else */

			movement_type = MOVEMENT_TYPE_AIR;

			task = create_supply_task (sender, factory, cargo, movement_type, task_database [ENTITY_SUB_TYPE_TASK_SUPPLY].task_priority, NULL, NULL);
	















		}
		else
		{
	








		}
	}
	else
	{








	}

	return (TRUE);
}





static int response_to_force_special_kill (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	force
		*force_raw;

	int
		kills;

	//campaign_criteria
		//criteria;

	//campaign_criteria_type
		//*campaign_criteria;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	force_raw = (force *) get_local_entity_data (receiver);

	kills = va_arg (pargs, int);






/*
	criteria = CAMPAIGN_CRITERIA_SPECIAL_KILLS;

	campaign_criteria = get_force_campaign_criteria (receiver, criteria, -1);

	if (campaign_criteria)
	{
		campaign_criteria->count = max (campaign_criteria->count, kills);
	}
*/
	return TRUE;
}





static int response_to_force_vacant_farp (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity_sides
		this_side;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}













	this_side = (entity_sides) get_local_entity_int_value (receiver, INT_TYPE_SIDE);

	if (!entity_is_object_of_task (sender, ENTITY_SUB_TYPE_TASK_RECON, this_side))
	{
//		create_farp_recon_task (this_side, sender);
	}

	return (TRUE);
}





static int response_to_force_waypoint_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity_sub_types
		sub_type;

	//campaign_criteria_type
		//*campaign_criteria;

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}

	sub_type = get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE);


/*
	campaign_criteria = get_force_campaign_criteria (receiver, CAMPAIGN_CRITERIA_REACHED_WAYPOINTS, sub_type);

	if (campaign_criteria)
	{
	
		campaign_criteria->count ++;
	}
*/

	






	return (TRUE);
}
	




#define REQUEST_ASSISTANCE_RANGE_MULTIPLIER		2000

static int response_to_request_assistance (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*task,
		*group,
		*victim,
		*victim_group,
		*member,
		*aggressor,
		*target_group;

	vec3d
		*pos,
		*group_pos;

	int
		flag,
		group_type,
		response;

	float
		range,
		max_range;

	aggressor = va_arg (pargs, entity *);

	ASSERT (aggressor);

	ASSERT (sender);

	victim = sender;

	if (get_local_entity_type (victim) == ENTITY_TYPE_KEYSITE)
	{
		victim_group = victim;


	}
	else
	{
		victim_group = get_local_entity_parent (victim, LIST_TYPE_MEMBER);


	}
	
	ASSERT (victim_group);

	pos = get_local_entity_vec3d_ptr (victim, VEC3D_TYPE_POSITION);

	ASSERT (pos);

	target_group = get_local_entity_parent (aggressor, LIST_TYPE_MEMBER);

	ASSERT (target_group);

	response = FALSE;







	//
	// Play Speech
	// 

	

	//
	// find nearby suitable group
	//

	group = get_local_entity_first_child (receiver, LIST_TYPE_AIR_REGISTRY);

	while (group)
	{
		if (group != victim_group)			// if sender is a group, we don't want itself responding to its own request !
		{
			if (group != target_group)			// In certain FF incidents the aggressor may try to attack itself !
			{
				if (get_local_entity_int_value (group, INT_TYPE_ALIVE))
				{
					if (get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
					{
						group_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);
		
						// group must be aircraft
						if (group_database [group_type].movement_type == MOVEMENT_TYPE_AIR)
						{
							// group must be able to intercept with reasonable speed
							if (group_database [group_type].ai_stats.movement_speed >= 3)
							{
								//
								// group must already be airborne (i.e. navigating)
								//
	
								task = get_local_group_primary_task (group);
	
								if (task)
								{
									if (get_local_entity_int_value (task, INT_TYPE_DEFAULT_ENGAGE_ENEMY))
									{
										//
										// Check group is not already engaging
										//
		
										if (!check_group_task_type (group, ENTITY_SUB_TYPE_TASK_ENGAGE))
										{
											flag = TRUE;
				
											member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);
				
											while (member)
											{
												if (get_local_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE) != OPERATIONAL_STATE_NAVIGATING)
												{
													flag = FALSE;
				
													break;
												}
				
												member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
											}
				
											if (flag)
											{
												//
												// calculate range
												//
				
												group_pos = get_local_entity_vec3d_ptr (group, VEC3D_TYPE_POSITION);
				
												range = get_sqr_2d_range (pos, group_pos);
				





				
												max_range = group_database [group_type].ai_stats.movement_speed * REQUEST_ASSISTANCE_RANGE_MULTIPLIER;
				
												max_range *= max_range;
				
												if (range < max_range)
												{
													//
													// assign engage tasks
													//
			
													if (engage_targets_in_group (group, target_group, TRUE))
													{
														ai_log ("(ASSIST) %s assisting %s under attack by %s",
																	get_local_entity_string (group, STRING_TYPE_FULL_NAME),
																	get_local_entity_string (victim, STRING_TYPE_FULL_NAME),
																	get_local_entity_string (aggressor, STRING_TYPE_FULL_NAME));
			
														member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);
				

			
														response = TRUE;

														break;
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}

		group = get_local_entity_child_succ (group, LIST_TYPE_AIR_REGISTRY);
	}

	return response;
}





static int response_to_task_created (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*objective;

	ASSERT (sender);

	ASSERT (sender->type == ENTITY_TYPE_TASK);

	if ((get_game_status () != GAME_STATUS_INITIALISED) || (get_comms_model () == COMMS_MODEL_CLIENT))
	{

		return (FALSE);
	}

	objective = va_arg (pargs, entity *);

	if (objective)
	{
	








	}
	else
	{
	







	}

	return (TRUE);
}
	




static int response_to_task_assigned (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	ASSERT (sender);

	ASSERT (sender->type == ENTITY_TYPE_TASK);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	create_task_assigned_reactionary_tasks (sender);

	return (TRUE);
}
	




static int response_to_task_completed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity_sub_types
		sub_type;

	force
		*force_raw;

	task_terminated_types
		task_terminated;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_TASK);

	task_terminated = (task_terminated_types) va_arg (pargs, int);

	sub_type = get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE);

	ASSERT (task_database [sub_type].primary_task);







	//
	// Create reactionary tasks
	//

	create_task_completed_reactionary_tasks (sender);

	//
	// Update force statistics
	//
	
	force_raw = (force *) get_local_entity_data (receiver);

	ASSERT (get_local_entity_int_value (sender, INT_TYPE_TASK_COMPLETED) != TASK_INCOMPLETE);

	if (get_local_entity_int_value (sender, INT_TYPE_TASK_COMPLETED) == TASK_COMPLETED_FAILURE)
	{
		force_raw->task_generation [sub_type].failed ++;
	}
	else
	{
		force_raw->task_generation [sub_type].completed ++;
	}

	return (TRUE);
}
	

























































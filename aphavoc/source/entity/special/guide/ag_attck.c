

































































#include "project.h"

#define DEBUG_MODULE 0









#define FIXED_WING_APPROACH_TIME		20.0

#define FIXED_WING_CLIMB_TIME			15.0

#define FIXED_WING_DIVE_TIME			10.0

#define FIXED_WING_EGRESS_TIME		10.0

#define INITIAL_BOB_UP_DISTANCE		10.0





static void set_attack_guide_approach_position (entity *en);

static void set_attack_guide_seek_cover_position (entity *en);

static void set_attack_guide_take_cover_position (entity *en);

static void set_attack_guide_climb_position (entity *en);

static void set_attack_guide_fire_position (entity *en);

static void set_attack_guide_disengage_position (entity *en);

static void set_attack_guide_egress_position (entity *en);

static void set_attack_guide_hasty_fire_position (entity *en);

static void set_attack_guide_hasty_take_cover_position (entity *en);

void set_attack_guide_dive_position (entity *en);





void initialise_air_to_ground_attack_guide (entity *en, entity *aggressor, entity *target)
{
	entity_sub_types
		best_weapon;

	ASSERT(target);
	ASSERT(aggressor);
	
	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) > 0.0)
	{
		return;
	}

	if (get_local_entity_type (aggressor) == ENTITY_TYPE_HELICOPTER)
	{
//		if (get_local_entity_first_child (aggressor, LIST_TYPE_TARGET))
		{
			//
			// if the aggressor is a helicopter, and is being targeted - attempt to fire back rapidly and then look for cover
			//

			best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_RANGE_CHECK);

			if (best_weapon != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);

				set_attack_guide_hasty_fire_position (en);

				return;
			}
		}
	}

	//
	// find best weapon for target (approach ranges will be based on this)
	//

	best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_CRITERIA_MINIMAL);

	set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);

	if (best_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		//
		// Entity is not capable of destroying the target
		//







		return;
	}

	//
	// calculate APPROACH position
	//

	set_attack_guide_approach_position (en);
}





void attack_guide_approach_reached (entity *en)
{
	entity
		*aggressor;

	//
	// change guide type to SEEK_COVER (Helicopter) or CLIMB (Jet) and calculate new position
	//

	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	ASSERT (aggressor);

	switch (get_local_entity_type (aggressor))
	{
		case ENTITY_TYPE_HELICOPTER:
		{
			set_attack_guide_seek_cover_position (en);

			break;
		}
		case ENTITY_TYPE_FIXED_WING:
		{
			set_attack_guide_climb_position (en);

			set_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 30.0); // cease fire until fire point not reached

			break;
		}
		default:
		{
			debug_fatal ("GD_ATTCK: Invalid Entity Type %d", get_local_entity_type (aggressor));
		}
	}
}





void attack_guide_seek_cover_reached (entity *en)
{
	//
	// search for cover
	//

	update_guide_seek_cover_results (en);
}





void attack_guide_fly_to_cover_reached (entity *en)
{

	//
	// change guide type to TAKE_COVER and calculate new position
	//

	set_attack_guide_take_cover_position (en);
}





void attack_guide_take_cover_reached (entity *en)
{
	entity
		*group,
		*leader,
		*task,
		*target,
		*aggressor;

	entity_sub_types
		best_weapon,
		selected_weapon;

	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);

	group = get_local_entity_parent (en, LIST_TYPE_GUIDE_STACK);

	ASSERT (group);

	//
	// Wait if mobile is still firing
	//

	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) > 0.0)
	{
		return;
	}

	//
	// check weapon
	//

	best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_RANGE_CHECK);

	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if ((selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON) || (best_weapon != selected_weapon))
	{
		if (best_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			//
			// No suitable weapon at current range
			//

			best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_CRITERIA_MINIMAL);

			if (best_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				//
				// Entity is not capable of destroying the target - abort the attack
				//

				delete_group_member_from_engage_guide (aggressor, en, TRUE);
			}
			else
			{
				//
				// Entity is capable of destroying the target, but must move to a different position
				//

				set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);

				set_attack_guide_approach_position (en);
			}

			return;
		}
		else
		{
			//
			// found new suitable weapon at current range
			//

			set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);
		}
	}

	//
	// wait if it still has guided AG missiles fired at the same target
	//

	if (check_guided_missile_type_alive(aggressor))
	{
		return;
	}


	//
	// notify the group leader that the cover position has been reached
	//

	leader = (entity *) get_local_entity_ptr_value (group, PTR_TYPE_GROUP_LEADER);

	ASSERT (leader);

	notify_local_entity (ENTITY_MESSAGE_GUIDE_COVER_POSITION_REACHED, leader, en);
}





void attack_guide_proceed_to_fire_positon (entity *en)
{
	//
	// change guide type to FIRE and calculate new position
	//

	ASSERT (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_TAKE_COVER);

	ASSERT (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER));
	
	set_attack_guide_fire_position (en);
}





void attack_guide_climb_reached (entity *en)
{
	//
	// change guide type to DIVE and calculate new position
	//

	set_local_entity_float_value ((entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER), FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 30.0); // cease fire until fire point not reached

	set_attack_guide_dive_position (en);
}





void attack_guide_dive_reached (entity *en)
{
	ASSERT(get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER));

	set_local_entity_float_value ((entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER), FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 30.0); // cease fire until fire point not reached

	set_attack_guide_fire_position (en); // change guide type to FIRE and calculate new position
}





void attack_guide_fire_reached (entity *en)
{
	entity
		*aggressor,
		*task,
		*target;
	
	//
	// Fire Weapon
	// Jet - Change guide type to DISENGAGE
	// Helicopter - Move back to TAKE_COVER
	//

	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT (aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);







	
	if (check_guided_missile_type_alive(aggressor))
	{







		if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
		{
			if (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING)
				set_attack_guide_egress_position (en);
			else
				set_attack_guide_seek_cover_position (en);
		}

		return;
	}

	if (get_2d_range (get_local_entity_vec3d_ptr (aggressor, VEC3D_TYPE_POSITION), get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION)) < (0.5 + 0.5 * frand1()) * (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING ? 500.0 : 100.0)) // too close, move back
	{







		if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
		{
			if (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING)
				set_attack_guide_egress_position (en);
			else
				set_attack_guide_seek_cover_position (en);
		}

		return;
	}

	switch (get_local_entity_type (aggressor))
	{

		//
		case ENTITY_TYPE_HELICOPTER:
		//

		{
			switch (aircraft_fire_weapon (aggressor, AIRCRAFT_FIRE_CHECK_ALL, FALSE))
			{
				case AIRCRAFT_FIRE_OK:
				{
					//
					// Aircraft fired weapon - move back to cover position
					//

					if (!get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER))
						set_local_entity_float_value(aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 5.0);

					set_attack_guide_take_cover_position (en);

					break;
				}

				case AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY:
				case AIRCRAFT_FIRE_NO_WEAPON:
				case AIRCRAFT_FIRE_NO_TARGET:
				case AIRCRAFT_FIRE_SUPPRESSED:
				{
					set_local_entity_float_value(aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 10.0);
					
					if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
						set_attack_guide_seek_cover_position (en);

					break;
				}

				case AIRCRAFT_FIRE_NO_LOS:
				{
					//
					// no LOS - increase altitude of guide
					//

					vec3d
						position;

					float
						altitude,
						terrain_elevation;

					get_local_entity_vec3d (en, VEC3D_TYPE_GUIDE_POSITION, &position);

					position.y += 10.0;

					//
					// check position isn't too high
					//

					ASSERT(point_inside_map_area(&position));
	
					terrain_elevation = get_3d_terrain_elevation (position.x, position.z);

					altitude = position.y - terrain_elevation;

					if (altitude >= 100.0)
					{
						//
						// something wrong - e.g. target has moved - find another cover position
						//

						if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
							set_attack_guide_seek_cover_position (en);
					}
					else
					{
						set_client_server_guide_entity_new_position (en, &position, NULL);
					}

					break;
				}
			}

			break;
		}


		//
		case ENTITY_TYPE_FIXED_WING:
		//

		{
			if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY) > 5.0)
				set_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 5.0);
			
			if (aircraft_fire_weapon (aggressor, AIRCRAFT_FIRE_CHECK_ALL, TRUE) == AIRCRAFT_FIRE_OK)
			{
				if (!get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER))
					set_local_entity_float_value(aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 5.0);
				set_attack_guide_disengage_position (en); // ready to fire, leave this loop
			}
			else if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
				set_attack_guide_fire_position(en); // update intercept point
			
			break;
		}
		default:
		{
			debug_fatal ("GD_ATTCK: Invalid Entity Type %d", get_local_entity_type (aggressor));
		}
	}
}





void attack_guide_disengage_reached (entity *en)
{
	//
	// Jet - Change guide type to EGRESS
	//

	set_attack_guide_egress_position (en);
}





void attack_guide_egress_reached (entity *en)
{
	entity_sub_types
		best_weapon;

	entity
		*task,
		*target,
		*aggressor;

	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);
	
	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) > 0.0)
	{
		return;
	}

	//
	// find best weapon for target (approach ranges will be based on this)
	//

	best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_CRITERIA_MINIMAL);

	set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);

	if (best_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		//
		// Entity is not capable of destroying the target - abort the attack
		//

		delete_group_member_from_engage_guide (aggressor, en, TRUE);

		return;
	}

	//
	// Change guide type back to APPROACH
	//

	set_attack_guide_approach_position (en);
}





void attack_guide_hasty_fire_reached (entity *en)
{
	entity
		*aggressor,
		*target;

	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	ASSERT (aggressor);

	ASSERT (get_local_entity_type (aggressor) == ENTITY_TYPE_HELICOPTER);

	target = get_local_entity_parent (aggressor, LIST_TYPE_TARGET);

	ASSERT (target);







	
	if (check_guided_missile_type_alive(aggressor))
	{







		if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
			set_attack_guide_seek_cover_position (en);

		return;
	}

	//
	// Take a pot-shot
	//

	if (get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON) != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		if (!get_local_entity_int_value (aggressor, INT_TYPE_WEAPONS_HOLD))
		{
			if (check_entity_line_of_sight (aggressor, target, MOBILE_LOS_CHECK_ALL))
			{
				//
				// (as long as it is in "weapons free" mode)
				//

				set_local_entity_float_value(aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 0.0);
				aircraft_fire_weapon (aggressor, AIRCRAFT_FIRE_CHECK_ALL, FALSE);

				if (!get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER))
					set_local_entity_float_value(aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 5.0);
			}
		}
	}

	//
	// and duck down low
	//

	if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
		set_attack_guide_hasty_take_cover_position (en);
}





void attack_guide_hasty_take_cover_reached (entity *en)
{
	entity
		*task,
		*target,
		*aggressor;

	entity_sub_types
		best_weapon;

	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);
	
	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) > 0.0)
	{
		return;
	}

	//
	// check weapon
	//

	if (get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON) == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_RANGE_CHECK);

		if (best_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			//
			// No suitable weapon at current range
			//

			best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_CRITERIA_MINIMAL);

			if (best_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				//
				// Entity is not capable of destroying the target - abort the attack
				//

				delete_group_member_from_engage_guide (aggressor, en, TRUE);

				return;
			}
		}

		set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);
	}

	//
	// change guide type to SEEK_COVER
	//

	if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER))
		set_attack_guide_seek_cover_position (en);
}





void set_attack_guide_approach_position (entity *en)
{




























	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_APPROACH);












	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}



























































































































}





void set_attack_guide_seek_cover_position (entity *en)
{













	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_SEEK_COVER);
































































}





void attack_guide_no_cover_found (entity *en)
{
	entity
		*task,
		*aggressor,
		*target;

	vec3d
		*aggressor_pos,
		*target_pos,
		position;

	vec2d
		direction;

	float
		distance;

	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);

	target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

	aggressor_pos = get_local_entity_vec3d_ptr (aggressor, VEC3D_TYPE_POSITION);

	//
	// No cover found - Check Line Of Sight
	//

	if (check_position_line_of_sight (aggressor, target, aggressor_pos, target_pos, MOBILE_LOS_CHECK_ALL))
	{
		//
		// Full Line Of Sight - forget cover just fire from here
		//

		set_local_entity_vec3d (aggressor, VEC3D_TYPE_COVER_POSITION, aggressor_pos);

		set_attack_guide_fly_to_cover_position (en);

		return;
	}

	//
	// Set guide to seek cover, but rotate around the target
	//

	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_SEEK_COVER);

	ASSERT (get_local_entity_type (aggressor) == ENTITY_TYPE_HELICOPTER);

	//
	// calculate vector of aggressor to target
	//

	direction.x = aggressor_pos->x - target_pos->x;
	direction.y = aggressor_pos->z - target_pos->z;

	distance = normalise_any_2d_vector (&direction);

	//
	// Reduce distance (spiral around target)
	//

	distance *= 0.8;

	//
	// calculate guide position
	//

	rotate_2d_vector (&direction, rad (25.0));

	position.x = target_pos->x + (direction.x * distance);
	position.z = target_pos->z + (direction.y * distance);

	bound_position_to_adjusted_map_volume (&position);

	position.y = get_3d_terrain_elevation (position.x, position.z) + get_local_entity_float_value (aggressor, FLOAT_TYPE_ATTACK_ALTITUDE);

	set_client_server_guide_entity_new_position (en, &position, NULL);

	//
	// Set criteria
	//

	set_client_server_guide_criteria_valid (en, GUIDE_CRITERIA_ALTITUDE, FALSE, 0.0);

	set_client_server_guide_criteria_valid (en, GUIDE_CRITERIA_HEADING, TRUE, rad (5.0));

	set_client_server_guide_criteria_valid (en, GUIDE_CRITERIA_WEAPON_VECTOR, FALSE, 0.0);
}





void set_attack_guide_fly_to_cover_position (entity *en)
{










	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FLY_TO_COVER);












































}





void set_attack_guide_take_cover_position (entity *en)
{










	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_TAKE_COVER);













































}





void set_attack_guide_climb_position (entity *en)
{






















	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_CLIMB);












	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}






























































}





void set_attack_guide_dive_position (entity *en)
{



















	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DIVE);








	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}


















	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DIVE);















}





void set_attack_guide_fire_position (entity *en)
{





















			set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE);





































			selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

			if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				delete_group_member_from_engage_guide (aggressor, en, FALSE);

				return;
			}





















			set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE);




























}





void set_attack_guide_disengage_position (entity *en)
{


























	switch (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE))
	{
		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE:
		{
			set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DISENGAGE);

			break;
		}

		case ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE_VIRTUAL:
		{
			set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DISENGAGE_VIRTUAL);

			virtual_ = TRUE;

			break;
		}

		default:
		{
			debug_fatal ("AG_ATTCK: Invalid Guide Type %d", get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE));

			break;
















	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}






























































}





void set_attack_guide_egress_position (entity *en)
{

























	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_EGRESS);





































































































































}





void set_attack_guide_hasty_fire_position (entity *en)
{










	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_HASTY_FIRE);






































}





void set_attack_guide_hasty_take_cover_position (entity *en)
{










	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_HASTY_TAKE_COVER);
















































}







































































#include "project.h"





#define DEBUG_MODULE 0


























































































aircraft_fire_result aircraft_fire_weapon (entity *en, unsigned int check_flags, int increase_accuracy)
{
	entity
		*target;

	aircraft
		*raw;

	vec3d
		*target_pos,
		en_pos,
		*weapon_vector,
		weapon_to_intercept_point_vector;
	float
		launch_angle_error,
		range;
	int
		debug_flag = FALSE;

	int loal_mode = FALSE;

	ASSERT (en);

	raw = (aircraft *) get_local_entity_data (en);

	get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &en_pos);









	if (raw->weapon_launch_delay)
	{
		if (debug_flag)
		{
			debug_log ("AC_WPN: weapon launch delay %.1f seconds", raw->weapon_launch_delay);
		}

		return AIRCRAFT_FIRE_SUPPRESSED;
	}
	
	// find target

	target = get_local_entity_parent (en, LIST_TYPE_TARGET);

	if (!target)
	{
		if (debug_flag)
		{
			debug_log ("AC_WPN: Fire Weapon Error - NO TARGET");
		}

		set_local_entity_float_value(en, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 0);

		return AIRCRAFT_FIRE_NO_TARGET;
	}

	target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);
	range = get_3d_range (&en_pos, target_pos);

	// Fire suppressed

	if (check_flags & AIRCRAFT_FIRE_SUPPRESSED)
	{
		if (get_local_entity_int_value (get_session_entity (), INT_TYPE_SUPPRESS_AI_FIRE))
		{
			return AIRCRAFT_FIRE_SUPPRESSED;
		}
	}

	// check weapon

	if (!aircraft_select_best_weapon(en, target))
	{
		if (debug_flag)
		{
			debug_log ("AC_WPN: Can't find suitable weapon, skip");
		}

		return AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY;
	}

	if (check_flags & AIRCRAFT_FIRE_NO_WEAPON)
	{
		if (get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON) == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			if (debug_flag)
			{
				debug_log ("AC_WPN: Fire Weapon Error - NO WEAPON");
			}
			
			set_local_entity_float_value(en, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 0);

			return AIRCRAFT_FIRE_NO_WEAPON;
		}
	}

	// weapon system_ready

	if (check_flags & AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY)
	{
		float launch_angle_modifier;
		
		if (!get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON_SYSTEM_READY))
		{
			if (debug_flag)
			{
				debug_log ("AC_WPN: Fire Weapon Error - WEAPON SYSTEM NOT READY");
			}
			
			return AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY;
		}

		if (!get_local_entity_int_value (en, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID))
		{
			if (debug_flag)
			{
				debug_log ("AC_WPN: Fire Weapon Error - outside launch angle (1)");
			}
			
			return AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY;
		}

		weapon_vector = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_WEAPON_VECTOR);

		ASSERT (weapon_vector);

		get_local_entity_vec3d (en, VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR, &weapon_to_intercept_point_vector);

		launch_angle_error = acos (get_3d_unit_vector_dot_product (weapon_vector, &weapon_to_intercept_point_vector));

		if (increase_accuracy && weapon_database[raw->selected_weapon].guidance_type) // make them not launch misiles into ground
			launch_angle_modifier = 0.25;
		else if (increase_accuracy)
			launch_angle_modifier = 0.5; // sharpshooter mode on!
		else
			launch_angle_modifier = 1.0;
		
		if (fabs (launch_angle_error) > launch_angle_modifier * weapon_database[raw->selected_weapon].max_launch_angle_error)
		{
			if (debug_flag)
			{
				debug_log ("AC_WPN: Fire Weapon Error - outside launch angle (%f > %f)", deg(fabs (launch_angle_error)), launch_angle_modifier * deg(weapon_database[raw->selected_weapon].max_launch_angle_error));
			}
			
			return AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY;
		}

	}

	// line of sight checks

	if (check_flags & AIRCRAFT_FIRE_NO_LOS)
	{
		int
			criteria;

		if (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			criteria = MOBILE_LOS_CHECK_ALL;
		}
		else
		{
			criteria = MOBILE_LOS_CHECK_COURSE_TERRAIN;
		}

		en_pos.y -= (get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE) + 2.0);

		if (!check_position_line_of_sight (en, target, &en_pos, target_pos, (mobile_los_check_criteria) criteria))
		{
			if (get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON) == ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE
				&& get_2d_range(&en_pos, target_pos) > 0.5 * weapon_database[ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE].max_range)
			{
				if (debug_flag)
				{
					debug_log("AC_WPN: Switching to LOAL mode to fire at target without LOS ((Aircraft %s (%d), Target %s (%d))",
										get_local_entity_string (en, STRING_TYPE_FULL_NAME), get_local_entity_index (en),
										get_local_entity_string (target, STRING_TYPE_FULL_NAME), get_local_entity_index (target));
				}
				
				loal_mode = TRUE;
			}
			else
			{
				if (debug_flag)
				{
					debug_log ("AC_WPN: Fire Weapon Error - NO LOS (Aircraft %s (%d), Target %s (%d))",
										get_local_entity_string (en, STRING_TYPE_FULL_NAME), get_local_entity_index (en),
										get_local_entity_string (target, STRING_TYPE_FULL_NAME), get_local_entity_index (target));
				}
				
				return AIRCRAFT_FIRE_NO_LOS;
			}
		}
	}

	// Play Speech

	play_aircraft_weapon_launched_speech (en, raw->selected_weapon);

	// Fire weapon

	set_local_entity_int_value(en, INT_TYPE_LOCK_ON_AFTER_LAUNCH, loal_mode);

	if (debug_flag)
	{
		debug_log ("AC_WPN: Launch Weapon!");
	}

	set_local_entity_float_value(en, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 0);
	
	launch_client_server_weapon (en, raw->selected_weapon, FALSE);
	
	return AIRCRAFT_FIRE_OK;
}





	// aircraft waiting to engage, or using cannons
























































































































































































































































































int aircraft_select_best_weapon (entity *en, entity *target)
{
	entity
		*guide,
		*task;

	int
		weapon_type;

	ASSERT(target);
	ASSERT(en);
	
	//
	// Find best weapon
	//

	weapon_type = get_best_weapon_for_target (en, target, BEST_WEAPON_CRITERIA_ALL);

	//
	// abort engage task and re-arm if no suitable weapon
	//

	if (weapon_type == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		weapon_type = get_best_weapon_for_target (en, target, BEST_WEAPON_CRITERIA_MINIMAL);

		if (weapon_type == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{










			guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

			if (guide)
			{
				task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

				ASSERT (task);

				if (!task_database [get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE)].persistent_task)
					notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, task, en, TASK_TERMINATED_ABORTED);
			}
		}

		if (weapon_type != get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON))
		{
			set_client_server_entity_int_value (en, INT_TYPE_SELECTED_WEAPON, weapon_type);
		}

		return FALSE;
	}
	else
	{
		if (weapon_type != get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON))
		{
			set_client_server_entity_int_value (en, INT_TYPE_SELECTED_WEAPON, weapon_type);
		}

		return TRUE;
	}
}





















































































































































































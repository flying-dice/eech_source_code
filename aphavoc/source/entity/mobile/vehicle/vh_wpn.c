

































































#include "project.h"
#include "../../../ai/taskgen/engage.h" // for CIWS





#define DEBUG_MODULE 0





static int vehicle_select_best_weapon (entity *en, entity *target);

static float get_fire_timer_difficulty_factor (entity *en);





void update_vehicle_weapon_fire (entity *en)
{
 	vehicle
      *raw;

	entity
		*target;

	int
		debug_flag = FALSE;

	float
		range,
		launch_angle_error,
		burst_timer_state;

	vec3d
		*pos,
		*target_pos,
		*weapon_vector,
		*weapon_to_intercept_point_vector;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (vehicle *) get_local_entity_data (en);

	burst_timer_state = raw->weapon_burst_timer;

	target = raw->mob.target_link.parent;











while(TRUE)
{
	// check vehicle not sleeping

	if (raw->sleep > 0.0)
	{
		if (debug_flag)
		{
			debug_log ("VH_WPN: Not Firing - Sleeping (%.1f seconds left", raw->sleep);
		}

		raw->weapon_burst_timer = 0.0;
		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;
		
		break;
	}

	// check valid target

	if (!target)
	{
		if (debug_flag)
		{
			debug_log ("VH_WPN: Not Firing - No Target");
		}

		raw->weapon_burst_timer = 0.0;
		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;
		
		break;
	}

	// Check Suppress AI fire

	if (get_local_entity_int_value (get_session_entity (), INT_TYPE_SUPPRESS_AI_FIRE))
	{
		if (debug_flag)
		{
			debug_log ("VH_WPN: Not Firing - AI fire suppressed");
		}

		raw->weapon_burst_timer = 0.0;
		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;

		break;
	}

	// check valid selected weapon

	if (!vehicle_select_best_weapon (en, target) && !raw->weapon_burst_timer)
	{
		if (debug_flag && (raw->target_fire_timer > 0.02))
		{
			debug_log ("VH_WPN: Not Firing - No weapon");
		}

		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;

		break;
	}

	// check selected weapon is ready

	if (!raw->selected_weapon_system_ready)
	{
		if (debug_flag)
		{
			debug_log ("VH_WPN: Not Firing - Weapon system not ready");
		}

		raw->weapon_burst_timer = 0.0;
		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;

		break;
	}

	// Check range

	if (!raw->weapon_burst_timer)
	{
		pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

		target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

		range = get_3d_range (pos, target_pos);

		if ((range < weapon_database [raw->selected_weapon].min_range) || (range > weapon_database [raw->selected_weapon].max_range))
		{
			if (debug_flag)
			{
				debug_log ("VH_WPN: Not Firing - out of range");
			}

			raw->weapon_salvo_timer = 0.0;
			raw->target_fire_timer = 0.01;

			break;
		}
	}

	// Check altitude for radar guided missiles

	if (get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT) && 
			(weapon_database [raw->selected_weapon].guidance_type == WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR || weapon_database [raw->selected_weapon].guidance_type == WEAPON_GUIDANCE_TYPE_ACTIVE_RADAR))
	{
		float
			min_alt,
			max_alt,
			altitude;

		int
			seed;

		altitude = get_local_entity_float_value (target, FLOAT_TYPE_RADAR_ALTITUDE);

		min_alt = get_local_entity_float_value (en, FLOAT_TYPE_AIR_SCAN_FLOOR);

		max_alt = get_local_entity_float_value (en, FLOAT_TYPE_AIR_SCAN_CEILING);

		seed = get_client_server_entity_random_number_seed (en);

		min_alt *= (1.0 + (0.5 * frand1x (&seed)));

		if ((altitude < min_alt) || (altitude > max_alt))
		{
			if (debug_flag)
			{
				debug_log ("VH_WPN: Not Firing - altitude");
			}

			raw->weapon_burst_timer = 0.0;
			raw->weapon_salvo_timer = 0.0;
			raw->target_fire_timer = 0.01;

			break;
		}
	}

	// check within max launch angle error

	if (!get_local_entity_int_value (en, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID))
	{
		if (debug_flag)
		{
			debug_log ("VH_WPN: Not Firing - outside launch angle (1)");
		}

		raw->weapon_burst_timer = 0.0;
		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;

		break;
	}

	weapon_vector = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_WEAPON_VECTOR);

	ASSERT (weapon_vector);

	weapon_to_intercept_point_vector = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR);

	ASSERT (weapon_to_intercept_point_vector);

	launch_angle_error = acos (get_3d_unit_vector_dot_product (weapon_vector, weapon_to_intercept_point_vector));

	if (fabs (launch_angle_error) > weapon_database[raw->selected_weapon].max_launch_angle_error)
	{
		if (debug_flag)
		{
			debug_log ("VH_WPN: Not Firing - outside launch angle (%f > %f)", deg(fabs (launch_angle_error)), deg(weapon_database[raw->selected_weapon].max_launch_angle_error));
		}

		raw->weapon_burst_timer = 0.0;
		raw->weapon_salvo_timer = 0.0;
		raw->target_fire_timer = 0.01;

		break;
	}

	// Check Line Of Sight (Non-projectile weapons only)

	if (!raw->weapon_burst_timer)
	{
		if (weapon_database[raw->selected_weapon].aiming_type != WEAPON_AIMING_TYPE_CALC_ANGLE_OF_PROJECTION)
		{
			if (!check_entity_line_of_sight (en, target, (mobile_los_check_criteria) MOBILE_LOS_CHECK_ALL))
			{
				if (debug_flag)
				{
					debug_log ("VH_WPN: Not Firing - No Line Of Sight");
				}

				raw->weapon_salvo_timer = 0.0;
				raw->target_fire_timer = 0.01;

				break;
			}
		}
	}
		
	// Update fire timer (unless continuing burst)

	if (raw->weapon_burst_timer <= 0.0)
	{
		if (raw->target_fire_timer > 0.0)
		{
			raw->target_fire_timer = max(0.0f, raw->target_fire_timer - get_delta_time ());
			
			if (raw->target_fire_timer > 0.0)
				break;
		}
		else
		{
			int
				seed;

			raw->target_fire_timer = weapon_database [raw->selected_weapon].reload_time;

			seed = get_client_server_entity_random_number_seed (en);

			raw->target_fire_timer *= (1.0 + (0.333 * sfrand1x (&seed)));

			if ((target) && (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI))
			{
				// then modify timer based on difficulty level

				raw->target_fire_timer *= get_fire_timer_difficulty_factor (target);
			}

			if (debug_flag)
			{
				debug_log("VH_WPN: Firing interval %f seconds", raw->target_fire_timer);
			}

			break;
		}
	}
	
	if (raw->weapon_burst_timer > 0.0)
	{
		set_local_entity_float_value (en, FLOAT_TYPE_WEAPON_BURST_TIMER, max(0.0f, raw->weapon_burst_timer - get_delta_time ()));

		if (raw->weapon_burst_timer > 0.0)
		{
			if (debug_flag >= 2)
			{
				debug_log ("VH_WPN: Continuing Burst");
			}
		}
		else
		{
			if (debug_flag)
			{
				debug_log ("VH_WPN: Stop Burst this frame");
			}

			raw->weapon_salvo_timer = 0.0;
			
			break;
		}
	}

	// fire weapon

	if (debug_flag > 1)
	{
		debug_log ("VH_WPN: Firing weapon");
	}

	launch_client_server_weapon (en, raw->selected_weapon, FALSE);

	break;
}

	if (burst_timer_state > 0.0 && raw->weapon_burst_timer <= 0.0)
	{
		pause_client_server_continuous_weapon_sound_effect (en, raw->selected_weapon);
	}


	//
	// SPECIAL_EFFECT_HOOK FOR LAUNCHING WEAPON
	//


	//
	// If current weapon is WEAPON_NO_WEAPON it means the last one was just fired - set vehicle to re-arming
	//

/*	if (raw->selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		pause_client_server_continuous_weapon_sound_effect (en, raw->selected_weapon);
	} */
}





int vehicle_select_best_weapon (entity *en, entity *target)
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





















































































































void update_vehicle_missile_defence (entity *en)
{
	entity
		*group;
	vehicle
		*raw;
	entity
		*persuer,
		*rocket = NULL;

	ASSERT (en);
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (vehicle *) get_local_entity_data (en);
	
	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	ASSERT (group);

	persuer = get_local_entity_first_child (en, LIST_TYPE_TARGET);

	if (!persuer)
		return;
	
	while (persuer) // stick to the first launched rocket
	{
		if (get_local_entity_type (persuer) == ENTITY_TYPE_WEAPON)
			rocket = persuer;
		persuer = get_local_entity_child_succ (persuer, LIST_TYPE_TARGET);
	}

	if (rocket)
		if (rocket != raw->mob.target_link.parent) // target already engaged
		{
			entity_sub_types type = get_local_entity_int_value (rocket, INT_TYPE_ENTITY_SUB_TYPE);

			if (weapon_database[type].boost_time > 0) // missiles and rockets only
				if (vehicle_select_best_weapon (en, rocket))
				{
					engage_specific_target (group, rocket, 1, TRUE);

					return;
				}
		}
}




static struct FIRE_TIMER_DIFFICULTY_TABLE
{
	game_difficulty_settings
		difficulty;

	float
		timer_scale;

} fire_timer_difficulty_table [] =
	{
		GAME_DIFFICULTY_HARD,		0.5,
		GAME_DIFFICULTY_MEDIUM,		0.66,
		GAME_DIFFICULTY_EASY,		1.0,
		(game_difficulty_settings) -1
	};












































void rearm_vehicle_weapons(entity *en)
{
		entity
			*group;
		float
			ammo;
		weapon_config_types
			config_type;
		weapon_package_status
			*package_status;
		int
			package;

		group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

		ASSERT (group);

		package_status = (weapon_package_status *) get_local_entity_ptr_value (en, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);
		config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

		ASSERT (weapon_config_type_valid (config_type));

		if (package_status)
			for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
			{
				if (!package_status[package].number && weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
				{
					if (!package_status[package].rearming_timer)
					{
						package_status[package].rearming_timer = get_local_entity_rearming_sleep_time (group, weapon_config_database[config_type][package].sub_type);





					}
					else
						package_status[package].rearming_timer = max(0.0f, package_status[package].rearming_timer - get_delta_time());
					
					if (!package_status[package].rearming_timer)
					{
						ammo = get_local_entity_float_value (group, FLOAT_TYPE_AMMO_SUPPLY_LEVEL);
						ammo -= get_local_entity_float_value (en, FLOAT_TYPE_AMMO_ECONOMY) * AMMO_USAGE_ACCELERATOR;
						ammo = bound (ammo, 0.0, 100.0);
						set_local_entity_float_value (group, FLOAT_TYPE_AMMO_SUPPLY_LEVEL, ammo);
						
						package_status[package].number = weapon_config_database[config_type][package].number;




					}
				}
			}
	}

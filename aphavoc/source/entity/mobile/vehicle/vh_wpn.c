

































































#include "project.h"
#include "../../../ai/taskgen/engage.h" // for CIWS





#define DEBUG_MODULE 0





static int vehicle_select_best_weapon (entity *en, entity *target);

static float get_fire_timer_difficulty_factor (entity *en);






























































































































































































































































































































































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

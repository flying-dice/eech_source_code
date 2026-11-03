





































































#include "project.h"

#define DEBUG_MODULE 0





void initialise_attack_guide (entity *en)
{
	entity
		*task,
		*aggressor,
		*target;

	ASSERT (en);

	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);

	ASSERT (task);

	ASSERT (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_ENGAGE);

	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	ASSERT (aggressor);

	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) > 0.0)
	{
		return;
	}

	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);

	ASSERT (target);

	if (!get_local_entity_int_value (aggressor, INT_TYPE_IDENTIFY_AIRCRAFT))
	{
		//
		// Surface-Air & Surface-Surface
		//

		entity_sub_types
			best_weapon;

		//
		// find best weapon for target
		//

		best_weapon = get_best_weapon_for_target (aggressor, target, BEST_WEAPON_CRITERIA_ALL);

		set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);

		return;
	}
	else
	{
		//
		// Air-Air & Air-Surface
		//

		if (get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT))
		{
			initialise_air_to_air_attack_guide (en, aggressor, target);
		}
		else
		{
			initialise_air_to_ground_attack_guide (en, aggressor, target);
		}
	}
}










































































































int attack_guide_find_best_weapon (entity *en)
{
	entity
		*target,
		*aggressor;

	entity_sub_types
		best_weapon;

	//
	// check weapon
	//

	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	ASSERT (aggressor);

	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) > 0.0)
	{
		return FALSE;
	}

	target = get_local_entity_parent (aggressor, LIST_TYPE_TARGET);

	ASSERT (target);

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

			return FALSE;
		}
	}

	set_client_server_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON, best_weapon);

	return TRUE;
}






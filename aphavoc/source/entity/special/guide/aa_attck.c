

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_attack_guide_move_six_position (entity *en);

static void set_attack_guide_move_circle_position (entity *en);

static void set_attack_guide_fire_intercept_position (entity *en);





void initialise_air_to_air_attack_guide (entity *en, entity *aggressor, entity *target)
{
	//
	// calculate FIRE position
	//
	
	if (get_local_entity_int_value (aggressor, INT_TYPE_WEAPONS_HOLD))
	{
		debug_log ("AA_ATTCK: Aborting attack - weapons hold (initialise)");

		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}
	
	if (attack_guide_find_best_weapon (en))
	{
		ASSERT(get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER));
		set_attack_guide_fire_intercept_position (en);
	}
}





void attack_guide_move_six_reached (entity *en)
{
	entity
		*task,
		*aggressor,
		*target;

	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);








	if (get_local_entity_int_value (aggressor, INT_TYPE_WEAPONS_HOLD))
	{
		debug_log ("AA_ATTCK: Aborting attack - weapons hold (move six)");

		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}
	
	//
	// check weapon
	//

	if (attack_guide_find_best_weapon (en))
	{
		ASSERT(get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER));
		set_attack_guide_fire_intercept_position (en);
	}
}





void attack_guide_move_circle_reached (entity *en)
{
	entity
		*task,
		*aggressor,
		*target;

	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);








	if (get_local_entity_int_value (aggressor, INT_TYPE_WEAPONS_HOLD))
	{
		debug_log ("AA_ATTCK: Aborting attack - weapons hold (move circle)");

		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}
	
	//
	// check weapon
	//

	if (attack_guide_find_best_weapon (en))
	{
		ASSERT(get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER));
		set_attack_guide_fire_intercept_position (en);
	}
}





void attack_guide_fire_intercept_reached (entity *en)
{
	entity
		*task,
		*aggressor,
		*target;

	vec3d
		*target_pos,
		*aggressor_pos;
		
	ASSERT(en);
	
	aggressor = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);
	ASSERT(aggressor);
	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);
	ASSERT(task);
	target = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);
	ASSERT(target);

	if (get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) && get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) < 0.2 && 
			attack_guide_find_best_weapon (en))
	{
		if (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING)
			set_attack_guide_move_six_position (en);
		else
			set_attack_guide_move_circle_position (en);

		return;
	}	
	
	if (get_local_entity_int_value (aggressor, INT_TYPE_WEAPONS_HOLD))
	{







		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}








	if (check_guided_missile_type_alive(aggressor))
	{








		if (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING)
			set_attack_guide_move_six_position (en);
		else
			set_attack_guide_move_circle_position (en);
		return;
	}

	aggressor_pos = get_local_entity_vec3d_ptr (aggressor, VEC3D_TYPE_POSITION);
	target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

	if (get_2d_range (aggressor_pos, target_pos) < (0.5 + 0.5 * frand1()) * (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING ? 300 : 200)) // too close, move back
	{








		if (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING)
			set_attack_guide_move_six_position (en);
		else
			set_attack_guide_move_circle_position (en);
		
		return;
	}

	// Fire Weapon

	if (!get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER))
	{
		if (aircraft_fire_weapon (aggressor, AIRCRAFT_FIRE_CHECK_ALL, TRUE) == AIRCRAFT_FIRE_OK)
		{
			if (!get_local_entity_float_value (aggressor, FLOAT_TYPE_WEAPON_BURST_TIMER) && attack_guide_find_best_weapon (en))
			{
				set_local_entity_float_value(aggressor, FLOAT_TYPE_WEAPON_LAUNCH_DELAY, 5.0);

				if (get_local_entity_type (aggressor) == ENTITY_TYPE_FIXED_WING)
					set_attack_guide_move_six_position (en);
				else
					set_attack_guide_move_circle_position (en);
			}
		}
		else if (get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER)) // update intercept point if it's possible
			set_attack_guide_fire_intercept_position(en);
	}
}





void set_attack_guide_move_six_position (entity *en)
{





















	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_MOVE_SIX);
















	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}




































}





void set_attack_guide_move_circle_position (entity *en)
{



















	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_MOVE_CIRCLE);


































































}





void set_attack_guide_fire_intercept_position (entity *en)
{



















	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_FIRE_INTERCEPT);












	selected_weapon = get_local_entity_int_value (aggressor, INT_TYPE_SELECTED_WEAPON);

	if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		delete_group_member_from_engage_guide (aggressor, en, FALSE);

		return;
	}
































}






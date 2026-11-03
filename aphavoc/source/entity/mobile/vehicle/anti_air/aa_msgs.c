

































































#include "project.h"





//
// 2 levels of debug
//

#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity
		*force,
		*aggressor,
		*enemy_force;

	int
		old_damage_state,
		total_damage_level;

	float
		damage_modifier;







	damage_modifier = va_arg (pargs, double);

	if (sender)
	{
		if (get_local_entity_type (sender) == ENTITY_TYPE_WEAPON)
		{
			aggressor = get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON);
		}
		else
		{
			aggressor = sender;
		}
	}
	else
	{
		aggressor = NULL;
	}

	force = get_local_force_entity ((entity_sides) get_local_entity_int_value (receiver, INT_TYPE_SIDE));

	enemy_force = get_local_force_entity (get_enemy_side (get_local_entity_int_value (receiver, INT_TYPE_SIDE)));

	//
	// assess damage
	//

	old_damage_state = vehicle_critically_damaged(receiver);
			
	damage_client_server_entity (receiver, sender, damage_modifier);

	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

	if (old_damage_state < vehicle_critically_damaged(receiver)) // vehicle critically damaged
		credit_client_server_mobile_kill (receiver, aggressor);
	
	if (total_damage_level <= 0)
	{
		if (aggressor)
		{
			play_vehicle_destroyed_speech (receiver, aggressor);

//			credit_client_server_mobile_kill (receiver, aggressor);
		}

		notify_local_entity (ENTITY_MESSAGE_FORCE_DESTROYED, force, receiver, aggressor);

		notify_local_entity (ENTITY_MESSAGE_FORCE_DESTROYED, enemy_force, receiver, aggressor);

		kill_client_server_entity (receiver);
	}
	else
	{

		//
		// SPECIAL_EFFECT_HOOK FOR GETTING_SHOT
		//


		if (aggressor)
		{
			play_vehicle_shot_at_speech (receiver, aggressor);
		}


		//
		//


		//notify hit. ?
	}

	return (TRUE);
}







































































































static int response_to_task_terminated (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	return (TRUE);
}
































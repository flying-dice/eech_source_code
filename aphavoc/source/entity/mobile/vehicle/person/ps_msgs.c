

































































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

	#if DEBUG_MODULE >= 2

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

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

	if (old_damage_state < vehicle_critically_damaged(receiver)) // vehicle critically damaged
		credit_client_server_mobile_kill (receiver, aggressor);
	
	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

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





#if DEBUG_MODULE >= 2

static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE >= 2

static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE >= 2

// WARNING: THIS IS OVERLOADED AT VEHICLE LEVEL //

static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE >= 2

// WARNING: THIS IS OVERLOADED AT VEHICLE LEVEL //

static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





static int response_to_set_entity_position (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	vec3d
		*position;

	float
		velocity,
		heading;

	person
		*raw;

	raw = (person *) get_local_entity_data (receiver);

	position = va_arg (pargs, vec3d *);

	heading = (float) va_arg (pargs, double);

	velocity = (float) va_arg (pargs, double);

	//
	// should this notify children of movement (for mobile landing entities) ?
	//

	set_local_entity_vec3d (receiver, VEC3D_TYPE_POSITION, position);

	return (TRUE);
}





static int response_to_task_terminated (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	return (TRUE);
}





void overload_person_message_responses (void)
{
	overload_vehicle_message_responses (ENTITY_TYPE_PERSON);

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_COLLISION] 								= response_to_collision;

	#if DEBUG_MODULE >= 2

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_LINK_CHILD]								= response_to_link_child;

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_UNLINK_CHILD]							= response_to_unlink_child;

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_LINK_PARENT]								= response_to_link_parent;

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_UNLINK_PARENT]							= response_to_unlink_parent;

	#endif

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_SET_ENTITY_POSITION]					= response_to_set_entity_position;

	message_responses[ENTITY_TYPE_PERSON][ENTITY_MESSAGE_TASK_TERMINATED]						= response_to_task_terminated;
}





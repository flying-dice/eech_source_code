

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity
		*force,
		*enemy_force,
		*aggressor;

	int
		damage_caused,
		old_damage_state,
		total_damage_level;

	float
		damage_modifier;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	damage_modifier = va_arg (pargs, double);

	old_damage_state = vehicle_critically_damaged(receiver);
			
	damage_caused = damage_client_server_entity (receiver, sender, damage_modifier);

	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

	if (sender)
	{
		if (get_local_entity_type (sender) == ENTITY_TYPE_WEAPON)
		{
			aggressor = get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON);

			//
			// a direct hit from a H.E. weapon causes smoke from impact point
			//

			if ((damage_modifier == 1.0) && (damage_caused > 100))
			{
				int
					random_seed;

				weapon_warhead_types
					warhead_type;

				random_seed = get_client_server_entity_random_number_seed (sender);

				if (frand1x (&random_seed) > 0.8)
				{
	
					warhead_type = (weapon_warhead_types) get_local_entity_int_value (sender, INT_TYPE_WEAPON_WARHEAD_TYPE);
	
					if ((warhead_type >= WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK) &&
							(warhead_type <= WEAPON_WARHEAD_TYPE_CONVENTIONAL_MUNITIONS))
					{
						vec3d
							rel_pos,
							rel_vec,
							*en_pos,
							*weapon_pos;
	
						matrix3x3
							m,
							inv;
	
						en_pos = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION);
						weapon_pos = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);

						rel_vec.x = weapon_pos->x - en_pos->x;
						rel_vec.y = weapon_pos->y - en_pos->y;
						rel_vec.z = weapon_pos->z - en_pos->z;
	
						get_local_entity_attitude_matrix (receiver, m);

						get_inverse_matrix (inv, m);

						multiply_matrix3x3_vec3d (&rel_pos, inv, &rel_vec);

						create_client_server_meta_smoke_list_specified_offset (META_SMOKE_LIST_TYPE_FIRE_SHORT_DURATION, receiver, &rel_pos);
					}
				}
			}
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





static int response_to_articulate_loading_doors (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	object_3d_instance
		*inst3d;

	sound_sample_indices
		sound_sample_index;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	inst3d = (object_3d_instance *) get_local_entity_ptr_value (receiver, PTR_TYPE_INSTANCE_3D_OBJECT);

	if (inst3d)
	{
		if (object_contains_sub_object_type (inst3d, OBJECT_3D_SUB_OBJECT_LOADING_DOOR, 0, NULL))
		{
			sound_sample_index = SOUND_SAMPLE_INDEX_SHIP_DOORS;

			create_client_server_sound_effect_entity
			(
				receiver,
				ENTITY_SIDE_NEUTRAL,
				ENTITY_SUB_TYPE_EFFECT_SOUND_MISC,
				SOUND_CHANNEL_SOUND_EFFECT,
				SOUND_LOCALITY_ALL,
				NULL,												// position
				1.0,												// amplification
				1.0,												// Werewolf pitch
				TRUE,												// valid sound effect
				FALSE,											// looping
				1,													// sample count
				&sound_sample_index							// sample index list
			);
		}
	}

	return (TRUE);
}





#if DEBUG_MODULE

static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

// WARNING: THIS IS OVERLOADED AT VEHICLE LEVEL //

static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

// WARNING: THIS IS OVERLOADED AT VEHICLE LEVEL //

static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





void overload_ship_vehicle_message_responses (void)
{
	overload_vehicle_message_responses (ENTITY_TYPE_SHIP_VEHICLE);

	message_responses[ENTITY_TYPE_SHIP_VEHICLE][ENTITY_MESSAGE_COLLISION] 						= response_to_collision;

	message_responses[ENTITY_TYPE_SHIP_VEHICLE][ENTITY_MESSAGE_ARTICULATE_LOADING_DOORS] 	= response_to_articulate_loading_doors;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_SHIP_VEHICLE][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	message_responses[ENTITY_TYPE_SHIP_VEHICLE][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;

	message_responses[ENTITY_TYPE_SHIP_VEHICLE][ENTITY_MESSAGE_LINK_PARENT]		= response_to_link_parent;

	message_responses[ENTITY_TYPE_SHIP_VEHICLE][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif
}





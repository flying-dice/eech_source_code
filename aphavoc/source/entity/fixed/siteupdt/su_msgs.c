

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*aggressor;

	int
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

		if (aggressor)
		{
			notify_keysite_structure_under_attack (receiver, aggressor);
		}
	}
	else
	{
		aggressor = NULL;
	}

	damage_client_server_entity (receiver, sender, damage_modifier);

	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

	if (total_damage_level <= 0)
	{
		if (aggressor)
		{
			if (get_local_entity_float_value (receiver, FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE) >= 0.5)
			{
				if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_LOCAL)
				{
					inc_player_log_kills (get_global_gunship_side (), get_current_player_log (), receiver);
				}
				else if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_REMOTE)
				{
					transmit_entity_comms_message (ENTITY_COMMS_PLAYER_KILL, aggressor, receiver);
				}
			}
		}

		kill_client_server_entity (receiver);
	}

	return (TRUE);
}





static int response_to_articulate_loading_doors (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	//
	// WATING FOR SOUND EFFECT DATA...
	//

	sound_sample_indices
		sample_index;

	vec3d
		*position;







	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	position = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION);

	ASSERT (position);

	sample_index = SOUND_SAMPLE_INDEX_HELICOPTER_DOORS;

	create_client_server_sound_effect_entity
	(
		get_local_sector_entity (position),
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
		&sample_index									// sample index list
	);

	return (TRUE);
}


























































































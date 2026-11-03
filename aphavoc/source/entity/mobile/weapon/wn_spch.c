

































































#include "project.h"





#define DEBUG_MODULE 0





void play_entity_weapon_selected_speech (entity *en, entity_sub_types weapon_type)
{
	int
		speech_index;

	entity_sides
		side;

	//
	// only play speech for players
	//

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	speech_index = weapon_database [weapon_type].weapon_selected_cpg_speech;

	if (speech_index != -1)
	{
		if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			side = (entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE);

			remove_speech_category_from_buffers (side, SPEECH_ORIGINATOR_CO_PILOT, SPEECH_CATEGORY_WEAPONS_SYSTEMS);

			play_client_server_speech
			(
				en,
				en,
				side,
				ENTITY_SUB_TYPE_EFFECT_SOUND_CPG_MESSAGE,
				SOUND_LOCALITY_RADIO,
				0.4,		// delay
				0.2,		// priority
				1.5,		// expire time
				SPEECH_ORIGINATOR_CO_PILOT,
				SPEECH_CATEGORY_WEAPONS_SYSTEMS,
				-1.0,		// category silence timer
				SPEECH_ARRAY_CPG_MESSAGES, speech_index,
				-1
			);
		}
	}
}





void play_entity_weapon_launched_speech (entity *en, entity_sub_types weapon_type)
{
	int
		speech_index;

	//
	// only play speech for players
	//

	speech_index = weapon_database [weapon_type].weapon_launched_cpg_speech;

	if (speech_index != -1)
	{
		if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			play_client_server_cpg_message
			(
				en,
				0.8,
				2.0,
				SPEECH_CATEGORY_FIRING,
				8.0,
				speech_index
			);
		}
	}
}





void play_entity_weapon_low_speech (entity *en, entity_sub_types weapon_type)
{
	int
		speech_index;

	//
	// only play speech for players
	//

	speech_index = weapon_database [weapon_type].weapon_low_cpg_speech;

	if (speech_index != -1)
	{
		if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			if (!get_local_entity_int_value (get_session_entity (), INT_TYPE_INFINITE_WEAPONS))
			{
				play_client_server_cpg_message
				(
					en,
					1.0,
					5.0,
					SPEECH_CATEGORY_AMMO_LOW,
					-1.0,
					speech_index
				);
			}
		}
	}
}





void play_entity_weapon_out_speech (entity *en, entity_sub_types weapon_type)
{
	int
		speech_index;

	//
	// only play speech for players
	//

	speech_index = weapon_database [weapon_type].weapon_out_cpg_speech;

	if (speech_index != -1)
	{
		if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			play_client_server_cpg_message
			(
				en,
				1.0,
				5.0,
				SPEECH_CATEGORY_AMMO_LOW,
				-1.0,
				speech_index
			);
		}
	}
}






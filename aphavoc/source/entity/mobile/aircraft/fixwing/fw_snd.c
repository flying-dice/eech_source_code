

































































#include "project.h"





#define DEBUG_MODULE 0





static void play_local_sound (entity *en, viewpoint *vp, float range)
{

	sound_sample_indices
		effect_type;

	int
		rate,
		looping,
		volume,
		panning;

	fixed_wing
		*raw;

	raw = get_local_entity_data (en);

	if (!raw->ac.sound_effect_data)
	{

		effect_type = aircraft_database [raw->ac.mob.sub_type].continuous_sound_effect_index;

		if (effect_type != SOUND_SAMPLE_INDEX_NONE)
		{

			#if DEBUG_MODULE

			debug_log ("FW_SND: creating sound effect %d", effect_type);

			#endif

			panning = 0;

			rate = 22050;

			volume = 0;
	
			looping = TRUE;
	
			raw->ac.sound_effect_data = create_single_system_sound_effect (effect_type, panning, rate, volume, looping, (void *) en, 0);

			if (raw->ac.sound_effect_data)
			{
	
				raw->ac.valid_sound_effect = TRUE;
	
				play_system_sound_effect (raw->ac.sound_effect_data);
			}
		}
	}
}





void overload_fixed_wing_sound_functions (void)
{

	fn_play_local_entity_sound[ENTITY_TYPE_FIXED_WING]	= play_local_sound;
}





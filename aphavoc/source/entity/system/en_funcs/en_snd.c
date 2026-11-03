

































































#include "project.h"





#define DEBUG_MODULE 0





#define DEBUG_3D_OBJECT_SOUND_RANGE		(5000.0)





static float
	max_3d_object_sound_range;





void scan_local_entity_sounds (viewpoint *vp)
{

	int
		x_sec_min,
		z_sec_min,
		x_sec_max,
		z_sec_max,
		x_sec,
		z_sec;

	entity
		*sctr,
		*en,
		*parent;

	vec3d
		scan_origin,
		*position;

	float
		range,
		max_range;

	struct SYSTEM_SOUND_EFFECT
		*snd_eff;

	ASSERT (vp);

	if (!get_application_sound_system_status ())
	{
		return;
	}

	if (get_sound_system_paused ())
	{
		return;
	}

	// Casm 07DEC09 OpenAL support
	set_system_sound_listener_orientation ( vp->zv, vp->yv, vp->zv );

	//
	// update playing samples
	//

	update_system_sound_effect_system ();

	//
	// play session sound effects
	//
			
	en = get_local_entity_first_child (get_session_entity (), LIST_TYPE_SPECIAL_EFFECT);

	while (en)
	{
		play_local_entity_sound (en, vp, 0.0);

		en = get_local_entity_child_succ (en, LIST_TYPE_SPECIAL_EFFECT);
	}

	//
	// play local sound effects
	//

	scan_origin = vp->position;

	bound_position_to_map_area (&scan_origin);

	get_x_sector (x_sec_min, scan_origin.x - max_3d_object_sound_range);
	get_z_sector (z_sec_min, scan_origin.z - max_3d_object_sound_range);

	get_x_sector (x_sec_max, scan_origin.x + max_3d_object_sound_range);
	get_z_sector (z_sec_max, scan_origin.z + max_3d_object_sound_range);

	x_sec_min = max (x_sec_min, MIN_MAP_X_SECTOR);
	z_sec_min = max (z_sec_min, MIN_MAP_Z_SECTOR);

	x_sec_max = min (x_sec_max, MAX_MAP_X_SECTOR);
	z_sec_max = min (z_sec_max, MAX_MAP_Z_SECTOR);

	//
	// Scan for valid sound effects
	//

	for (z_sec = z_sec_min; z_sec <= z_sec_max; z_sec++)
	{

		for (x_sec = x_sec_min; x_sec <= x_sec_max; x_sec++)
		{

			sctr = get_local_raw_sector_entity (x_sec, z_sec);

			//
			// play sound effects in the sector
			//
			
			en = get_local_entity_first_child (sctr, LIST_TYPE_SPECIAL_EFFECT);

			while (en)
			{
				position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

				range = get_approx_3d_range (&vp->position, position);

				max_range = DEBUG_3D_OBJECT_SOUND_RANGE;

				if (range < max_range)
				{
					play_local_entity_sound (en, vp, range);
				}

				en = get_local_entity_child_succ (en, LIST_TYPE_SPECIAL_EFFECT);
			}

			//
			// play sound effects attached to other entities in the sector
			//
			
			parent = get_local_entity_first_child (sctr, LIST_TYPE_SECTOR);

			while (parent)
			{
				if (get_local_entity_int_value (parent, INT_TYPE_SOUND_LIST_VALID))
				{
					position = get_local_entity_vec3d_ptr (parent, VEC3D_TYPE_POSITION);
	
					range = get_approx_3d_range (&vp->position, position);
	
					max_range = DEBUG_3D_OBJECT_SOUND_RANGE;
	
					if (range < max_range)
					{
						en = get_local_entity_first_child (parent, LIST_TYPE_SPECIAL_EFFECT);
	
						while (en)
						{
							if (get_local_entity_type (en) == ENTITY_TYPE_SOUND_EFFECT)
							{
								play_local_entity_sound (en, vp, range);
							}
	
							en = get_local_entity_child_succ (en, LIST_TYPE_SPECIAL_EFFECT);
						}
					}
				}

				parent = get_local_entity_child_succ (parent, LIST_TYPE_SECTOR);
			}
		}
	}

	//
	// Update playing sound effects
	//

	snd_eff = get_first_system_sound_effect ();

	while (snd_eff)
	{

		en = (entity *) get_system_sound_effect_user_data (snd_eff);

		if (!get_system_sound_effect_playing (snd_eff))
		{
			#if DEBUG_MODULE

			debug_log ("EN_SND : destroying sound coz it's not being played");

			#endif

			destroy_system_sound_effect (snd_eff);

			if (en)
			{

				set_local_entity_ptr_value (en, PTR_TYPE_SOUND_EFFECT_DATA, NULL);
			}
		}
		else if (en)
		{
			if (get_local_entity_parent (en, LIST_TYPE_SPECIAL_EFFECT) != get_session_entity ())
			{
				position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	
				range = get_approx_3d_range (&vp->position, position);
	
				max_range = 5000;
	
				if (range > max_range)
				{
	
					#if DEBUG_MODULE
	
					debug_log ("EN_SND: destroying sound effect (too far away)");
	
					#endif
	
					destroy_system_sound_effect (snd_eff);
	
					set_local_entity_ptr_value (en, PTR_TYPE_SOUND_EFFECT_DATA, NULL);
				}
			}
		}

		snd_eff = get_next_system_sound_effect (snd_eff);
	}
}





void stop_local_entity_sounds ()
{
	struct SYSTEM_SOUND_EFFECT
		*snd_eff;

	entity
		*en;

	//
	// stop all sounds (not in the 3d visual)
	//

	snd_eff = get_first_system_sound_effect ();

	while (snd_eff)
	{

		en = (entity *) get_system_sound_effect_user_data (snd_eff);

		if (en)
		{
			destroy_system_sound_effect (snd_eff);

			set_local_entity_ptr_value (en, PTR_TYPE_SOUND_EFFECT_DATA, NULL);
		}

		snd_eff = get_next_system_sound_effect (snd_eff);
	}
}





void (*fn_play_local_entity_sound[NUM_ENTITY_TYPES]) (entity *en, viewpoint *vp, float range);





static void default_play_entity_sound (entity *en, viewpoint *vp, float range)
{
}





void initialise_entity_sound_default_functions (void)
{

	int
		i;

	max_3d_object_sound_range = DEBUG_3D_OBJECT_SOUND_RANGE;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{

		fn_play_local_entity_sound[i]	= default_play_entity_sound;
	}
}





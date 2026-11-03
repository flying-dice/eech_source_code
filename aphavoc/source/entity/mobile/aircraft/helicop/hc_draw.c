

































































#include "project.h"
#include "../../../../gunships/dynamics/common/co_undercarriage.h"




#define DEBUG_MODULE 0





static float
	sprite_light_timer = 0.0;

#define SPRITE_LIGHT_TIMER_FREQUENCY 0.5

#define SPRITE_LIGHT_ON_THRESHOLD 0.1

#ifndef OGRE_EE
static int object_internal_lighting_valid (entity *en);

static int sprite_light_valid (entity *en);
#endif





static void draw_local_3d_object (entity *en, float range)
{
	helicopter
		*raw;

	int
		internal_view,
		draw_gunship;

	ASSERT(en);

	raw = (helicopter *) get_local_entity_data (en);

	//
	// the player's gunship is visible from the menus - where the in_cockpit flag is invalid
	//

	if (en == get_gunship_entity ())
	{
		internal_view = ((in_cockpit) && (get_in_flight_game_mode () == IN_FLIGHT_GAME_MODE_SIMULATOR));
	}
	else
	{
		internal_view = FALSE;
	}

	draw_gunship = TRUE;


	//
	// SORT COMANCHE AND HOKUM INSTANCES
	//
	// It is necessary to switch parts on and off for all Comanche and Hokum instances
	// because the player can change gunship so the instances need to be fixed up.
	//


	{
		int
			draw_virtual_cockpit_parts,
			ejected;

		object_3d_sub_object_search_data
			search;

		switch (raw->ac.object_3d_shape)
		{
		case OBJECT_3D_RAH66:
		{

			//
			// Comanche
			//


			//
			// draw 'canopy' doors on player object and 'loading' doors on AI object
			//

			draw_virtual_cockpit_parts = FALSE;

			if (en == get_gunship_entity () && internal_view)
				draw_virtual_cockpit_parts = TRUE;

			//
			// fuselage
			//

			search.search_depth = 0;
			search.search_object = raw->ac.inst3d;
			search.sub_object_index = OBJECT_3D_SUB_OBJECT_RAH66_FUSELAGE;

			if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
			{
				search.result_sub_object->visible_object = !draw_virtual_cockpit_parts;
			}

			break;
		}
		case OBJECT_3D_KA_52:
		{

			//
			// Hokum
			//


			ejected = get_local_entity_int_value (en, INT_TYPE_EJECTED);

			draw_virtual_cockpit_parts = FALSE;

			if (en == get_gunship_entity () && internal_view)
				draw_virtual_cockpit_parts = TRUE;

			//
			// stub wings
			//

			search.search_depth = 0;
			search.search_object = raw->ac.inst3d;
			search.sub_object_index = OBJECT_3D_SUB_OBJECT_KA52_WINGS;

			if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
			{
				search.result_sub_object->visible_object = draw_virtual_cockpit_parts;
			}

			//
			// fuselage
			//

			search.search_depth = 0;
			search.search_object = raw->ac.inst3d;
			search.sub_object_index = OBJECT_3D_SUB_OBJECT_KA52_FUSELAGE;

			if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
			{
				search.result_sub_object->visible_object = !draw_virtual_cockpit_parts;
			}

			if (ejected)
			{
				//
				// canopy/loading doors
				//

				search.search_depth = 0;
				search.search_object = raw->ac.inst3d;
				search.sub_object_index = OBJECT_3D_SUB_OBJECT_CANOPY_DOORS;

				if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					search.result_sub_object->visible_object = FALSE;
				}

				search.search_depth = 1;
				search.search_object = raw->ac.inst3d;
				search.sub_object_index = OBJECT_3D_SUB_OBJECT_CANOPY_DOORS;

				if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					search.result_sub_object->visible_object = FALSE;
				}

				//
				// ejector seats & crew
				//

				search.search_depth = 0;
				search.search_object = raw->ac.inst3d;
				search.sub_object_index = OBJECT_3D_SUB_OBJECT_KA52_EJECTOR_SEAT;

				if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					search.result_sub_object->visible_object = FALSE;
				}

				search.search_depth = 1;
				search.search_object = raw->ac.inst3d;
				search.sub_object_index = OBJECT_3D_SUB_OBJECT_KA52_EJECTOR_SEAT;

				if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					search.result_sub_object->visible_object = FALSE;
				}
			}

			break;
		}
		case OBJECT_3D_AH1Z:
		case OBJECT_3D_OH58D:
		{
			draw_virtual_cockpit_parts = internal_view;

			search.search_depth = 0;
			search.search_object = raw->ac.inst3d;
			search.sub_object_index = OBJECT_3D_SUB_OBJECT_RAH66_FUSELAGE;

			if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
			{
				search.result_sub_object->visible_object = !draw_virtual_cockpit_parts;
			}
			break;
		}
		default:
		{
			draw_gunship = FALSE;
			break;
		}
		}
	}



	ASSERT(en);

	if (en == get_gunship_entity ())
	{
		//
		// update viewpoint
		//

		memcpy (&raw->ac.inst3d->vp.position, &raw->ac.mob.position, sizeof (vec3d));

		memcpy (&raw->ac.inst3d->vp.attitude, &raw->ac.mob.attitude, sizeof (matrix3x3));

		//
		// animate
		//

		animate_helicopter_main_rotors (en, FALSE);

		animate_helicopter_tail_rotor (en);

		animate_helicopter_controls (en);

		animate_helicopter_wipers (en);

		animate_helicopter_eo (en);

		animate_aircraft_suspension (en);

		animate_helicopter_wheels (en);

		animate_aircraft_loading_doors (en);

		animate_aircraft_cargo_doors (en);

		animate_aircraft_undercarriage (en);

		animate_aircraft_weapon_system_ready (en);

		animate_aircraft_shadow (en);

#ifndef OGRE_EE
		if (active_3d_environment->render_filter == RENDER_INFRARED)
		{
			raw->ac.inst3d->object_diffuse_value = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 230 : 255;
			raw->ac.inst3d->object_internal_lighting = raw->ac.inst3d->object_sprite_lights = FALSE;
		}
		else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
		{
			raw->ac.inst3d->object_diffuse_value = 0; //32 * (1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5 * ONE_MINUTE) / (2 * ONE_MINUTE));
			raw->ac.inst3d->object_internal_lighting = raw->ac.inst3d->object_sprite_lights = FALSE;
		}
		else
		{
			raw->ac.inst3d->object_diffuse_value = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 255 : 127;
			raw->ac.inst3d->object_diffuse_value *= 1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5f * ONE_MINUTE) / (2 * ONE_MINUTE);
			raw->ac.inst3d->object_internal_lighting = object_internal_lighting_valid (en);
			raw->ac.inst3d->object_sprite_lights = (raw->ac.inst3d->object_internal_lighting && sprite_light_valid (en));
		}
#endif

		//
		// draw
		//

		if (!internal_view)
		{
			if (!in_flight_invisible_object)
			{
				animate_and_draw_entity_muzzle_flash_effect (en);

#ifndef OGRE_EE
				insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->ac.inst3d);
#else
				object_3d_draw (raw->ac.inst3d);
#endif
			}
		}
		else
		{
			if (get_comanche_hokum_gunship () && draw_gunship)
			{
				if (get_global_draw_cockpit_graphics ())
				{
					if (!get_global_glass_cockpit ())
					{
						if
						(
							(get_view_mode () == VIEW_MODE_VIRTUAL_COCKPIT_CREW) ||
							(get_global_cockpit_detail_level () == COCKPIT_DETAIL_LEVEL_HIGH)
						)
						{
							animate_and_draw_entity_muzzle_flash_effect (en);

							raw->ac.inst3d->vp.position = virtual_cockpit_inst3d->vp.position;

							memcpy (&raw->ac.inst3d->vp.attitude, &virtual_cockpit_inst3d->vp.attitude, sizeof (matrix3x3));

#ifndef OGRE_EE
							insert_relative_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_ZBUFFERED_OBJECT, &raw->ac.inst3d->vp.position, raw->ac.inst3d);
#else
							object_3d_draw (raw->ac.inst3d);
#endif
						}
					}
				}
			}
		}
	}
	else
	{
		//
		// update viewpoint
		//

		memcpy (&raw->ac.inst3d->vp.position, &raw->ac.mob.position, sizeof (vec3d));

		memcpy (&raw->ac.inst3d->vp.attitude, &raw->ac.mob.attitude, sizeof (matrix3x3));

		//
		// animate
		//

		animate_helicopter_main_rotors (en, FALSE);

		animate_helicopter_tail_rotor (en);

		animate_helicopter_controls (en);

		animate_aircraft_suspension (en);

		animate_aircraft_loading_doors (en);

		animate_aircraft_cargo_doors (en);

		animate_aircraft_undercarriage (en);

		animate_aircraft_weapon_system_ready (en);

		animate_aircraft_shadow (en);

#ifndef OGRE_EE
		if (active_3d_environment->render_filter == RENDER_INFRARED)
		{
			raw->ac.inst3d->object_diffuse_value = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 230 : 255;
			raw->ac.inst3d->object_internal_lighting = raw->ac.inst3d->object_sprite_lights = FALSE;
		}
		else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
		{
			raw->ac.inst3d->object_diffuse_value = 0; //32 * (1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5 * ONE_MINUTE) / (2 * ONE_MINUTE));
			raw->ac.inst3d->object_internal_lighting = raw->ac.inst3d->object_sprite_lights = FALSE;
		}
		else
		{
			animate_and_draw_entity_muzzle_flash_effect (en);

			raw->ac.inst3d->object_diffuse_value = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 255 : 127;
			raw->ac.inst3d->object_diffuse_value *= 1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5f * ONE_MINUTE) / (2 * ONE_MINUTE);
			raw->ac.inst3d->object_internal_lighting = object_internal_lighting_valid (en);
			raw->ac.inst3d->object_sprite_lights = (raw->ac.inst3d->object_internal_lighting && sprite_light_valid (en));
		}

		//
		// draw
		//

		insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->ac.inst3d);
#else
		animate_and_draw_entity_muzzle_flash_effect (en);
		object_3d_draw (raw->ac.inst3d);
#endif
	}

#ifndef OGRE_EE

	//
	// draw comanche and hokum shadows
	//


	if (raw->ac.object_3d_shape == OBJECT_3D_RAH66)
	{
		object_3d_instance
			*shadow_object;

		shadow_object = construct_temporary_3d_object (OBJECT_3D_RAH66_FUSELAGE, FALSE);

		shadow_object->vp = raw->ac.inst3d->vp;

		insert_object_shadow_into_3d_scene (shadow_object);
	}
	else if (raw->ac.object_3d_shape == OBJECT_3D_KA_52)
	{
		object_3d_instance
			*shadow_object;

		shadow_object = construct_temporary_3d_object (OBJECT_3D_KA_52_FUSELAGE, FALSE);

		shadow_object->vp = raw->ac.inst3d->vp;

		insert_object_shadow_into_3d_scene (shadow_object);
	}
#endif




	// Xhit: added for downwash effect. (030328)
	// VJ 030512 changed from no_downbwash to downwash

	if(command_line_downwash)
	{
		draw_downwash_effect(en);
	}


	#if DEBUG_MODULE

	if (en == get_external_view_entity ())
	{
		draw_mobile_entity_debug_info (en);
	}

	#endif
}





void overload_helicopter_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_HELICOPTER]	= draw_local_3d_object;
}





#ifndef OGRE_EE
int object_internal_lighting_valid (entity *en)
{
	day_segment_types
		day_segment_type;

	if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
	{
		return (get_local_entity_int_value (en, INT_TYPE_LIGHTS_ON));
	}
	else
	{
		day_segment_type = (day_segment_types) get_local_entity_int_value (get_session_entity (), INT_TYPE_DAY_SEGMENT_TYPE);

		return (get_local_entity_int_value (en, INT_TYPE_MOBILE_MOVING) &&
				((day_segment_type == DAY_SEGMENT_TYPE_NIGHT) || (day_segment_type == DAY_SEGMENT_TYPE_DUSK) || (day_segment_type == DAY_SEGMENT_TYPE_DAWN)));
	}
}
#endif





void update_helicopter_sprite_light_timers (void)
{
	sprite_light_timer += (get_delta_time () * SPRITE_LIGHT_TIMER_FREQUENCY);

	if (sprite_light_timer >= 1.0)
	{
		sprite_light_timer = frac (sprite_light_timer);
	}
}





#ifndef OGRE_EE
int sprite_light_valid (entity *en)
{
	int
		integer_offset;

	float
		val,
		float_offset;

	//
	// uses entity index to offset sprite lights so that they are not all flashing in sync on all objects
	//

	if (!get_local_entity_int_value(en, INT_TYPE_ALIVE))
		return FALSE;

	integer_offset = get_local_entity_index (en);

	integer_offset = integer_offset & 0x0f;

	float_offset = ((float)integer_offset) * 0.0625;

	val = frac (sprite_light_timer + float_offset);

	return (val < SPRITE_LIGHT_ON_THRESHOLD);
}
#endif





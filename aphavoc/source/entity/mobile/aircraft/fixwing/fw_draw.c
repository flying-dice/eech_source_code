

































































#include "project.h"





#define DEBUG_MODULE 0





static float
	sprite_light_timer = 0.0;

#define SPRITE_LIGHT_TIMER_FREQUENCY 0.5

#define SPRITE_LIGHT_ON_THRESHOLD 0.1

#ifndef OGRE_EE
static int sprite_light_valid (entity *en);
#endif





static void draw_local_3d_object (entity *en, float range)
{
	fixed_wing
		*raw;

#ifndef OGRE_EE
	day_segment_types
		day_segment_type;
#endif

	raw = (fixed_wing *) get_local_entity_data (en);

	//
	// update viewpoint
	//

	memcpy (&raw->ac.inst3d->vp.position, &raw->ac.mob.position, sizeof (vec3d));

	memcpy (&raw->ac.inst3d->vp.attitude, &raw->ac.mob.attitude, sizeof (matrix3x3));

	//
	// animate
	//

	animate_fixed_wing_afterburners (en);

	animate_fixed_wing_airbrakes (en);

	animate_fixed_wing_flaps (en);

	animate_fixed_wing_propellors (en);

	animate_aircraft_loading_doors (en);

	animate_aircraft_cargo_doors (en);

	animate_aircraft_undercarriage (en);

	animate_aircraft_weapon_system_ready (en);

	animate_aircraft_shadow (en);

	animate_aircraft_rudder (en);

	//
	// draw
	//

#ifndef OGRE_EE
	day_segment_type = (day_segment_types) get_local_entity_int_value (get_session_entity (), INT_TYPE_DAY_SEGMENT_TYPE);

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
		raw->ac.inst3d->object_diffuse_value *= 1.0f - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5f * ONE_MINUTE) / (2 * ONE_MINUTE);
		raw->ac.inst3d->object_internal_lighting = ((day_segment_type == DAY_SEGMENT_TYPE_NIGHT) || (day_segment_type == DAY_SEGMENT_TYPE_DUSK));
		raw->ac.inst3d->object_sprite_lights = (raw->ac.inst3d->object_internal_lighting && sprite_light_valid (en));
	}

	insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->ac.inst3d);
#else
	animate_and_draw_entity_muzzle_flash_effect (en);
	object_3d_draw (raw->ac.inst3d);
#endif

#ifndef OGRE_EE
	#if DEBUG_MODULE

	if (en == get_external_view_entity ())
	{
		vec3d
			*pos,
			wp_pos;

		draw_mobile_entity_debug_info (en);

		if (get_local_entity_primary_task (en))
		{
			fixed_wing_movement_get_waypoint_position (en, &wp_pos);

			pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

			create_debug_3d_line (pos, &wp_pos, sys_col_dark_green, 0.0);
		}
	}

	#endif
#endif
}





void overload_fixed_wing_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_FIXED_WING]	= draw_local_3d_object;
}





void update_fixed_wing_sprite_light_timers (void)
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

	integer_offset = get_local_entity_index (en);

	integer_offset = integer_offset & 0x0f;

	float_offset = ((float)integer_offset) * 0.0625;

	val = frac (sprite_light_timer + float_offset);

	return (val < SPRITE_LIGHT_ON_THRESHOLD);
}
#endif





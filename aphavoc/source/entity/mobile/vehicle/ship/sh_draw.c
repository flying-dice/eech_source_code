

































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{
	ship_vehicle
		*raw;

#ifndef OGRE_EE
	day_segment_types
		day_segment_type;
#endif

	raw = (ship_vehicle *) get_local_entity_data (en);

	//
	// update viewpoint
	//

	memcpy (&raw->vh.inst3d->vp.position, &raw->vh.mob.position, sizeof (vec3d));

	memcpy (&raw->vh.inst3d->vp.attitude, &raw->vh.mob.attitude, sizeof (matrix3x3));

	//
	// animate
	//

	animate_ship_rotors (en);

	animate_vehicle_loading_doors (en);

	animate_vehicle_radar (en);

	animate_vehicle_weapon_system_ready (en);

#ifndef OGRE_EE
	day_segment_type = (day_segment_types) get_local_entity_int_value (get_session_entity (), INT_TYPE_DAY_SEGMENT_TYPE);

	//
	// draw
	//

	raw->vh.inst3d->object_has_shadow = FALSE;

	if (active_3d_environment->render_filter == RENDER_INFRARED)
	{
		raw->vh.inst3d->object_diffuse_value = 80;
		raw->vh.inst3d->object_internal_lighting = raw->vh.inst3d->object_sprite_lights = FALSE;
	}
	else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
	{
		raw->vh.inst3d->object_diffuse_value = 0; //32 * (1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5 * ONE_MINUTE) / (2 * ONE_MINUTE));
		raw->vh.inst3d->object_internal_lighting = raw->vh.inst3d->object_sprite_lights = FALSE;
	}
	else
	{
		animate_and_draw_entity_muzzle_flash_effect (en);

		raw->vh.inst3d->object_diffuse_value = 255;
		raw->vh.inst3d->object_internal_lighting = ((day_segment_type == DAY_SEGMENT_TYPE_NIGHT) || (day_segment_type == DAY_SEGMENT_TYPE_DUSK));
		raw->vh.inst3d->object_sprite_lights = raw->vh.inst3d->object_internal_lighting;
	}

	insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->vh.inst3d);
#else
	animate_and_draw_entity_muzzle_flash_effect (en);
	object_3d_draw (raw->vh.inst3d);
#endif
}





void overload_ship_vehicle_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_SHIP_VEHICLE]	= draw_local_3d_object;
}





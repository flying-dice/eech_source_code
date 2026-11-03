

































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{
	anti_aircraft
		*raw;

	raw = (anti_aircraft *) get_local_entity_data (en);

	//
	// update viewpoint
	//

	memcpy (&raw->vh.inst3d->vp.position, &raw->vh.mob.position, sizeof (vec3d));

	memcpy (&raw->vh.inst3d->vp.attitude, &raw->vh.mob.attitude, sizeof (matrix3x3));

	//
	// animate
	//

	animate_vehicle_loading_doors (en);

	animate_vehicle_radar (en);

	animate_vehicle_weapon_system_ready (en);

	//
	// draw
	//

#ifndef OGRE_EE
	if (active_3d_environment->render_filter == RENDER_INFRARED)
	{
		raw->vh.inst3d->object_diffuse_value = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 230 : 255;
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

		raw->vh.inst3d->object_diffuse_value = get_local_entity_int_value (en, INT_TYPE_ALIVE) ? 255 : 127;
		raw->vh.inst3d->object_diffuse_value *= 1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 1.5f * ONE_MINUTE) / (2 * ONE_MINUTE);
	}

	insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->vh.inst3d);
#else
	animate_and_draw_entity_muzzle_flash_effect (en);
	object_3d_draw (raw->vh.inst3d);
#endif
}





void overload_anti_aircraft_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_ANTI_AIRCRAFT]	= draw_local_3d_object;
}





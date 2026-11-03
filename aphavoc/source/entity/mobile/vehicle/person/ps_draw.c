

































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{
	person
		*raw;

	raw = (person *) get_local_entity_data (en);

	//
	// update viewpoint
	//

	memcpy (&raw->vh.inst3d->vp.position, &raw->vh.mob.position, sizeof (vec3d));

	memcpy (&raw->vh.inst3d->vp.attitude, &raw->vh.mob.attitude, sizeof (matrix3x3));

	//
	// animate
	//

	animate_person (en);

	animate_vehicle_weapon_system_ready (en);

	//
	// draw
	//

#ifndef OGRE_EE
	raw->vh.inst3d->object_has_shadow = FALSE;

	if (active_3d_environment->render_filter == RENDER_INFRARED)
		raw->vh.inst3d->object_diffuse_value = 255 * (1 - min(get_local_entity_float_value (en, FLOAT_TYPE_DEATH_TIMER), 3.0f * ONE_MINUTE) / (4 * ONE_MINUTE));
	else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
		raw->vh.inst3d->object_diffuse_value = 0; //32;
	else
	{
		raw->vh.inst3d->object_diffuse_value = 255;
		animate_and_draw_entity_muzzle_flash_effect (en);
	}

	insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->vh.inst3d);
#else
	animate_and_draw_entity_muzzle_flash_effect (en);
	object_3d_draw (raw->vh.inst3d);
#endif
}





void overload_person_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_PERSON]	= draw_local_3d_object;
}





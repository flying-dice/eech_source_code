

































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{
	crater
		*raw;

	object_3d_instance
		*inst3d;

	vec3d
		face_normal;

	raw = (crater *) get_local_entity_data (en);

	inst3d = construct_temporary_3d_object (raw->fix.object_3d_shape, FALSE);

	if (inst3d)
	{
#ifndef OGRE_EE
		inst3d->object_has_shadow = FALSE;

		inst3d->requires_immediate_render = TRUE;
#endif

		inst3d->relative_scale.x = raw->scale;
		inst3d->relative_scale.y = raw->scale;
		inst3d->relative_scale.z = raw->scale;

		memcpy (&inst3d->vp.position, &raw->fix.position, sizeof (vec3d));

		get_3d_terrain_face_normal (&face_normal, raw->fix.position.x, raw->fix.position.z);

		get_3d_transformation_matrix_from_face_normal_and_heading (inst3d->vp.attitude, &face_normal, raw->fix.heading);

#ifndef OGRE_EE
		if (active_3d_environment->render_filter == RENDER_INFRARED)
			inst3d->object_diffuse_value = 10;
		else
			inst3d->object_diffuse_value = 255;

		insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, inst3d);
#else
		object_3d_draw (inst3d);
#endif
	}
}




/*
static void draw_local_2d_symbol (entity *en)
{
}
*/




void overload_crater_draw_functions (void)
{
	fn_draw_local_entity_3d_object [ENTITY_TYPE_CRATER]	 = draw_local_3d_object;

	//fn_draw_local_entity_2d_symbol [ENTITY_TYPE_CRATER]	 = draw_local_2d_symbol;
}





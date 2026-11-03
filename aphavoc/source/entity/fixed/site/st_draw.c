

































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{

	site
		*raw;

	object_3d_instance
		*inst3d;

	vec3d
		face_normal;

	day_segment_types
		day_segment_type;

	raw = (site *) get_local_entity_data (en);

	inst3d = construct_temporary_3d_object (raw->fix.object_3d_shape, FALSE);

	if (inst3d)
	{
		memcpy (&inst3d->vp.position, &raw->fix.position, sizeof (vec3d));

		if (fixed_database[raw->fix.sub_type].align_with_terrain)
		{
			get_3d_terrain_face_normal (&face_normal, raw->fix.position.x, raw->fix.position.z);

			get_3d_transformation_matrix_from_face_normal_and_heading (inst3d->vp.attitude, &face_normal, raw->fix.heading);
		}
		else
		{
			get_3d_transformation_matrix (inst3d->vp.attitude, raw->fix.heading, 0.0, 0.0);
		}

		day_segment_type = (day_segment_types) get_local_entity_int_value (get_session_entity (), INT_TYPE_DAY_SEGMENT_TYPE);

#ifndef OGRE_EE
		inst3d->object_has_shadow = FALSE;

		if (active_3d_environment->render_filter == RENDER_INFRARED)
		{
			inst3d->object_diffuse_value = 64;
			inst3d->object_sprite_lights = FALSE;
		}
		else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
		{
			inst3d->object_diffuse_value = 64;
			inst3d->object_sprite_lights = FALSE;
		}
		else
		{
			inst3d->object_diffuse_value = 255;
			inst3d->object_sprite_lights = ((day_segment_type == DAY_SEGMENT_TYPE_NIGHT) || (day_segment_type == DAY_SEGMENT_TYPE_DUSK));
		}

		insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, inst3d);
#else
		object_3d_draw (inst3d);
#endif
	}
}





void overload_site_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_SITE]	= draw_local_3d_object;
}





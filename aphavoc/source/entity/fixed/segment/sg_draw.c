

































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{
	segment
		*raw;

	entity
		*parent;

	object_3d_instance
		*inst3d;

	float
		pitch;

	raw = (segment *) get_local_entity_data (en);

	if (raw->fix.object_3d_shape == OBJECT_3D_INVALID_OBJECT_INDEX)
	{
		return;
	}

	inst3d = construct_temporary_3d_object (raw->fix.object_3d_shape, TRUE);

	if (inst3d)
	{
#ifndef OGRE_EE
		inst3d->object_has_shadow = FALSE;
#endif

		parent = get_local_entity_parent (en, LIST_TYPE_SEGMENT);

		ASSERT (parent);

		memcpy (&inst3d->vp.position, &raw->fix.position, sizeof (vec3d));

		pitch = get_local_entity_float_value (parent, FLOAT_TYPE_PITCH);

		get_3d_transformation_matrix (inst3d->vp.attitude, raw->fix.heading, pitch, 0.0);

		if (parent->type == ENTITY_TYPE_BRIDGE)
		{
			//
			// special case for bridges - scale the legs
			//

			vec3d
				relative_scale;

			float
				leg_scale;

			leg_scale = get_local_entity_float_value (parent, FLOAT_TYPE_SCALE);

			if (leg_scale != 1.0)
			{
				relative_scale.x = 1.0;
				relative_scale.y = leg_scale;
				relative_scale.z = 1.0;

				set_sub_object_type_relative_scale(inst3d, OBJECT_3D_SUB_OBJECT_BRIDGE_LEG, &relative_scale);
			}

			set_sub_object_type_heading_pitch_and_roll(inst3d, OBJECT_3D_SUB_OBJECT_BRIDGE_LEG, 0.0, pitch, 0.0);

		}

#ifndef OGRE_EE
		if (active_3d_environment->render_filter == RENDER_INFRARED)
			inst3d->object_diffuse_value = 64;
		else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
			inst3d->object_diffuse_value = 32;
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
	segment
		*raw;

	struct OBJECT_3D_BOUNDS
		*bounding_box;

	if (command_line_run_mode == RUN_MODE_AITOOL)
	{

		raw = get_local_entity_data (en);

		set_2d_instance_position (active_2d_environment, raw->fix.position.x, raw->fix.position.z);

		bounding_box = get_object_3d_bounding_box (raw->fix.object_3d_shape);

		set_2d_instance_scaling (active_2d_environment, bounding_box->xmax - bounding_box->xmin, bounding_box->zmax - bounding_box->zmin);

		set_2d_instance_rotation (active_2d_environment, -get_local_entity_float_value (en, FLOAT_TYPE_HEADING));

		draw_2d_square (sys_col_dark_red);

		reset_2d_instance (planner_map_env);
	}
}
*/





void overload_segment_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_SEGMENT]	= draw_local_3d_object;

	//fn_draw_local_entity_2d_symbol[ENTITY_TYPE_SEGMENT]	= draw_local_2d_symbol;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void draw_local_3d_object (entity *en, float range)
{
	city
		*raw;

	entity
		*building;

	object_3d_instance
		*inst3d;

	float rangefactor; //Werewolf 16Apr03

	raw = (city *) get_local_entity_data (en);

	//
	// Draw all city buildings
	//

#ifndef OGRE_EE
	range *= current_3d_viewangle_distance_conversion_factor;
#endif

	//Werewolf 16Apr03
	//rangefactor is 1 for default cbar value
	rangefactor = (command_line_city_block_approximation_range / 500.0);

	if ((range < command_line_city_block_approximation_range * METRE * rangefactor) || (raw->approximation_object_3d_shape == OBJECT_3D_INVALID_OBJECT_INDEX))
	{

		if ( raw->fix.object_3d_shape != OBJECT_3D_INVALID_OBJECT_INDEX )
		{

			if ( range < object_3d_information_database[raw->fix.object_3d_shape].maximum_distance*rangefactor )
			{

				inst3d = construct_temporary_3d_object (raw->fix.object_3d_shape, FALSE);

				if (inst3d)
				{

					memcpy (&inst3d->vp.position, &raw->fix.position, sizeof (vec3d));

					get_3d_transformation_heading_matrix (inst3d->vp.attitude, raw->fix.heading);

#ifndef OGRE_EE
					inst3d->object_has_shadow = FALSE;

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
		}

		building = raw->city_building_root.first_child;

		while (building)
		{

			draw_local_city_building_object (building, range);

			building = get_local_entity_child_succ (building, LIST_TYPE_CITY_BUILDING);
		}
	}
	else
	{

		//
		// Draw approximation object
		//

		if ((raw->fix.alive) && (raw->approximation_object_3d_shape != OBJECT_3D_INVALID_OBJECT_INDEX))
		{

//			if ( range < object_3d_information_database[raw->approximation_object_3d_shape].maximum_distance )
			{

				inst3d = construct_temporary_3d_object (raw->approximation_object_3d_shape, FALSE);

				if (inst3d)
				{

					memcpy (&inst3d->vp.position, &raw->fix.position, sizeof (vec3d));

					get_3d_transformation_heading_matrix (inst3d->vp.attitude, raw->fix.heading);

#ifndef OGRE_EE
					inst3d->object_has_shadow = FALSE;

					if (active_3d_environment->render_filter == RENDER_INFRARED)
						inst3d->object_diffuse_value = 64;
					else if (active_3d_environment->render_filter == RENDER_MONOCHROME)
						inst3d->object_diffuse_value = 220;
					else
						inst3d->object_diffuse_value = 255;

					insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, inst3d);
#else
					object_3d_draw (inst3d);
#endif
				}
			}
		}
	}
}





/*
static void draw_local_2d_symbol (entity *en)
{

	city
		*raw;

	int
		screen_x,
		screen_y;

	struct OBJECT_3D_BOUNDS
		*bounding_box;

	raw = get_local_entity_data (en);

	switch (command_line_run_mode)
	{

		case RUN_MODE_AITOOL:
		{

			set_2d_instance_position (active_2d_environment, raw->fix.position.x, raw->fix.position.z);

			bounding_box = get_object_3d_bounding_box (raw->fix.object_3d_shape);

			set_2d_instance_scaling (active_2d_environment, bounding_box->xmax - bounding_box->xmin, bounding_box->zmax - bounding_box->zmin);

			set_2d_instance_rotation (active_2d_environment, -get_local_entity_float_value (en, FLOAT_TYPE_HEADING));

			draw_2d_square (sys_col_dark_red);

			reset_2d_instance (aitool_map_env);

			break;
		}

		case RUN_MODE_NORMAL:
		default:
		{

			if (!raw->fix.alive)
			{
				return;
			}

			reset_2d_instance (planner_map_env);

			//
			// Calculate screen coords
			//

			get_2d_int_screen_coordinates (raw->fix.position.x, raw->fix.position.z, &screen_x, &screen_y);

			//
			// Draw correct icon
			//

			switch (raw->fix.side)
			{

				case ENTITY_SIDE_BLUE_FORCE:
				{

					blit_rgb_centralised_clipped_graphic (get_graphics_file_data (GRAPHICS_UI_COMMON_US_BUILDING_ICON), screen_x, screen_y);

					break;
				}

				case ENTITY_SIDE_RED_FORCE:
				{

					blit_rgb_centralised_clipped_graphic (get_graphics_file_data (GRAPHICS_UI_COMMON_CIS_BUILDING_ICON), screen_x, screen_y);

					break;
				}
			}
		}
	}
}
*/





void overload_city_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_CITY]	= draw_local_3d_object;

	//fn_draw_local_entity_2d_symbol[ENTITY_TYPE_CITY]	= draw_local_2d_symbol;
}





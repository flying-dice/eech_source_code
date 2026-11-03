

































































#include "project.h"





#define DEBUG_MODULE 0





#define DEBUG_3D_OBJECT_VISUAL_RANGE   (10000.0)

#define DEBUG_3D_LINE_VISUAL_RANGE     (10000.0)





int
	draw_eo_3d_scene;





void reset_local_entity_drawn_flags (void)
{
	entity
		*en;

	ASSERT (get_update_entity ());

	en = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);

	while (en)
	{
		set_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME, FALSE);

		en = get_local_entity_child_succ (en, LIST_TYPE_UPDATE);
	}
}





void scan_local_entity_3d_objects (viewpoint *vp)
{
	int
		x_sec_org,
		z_sec_org,
		x_sec_min,
		z_sec_min,
		x_sec_max,
		z_sec_max,
		x_sec,
		z_sec;

	float
		hither,
		range,
		env_max_range,
		object_max_range,
		fog_start,
		fog_end;

	entity
		*sctr,
		*en;

	vec3d
		scan_origin,
		*position;

	env_3d
		*env;

	ASSERT (vp);

	//
	// get active 3D environment max range
	//

	env = get_3d_active_environment ();

	ASSERT (env);

	get_3d_view_distances (env, &env_max_range, &hither);

	//
	// get sector scan limits
	//

	scan_origin = vp->position;

	bound_position_to_map_area (&scan_origin);

	get_x_sector (x_sec_org, scan_origin.x);
	get_z_sector (z_sec_org, scan_origin.z);

	get_x_sector (x_sec_min, scan_origin.x - env_max_range);
	get_z_sector (z_sec_min, scan_origin.z - env_max_range);

	get_x_sector (x_sec_max, scan_origin.x + env_max_range);
	get_z_sector (z_sec_max, scan_origin.z + env_max_range);

	x_sec_min = max (x_sec_min, MIN_MAP_X_SECTOR);
	z_sec_min = max (z_sec_min, MIN_MAP_Z_SECTOR);

	x_sec_max = min (x_sec_max, MAX_MAP_X_SECTOR);
	z_sec_max = min (z_sec_max, MAX_MAP_Z_SECTOR);

	//
	// scan sectors
	//

	for (z_sec = z_sec_min; z_sec <= z_sec_max; z_sec++)
	{
		for (x_sec = x_sec_min; x_sec <= x_sec_max; x_sec++)
		{
			sctr = get_local_raw_sector_entity (x_sec, z_sec);

			en = get_local_entity_first_child (sctr, LIST_TYPE_SECTOR);

			while (en)
			{

				if (get_local_entity_valid_3d_draw_function (en))
				{
					position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

					range = get_approx_3d_range (&vp->position, position);

					if (range < env_max_range)
					{
						get_3d_fog_distances (env, &fog_start, &fog_end);

						if (draw_eo_3d_scene)
						{
							object_max_range = fog_end;
						}
						else
						{
							if (command_line_high_lod_hack < 0 || command_line_high_lod_hack > 5)
							{
								debug_fatal ("wrong high_lod_hack value, please check eech.ini");
							}
							object_max_range = ((5 - command_line_high_lod_hack) * get_local_entity_float_value (en, FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE) + command_line_high_lod_hack * fog_end) / 5;
						}

						if (range < object_max_range)
						{

							draw_local_entity_3d_object (en, range);

							set_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME, TRUE);
						}
					}
				}

				en = get_local_entity_child_succ (en, LIST_TYPE_SECTOR);
			}
		}
	}

	//
	// debug
	//

	draw_debug_3d_objects (vp, DEBUG_3D_OBJECT_VISUAL_RANGE);

	draw_debug_3d_lines (vp, DEBUG_3D_LINE_VISUAL_RANGE);
}





void (*fn_draw_local_entity_3d_object[NUM_ENTITY_TYPES]) (entity *en, float range);





static void default_draw_entity_3d_object (entity *en, float range)
{
}





void initialise_entity_draw_default_functions (void)
{
	int
		i;

	draw_eo_3d_scene = FALSE;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		fn_draw_local_entity_3d_object[i]	= default_draw_entity_3d_object;
	}

	//
	// initialise maximum 3D object visual range
	//

	for (i = OBJECT_3D_INVALID_OBJECT_INDEX + 1; i < OBJECT_3D_LAST; i++)
	{
		//
		// if maximum_distance == 0.0 (no approximations) then set to nominal value
		//

		if (object_3d_information_database[i].maximum_distance == 0.0)
		{
			object_3d_information_database[i].maximum_distance = NOMINAL_3D_OBJECT_VISUAL_RANGE;
		}
	}
}





int get_local_entity_valid_3d_draw_function (entity *en)
{

	if (fn_draw_local_entity_3d_object[get_local_entity_type (en)] != default_draw_entity_3d_object)
	{

		return TRUE;
	}

	return FALSE;
}





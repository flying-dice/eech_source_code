

































































#include "project.h"

#include "ai/highlevl/highlevl.h"
#include "ai/taskgen/engage.h"





#define DEBUG_MODULE 0





static void aircraft_target_scan (entity *en);

static entity *get_aircraft_air_to_surface_scan_group_targets (entity *source, int *target_count);

static entity *get_aircraft_air_to_air_scan_group_targets (entity *source, int *target_count);





static int check_aircraft_target_scan_valid (entity *en)
{

	entity
		*group;

	//
	// Is entity allowed to scan for targets 
	//

	if (!get_local_entity_int_value (en, INT_TYPE_OFFENSIVE_CAPABILITY))
	{
		return FALSE;
	}

	if (get_local_entity_float_value (en, FLOAT_TYPE_TARGET_SCAN_DELAY) == 0.0)
	{
		return FALSE;
	}

	//
	// Can group engage (by default or overridden by force)
	//

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (!group)
	{
		return FALSE;
	}

	if (!get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
	{






		return FALSE;
	}

	//
	// Is entity landed or landing/takeing off ?
	//

	if (!get_local_entity_int_value (en, INT_TYPE_AIRBORNE_AIRCRAFT))
	{






		return FALSE;
	}

	return TRUE;
}





void update_aircraft_target_scan (entity *en)
{
	if (check_aircraft_target_scan_valid (en))
	{
		aircraft_target_scan (en);
	}
}





void aircraft_target_scan (entity *en)
{
	aircraft
		*raw;

	entity
		*task,
		*group,
		*target_group;

	int
		target_count;

	task_target_source_types
		target_source;

	raw = (aircraft *) get_local_entity_data (en);

	raw->target_scan_timer -= get_delta_time (); 
		
	if (raw->target_scan_timer > 0.0)
	{
		return;
	}

	//
	// reset target_scan_timer (okay to use a random number as this is a server only variable)
	//
		
	raw->target_scan_timer = aircraft_database[raw->mob.sub_type].target_scan_delay * ((frand1 () + 1.0) * 0.5);
		
	//
	// Only task leader of primary task to do targetting
	// (This will stop things landing/taking off from scanning, as well as group members that are already engaging)
	//

	if (!get_local_entity_int_value (en, INT_TYPE_TASK_LEADER))
	{
		return;
	}

	task = get_local_entity_current_task (en);

	if (!task)
	{
		return;
	}

	if (!get_local_entity_int_value (task, INT_TYPE_PRIMARY_TASK))
	{
		return;
	}

	//
	// If task leader human, don't do any scanning
	//

	if (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
	{
		return;
	}

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	//
	// What type of target does task allow 
	//

	target_source = (task_target_source_types) get_local_entity_int_value (task, INT_TYPE_TASK_TARGET_SOURCE);

	switch (target_source)
	{
		case TASK_TARGET_SOURCE_TASK_OBJECTIVE:
		{
			return; 
		}

		case TASK_TARGET_SOURCE_SCAN_ALL:
		case TASK_TARGET_SOURCE_SCAN_AIR:
		{
			//
			// Just notify force entity of targets
			//

			target_group = get_aircraft_air_to_air_scan_group_targets (en, &target_count);

			if ((target_group) && (target_count > 0))
			{
				ai_log ("AC_TGT: %s doing %s Scanned %d air targets", get_local_entity_string (en, STRING_TYPE_FULL_NAME), get_local_entity_string (task, STRING_TYPE_FULL_NAME), target_count);






	

				//
				// SPECIAL_EFFECT_HOOK FOR AIR TARGETS SIGHTED
				//



				//
				//


				engage_targets_in_group (group, target_group, TRUE);
			}

			if (target_source == TASK_TARGET_SOURCE_SCAN_AIR)
			{

				break;
			}
		}

		case TASK_TARGET_SOURCE_SCAN_GROUND:
		{
			//
			// Just notify force entity of targets
			//







			target_group = get_aircraft_air_to_surface_scan_group_targets (en, &target_count);

			if ((target_group) && (target_count > 0))
			{
	







				//
				// SPECIAL_EFFECT_HOOK FOR GROUND TARGETS SIGHTED
				//

	

				//
				//


				engage_targets_in_group (group, target_group, TRUE);
			}

			break;
		}
	}
}





entity *get_aircraft_air_to_surface_scan_group_targets (entity *source, int *target_count)
{
	int
		source_side,
		x_sec_org,
		z_sec_org,
		x_sec_min,
		z_sec_min,
		x_sec_max,
		z_sec_max,
		x_sec,
		z_sec;

	entity
		*best_group,
		*best_target,
		*target,
		*group,
		*sec;

	matrix3x3
		attitude;

	vec3d
		*source_position,
		*target_position,
		target_vector;

	float
		cos_angle,
		target_range,
		target_result,
		best_target_result,
		source_surface_scan_range;

	ASSERT (source);

	(*target_count) = 0;

	group = NULL;

	best_group = NULL;

	best_target = NULL;

	best_target_result = FLT_MAX;

	source_side = get_local_entity_int_value (source, INT_TYPE_SIDE);

	source_position = get_local_entity_vec3d_ptr (source, VEC3D_TYPE_POSITION);

	source_surface_scan_range = get_local_entity_float_value (source, FLOAT_TYPE_SURFACE_SCAN_RANGE);

	if (source_surface_scan_range == 0.0)
	{
		return NULL;
	}

	source_surface_scan_range *= source_surface_scan_range;


	//
	// get sector scan area
	//


	get_x_sector (x_sec_org, source_position->x);
	get_z_sector (z_sec_org, source_position->z);

	ASSERT (sector_inside_map_area (x_sec_org, z_sec_org));

	get_x_sector (x_sec_min, source_position->x - source_surface_scan_range);
	get_z_sector (z_sec_min, source_position->z - source_surface_scan_range);

	get_x_sector (x_sec_max, source_position->x + source_surface_scan_range);
	get_z_sector (z_sec_max, source_position->z + source_surface_scan_range);

	x_sec_min = max (x_sec_min, MIN_MAP_X_SECTOR);
	z_sec_min = max (z_sec_min, MIN_MAP_Z_SECTOR);

	x_sec_max = min (x_sec_max, MAX_MAP_X_SECTOR);
	z_sec_max = min (z_sec_max, MAX_MAP_Z_SECTOR);


	//
	// sector scan
	//


	for (z_sec = z_sec_min; z_sec <= z_sec_max; z_sec++)
	{
		for (x_sec = x_sec_min; x_sec <= x_sec_max; x_sec++)
		{
			sec = get_local_raw_sector_entity (x_sec, z_sec);

			target = get_local_entity_first_child (sec, LIST_TYPE_SECTOR);

			if (target)
			{
				do
				{

					//
					// simple target select criteria
					//


					if (get_local_entity_int_value (target, INT_TYPE_SIDE) == source_side)
					{
						continue;
					}

					//
					// Allow vehicles and aircraft....
					//
						
					if ((!get_local_entity_int_value (target, INT_TYPE_IDENTIFY_AIRCRAFT)) && 
							(!get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE)))
					{
						continue;
					}

					//
					// ... But only aircraft that are on the ground
					//

					if (get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT))
					{
						continue;
					}

					if (get_local_entity_int_value (target, INT_TYPE_TARGET_TYPE) == TARGET_TYPE_INVALID)
					{
						continue;
					}

					group = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if (!group)
					{
						continue;
					}

					if (group == best_group)
					{
						continue;
					}

					if (!get_local_entity_int_value (target, INT_TYPE_ALIVE))
					{
						continue;
					}

					if (target == source)
					{
						continue;
					}

					target_position = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

					target_range = get_sqr_2d_range (source_position, target_position);

					if (target_range > source_surface_scan_range)
					{
						continue;
					}

					(*target_count) ++;

					target_vector.x = target_position->x - source_position->x;
					target_vector.y = target_position->y - source_position->y;
					target_vector.z = target_position->z - source_position->z;

					get_local_entity_attitude_matrix (source, attitude);

					cos_angle = get_3d_vector_dot_product (&target_vector, (vec3d *) &attitude [2]);

					//
					//
					//

					target_result = target_range * (2.0 - (cos_angle + 1.0));

					//
					//
					//

					if (target_result < best_target_result)
					{
						best_target_result = target_result;

						best_target = target;

						best_group = group;
					}
				}
				while (target = get_local_entity_child_succ (target, LIST_TYPE_SECTOR));
			}
		}
	}

	return (best_group);
}





entity *get_aircraft_air_to_air_scan_group_targets (entity *source, int *target_count)
{
	int
		source_side,
		x_sec_org,
		z_sec_org,
		x_sec_min,
		z_sec_min,
		x_sec_max,
		z_sec_max,
		x_sec,
		z_sec;

	entity
		*best_group,
		*best_target,
		*target,
		*group,
		*sec;

	matrix3x3
		attitude;

	vec3d
		*source_position,
		*target_position,
		target_vector;

	float
		cos_angle,
		target_range,
		target_result,
		best_target_result,
		source_air_scan_range;

	ASSERT (source);

	(*target_count) = 0;

	best_group = NULL;

	group = NULL;

	best_target = NULL;

	best_target_result = FLT_MAX;

	source_side = get_local_entity_int_value (source, INT_TYPE_SIDE);

	source_position = get_local_entity_vec3d_ptr (source, VEC3D_TYPE_POSITION);

	source_air_scan_range = get_local_entity_float_value (source, FLOAT_TYPE_AIR_SCAN_RANGE);

	if (source_air_scan_range == 0.0)
	{
		return NULL;
	}

	source_air_scan_range *= source_air_scan_range;


	//
	// get sector scan area
	//


	get_x_sector (x_sec_org, source_position->x);
	get_z_sector (z_sec_org, source_position->z);

	ASSERT (sector_inside_map_area (x_sec_org, z_sec_org));

	get_x_sector (x_sec_min, source_position->x - source_air_scan_range);
	get_z_sector (z_sec_min, source_position->z - source_air_scan_range);

	get_x_sector (x_sec_max, source_position->x + source_air_scan_range);
	get_z_sector (z_sec_max, source_position->z + source_air_scan_range);

	x_sec_min = max (x_sec_min, MIN_MAP_X_SECTOR);
	z_sec_min = max (z_sec_min, MIN_MAP_Z_SECTOR);

	x_sec_max = min (x_sec_max, MAX_MAP_X_SECTOR);
	z_sec_max = min (z_sec_max, MAX_MAP_Z_SECTOR);


	//
	// sector scan
	//


	for (z_sec = z_sec_min; z_sec <= z_sec_max; z_sec++)
	{
		for (x_sec = x_sec_min; x_sec <= x_sec_max; x_sec++)
		{
			sec = get_local_raw_sector_entity (x_sec, z_sec);

			target = get_local_entity_first_child (sec, LIST_TYPE_SECTOR);

			if (target)
			{
				do
				{

					//
					// simple target select criteria
					//


					if (get_local_entity_int_value (target, INT_TYPE_SIDE) == source_side)
					{
						continue;
					}

					if (!get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT))
					{
						continue;
					}

					if (get_local_entity_int_value (target, INT_TYPE_TARGET_TYPE) == TARGET_TYPE_INVALID)
					{
						continue;
					}

					group = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if (!group)
					{
						continue;
					}

					if (group == best_group)
					{
						continue;
					}

					if (!get_local_entity_int_value (target, INT_TYPE_ALIVE))
					{
						continue;
					}

					if (target == source)
					{
						continue;
					}

					target_position = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

					target_range = get_sqr_2d_range (source_position, target_position);

					//
					// adjust range if target is player - each jammer / radar turned off allows player to fly 10% closer
					//
					{
						float
							adj_range;

						adj_range = target_range;

						if (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
						{
							if (!get_local_entity_int_value (target, INT_TYPE_RADAR_ON))
							{
								adj_range *= 1.1;
							}
							if (!get_local_entity_int_value (target, INT_TYPE_RADAR_JAMMER_ON))
							{
								adj_range *= 1.1;
							}
							if (!get_local_entity_int_value (target, INT_TYPE_INFRA_RED_JAMMER_ON))
							{
								adj_range *= 1.1;
							}
						}

						if (adj_range > (source_air_scan_range * get_local_entity_float_value (target, FLOAT_TYPE_RADAR_STEALTH_FACTOR)))
						{
							continue;
						}
					}

					(*target_count) ++;

					target_vector.x = target_position->x - source_position->x;
					target_vector.y = target_position->y - source_position->y;
					target_vector.z = target_position->z - source_position->z;

					get_local_entity_attitude_matrix (source, attitude);

					cos_angle = get_3d_vector_dot_product (&target_vector, (vec3d *) &attitude [2]);

					//
					//
					//

					target_result = target_range * (2.0 - (cos_angle + 1.0));

					//
					//
					//

					if (target_result < best_target_result)
					{
						best_target_result = target_result;

						best_target = target;

						best_group = group;
					}
				}
				while (target = get_local_entity_child_succ (target, LIST_TYPE_SECTOR));
			}
		}
	}

	return (best_group);
}





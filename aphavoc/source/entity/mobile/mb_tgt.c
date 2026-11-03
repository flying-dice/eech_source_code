

































































#include "project.h"




//
// ** These functions used by AI ONLY **
//




#define TEXT_DEBUG_MODULE 0

#define LINE_DEBUG_MODULE 0





#define LOS_COARSE_CHECK_DISTANCE					200.0f

#define LOS_FINE_CHECK_DISTANCE_SOURCE_END		40.0f
#define LOS_NUMBER_OF_FINE_CHECKS_SOURCE_END		8.0f

#define LOS_FINE_CHECK_DISTANCE_TARGET_END		24.0f
#define LOS_NUMBER_OF_FINE_CHECKS_TARGET_END		8.0f

#define LOS_OBJECT_CHECK_DISTANCE_SOURCE_END		200.0f

#define LOS_OBJECT_CHECK_DISTANCE_TARGET_END		200.0f

//
// Ignore any obstacles within a certain radius of the target
//

#define LOS_OBJECT_EXCLUDE_DISTANCE_TARGET_END	12.0f





// raise vehicle's base to avoid targeting problems

int check_entity_line_of_sight (entity *source, entity *target, mobile_los_check_criteria criteria)
{
	vec3d
		source_position,
		target_position;

	ASSERT (source);

	ASSERT (target);

	get_local_entity_vec3d (source, VEC3D_TYPE_POSITION, &source_position);
	
	if (get_local_entity_int_value (source, INT_TYPE_IDENTIFY_VEHICLE))
		source_position.y += 1.5;

	get_local_entity_vec3d (target, VEC3D_TYPE_POSITION, &target_position);

	if (get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE))
		target_position.y += 1.5;

	return check_position_line_of_sight (source, target, &source_position, &target_position, criteria);
}





int check_position_line_of_sight (entity *source, entity *target, vec3d *source_position, vec3d *target_position, mobile_los_check_criteria criteria)
{

	entity
		*collision_en;

	vec3d
		increment,
		collision_point,
		normal,
		#if LINE_DEBUG_MODULE
		old_position,
		#endif
		check_position,
		direction;

	float
		target_range,
		collision_distance,
		number_of_terrain_checks,
		terrain_elevation;

	terrain_3d_point_data
		terrain_info;

	ASSERT (source);

	ASSERT (target);

	ASSERT (source_position);

	ASSERT (target_position);

	direction.x = target_position->x - source_position->x;
	direction.y = target_position->y - source_position->y;
	direction.z = target_position->z - source_position->z;

	target_range = sqrt ((direction.x * direction.x) + (direction.z * direction.z));

	normalise_3d_vector (&direction);
	

	// COARSE line of sight check with terrain


	if (criteria &	MOBILE_LOS_CHECK_COURSE_TERRAIN)
	{
		number_of_terrain_checks = target_range / LOS_COARSE_CHECK_DISTANCE;
	
		increment.x = direction.x * LOS_COARSE_CHECK_DISTANCE;
		increment.y = direction.y * LOS_COARSE_CHECK_DISTANCE;
		increment.z = direction.z * LOS_COARSE_CHECK_DISTANCE;
	
		check_position = *source_position;
	
		memset (&terrain_info, 0, sizeof (terrain_3d_point_data));
	
		while (number_of_terrain_checks > 1.0)
		{
			#if LINE_DEBUG_MODULE
	
			old_position = check_position;
	
			#endif
	
			check_position.x += increment.x;
			check_position.y += increment.y;
			check_position.z += increment.z;
	
			ASSERT(point_inside_map_area(&check_position));

			get_3d_terrain_point_data (check_position.x, check_position.z, &terrain_info);
	
			terrain_elevation = get_3d_terrain_point_data_elevation (&terrain_info);
	
			if (terrain_elevation > check_position.y)
			{
	
				#if TEXT_DEBUG_MODULE
	
				debug_log ("MB_TGT: (%s -> %s) failed COURSE terrain LOS", get_local_entity_string (source, STRING_TYPE_FULL_NAME), get_local_entity_string (target, STRING_TYPE_FULL_NAME));
	
				#endif
	
				return FALSE;
			}
	
			#if LINE_DEBUG_MODULE
	
			create_debug_3d_line (&old_position, &check_position, sys_col_yellow, 10.0);
	
			#endif
	
			number_of_terrain_checks --;
		}
	}
	

	// SOURCE END FINE line of sight check with terrain


	if (criteria &	MOBILE_LOS_CHECK_SOURCE_END_TERRAIN)
	{
		number_of_terrain_checks = target_range / LOS_FINE_CHECK_DISTANCE_SOURCE_END;
	
		number_of_terrain_checks = min (number_of_terrain_checks, LOS_NUMBER_OF_FINE_CHECKS_SOURCE_END);
	
		increment.x = direction.x * LOS_FINE_CHECK_DISTANCE_SOURCE_END;
		increment.y = direction.y * LOS_FINE_CHECK_DISTANCE_SOURCE_END;
		increment.z = direction.z * LOS_FINE_CHECK_DISTANCE_SOURCE_END;
	
		check_position = *source_position;
	
		while (number_of_terrain_checks > 1.0)
		{
			#if LINE_DEBUG_MODULE
	
			old_position = check_position;
	
			#endif
	
			check_position.x += increment.x;
			check_position.y += increment.y;
			check_position.z += increment.z;
	
			ASSERT(point_inside_map_area(&check_position));

			get_3d_terrain_point_data (check_position.x, check_position.z, &terrain_info);
	
			terrain_elevation = get_3d_terrain_point_data_elevation (&terrain_info);
	
			if (terrain_elevation > check_position.y)
			{
	
				#if TEXT_DEBUG_MODULE
	
				debug_log ("MB_TGT: (%s -> %s) failed FINE terrain LOS (source end)", get_local_entity_string (source, STRING_TYPE_FULL_NAME), get_local_entity_string (target, STRING_TYPE_FULL_NAME));
	
				#endif
	
				return FALSE;
			}
	
			#if LINE_DEBUG_MODULE
	
			create_debug_3d_line (&old_position, &check_position, sys_col_yellow, 10.0);
	
			#endif
	
			number_of_terrain_checks --;
		}
	}
	

	// TARGET END FINE line of sight check with terrain


	if (criteria &	MOBILE_LOS_CHECK_TARGET_END_TERRAIN)
	{
		number_of_terrain_checks = target_range / LOS_FINE_CHECK_DISTANCE_TARGET_END;
	
		number_of_terrain_checks = min (number_of_terrain_checks, LOS_NUMBER_OF_FINE_CHECKS_TARGET_END);
	
		increment.x = direction.x * LOS_FINE_CHECK_DISTANCE_TARGET_END;
		increment.y = direction.y * LOS_FINE_CHECK_DISTANCE_TARGET_END;
		increment.z = direction.z * LOS_FINE_CHECK_DISTANCE_TARGET_END;
	
		check_position = *target_position;
	
		while (number_of_terrain_checks > 1.0)
		{
			#if LINE_DEBUG_MODULE
	
			old_position = check_position;
	
			#endif
	
			check_position.x -= increment.x;
			check_position.y -= increment.y;
			check_position.z -= increment.z;
	
			ASSERT(point_inside_map_area(&check_position));

			get_3d_terrain_point_data (check_position.x, check_position.z, &terrain_info);
	
			terrain_elevation = get_3d_terrain_point_data_elevation (&terrain_info);
	
			if (terrain_elevation > check_position.y)
			{
	
				#if TEXT_DEBUG_MODULE
	
				debug_log ("MB_TGT: (%s -> %s) failed FINE terrain LOS (target end)", get_local_entity_string (source, STRING_TYPE_FULL_NAME), get_local_entity_string (target, STRING_TYPE_FULL_NAME));
	
				#endif
	
				return FALSE;
			}
	
			#if LINE_DEBUG_MODULE
	
			create_debug_3d_line (&old_position, &check_position, sys_col_yellow, 10.0);
	
			#endif
	
			number_of_terrain_checks --;
		}
	}


	// SOURCE END line of sight check with objects


	if (criteria &	MOBILE_LOS_CHECK_SOURCE_END_OBJECTS)
	{
		check_position.x = source_position->x + (direction.x * min(target_range, LOS_OBJECT_CHECK_DISTANCE_SOURCE_END));
		check_position.y = source_position->y + (direction.y * min(target_range, LOS_OBJECT_CHECK_DISTANCE_SOURCE_END));
		check_position.z = source_position->z + (direction.z * min(target_range, LOS_OBJECT_CHECK_DISTANCE_SOURCE_END));

		collision_en = get_line_of_sight_collision_entity
							(
								source,
								target,
								source_position,
								&check_position,
								&collision_point,
								&normal
							);
	
		if (collision_en)
		{
	
			#if TEXT_DEBUG_MODULE

			debug_log ("MB_TGT: (%s -> %s) failed OBJECT LOS with %s (source end)", get_local_entity_string (source, STRING_TYPE_FULL_NAME), get_local_entity_string (target, STRING_TYPE_FULL_NAME), get_local_entity_type_name (collision_en));
			
			#endif
	
			return FALSE;
		}
	}
	

	// TARGET END line of sight check with objects


	if (criteria &	MOBILE_LOS_CHECK_TARGET_END_OBJECTS)
	{	
		check_position.x = target_position->x - (direction.x * min(target_range, LOS_OBJECT_CHECK_DISTANCE_TARGET_END));
		check_position.y = target_position->y - (direction.y * min(target_range, LOS_OBJECT_CHECK_DISTANCE_TARGET_END));
		check_position.z = target_position->z - (direction.z * min(target_range, LOS_OBJECT_CHECK_DISTANCE_TARGET_END));

		collision_en = get_line_of_sight_collision_entity
							(
								source,
								target,
								target_position,
								&check_position,
								&collision_point,
								&normal
							);
	
		if (collision_en)
		{
			collision_distance = get_sqr_3d_range (&collision_point, target_position);

			if (collision_distance > (LOS_OBJECT_EXCLUDE_DISTANCE_TARGET_END * LOS_OBJECT_EXCLUDE_DISTANCE_TARGET_END))
			{
				#if TEXT_DEBUG_MODULE
			
				debug_log ("MB_TGT: (%s -> %s) failed OBJECT LOS with %s (target end)", get_local_entity_string (source, STRING_TYPE_FULL_NAME), get_local_entity_string (target, STRING_TYPE_FULL_NAME), get_local_entity_type_name (collision_en));
			
				#endif
	
				return FALSE;
			}
		}
	}

	return TRUE;
}






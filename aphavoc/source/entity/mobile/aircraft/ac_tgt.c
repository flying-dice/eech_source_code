// 
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
// 
// 	 Please see the document LICENSE.TXT for the full licence agreement
// 
// 2. LICENCE
//  2.1 	
//  	Subject to the provisions of this Agreement we now grant to you the 
//  	following rights in respect of the Source Code:
//   2.1.1 
//   	the non-exclusive right to Exploit  the Source Code and Executable 
//   	Code on any medium; and 
//   2.1.2 
//   	the non-exclusive right to create and distribute Derivative Works.
//  2.2 	
//  	Subject to the provisions of this Agreement we now grant you the
// 	following rights in respect of the Object Code:
//   2.2.1 
// 	the non-exclusive right to Exploit the Object Code on the same
// 	terms and conditions set out in clause 3, provided that any
// 	distribution is done so on the terms of this Agreement and is
// 	accompanied by the Source Code and Executable Code (as
// 	applicable).
// 
// 3. GENERAL OBLIGATIONS
//  3.1 
//  	In consideration of the licence granted in clause 2.1 you now agree:
//   3.1.1 
// 	that when you distribute the Source Code or Executable Code or
// 	any Derivative Works to Recipients you will also include the
// 	terms of this Agreement;
//   3.1.2 
// 	that when you make the Source Code, Executable Code or any
// 	Derivative Works ("Materials") available to download, you will
// 	ensure that Recipients must accept the terms of this Agreement
// 	before being allowed to download such Materials;
//   3.1.3 
// 	that by Exploiting the Source Code or Executable Code you may
// 	not impose any further restrictions on a Recipient's subsequent
// 	Exploitation of the Source Code or Executable Code other than
// 	those contained in the terms and conditions of this Agreement;
//   3.1.4 
// 	not (and not to allow any third party) to profit or make any
// 	charge for the Source Code, or Executable Code, any
// 	Exploitation of the Source Code or Executable Code, or for any
// 	Derivative Works;
//   3.1.5 
// 	not to place any restrictions on the operability of the Source 
// 	Code;
//   3.1.6 
// 	to attach prominent notices to any Derivative Works stating
// 	that you have changed the Source Code or Executable Code and to
// 	include the details anddate of such change; and
//   3.1.7 
//   	not to Exploit the Source Code or Executable Code otherwise than
// 	as expressly permitted by  this Agreement.
// 



/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#include "project.h"

#include "ai/highlevl/highlevl.h"
#include "ai/taskgen/engage.h"
#include "entity/mobile/mb_sensorfx.h"
#include "entity/mobile/mb_hostsense.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static void aircraft_target_scan (entity *en);

static entity *get_aircraft_air_to_surface_scan_group_targets (entity *source, int *target_count);

static entity *get_aircraft_air_to_air_scan_group_targets (entity *source, int *target_count);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
		#if DEBUG_MODULE

		debug_log ("AC_TGT: group %s not allowed to engage enemy", get_local_entity_string_value (group, STRING_TYPE_FULL_NAME));

		#endif

		return FALSE;
	}

	//
	// Is entity landed or landing/takeing off ?
	//

	if (!get_local_entity_int_value (en, INT_TYPE_AIRBORNE_AIRCRAFT))
	{
		#if DEBUG_MODULE

		debug_log ("AC_TGT: entity %s not allowed to engage enemy, its not airborne", get_local_entity_string (en, STRING_TYPE_FULL_NAME));

		#endif

		return FALSE;
	}

	return TRUE;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void update_aircraft_target_scan (entity *en)
{
	if (check_aircraft_target_scan_valid (en))
	{
		aircraft_target_scan (en);
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

				#if DEBUG_MODULE
	
				debug_log ("AC_TGT: %s(%d) Scanned %d air group targets", entity_type_names [en->type], en, target_count);
	
				#endif
	
				/////////////////////////////////////////////////////////////////
				//
				// SPECIAL_EFFECT_HOOK FOR AIR TARGETS SIGHTED
				//
				/////////////////////////////////////////////////////////////////

				/////////////////////////////////////////////////////////////////
				//
				//
				/////////////////////////////////////////////////////////////////

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

			#if DEBUG_MODULE

			debug_log ("AC_TGT: %s(%d) Scanning ground", entity_type_names [en->type], en);

			#endif

			target_group = get_aircraft_air_to_surface_scan_group_targets (en, &target_count);

			if ((target_group) && (target_count > 0))
			{
	
				#if DEBUG_MODULE
	
				debug_log ("AC_TGT: %s(%d) Scanned %d ground group targets %s", entity_type_names [en->type], en, target_count, entity_sub_type_group_names [get_local_entity_int_value (tgt, INT_TYPE_ENTITY_SUB_TYPE)]);
	
				#endif

				/////////////////////////////////////////////////////////////////
				//
				// SPECIAL_EFFECT_HOOK FOR GROUND TARGETS SIGHTED
				//
				/////////////////////////////////////////////////////////////////
	
				/////////////////////////////////////////////////////////////////
				//
				//
				/////////////////////////////////////////////////////////////////

				engage_targets_in_group (group, target_group, TRUE);
			}

			break;
		}
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

	SENSOR_FX_BEGIN (EECH_SENSOR_AIR_TO_SURFACE, source)

	if (source_surface_scan_range == 0.0)
	{
		SENSOR_FX_END (0, NULL, 0.0)

		return NULL;
	}

	SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_X, source_position->x)
	SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_Y, source_position->y)
	SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_Z, source_position->z)
	SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_RANGE, source_surface_scan_range)

	//
	// the host owns what this sensor perceives (range test, EEDCS-43); the raw scan range is its fact
	//

	campaign_sense_begin (CAMPAIGN_SENSE_AIR_TO_SURFACE, source, source_position, source_surface_scan_range, 0.0, 0.0);

	//
	// the candidate enumeration below squares the range before the sector bounds (a long-standing campaign quirk, kept as it was)
	//

	source_surface_scan_range *= source_surface_scan_range;

	SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_RANGE_SQUARED, source_surface_scan_range)

	////////////////////////////////////////
	//
	// get sector scan area
	//
	////////////////////////////////////////

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

	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_X_MIN, x_sec_min)
	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_Z_MIN, z_sec_min)
	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_X_MAX, x_sec_max)
	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_Z_MAX, z_sec_max)

	////////////////////////////////////////
	//
	// sector scan
	//
	////////////////////////////////////////

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
					////////////////////////////////////////
					//
					// simple target select criteria
					//
					////////////////////////////////////////

					if (get_local_entity_int_value (target, INT_TYPE_SIDE) == source_side)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_SIDE)

						continue;
					}

					//
					// Allow vehicles and aircraft....
					//

					if ((!get_local_entity_int_value (target, INT_TYPE_IDENTIFY_AIRCRAFT)) &&
							(!get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE)))
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_IDENTIFY)

						continue;
					}

					//
					// ... But only aircraft that are on the ground
					//

					if (get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT))
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_AIRBORNE)

						continue;
					}

					if (get_local_entity_int_value (target, INT_TYPE_TARGET_TYPE) == TARGET_TYPE_INVALID)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_TARGET_TYPE)

						continue;
					}

					group = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if (!group)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_NO_GROUP)

						continue;
					}

					if (group == best_group)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_BEST_GROUP)

						continue;
					}

					if (!get_local_entity_int_value (target, INT_TYPE_ALIVE))
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_DEAD)

						continue;
					}

					if (target == source)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_SELF)

						continue;
					}

					target_position = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

					if (!campaign_sense_candidate (source, target, target_position, &target_range))
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

					SENSOR_FX_DEC_F (EECH_SENSOR_TAG_COS_ANGLE, cos_angle)
					SENSOR_FX_DEC_F (EECH_SENSOR_TAG_RESULT, target_result)
					SENSOR_FX_DEC_I (EECH_SENSOR_TAG_REPLACED, target_result < best_target_result)

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

	campaign_sense_end ();

	SENSOR_FX_END (*target_count, best_group, best_target_result)

	return (best_group);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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

	SENSOR_FX_BEGIN (EECH_SENSOR_AIR_TO_AIR, source)

	if (source_air_scan_range == 0.0)
	{
		SENSOR_FX_END (0, NULL, 0.0)

		return NULL;
	}

	SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_X, source_position->x)
	SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_Y, source_position->y)
	SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_Z, source_position->z)
	SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_RANGE, source_air_scan_range)

	//
	// the host owns what this sensor perceives (squared range, emitter and stealth adjustment, EEDCS-43); the raw scan range is its fact
	//

	campaign_sense_begin (CAMPAIGN_SENSE_AIR_TO_AIR, source, source_position, source_air_scan_range, 0.0, 0.0);

	//
	// the candidate enumeration below squares the range before the sector bounds (a long-standing campaign quirk, kept as it was, including
	// the empty rectangle a range above about 46 km gives)
	//

	source_air_scan_range *= source_air_scan_range;

	SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_RANGE_SQUARED, source_air_scan_range)

	////////////////////////////////////////
	//
	// get sector scan area
	//
	////////////////////////////////////////

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

	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_X_MIN, x_sec_min)
	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_Z_MIN, z_sec_min)
	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_X_MAX, x_sec_max)
	SENSOR_FX_I (EECH_SENSOR_TAG_SECTOR_Z_MAX, z_sec_max)

	////////////////////////////////////////
	//
	// sector scan
	//
	////////////////////////////////////////

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
					////////////////////////////////////////
					//
					// simple target select criteria
					//
					////////////////////////////////////////

					if (get_local_entity_int_value (target, INT_TYPE_SIDE) == source_side)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_SIDE)

						continue;
					}

					if (!get_local_entity_int_value (target, INT_TYPE_AIRBORNE_AIRCRAFT))
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_AIRBORNE)

						continue;
					}

					if (get_local_entity_int_value (target, INT_TYPE_TARGET_TYPE) == TARGET_TYPE_INVALID)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_TARGET_TYPE)

						continue;
					}

					group = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if (!group)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_NO_GROUP)

						continue;
					}

					if (group == best_group)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_BEST_GROUP)

						continue;
					}

					if (!get_local_entity_int_value (target, INT_TYPE_ALIVE))
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_DEAD)

						continue;
					}

					if (target == source)
					{
						SENSOR_FX_SKIP (EECH_SENSOR_SKIP_SELF)

						continue;
					}

					target_position = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

					if (!campaign_sense_candidate (source, target, target_position, &target_range))
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

					SENSOR_FX_DEC_F (EECH_SENSOR_TAG_COS_ANGLE, cos_angle)
					SENSOR_FX_DEC_F (EECH_SENSOR_TAG_RESULT, target_result)
					SENSOR_FX_DEC_I (EECH_SENSOR_TAG_REPLACED, target_result < best_target_result)

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

	campaign_sense_end ();

	SENSOR_FX_END (*target_count, best_group, best_target_result)

	return (best_group);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_SENSOR_FIXTURE

//
// TEST-ONLY (EEDCS-43): the two group scans above are file-local; the sensor fixture's read-only probe reaches them here.
//

entity *eech_sensor_probe_aircraft_scan (entity *source, int air_to_air, int *target_count)
{
	if (air_to_air)
	{
		return get_aircraft_air_to_air_scan_group_targets (source, target_count);
	}

	return get_aircraft_air_to_surface_scan_group_targets (source, target_count);
}

#endif

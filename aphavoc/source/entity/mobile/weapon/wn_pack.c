

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			weapon
				*raw;

			raw = (weapon *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			pack_mobile_data (en, &raw->mob, mode);

			pack_float_value (en, FLOAT_TYPE_VERY_HIGH_VELOCITY, raw->mob.velocity);

			pack_float_value (en, FLOAT_TYPE_WEAPON_LIFETIME, raw->weapon_lifetime);

			if (mode == PACK_MODE_SERVER_SESSION)
			{
				pack_float_value (en, FLOAT_TYPE_DECOY_TIMER, raw->decoy_timer);
			}

			// burst_size

			// kill_code

			pack_int_value (en, INT_TYPE_WEAPON_ON_GROUND, raw->weapon_on_ground);

			// parachute_status

			pack_list_link (en, LIST_TYPE_LAUNCHED_WEAPON, &raw->launched_weapon_link);

			// view_weapon_link

			pack_int_value (en, INT_TYPE_LOCK_ON_AFTER_LAUNCH, raw->loal_mode);
			pack_int_value (en, INT_TYPE_WEAPON_MISSILE_PHASE, raw->missile_phase);

			if (mode == PACK_MODE_SERVER_SESSION)
			{

				if (raw->task_dependent_root.first_child)
				{

					pack_int_value (en, INT_TYPE_VALID, TRUE);

					pack_list_root (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_root);
				}
				else
				{

					pack_int_value (en, INT_TYPE_VALID, FALSE);
				}
			}

			#if DEBUG_MODULE

			debug_log ("WN_PACK: Packed %s (%d)", weapon_database [raw->mob.sub_type].full_name, get_local_entity_safe_index (en));

			#endif

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			break;
		}
	}
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			int
				index;

			weapon
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (weapon *) malloc_fast_mem (sizeof (weapon));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (weapon));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			unpack_mobile_data (en, &raw->mob, mode);

			//
			// safeguard position from truncation errors
			//

			raw->mob.position.x = bound (raw->mob.position.x, MIN_MAP_X, MAX_MAP_X);

			raw->mob.position.z = bound (raw->mob.position.z, MIN_MAP_Z, MAX_MAP_Z);

			ASSERT(point_inside_map_area(&raw->mob.position));
	
			raw->mob.position.y = max (raw->mob.position.y, get_3d_terrain_elevation (raw->mob.position.x, raw->mob.position.z) + 0.10f);

			//
			//
			//

			raw->mob.velocity = unpack_float_value (en, FLOAT_TYPE_VERY_HIGH_VELOCITY);

			raw->weapon_lifetime = unpack_float_value (en, FLOAT_TYPE_WEAPON_LIFETIME);

			if (mode == PACK_MODE_SERVER_SESSION)
			{
				raw->decoy_timer = unpack_float_value (en, FLOAT_TYPE_DECOY_TIMER);
			}

			// burst_size

			// kill_code

			raw->weapon_on_ground = unpack_int_value (en, INT_TYPE_WEAPON_ON_GROUND);

			// parachute_status

			unpack_list_link (en, LIST_TYPE_LAUNCHED_WEAPON, &raw->launched_weapon_link);

			// view_weapon_link

			raw->loal_mode = unpack_int_value (en, INT_TYPE_LOCK_ON_AFTER_LAUNCH);
			raw->missile_phase = unpack_int_value (en, INT_TYPE_WEAPON_MISSILE_PHASE);

			if (mode == PACK_MODE_SERVER_SESSION)
			{
				if (unpack_int_value (en, INT_TYPE_VALID))
				{

					unpack_list_root (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_root);
				}
			}

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->mob.position), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

			#if DEBUG_MODULE

			debug_log ("WN_PACK: Unpacked %s (%d)", weapon_database [raw->mob.sub_type].full_name, get_local_entity_safe_index (en));

			#endif

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			//
			// always use access functions to set the data
			//

			break;
		}
	}
}





void overload_weapon_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_WEAPON]	= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_WEAPON]	= unpack_local_data;
}





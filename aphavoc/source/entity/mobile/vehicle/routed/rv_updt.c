

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{

	routed_vehicle
		*raw;

	int
		loop,
		damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL),
		explosive_quality = get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_QUALITY);

	raw = (routed_vehicle *) get_local_entity_data (en);

	update_local_entity_view_interest_level (en);

	raw->vh.sleep -= get_delta_time ();

	raw->vh.sleep = max (raw->vh.sleep, 0.0f);

	// fire damage
	
	if (damage_level && (explosive_quality & EXPLOSIVE_QUALITY_EXPLOSIVE || explosive_quality & EXPLOSIVE_QUALITY_FLAMMABLE) && 
			vehicle_critically_damaged(en) && frand1() < 2 * get_delta_time())
	{
		damage_level--;

		set_client_server_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, max(0, damage_level));

		if (damage_level <= 0)
			kill_client_server_entity (en);
	}

	#if DEBUG_MODULE
	// debug
	{

		entity
			*group;

		group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

		if (group)
		{

			if (group_database [get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].frontline_flag)
			{

				if (get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP))
				{

					if (get_local_entity_type (get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP)) == ENTITY_TYPE_KEYSITE)
					{

						if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) != OPERATIONAL_STATE_LANDED)
						{

							entity
								*task;

							task = get_local_entity_current_task (en);

							if (task)
							{

								if ((get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_LANDING) &&
									(get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_TAKEOFF) &&
									(get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING))
								{

									debug_fatal ("GP_UPDAT: frontline entity %s, %d connected to KEYSITE", get_local_entity_string (en, STRING_TYPE_FULL_NAME), get_local_entity_index (en));
								}
							}
						}
					}
				}
			}
		}

	}
	#endif
	// debug

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{

		//
		// Update
		//

		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			routed_vehicle_movement (en);
		}



		update_vehicle_loading_doors (en);

		update_vehicle_radar (en);


		//
		// ORDER IS CRITICAL
		//

		update_vehicle_target_scan (en);

		update_vehicle_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		update_vehicle_weapon_fire (en);

		update_vehicle_decoy_release (en);

		rearm_vehicle_weapons(en);
		
		//


		if (tacview_is_logging() && raw->vh.mob.alive)
		{
			int
				moved = get_local_entity_int_value(en, INT_TYPE_MOBILE_MOVING),
				rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);

			if (moved || rotated || command_line_tacview_logging < 3)
				write_tacview_unit_update(en, moved, rotated, FALSE);
		}
	}
	else
	{
		if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
		{
			raw->vh.death_timer += get_delta_time ();

			if (!((int)(raw->vh.death_timer + 1) % 300))
				if (raw->vh.death_timer >= calculate_mobile_death_timer_value (en))
				{
					destroy_client_server_entity_family (en);
				}
		}
		else
		{
			for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
			{
				ASSERT(point_inside_map_area(&raw->vh.mob.position));

				get_3d_terrain_point_data (raw->vh.mob.position.x, raw->vh.mob.position.z, &raw->vh.terrain_info);

				routed_vehicle_death_movement (en);
			}
		}
	}
}





static void update_client (entity *en)
{
	routed_vehicle
		*raw;

	int
		loop;

	raw = (routed_vehicle *) get_local_entity_data (en);

	raw->vh.sleep -= get_delta_time ();

	raw->vh.sleep = max (raw->vh.sleep, 0.0f);

	update_local_entity_view_interest_level (en);

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			routed_vehicle_movement (en);
		}



		update_vehicle_loading_doors (en);

		update_vehicle_radar (en);


		//
		// ORDER IS CRITICAL
		//

		update_vehicle_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		rearm_vehicle_weapons(en);
		
		//

		
		if (tacview_is_logging() && raw->vh.mob.alive)
		{
			int
				moved = get_local_entity_int_value(en, INT_TYPE_MOBILE_MOVING),
				rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);

			if (moved || rotated || command_line_tacview_logging < 3)
				write_tacview_unit_update(en, moved, rotated, FALSE);
		}
	}
	else
	{
		if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
		{
			raw->vh.death_timer += get_delta_time ();
		}
		else
		{
			for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
			{
				ASSERT(point_inside_map_area(&raw->vh.mob.position));

				get_3d_terrain_point_data (raw->vh.mob.position.x, raw->vh.mob.position.z, &raw->vh.terrain_info);

				routed_vehicle_death_movement (en);
			}
		}
	}
}





void overload_routed_vehicle_update_functions (void)
{
	fn_update_client_server_entity[ENTITY_TYPE_ROUTED_VEHICLE][COMMS_MODEL_SERVER]	= update_server;

	fn_update_client_server_entity[ENTITY_TYPE_ROUTED_VEHICLE][COMMS_MODEL_CLIENT]	= update_client;
}





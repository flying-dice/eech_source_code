

































































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



	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			routed_vehicle_movement (en);
		}








		//
		// ORDER IS CRITICAL
		//

		update_vehicle_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		rearm_vehicle_weapons(en);
		
		//

		









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
















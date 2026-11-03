

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	ship_vehicle
		*raw;

	int	
		loop,
		damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL),
		explosive_quality = get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_QUALITY);

	raw = (ship_vehicle *) get_local_entity_data (en);

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

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			ship_vehicle_movement (en);
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

		rearm_vehicle_weapons(en);
		
		update_vehicle_missile_defence(en);
				
		if (tacview_is_logging() && command_line_tacview_logging < 3)
		{
			// for modes < 3 we have to check if it's visible now and add or remove it accordingly
			// ships never move, so no need to update for that
			write_tacview_unit_update(en, FALSE, FALSE, FALSE);
		}

		//

	}
	else if (!raw->vh.mob.alive) // do not sink ship until it burn out!
	{
		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			ship_vehicle_death_movement (en);
		}
	}
}





static void update_client (entity *en)
{
	ship_vehicle
		*raw;

	int
		loop;

	raw = (ship_vehicle *) get_local_entity_data (en);

	update_local_entity_view_interest_level (en);

	raw->vh.sleep -= get_delta_time ();

	raw->vh.sleep = max (raw->vh.sleep, 0.0f);

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			ship_vehicle_movement (en);
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

		
		if (tacview_is_logging() && command_line_tacview_logging < 3)
		{
			// for modes < 3 we have to check if it's visible now and add or remove it accordingly
			// ships never move, so no need to update for that
			write_tacview_unit_update(en, FALSE, FALSE, FALSE);
		}
	}
	else if (!raw->vh.mob.alive) // do not sink ship until it burn out!
	{
		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			ship_vehicle_death_movement (en);
		}
	}
}
















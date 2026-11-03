

































































#include "project.h"







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






















































































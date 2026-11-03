

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{

	anti_aircraft
		*raw;
	int
		damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL),
		explosive_quality = get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_QUALITY);

	raw = (anti_aircraft *) get_local_entity_data (en);



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
	else if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
	{
		raw->vh.death_timer += get_delta_time ();
	}
}





static void update_client (entity *en)
{
	anti_aircraft
		*raw;

	raw = (anti_aircraft *) get_local_entity_data (en);



	raw->vh.sleep -= get_delta_time ();

	raw->vh.sleep = max (raw->vh.sleep, 0.0f);

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{








		//
		// ORDER IS CRITICAL
		//

		update_vehicle_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		rearm_vehicle_weapons(en);
		
		//














	}
	else if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
	{
		raw->vh.death_timer += get_delta_time ();
	}
}
















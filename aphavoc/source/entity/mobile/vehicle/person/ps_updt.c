

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	person
		*raw;

	int
		loop,
		damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL);

	raw = (person *) get_local_entity_data (en);

	update_local_entity_view_interest_level (en);

	raw->vh.sleep -= get_delta_time ();

	raw->vh.sleep = max (raw->vh.sleep, 0.0f);

	// bleeding damage
	
	if (damage_level &&  vehicle_critically_damaged(en) && frand1() < 2 * get_delta_time())
	{
		damage_level--;

		set_client_server_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, max(0, damage_level));

		if (damage_level <= 0)
			kill_client_server_entity (en);
	}

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{
		//
		// Update
		//

		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			person_movement (en);
		}



		update_person_animation (en);


		//
		// ORDER IS CRITICAL
		//

		update_vehicle_target_scan (en);

		update_vehicle_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		update_vehicle_weapon_fire (en);

		update_vehicle_decoy_release (en);

		if (get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON) != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			update_infantry_weapons(en, get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON));

		rearm_vehicle_weapons(en);
		
		//

	}
	else
	{
		update_person_animation (en);

		if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
		{
			raw->vh.death_timer += get_delta_time ();

			if (!((int)(raw->vh.death_timer + 1) % 300))
				if (raw->vh.death_timer >= calculate_mobile_death_timer_value (en))
				{
					destroy_client_server_entity_family (en);
				}
		}
	}
}





static void update_client (entity *en)
{
	int
		loop;

	person
		*raw;

	raw = (person *) get_local_entity_data (en);

	update_local_entity_view_interest_level (en);

	raw->vh.sleep -= get_delta_time ();

	raw->vh.sleep = max (raw->vh.sleep, 0.0f);

	if (raw->vh.mob.alive && !vehicle_critically_damaged(en))
	{

		//
		// Update
		//

		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			person_movement (en);
		}



		update_person_animation (en);




		//
		// ORDER IS CRITICAL
		//

		update_vehicle_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		if (get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON) != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			update_infantry_weapons(en, get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON));

		rearm_vehicle_weapons(en);
		
		//

	}
	else
	{
		update_person_animation (en);

		if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
		{
			raw->vh.death_timer += get_delta_time ();
		}
	}
}
















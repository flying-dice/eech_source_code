

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





static void update_propellors (entity *en)
{

	fixed_wing
		*raw;

	entity
		*group;

	raw = (fixed_wing *) get_local_entity_data (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	ASSERT (group);

	if (!in_flight_articulation_test)
	{
		if (get_local_entity_int_value (en, INT_TYPE_LANDED))
		{
			raw->propellor_rpm -= 5.0 * get_delta_time ();
		}
		else
		{
			raw->propellor_rpm += 5.0 * get_delta_time ();
		}
	}

	raw->propellor_rpm = bound (raw->propellor_rpm, 0.0, 100.0);
}





static void update_server (entity *en)
{
	fixed_wing
		*raw;

	int
		loop;

	aircraft_damage_types
		damage_type;

	raw = (fixed_wing *) get_local_entity_data (en);

	update_local_entity_view_interest_level (en);

	damage_type = aircraft_critically_damaged (en);

	if (raw->ac.mob.alive)
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			if (!get_local_entity_int_value (en, INT_TYPE_LANDED))
			{

				ASSERT(point_inside_map_area(&raw->ac.mob.position));

				get_3d_terrain_point_data (raw->ac.mob.position.x, raw->ac.mob.position.z, &raw->ac.terrain_info);
			}

			basic_fixed_wing_movement (en);
		}



		update_propellors (en);

		update_fixed_wing_afterburners (en);

		update_fixed_wing_airbrakes (en);

		update_fixed_wing_flaps (en);

		update_aircraft_loading_doors (en);

		update_aircraft_cargo_doors (en);

		update_aircraft_undercarriage (en);

		update_aircraft_rudder (en);


		//
		// ORDER IS CRITICAL
		//

		update_aircraft_target_scan (en);

		update_aircraft_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		update_aircraft_weapon_fire (en);

		update_aircraft_decoy_release (en);

		//


		update_fixed_wing_crate_dropping (en);

		if (tacview_is_logging() && raw->ac.mob.alive)
		{
			int
				moved = get_local_entity_int_value(en, INT_TYPE_MOBILE_MOVING),
				rotated = get_local_entity_int_value(en, INT_TYPE_ROTATED);

			if (moved || rotated || command_line_tacview_logging < 3)
				write_tacview_unit_update(en, moved, rotated, FALSE);
		}
		
		if (damage_type > AIRCRAFT_DAMAGE_NONE)
			if (frand1() < pow((float)damage_type, 4) * get_delta_time())
			{
				int damage_level = max(0, get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL) - 1);

				set_client_server_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, damage_level);

				if (damage_level <= 0)
					kill_client_server_entity (en);

				return;
			}
	}
	else
	{
		update_fixed_wing_afterburners (en);

		if (get_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_DEAD)
		{
			raw->ac.death_timer += get_delta_time ();

			if (!((int)(raw->ac.death_timer + 1) % 300))
				if (raw->ac.death_timer >= calculate_mobile_death_timer_value (en))
				{
					destroy_client_server_entity_family (en);
				}
		}
		else
		{
			for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
			{
				basic_fixed_wing_death_movement (en);
			}
		}
	}
}





static void update_client (entity *en)
{
	fixed_wing
		*raw;

	int
		loop;

	raw = (fixed_wing *) get_local_entity_data (en);

	update_local_entity_view_interest_level (en);

	if (raw->ac.mob.alive)
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			if (!get_local_entity_int_value (en, INT_TYPE_LANDED))
			{

				ASSERT(point_inside_map_area(&raw->ac.mob.position));

				get_3d_terrain_point_data (raw->ac.mob.position.x, raw->ac.mob.position.z, &raw->ac.terrain_info);
			}

			basic_fixed_wing_movement (en);
		}



		update_propellors (en);

		update_fixed_wing_afterburners (en);

		update_fixed_wing_airbrakes (en);

		update_fixed_wing_flaps (en);

		update_aircraft_loading_doors (en);

		update_aircraft_cargo_doors (en);

		update_aircraft_undercarriage (en);

		update_aircraft_rudder (en);


		//
		// ORDER IS CRITICAL
		//

		update_aircraft_weapon_system_ready (en);

		update_entity_weapon_systems (en);

		update_entity_weapon_system_weapon_and_target_vectors (en);

		//


		if (tacview_is_logging() && raw->ac.mob.alive)
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
			raw->ac.death_timer += get_delta_time ();
		}
		else
		{
			for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
			{
				basic_fixed_wing_death_movement (en);
			}
		}
	}
}
















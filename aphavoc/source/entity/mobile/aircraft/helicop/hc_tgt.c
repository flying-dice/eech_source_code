

































































#include "project.h"





#define DEBUG_MODULE	0





/*
void update_helicopter_target_scan (entity *en)
{
	helicopter
		*raw;

	entity
		*force,
		*target;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

   raw = get_local_entity_data (en);

	target = NULL;

	raw->ac.ready_to_fire = FALSE;

	//
	// aircraft must be armed
	//

	if (aircraft_database[raw->ac.mob.sub_type].default_weapon_type == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		return;
	}

	//
	//
	//

	if (raw->ac.selected_weapon != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		raw->ac.target_scan_timer -= get_delta_time ();

		if (raw->ac.target_scan_timer < 0.0)
		{
			//
			// reset target_scan_timer (okay to use a random number as this is a server only variable)
			//

			raw->ac.target_scan_timer = aircraft_database[raw->ac.mob.sub_type].target_scan_delay * ((frand1 () + 1.0) * 0.5);

			//
			// scan for target
			//

			//DEBUG//
			if (weapon_database[raw->ac.selected_weapon].weapon_class & WEAPON_CLASS_AIR_TO_AIR)
			{
				target = get_aircraft_air_to_air_scan_target (en);
			}
			else if (weapon_database[raw->ac.selected_weapon].weapon_class & WEAPON_CLASS_AIR_TO_SURFACE)
			{
				target = get_aircraft_air_to_surface_scan_target (en);
			}
			//DEBUG//

			//
			// set target if different to current target
			//

			if (target != get_local_entity_parent (en, LIST_TYPE_TARGET))
			{
				set_client_server_entity_parent (en, LIST_TYPE_TARGET, target);

				force = get_local_force_entity (raw->ac.mob.side);

				notify_local_entity (ENTITY_MESSAGE_FORCE_ALLIED_TARGET, force, en, target);
			}

			if (target)
			{
				raw->ac.ready_to_fire = TRUE;
			}
		}
	}
	else
	{
		//
		// if no selected weapon then lose target (if any)
		//

		if (target != get_local_entity_parent (en, LIST_TYPE_TARGET))
		{
			set_client_server_entity_parent (en, LIST_TYPE_TARGET, target);

			force = get_local_force_entity (raw->ac.mob.side);

			notify_local_entity (ENTITY_MESSAGE_FORCE_ALLIED_TARGET, force, en, target);
		}
	}
}
*/





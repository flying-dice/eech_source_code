

































































#include "project.h"





#define DEBUG_MODULE	0




#ifdef 0
void update_fixed_wing_weapon_fire (entity *en)
{
   fixed_wing
      *raw;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (fixed_wing *) get_local_entity_data (en);

	if (raw->ac.weapon_burst_timer > 0.0)
	{
		//
		// check valid selected weapon
		//

		if (raw->ac.selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			pause_client_server_continuous_weapon_sound_effect (en, raw->ac.selected_weapon);

			raw->ac.weapon_burst_timer = 0.0;

			return;
		}

		//
		// check selected weapon is ready
		//

		if (!raw->ac.selected_weapon_system_ready)
		{
			pause_client_server_continuous_weapon_sound_effect (en, raw->ac.selected_weapon);

			raw->ac.weapon_burst_timer = 0.0;

			return;
		}

		//
		// check valid target
		//

		if (!raw->ac.mob.target_link.parent)
		{
			pause_client_server_continuous_weapon_sound_effect (en, raw->ac.selected_weapon);

			raw->ac.weapon_burst_timer = 0.0;

			return;
		}
	
		//
		// fire weapon
		//

		launch_client_server_weapon (en, raw->ac.selected_weapon, FALSE);

		//
		// update burst timer
		//
		
		raw->ac.weapon_burst_timer -= get_delta_time ();

		if (raw->ac.weapon_burst_timer <= 0.0)
		{
			pause_client_server_continuous_weapon_sound_effect (en, raw->ac.selected_weapon);

			raw->ac.weapon_burst_timer = 0.0;

			return;
		}
	}
}
#endif




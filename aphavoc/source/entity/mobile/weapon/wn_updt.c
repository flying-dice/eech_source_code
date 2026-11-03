

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	weapon
		*raw;

	raw = (weapon *) get_local_entity_data (en);

	if (raw->mob.kill_next_update)
	{
		destroy_client_server_entity_family (en);

		return;
	}

	weapon_movement (en);

	update_weapon_animation (en);

	if (weapon_database[raw->mob.sub_type].decoy_type != WEAPON_DECOY_TYPE_NONE)
	{
		update_decoy (en);
	}

	if (raw->kill_code != WEAPON_KILL_CODE_OK)
	{
		create_client_server_weapon_killed_explosion_effect (en);

		raw->mob.kill_next_update = TRUE;
	}
}





static void update_client (entity *en)
{
	weapon
		*raw;

	raw = (weapon *) get_local_entity_data (en);

	if (raw->mob.kill_next_update)
	{
		return;
	}

	weapon_movement (en);

	update_weapon_animation (en);

	if (raw->kill_code != WEAPON_KILL_CODE_OK)
	{
		raw->mob.kill_next_update = TRUE;
	}
}
















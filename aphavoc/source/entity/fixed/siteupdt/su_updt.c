

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{

	site_updatable
		*raw;

	raw = (site_updatable *) get_local_entity_data (en);

	ASSERT (raw->fix.alive);

	update_site_loading_doors (en);

	update_site_radar (en);

	update_ground_crew (en);

	if (raw->loading_door_state == SITE_LOADING_DOORS_OPEN_FLOAT_VALUE)
	{

		raw->loading_door_timer -= get_delta_time ();

		if (raw->loading_door_timer < 0.0)
		{
			raw->loading_door_timer = 0.0;

			close_client_server_entity_loading_doors (en);
		}
	}
}





static void update_client (entity *en)
{

	site_updatable
		*raw;

	raw = (site_updatable *) get_local_entity_data (en);

	ASSERT (raw->fix.alive);
	
	update_site_loading_doors (en);

	update_site_radar (en);

	update_ground_crew (en);
}
















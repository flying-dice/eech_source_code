

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_server (entity *en)
{
	cargo
		*raw;

	int
		loop;

	raw = (cargo *) get_local_entity_data (en);

	if (raw->mob.alive)
	{



		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			ASSERT(point_inside_map_area(&raw->mob.position));

			get_3d_terrain_point_data (raw->mob.position.x, raw->mob.position.z, &raw->terrain_info);

			basic_cargo_movement (en);
		}


	}
	else
	{

		for (loop = 0; loop < get_entity_movement_iterations (); loop ++)
		{

			basic_cargo_death_movement (en);
		}
	}
}



























































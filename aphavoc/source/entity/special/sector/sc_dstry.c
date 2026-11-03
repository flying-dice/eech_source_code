

































































#include "project.h"





#define DEBUG_MODULE 0





static void destroy_local (entity *en)
{
	sector
		*raw;


	//
	// PRE-AMBLE
	//


	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_DESTROY, en);

	#endif

	raw = (sector *) get_local_entity_data (en);


	//
	// VALIDATE
	//



	//
	// DESTROY COMPONENTS
	//



	//
	// UNLINK FROM SYSTEM
	//


	destroy_local_entity_children (en, LIST_TYPE_SPECIAL_EFFECT);

	unlink_local_entity_children (en, LIST_TYPE_SECTOR);


	//
	// FREE ENTITY DATA
	//


	free_mem (raw);

	set_free_entity (en);
}





void overload_sector_destroy_functions (void)
{
	fn_destroy_local_entity				[ENTITY_TYPE_SECTOR]	= destroy_local;

	fn_destroy_local_entity_family	[ENTITY_TYPE_SECTOR]	= destroy_local;

	fn_kill_local_entity					[ENTITY_TYPE_SECTOR]	= destroy_local;
}





void destroy_local_sector_entities (void)
{

	int
		x_sec,
		z_sec;

	if (entity_sector_map)
	{

		for (z_sec = MIN_MAP_Z_SECTOR; z_sec <= MAX_MAP_Z_SECTOR; z_sec++)
		{

			for (x_sec = MIN_MAP_X_SECTOR; x_sec <= MAX_MAP_X_SECTOR; x_sec++)
			{

				if (entity_sector_map [x_sec + (z_sec * NUM_MAP_X_SECTORS)]->type != ENTITY_TYPE_UNKNOWN)
				{

					destroy_local_entity (entity_sector_map [x_sec + (z_sec * NUM_MAP_X_SECTORS)]);
				}
			}
		}

		free_mem (entity_sector_map);

		entity_sector_map = NULL;
	}
}





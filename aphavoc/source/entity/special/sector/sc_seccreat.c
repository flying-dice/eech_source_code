

































































#include "project.h"





#define DEBUG_MODULE 0




//
// Sector entities are local only.
//
// They must be created before any client/server entities to ensure that the index numbers correspond on each machine.
//




void create_local_sector_entities (void)
{
	static int
		old_min_map_z_sector = -1,
		old_max_map_z_sector = -1,
		old_num_map_z_sectors = -1,
		old_min_map_x_sector = -1,
		old_max_map_x_sector = -1,
		old_num_map_x_sectors = -1;

	int
		x_sec,
		z_sec;

	//
	// First time setup
	//

	if (old_min_map_z_sector == -1)
	{
		old_min_map_z_sector = MIN_MAP_Z_SECTOR;

		old_max_map_z_sector = MAX_MAP_Z_SECTOR;

		old_num_map_z_sectors = NUM_MAP_Z_SECTORS;

		old_min_map_x_sector = MIN_MAP_X_SECTOR;

		old_max_map_x_sector = MAX_MAP_X_SECTOR;

		old_num_map_x_sectors = NUM_MAP_X_SECTORS;
	}

	//
	// Deinitialise sectors (must be the 'old' map size)
	//

	if (entity_sector_map)
	{
		for (z_sec = old_min_map_z_sector; z_sec <= old_max_map_z_sector; z_sec ++)
		{
			for (x_sec = old_min_map_x_sector; x_sec <= old_max_map_x_sector; x_sec ++)
			{
				if (entity_sector_map [x_sec + (z_sec * old_num_map_x_sectors)]->type != ENTITY_TYPE_UNKNOWN)
				{
					debug_fatal ("SC_CREAT: uninitialised sector entity");
				}
			}
		}

		free_mem (entity_sector_map);
	}

	//
	// Store new map size
	//

	old_min_map_z_sector = MIN_MAP_Z_SECTOR;

	old_max_map_z_sector = MAX_MAP_Z_SECTOR;

	old_num_map_z_sectors = NUM_MAP_Z_SECTORS;

	old_min_map_x_sector = MIN_MAP_X_SECTOR;

	old_max_map_x_sector = MAX_MAP_X_SECTOR;

	old_num_map_x_sectors = NUM_MAP_X_SECTORS;

	//
	// Malloc memory
	//

	entity_sector_map = (entity **) malloc_heap_mem (sizeof (entity) * NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS);

	//
	// Create sector entities
	//

	for (z_sec = MIN_MAP_Z_SECTOR; z_sec <= MAX_MAP_Z_SECTOR; z_sec++)
	{
		for (x_sec = MIN_MAP_X_SECTOR; x_sec <= MAX_MAP_X_SECTOR; x_sec++)
		{
			entity_sector_map[x_sec + (z_sec * NUM_MAP_X_SECTORS)] = create_local_entity
			(
				ENTITY_TYPE_SECTOR,
				ENTITY_INDEX_DONT_CARE,
				ENTITY_ATTR_INT_VALUE (INT_TYPE_X_SECTOR, x_sec),
				ENTITY_ATTR_INT_VALUE (INT_TYPE_Z_SECTOR, z_sec),
				ENTITY_ATTR_END
			);
		}
	}

	debug_log ("SC_CREAT: creating sectors. Start entity %d", get_local_entity_index (entity_sector_map [0]));
}





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	sector
		*raw;


  	//
  	// VALIDATE
  	//


	validate_local_create_entity_index (index);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_CREATE, NULL, type, index);

	#endif

	en = get_free_entity (index);

	if (en)
	{

   	//
   	// MALLOC ENTITY DATA
   	//


		set_local_entity_type (en, type);

		raw = (sector *) malloc_fast_mem (sizeof (sector));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (sector));


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//



		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//

	}

	return (en);
}





void overload_sector_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_SECTOR] = create_local;
}





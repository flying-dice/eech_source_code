

































































#include "project.h"

#include "ai_dbase.h"





#define AI_DATABASE_DEBUG 0

#define AI_POPULATION_NAME_FILENAME "route\\popname.dat"





static const char
	*population_name_type_strings [] =
	{
		"KEYSITE",
		"TOWN",

		"NUM_POPULATION_NAME_TYPES"
	};





population_name_database_type
	*population_name_database = NULL;





void initialise_ai_database (void)
{
}





void deinitialise_ai_database (void)
{
}





void load_ai_sector_data (const char *sector_filename)
{

	entity
		*sector;

	int
		size,
		side,
		loop_x,
		loop_z;

	FILE
		*file_ptr;

	// JB 030311 Enable running out of separate directories
	file_ptr = safe_fopen (sector_filename, "rb");

	if (!file_ptr)
	{

		return;
	}

	size = file_size (sector_filename);

	ASSERT (size == (NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS) * sizeof (int));

	if (file_ptr)
	{
	
		for (loop_x = 0; loop_x < NUM_MAP_X_SECTORS; loop_x ++)
		{
	
			for (loop_z = 0; loop_z < NUM_MAP_Z_SECTORS; loop_z ++)
			{
	
				fread (&side, 1, sizeof (int), file_ptr);
	
				sector = get_local_raw_sector_entity (loop_x, loop_z);
	
				set_local_entity_int_value (sector, INT_TYPE_SIDE, side);
			}
		}
	}

	fclose (file_ptr);
}









































































































































































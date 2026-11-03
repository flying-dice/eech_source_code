

































































#include "project.h"

#include "ai_sect.h"





#define DEBUG_MODULE 0





/*static*/ void initialise_ai_sector_exposure (void);





void initialise_ai_sectors (void)
{

//	initialise_ai_sector_exposure ();
}





void initialise_ai_sector_exposure (void)
{

	entity
		*sector;

   int
		min_terrain_height = 0.0,
		max_terrain_height = 0.0,
		delta_value,
		value,
      x,
      z;

	//
	// find the min and max heights of the terrain to allow the exposure value to be scaled.
	//

   for (x = 0; x < (NUM_MAP_X_SECTORS * 2); x ++)
   {

      for (z = 0; z < (NUM_MAP_Z_SECTORS * 2); z ++)
      {

			if (!x && !z || terrain_sectors [z][x].maximum_height > max_terrain_height)
			{
				max_terrain_height = terrain_sectors [z][x].maximum_height;
			}

			if (!x && !z || terrain_sectors [z][x].minimum_height < min_terrain_height)
			{
				min_terrain_height = terrain_sectors [z][x].minimum_height;
			}
		}
	}







	//
	// exposure is the average delta height of the 4 sectors in an ai sector. Roughly scaled to 0 -> 255
	//

   for (x = 0; x < NUM_MAP_X_SECTORS; x ++)
   {

      for (z = 0; z < NUM_MAP_Z_SECTORS; z ++)
      {

         value =
            (((terrain_sectors [(z * 2) + 0][(x * 2) + 0].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height) +
            ((terrain_sectors [(z * 2) + 1][(x * 2) + 0].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height) +
            ((terrain_sectors [(z * 2) + 0][(x * 2) + 1].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height) +
            ((terrain_sectors [(z * 2) + 1][(x * 2) + 1].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height)) / 4;

			delta_value =
            (abs (value - ((terrain_sectors [(z * 2) + 0][(x * 2) + 0].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height)) +
            abs (value - ((terrain_sectors [(z * 2) + 1][(x * 2) + 0].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height)) +
            abs (value - ((terrain_sectors [(z * 2) + 0][(x * 2) + 1].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height)) +
            abs (value - ((terrain_sectors [(z * 2) + 1][(x * 2) + 1].average_height - min_terrain_height) * 256) / (max_terrain_height - min_terrain_height))) * 2;

			// add some kind of forest check.

			delta_value = bound (delta_value, 0, 255);







			sector = get_local_raw_sector_entity (x, z);

			set_local_entity_int_value (sector, INT_TYPE_EXPOSURE_LEVEL, delta_value);
      }
   }
}







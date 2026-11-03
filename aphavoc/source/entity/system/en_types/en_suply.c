

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_supply_heat_map_cargo_spot (entity *en, int x, int z, int value);




	
unsigned char
	*supply_heat_bitmap = NULL;

unsigned char
	*supply_heat_map = NULL;




	
float get_local_entity_rearming_sleep_time (entity *group, int weapon_type)
{

	float
		base_time,	
		sleep_time;

	if(!weapon_type)
		base_time = get_local_entity_float_value (group, FLOAT_TYPE_REARMING_TIME);
	else
		base_time = weapon_database[weapon_type].rearming_time;

	sleep_time = base_time * ( -(MAX_REARMING_TIME_SCALING_FACTOR - 1.0) / 100.0 * get_local_entity_float_value (group, FLOAT_TYPE_AMMO_SUPPLY_LEVEL) + MAX_REARMING_TIME_SCALING_FACTOR);

	#if DEBUG_MODULE || DEBUG_SUPPLY

	debug_log ("EN_SUPLY: SUPPLY_INFO: group %s rearming sleep time = %f (normal_time = %f, supply level = %f)",
					get_local_entity_string (group, STRING_TYPE_FULL_NAME),
					sleep_time,
					base_time,
					get_local_entity_float_value (group, FLOAT_TYPE_AMMO_SUPPLY_LEVEL));

	#endif

	return sleep_time;
}





float get_local_entity_refueling_sleep_time (entity *group)
{

	float
		sleep_time;

	// using rearming time for the moment...
	sleep_time = get_local_entity_float_value (group, FLOAT_TYPE_REARMING_TIME);

	sleep_time *= -(((MAX_REFUELING_TIME_SCALING_FACTOR - 1.0) / 100.0) * get_local_entity_float_value (group, FLOAT_TYPE_FUEL_SUPPLY_LEVEL)) + MAX_REARMING_TIME_SCALING_FACTOR;

	#if DEBUG_MODULE || DEBUG_SUPPLY

	debug_log ("EN_SUPLY: SUPPLY_INFO: group %s refueling sleep time = %f (normal_time = %f, supply level = %f)",
					get_local_entity_string (group, STRING_TYPE_FULL_NAME),
					sleep_time,
					get_local_entity_float_value (group, FLOAT_TYPE_REARMING_TIME),
					get_local_entity_float_value (group, FLOAT_TYPE_FUEL_SUPPLY_LEVEL));

	#endif

	return sleep_time;
}





void build_supply_heat_map (void)
{

	int
		x,
		z;

	entity
		*en,
		*group,
		*sector;

	float
		supplies;

	if (!supply_heat_map)
	{
		supply_heat_bitmap = (unsigned char *) malloc_heap_mem (sizeof (unsigned char) * (NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS * 3));

		supply_heat_map = (unsigned char *) malloc_heap_mem (sizeof (unsigned char) * (NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS));
	}

	#if DEBUG_MODULE

		debug_log ("EN_SUPLY: building supply heat map");

	#endif

	memset (supply_heat_map, 0, sizeof (unsigned char) * (NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS));

	memset (supply_heat_bitmap, 0, sizeof (unsigned char) * (NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS * 3));
	
	// draw cargo

   for (x = 0; x < NUM_MAP_X_SECTORS; x ++)
   {

      for (z = 0; z < NUM_MAP_Z_SECTORS; z ++)
      {

			sector = get_local_raw_sector_entity (x, z);

			en = get_local_entity_first_child (sector, LIST_TYPE_SECTOR);

			while (en)
			{

				if (get_local_entity_type (en) == ENTITY_TYPE_CARGO)
				{
					set_supply_heat_map_cargo_spot (en, x, z, 1);
				}

				else if (get_local_entity_int_value (en, INT_TYPE_ALIVE))
				{
					if (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_VEHICLE))
					{
						if (get_local_entity_int_value (en, INT_TYPE_GROUP_LEADER))
						{
	
							group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	
							// get combined value of supplies - (each in range 0-100) into range 0-127

							supplies = (get_local_entity_float_value (group, FLOAT_TYPE_FUEL_SUPPLY_LEVEL) +
											get_local_entity_float_value (group, FLOAT_TYPE_AMMO_SUPPLY_LEVEL)) *
											0.05;
							
							set_supply_heat_map_cargo_spot (en, x, z, (int) supplies);
						}
					}
				}

				en = get_local_entity_child_succ (en, LIST_TYPE_SECTOR);
			}
		}
	}

	// draw frontline groups
}





void set_supply_heat_map_cargo_spot (entity *en, int x, int z, int value)
{

	rgb_colour
		col;

	int
		x_loop,
		z_loop,
		x_a,
		z_a,
		offset,
		str;

	float
		scale;
	
	offset = supply_heat_map[x + (NUM_MAP_X_SECTORS * z)];

	if (offset > 128)
	{
		offset = 3;
	}
	else if (offset > 64)
	{
		offset = 2;
	}
	else if (offset > 32)
	{
		offset = 1;
	}
	else
	{
		offset = 0;
	}
	
	offset = min (offset, 3);

	if (offset)
	{
		scale = 255 / (offset * 2);
	}
	else
	{
		scale = 0;
		
		offset = 0;
	}
	
	for (z_loop = max (z - offset, 0); z_loop < min (z + offset + 1, NUM_MAP_Z_SECTORS); z_loop ++)
	{
		z_a = abs (z_loop - (z));

		for (x_loop = max (x - offset, 0); x_loop < min (x + offset + 1, NUM_MAP_X_SECTORS); x_loop ++)
		{

			col.r = supply_heat_bitmap [(z_loop * NUM_MAP_X_SECTORS * 3) + (x_loop * 3)];
			col.g = supply_heat_bitmap [(z_loop * NUM_MAP_X_SECTORS * 3) + (x_loop * 3) + 1];
			col.b = supply_heat_bitmap [(z_loop * NUM_MAP_X_SECTORS * 3) + (x_loop * 3) + 2];

			x_a = abs (x_loop - (x));

			str = 255 - (scale * (x_a + z_a));

			if (str > supply_heat_map[(z_loop * NUM_MAP_X_SECTORS) + x_loop])
			{
				supply_heat_map[(z_loop * NUM_MAP_X_SECTORS) + x_loop] = bound (str, 0.0, 255.0);
			}
			
			if (get_local_entity_int_value (en, INT_TYPE_SIDE) == ENTITY_SIDE_BLUE_FORCE)
			{
				if (str > col.b)
				{
					col.b = bound ((str), 0.0, 255.0);
				}
			}
			else if (get_local_entity_int_value (en, INT_TYPE_SIDE) == ENTITY_SIDE_RED_FORCE)
			{
				if (str > col.r)
				{
					col.r = bound ((str), 0.0, 255.0);
				}
			}
			else
			{
				if (str > col.g)
				{
					col.g = bound ((str), 0.0, 255.0);
				}
			}

			supply_heat_bitmap [(z_loop * NUM_MAP_X_SECTORS * 3) + (x_loop * 3)] = col.r;
			supply_heat_bitmap [(z_loop * NUM_MAP_X_SECTORS * 3) + (x_loop * 3) + 1] = col.g;
			supply_heat_bitmap [(z_loop * NUM_MAP_X_SECTORS * 3) + (x_loop * 3) + 2] = col.b;

		}
	}
}





void output_supply_heat_map (void)
{
	static int
		limit = 10,
		numzeros = 4,
		count = 0;

	char
		filename[13],
		temp_string[7];
	
	int
		z;

	// initialise filename & output map

	sprintf (filename, "sm");

	sprintf (temp_string, "%d.psd", count);
	
	if (count >= limit)
	{
		limit *= 10;
		
		numzeros--;
	}

	for (z = 0; z < numzeros; z++)
	{
		strcat (filename, "0");
	}

	strcat (filename, temp_string);


	if (supply_heat_bitmap == NULL)
	{
		build_supply_heat_map ();
	}
				
	write_psd_rgb_file (filename, NUM_MAP_X_SECTORS, NUM_MAP_Z_SECTORS, NUM_MAP_X_SECTORS * 3, supply_heat_bitmap);

	#if DEBUG_MODULE

		debug_log ("EN_SUPLY: generating supply heat map psd no. %d", count);

	#endif

	count++;

}





	

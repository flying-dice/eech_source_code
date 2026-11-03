

































































#include "project.h"





#define DEBUG_MODULE 0





world_map_data
	world_map;





void set_entity_world_map_size (int num_map_x_sectors, int num_map_z_sectors, int sector_side_length)
{
	ASSERT (num_map_x_sectors > 0);

	ASSERT (num_map_z_sectors > 0);

	ASSERT (int_bit_count (sector_side_length) == 1);

	world_map.sector_side_length			= sector_side_length;

	world_map.sector_side_length_mask	= sector_side_length - 1;

	world_map.num_map_x_sectors			= num_map_x_sectors;
	world_map.num_map_z_sectors			= num_map_z_sectors;

	world_map.num_map_sectors				= num_map_x_sectors * num_map_z_sectors;

	world_map.min_map_x_sector	 			= 0;
	world_map.min_map_z_sector	  	 		= 0;

	world_map.max_map_x_sector		  		= num_map_x_sectors - 1;
	world_map.max_map_z_sector				= num_map_z_sectors - 1;

	world_map.min_map_x	 					= 0.0;
	world_map.min_map_y						= -8000.0;
	world_map.min_map_z						= 0.0;

	world_map.max_map_x						= (float) (num_map_x_sectors * sector_side_length) - 1.0;	// ***** DO NOT CHANGE 1.0 *****
	world_map.max_map_y						= 65535.0;				 										
	world_map.max_map_z		  				= (float) (num_map_z_sectors * sector_side_length) - 1.0;	// ***** DO NOT CHANGE 1.0 *****

	world_map.mid_map_x 						= world_map.min_map_x + ((world_map.max_map_x - world_map.min_map_x) * 0.5);
	world_map.mid_map_y						= world_map.min_map_y + ((world_map.max_map_y - world_map.min_map_y) * 0.5);
	world_map.mid_map_z						= world_map.min_map_z + ((world_map.max_map_z - world_map.min_map_z) * 0.5);
}





int bound_position_to_map_area (vec3d *position)
{
	int
		result;

	result = FALSE;

	if (position->x < MIN_MAP_X)
	{
		position->x = MIN_MAP_X;

		result = TRUE;
	}
	else if (position->x > MAX_MAP_X)
	{
		position->x = MAX_MAP_X;

		result = TRUE;
	}

	if (position->z < MIN_MAP_Z)
	{
		position->z = MIN_MAP_Z;

		result = TRUE;
	}
	else if (position->z > MAX_MAP_Z)
	{
		position->z = MAX_MAP_Z;

		result = TRUE;
	}

	return (result);
}





int bound_position_to_adjusted_map_area (vec3d *position)
{
	int
		result;

	result = FALSE;

	if (position->x < MIN_MAP_X + MAP_PERIMETER_SIZE)
	{
		position->x = MIN_MAP_X + MAP_PERIMETER_SIZE;

		result = TRUE;
	}
	else if (position->x > MAX_MAP_X - MAP_PERIMETER_SIZE)
	{
		position->x = MAX_MAP_X - MAP_PERIMETER_SIZE;

		result = TRUE;
	}

	if (position->z < MIN_MAP_Z + MAP_PERIMETER_SIZE)
	{
		position->z = MIN_MAP_Z + MAP_PERIMETER_SIZE;

		result = TRUE;
	}
	else if (position->z > MAX_MAP_Z - MAP_PERIMETER_SIZE)
	{
		position->z = MAX_MAP_Z - MAP_PERIMETER_SIZE;

		result = TRUE;
	}

	return (result);
}





int bound_position_to_map_volume (vec3d *position)
{
	int
		result;

	result = FALSE;

	if (position->x < MIN_MAP_X)
	{
		position->x = MIN_MAP_X;

		result = TRUE;
	}
	else if (position->x > MAX_MAP_X)
	{
		position->x = MAX_MAP_X;

		result = TRUE;
	}

	if (position->y < MIN_MAP_Y)
	{
		position->y = MIN_MAP_Y;

		result = TRUE;
	}
	else if (position->y > MAX_MAP_Y)
	{
		position->y = MAX_MAP_Y;

		result = TRUE;
	}

	if (position->z < MIN_MAP_Z)
	{
		position->z = MIN_MAP_Z;

		result = TRUE;
	}
	else if (position->z > MAX_MAP_Z)
	{
		position->z = MAX_MAP_Z;

		result = TRUE;
	}

	return (result);
}

int bound_double_position_to_map_volume (double_vec3d *position)
{
	int
		result;

	result = FALSE;

	if (position->x < MIN_MAP_X)
	{
		position->x = MIN_MAP_X;

		result = TRUE;
	}
	else if (position->x > MAX_MAP_X)
	{
		position->x = MAX_MAP_X;

		result = TRUE;
	}

	if (position->y < MIN_MAP_Y)
	{
		position->y = MIN_MAP_Y;

		result = TRUE;
	}
	else if (position->y > MAX_MAP_Y)
	{
		position->y = MAX_MAP_Y;

		result = TRUE;
	}

	if (position->z < MIN_MAP_Z)
	{
		position->z = MIN_MAP_Z;

		result = TRUE;
	}
	else if (position->z > MAX_MAP_Z)
	{
		position->z = MAX_MAP_Z;

		result = TRUE;
	}

	return (result);
}





int bound_position_to_adjusted_map_volume (vec3d *position)
{
	int
		result;

	result = FALSE;

	if (position->x < MIN_MAP_X + MAP_PERIMETER_SIZE)
	{
		position->x = MIN_MAP_X + MAP_PERIMETER_SIZE;

		result = TRUE;
	}
	else if (position->x > MAX_MAP_X - MAP_PERIMETER_SIZE)
	{
		position->x = MAX_MAP_X - MAP_PERIMETER_SIZE;

		result = TRUE;
	}

	if (position->y < MIN_MAP_Y)
	{
		position->y = MIN_MAP_Y;

		result = TRUE;
	}
	else if (position->y > MAX_MAP_Y)
	{
		position->y = MAX_MAP_Y;

		result = TRUE;
	}

	if (position->z < MIN_MAP_Z + MAP_PERIMETER_SIZE)
	{
		position->z = MIN_MAP_Z + MAP_PERIMETER_SIZE;

		result = TRUE;
	}
	else if (position->z > MAX_MAP_Z - MAP_PERIMETER_SIZE)
	{
		position->z = MAX_MAP_Z - MAP_PERIMETER_SIZE;

		result = TRUE;
	}

	return (result);
}





int bound_double_position_to_adjusted_map_volume (double_vec3d *position)
{
	int
		result;

	result = FALSE;

	if (position->x < MIN_MAP_X + MAP_PERIMETER_SIZE)
	{
		position->x = MIN_MAP_X + MAP_PERIMETER_SIZE;

		result = TRUE;
	}
	else if (position->x > MAX_MAP_X - MAP_PERIMETER_SIZE)
	{
		position->x = MAX_MAP_X - MAP_PERIMETER_SIZE;

		result = TRUE;
	}

	if (position->y < MIN_MAP_Y)
	{
		position->y = MIN_MAP_Y;

		result = TRUE;
	}
	else if (position->y > MAX_MAP_Y)
	{
		position->y = MAX_MAP_Y;

		result = TRUE;
	}

	if (position->z < MIN_MAP_Z + MAP_PERIMETER_SIZE)
	{
		position->z = MIN_MAP_Z + MAP_PERIMETER_SIZE;

		result = TRUE;
	}
	else if (position->z > MAX_MAP_Z - MAP_PERIMETER_SIZE)
	{
		position->z = MAX_MAP_Z - MAP_PERIMETER_SIZE;

		result = TRUE;
	}

	return (result);
}





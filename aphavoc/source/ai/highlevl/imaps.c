

































































#include "project.h"

#include "imaps.h"





#define DEBUG_MODULE   0





static unsigned char
	**imaps [NUM_IMAP_TYPES];

static float
	*imap_temp_array = NULL;

static int
	imaps_initialised = FALSE;





void initialise_imaps (void)
{
	int
		loop,
		side;
		
	ASSERT (get_session_entity ());

	for (loop = 0; loop < NUM_IMAP_TYPES; loop ++)
	{
		imaps [loop] = (unsigned char**) malloc_heap_mem (sizeof (unsigned char *) * NUM_ENTITY_SIDES);

		for (side = 0; side < NUM_ENTITY_SIDES; side ++)
		{
			imaps [loop][side] = (unsigned char*) malloc_heap_mem (sizeof (unsigned char) * NUM_MAP_Z_SECTORS * NUM_MAP_X_SECTORS);
		}
	}

	// temporary array

	imap_temp_array = (float*)malloc_heap_mem (sizeof (float) * NUM_MAP_X_SECTORS * NUM_MAP_Z_SECTORS);

	imaps_initialised = TRUE;
}





void deinitialise_imaps (void)
{
	int
		loop,
		side;
		
	for (loop = 0; loop < NUM_IMAP_TYPES; loop ++)
	{
		ASSERT (imaps [loop]);

		for (side = 0; side < NUM_ENTITY_SIDES; side ++)
		{
			free_mem (imaps [loop][side]);
		}

		free_mem (imaps [loop]);

		imaps [loop] = NULL;
	}	

	// temporary array

	free_mem (imap_temp_array);

	imap_temp_array = NULL;

	imaps_initialised = FALSE;
}





float get_imap_value (imap_types type, entity_sides side, int x, int z)
{
	float
		f;

	unsigned char
		v;

	v = imaps [type][side][(z * NUM_MAP_X_SECTORS) + x];

	f = (float)v;

	return (f / 255.0);
}





unsigned char get_imap_char_value (imap_types type, entity_sides side, int x, int z)
{
	unsigned char
		v;

	v = imaps [type][side][(z * NUM_MAP_X_SECTORS) + x];

	return v;
}





static void normalise_inlfuence_map (float *original, unsigned char *output, int xsize, int zsize)
{
	float
		val,
		min_val,
		max_val;

	int
		x,
		z;

	ASSERT (original);

	ASSERT (output);

	//
	// Find largest value
	//

	min_val = FLT_MAX;

	max_val = 0.0;

	for (z = 0; z < zsize; z++)
	{
		for (x = 0; x < xsize; x++)
		{
			min_val = min (min_val, original [(z * xsize) + x]);

			max_val = max (max_val, original [(z * xsize) + x]);
		}
	}

	if (max_val == min_val)
	{
		return;
	}

	//
	// Normalise
	//

	for (z = 0; z < zsize; z++)
	{
		for (x = 0; x < xsize; x++)
		{
			val = 255.0 * ((original [(z * xsize) + x] - min_val) / (max_val - min_val));

			ASSERT ((val >= 0.0) && (val < 256.0));

			output [(z * xsize) + x] = (unsigned char)val;
		}
	}
}












































//
// SECTOR SIDES
//




static float update_sector_side (int sx, int sz, entity_sides side, int in_use, float scale)
{
	entity
		*en;

	sector
		*raw;
		
	en = get_local_raw_sector_entity (sx, sz);

	ASSERT (en);

	raw = (sector*)get_local_entity_data (en);

	if (scale > 0.0)
	{
		if (in_use)
		{
			raw->sector_side [side] += scale;
		}
		else
		{
			ASSERT (raw->sector_side [side] > 0.0);
	
			raw->sector_side [side] -= scale;
		}
	





	}

	return raw->sector_side [side];
}





void update_imap_sector_side (entity *en, int in_use)
{
	int
		x,
		z,
		sx,
		sz,
		side;

	vec3d
		*pos;

	float
		max_val,
		d2,
		dx,
		dz,
		scale,
		constant;

	ASSERT (en);

	max_val = get_local_entity_float_value (en, FLOAT_TYPE_SECTOR_SIDE_MAX_VALUE);

	if (max_val > 0.0)
	{
		constant = 0.3;

		side = get_local_entity_int_value (en, INT_TYPE_SIDE);

		pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	
		get_x_sector (sx, pos->x);
		get_z_sector (sz, pos->z);
	













		for (z = MIN_MAP_Z_SECTOR; z <= MAX_MAP_Z_SECTOR ; z ++)
		{
			dz = (constant * fabs ((float)(z - sz))) + 1.0;

			for (x = MIN_MAP_X_SECTOR; x <= MAX_MAP_X_SECTOR ; x ++)
			{
				dx = (constant * fabs ((float)(x - sx))) + 1.0;

				d2 = ((dx * dx) + (dz * dz));

				scale = max_val / d2;

				update_sector_side (x, z, (entity_sides)side, in_use, scale);
			}
		}
	}
}




//
// KEYSITE IMPORTANCE
//




static float update_sector_importance_level (int sx, int sz, entity_sides side, int in_use, float scale)
{
	entity
		*en;

	sector
		*raw;
		
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	en = get_local_raw_sector_entity (sx, sz);

	ASSERT (en);

	raw = (sector*) get_local_entity_data (en);

	if (scale > 0.0)
	{
		if (in_use)
		{
			raw->importance_level [side] += scale;
		}
		else
		{
			ASSERT (raw->importance_level [side] > 0.0);
	
			raw->importance_level [side] -= scale;
		}
	





	}

	return raw->importance_level [side];
}





void update_imap_importance_level (entity *en, int in_use)
{
	int
		x,
		z,
		sx,
		sz,
		side,
		sub_type,
		sector_radius,
		sr2;

	vec3d
		*pos;

	float
		scale,
		radius,
		importance;

	ASSERT (en);

	ASSERT (get_local_entity_type (en) == ENTITY_TYPE_KEYSITE);

	importance = get_local_entity_float_value (en, FLOAT_TYPE_KEYSITE_IMPORTANCE);

	if (importance > 0.0)
	{
		side = get_local_entity_int_value (en, INT_TYPE_SIDE);

		sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

		radius = keysite_database [sub_type].importance_radius / SECTOR_SIDE_LENGTH;

		convert_float_to_int (radius, &sector_radius);

		sector_radius = max (sector_radius, 1);

		sr2 = sector_radius * sector_radius;

		pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	
		get_x_sector (sx, pos->x);
		get_z_sector (sz, pos->z);
	






		for (z = -sector_radius; z <= sector_radius; z ++)
		{
			if (((sz + z) < MIN_MAP_Z_SECTOR) || ((sz + z) > MAX_MAP_Z_SECTOR))
			{
				continue;
			}

			for (x = -sector_radius; x <= sector_radius; x ++)
			{
				if (((sx + x) < MIN_MAP_X_SECTOR) || ((sx + x) > MAX_MAP_X_SECTOR))
				{
					continue;
				}

				scale = (float)((x * x) + (z * z)) / (float)sr2;

				if (scale < 1.0)
				{
					scale = 1.0 - scale;

					update_sector_importance_level (sx + x, sz + z, (entity_sides)side, in_use, importance * scale);
				}
			}
		}
	}
}





void normalise_importance_imaps (void)
{
	int
		sx,
		sz,
		side;

	entity
		*sec;

	sector
		*raw;

	float
		val;

	for (side = 0; side < NUM_ENTITY_SIDES; side ++)
	{
		for (sz = 0; sz < NUM_MAP_Z_SECTORS; sz++)
		{
			for (sx = 0; sx < NUM_MAP_X_SECTORS; sx++)
			{
				sec = get_local_raw_sector_entity (sx, sz);

				ASSERT (sec);

				raw = (sector*) get_local_entity_data (sec);

				val = raw->importance_level [side];

				imap_temp_array [(sz * NUM_MAP_X_SECTORS) + sx] = val;
			}
		}

		normalise_inlfuence_map (imap_temp_array, imaps [IMAP_IMPORTANCE][side], NUM_MAP_X_SECTORS, NUM_MAP_Z_SECTORS);
	}
}




//
// DISTANCE TO FRIENDLY BASE 
//




static float update_sector_distance_to_friendly_base (int sx, int sz, entity_sides side, float scale)
{
	entity
		*en;

	sector
		*raw;
		
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	en = get_local_raw_sector_entity (sx, sz);

	ASSERT (en);

	raw = (sector*) get_local_entity_data (en);

	if (scale > 0.0)
	{
		raw->distance_to_friendly_base [side] = max (raw->distance_to_friendly_base [side], scale);
	





	}

	return raw->distance_to_friendly_base [side];
}





void update_keysite_distance_to_friendly_base (entity *en, entity_sides side)
{
	int
		x,
		z,
		sx,
		sz,
		sub_type,
		sector_radius,
		sr2;

	vec3d
		*pos;

	float
		scale,
		radius;

	ASSERT (en);

	ASSERT (get_local_entity_type (en) == ENTITY_TYPE_KEYSITE);

	sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

	radius = keysite_database [sub_type].air_coverage_radius / SECTOR_SIDE_LENGTH;

	if (radius > 0.0)
	{
		convert_float_to_int (radius, &sector_radius);

		sector_radius = max (sector_radius, 1);

		sr2 = sector_radius * sector_radius;

		pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	
		get_x_sector (sx, pos->x);
		get_z_sector (sz, pos->z);
	






		for (z = -sector_radius; z <= sector_radius; z ++)
		{
			if (((sz + z) < MIN_MAP_Z_SECTOR) || ((sz + z) > MAX_MAP_Z_SECTOR))
			{
				continue;
			}

			for (x = -sector_radius; x <= sector_radius; x ++)
			{
				if (((sx + x) < MIN_MAP_X_SECTOR) || ((sx + x) > MAX_MAP_X_SECTOR))
				{
					continue;
				}

				scale = (float)((x * x) + (z * z)) / (float)sr2;

				if (scale < 1.0)
				{
					scale = 1.0 - scale;

					update_sector_distance_to_friendly_base (sx + x, sz + z, side, scale);
				}
			}
		}
	}
}





void update_imap_distance_to_friendly_base (entity_sides side)
{
	entity
		*sec,
		*force,
		*keysite;

	int
		x,
		z;

	sector
		*raw;

	for (z = MIN_MAP_Z_SECTOR; z < MAX_MAP_Z_SECTOR; z ++)
	{
		for (x = MIN_MAP_X_SECTOR; x < MAX_MAP_X_SECTOR; x ++)
		{
			sec = get_local_raw_sector_entity (x, z);

			ASSERT (sec);

			raw = (sector*) get_local_entity_data (sec);

			raw->distance_to_friendly_base [side] = 0.0;
		}
	}

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);
	
	force = get_local_force_entity (side);

	ASSERT (force);

	keysite = get_local_entity_first_child (force, LIST_TYPE_KEYSITE_FORCE);

	while (keysite)
	{
		if (get_local_entity_int_value (keysite, INT_TYPE_IN_USE))
		{
			if (get_local_entity_int_value (keysite, INT_TYPE_ALIVE))
			{
				update_keysite_distance_to_friendly_base (keysite, side);
			}
		}

		keysite = get_local_entity_child_succ (keysite, LIST_TYPE_KEYSITE_FORCE);
	}

	if (imaps_initialised)
	{
		normalise_base_distance_imaps ();
	}
}





void normalise_base_distance_imaps (void)
{
	int
		sx,
		sz,
		side;

	entity
		*sec;

	sector
		*raw;

	float
		val;

	for (side = 0; side < NUM_ENTITY_SIDES; side ++)
	{
		for (sz = 0; sz < NUM_MAP_Z_SECTORS; sz++)
		{
			for (sx = 0; sx < NUM_MAP_X_SECTORS; sx++)
			{
				sec = get_local_raw_sector_entity (sx, sz);

				ASSERT (sec);

				raw = (sector*) get_local_entity_data (sec);

				val = raw->distance_to_friendly_base [side];

				imap_temp_array [(sz * NUM_MAP_X_SECTORS) + sx] = val;
			}
		}

		normalise_inlfuence_map (imap_temp_array, imaps [IMAP_BASE_DISTANCE][side], NUM_MAP_X_SECTORS, NUM_MAP_Z_SECTORS);
	}
}




//
// SURFACE TO AIR DEFENCE LEVELS
//




static float update_sector_surface_to_air_defence_level (int sx, int sz, entity_sides side, int in_use, float scale)
{
	entity
		*en;

	sector
		*raw;
		
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	en = get_local_raw_sector_entity (sx, sz);

	ASSERT (en);

	raw = (sector*) get_local_entity_data (en);

	if (scale > 0.0)
	{
		if (in_use)
		{
			raw->surface_to_air_defence_level [side] += scale;
		}
		else
		{
			ASSERT (raw->surface_to_air_defence_level [side] > 0.0);

			raw->surface_to_air_defence_level [side] -= scale;
		}
	





	}

	return raw->surface_to_air_defence_level [side];
}





void update_imap_surface_to_air_defence_level (entity *en, entity *sector, int in_use)
{
	int
		x,
		z,
		sx,
		sz,
		side,
		sector_radius,
		sr2;

	float
		scale,
		radius,
		value;

	ASSERT (en);

	ASSERT (sector);

	value = get_local_entity_float_value (en, FLOAT_TYPE_POTENTIAL_SURFACE_TO_AIR_THREAT);

	if (value > 0.0)
	{
		side = get_local_entity_int_value (en, INT_TYPE_SIDE);

		radius = get_local_entity_float_value (en, FLOAT_TYPE_AIR_SCAN_RANGE) * 10.0 / SECTOR_SIDE_LENGTH;

		convert_float_to_int (radius, &sector_radius);

		sector_radius = max (sector_radius, 1);

		sr2 = sector_radius * sector_radius;

		sx = get_local_entity_int_value (sector, INT_TYPE_X_SECTOR);
		sz = get_local_entity_int_value (sector, INT_TYPE_Z_SECTOR);
	






		for (z = -sector_radius; z <= sector_radius; z ++)
		{
			if (((sz + z) < MIN_MAP_Z_SECTOR) || ((sz + z) > MAX_MAP_Z_SECTOR))
			{
				continue;
			}

			for (x = -sector_radius; x <= sector_radius; x ++)
			{
				if (((sx + x) < MIN_MAP_X_SECTOR) || ((sx + x) > MAX_MAP_X_SECTOR))
				{
					continue;
				}

				scale = (float)((x * x) + (z * z)) / (float)sr2;

				if (scale < 1.0)
				{
					scale = 1.0 - scale;

					update_sector_surface_to_air_defence_level (sx + x, sz + z, (entity_sides)side, in_use, value * scale);
				}
			}
		}
	}
}





void normalise_air_defence_imaps (void)
{
	int
		sx,
		sz,
		side;

	entity
		*sec;

	sector
		*raw;

	float
		val;

	for (side = 0; side < NUM_ENTITY_SIDES; side ++)
	{
		for (sz = 0; sz < NUM_MAP_Z_SECTORS; sz++)
		{
			for (sx = 0; sx < NUM_MAP_X_SECTORS; sx++)
			{
				sec = get_local_raw_sector_entity (sx, sz);

				ASSERT (sec);

				raw = (sector*) get_local_entity_data (sec);

				val = raw->surface_to_air_defence_level [side];

				imap_temp_array [(sz * NUM_MAP_X_SECTORS) + sx] = val;
			}
		}

		normalise_inlfuence_map (imap_temp_array, imaps [IMAP_AIR_DEFENCE][side], NUM_MAP_X_SECTORS, NUM_MAP_Z_SECTORS);
	}
}




//
// SURFACE TO SURFACE DEFENCE LEVELS
//




static float update_sector_surface_to_surface_defence_level (int sx, int sz, entity_sides side, int in_use, float scale)
{
	entity
		*en;

	sector
		*raw;
		
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	en = get_local_raw_sector_entity (sx, sz);

	ASSERT (en);

	raw = (sector*) get_local_entity_data (en);

	if (scale > 0.0)
	{
		if (in_use)
		{
			raw->surface_to_surface_defence_level [side] += scale;
		}
		else
		{
			ASSERT (raw->surface_to_surface_defence_level [side] > 0.0);

			raw->surface_to_surface_defence_level [side] -= scale;
		}
	





	}

	return raw->surface_to_surface_defence_level [side];
}





void update_imap_surface_to_surface_defence_level (entity *en, entity *sector, int in_use)
{
	int
		x,
		z,
		sx,
		sz,
		side,
		sector_radius,
		sr2;

	float
		scale,
		radius,
		value;

	ASSERT (en);

	ASSERT (sector);

	value = get_local_entity_float_value (en, FLOAT_TYPE_POTENTIAL_SURFACE_TO_SURFACE_THREAT);

	if (value > 0.0)
	{
		side = get_local_entity_int_value (en, INT_TYPE_SIDE);

		radius = get_local_entity_float_value (en, FLOAT_TYPE_SURFACE_SCAN_RANGE) * 10.0 / SECTOR_SIDE_LENGTH;

		convert_float_to_int (radius, &sector_radius);

		sector_radius = max (sector_radius, 1);

		sr2 = sector_radius * sector_radius;

		sx = get_local_entity_int_value (sector, INT_TYPE_X_SECTOR);
		sz = get_local_entity_int_value (sector, INT_TYPE_Z_SECTOR);
	






		for (z = -sector_radius; z <= sector_radius; z ++)
		{
			if (((sz + z) < MIN_MAP_Z_SECTOR) || ((sz + z) > MAX_MAP_Z_SECTOR))
			{
				continue;
			}

			for (x = -sector_radius; x <= sector_radius; x ++)
			{
				if (((sx + x) < MIN_MAP_X_SECTOR) || ((sx + x) > MAX_MAP_X_SECTOR))
				{
					continue;
				}

				scale = (float)((x * x) + (z * z)) / (float)sr2;

				if (scale < 1.0)
				{
					scale = 1.0 - scale;

					update_sector_surface_to_surface_defence_level (sx + x, sz + z, (entity_sides)side, in_use, value * scale);
				}
			}
		}
	}
}





void normalise_surface_defence_imaps (void)
{
	int
		sx,
		sz,
		side;

	entity
		*sec;

	sector
		*raw;

	float
		val;

	for (side = 0; side < NUM_ENTITY_SIDES; side ++)
	{
		for (sz = 0; sz < NUM_MAP_Z_SECTORS; sz++)
		{
			for (sx = 0; sx < NUM_MAP_X_SECTORS; sx++)
			{
				sec = get_local_raw_sector_entity (sx, sz);

				ASSERT (sec);

				raw = (sector*) get_local_entity_data (sec);

				val = raw->surface_to_surface_defence_level [side];

				imap_temp_array [(sz * NUM_MAP_X_SECTORS) + sx] = val;
			}
		}

		normalise_inlfuence_map (imap_temp_array, imaps [IMAP_SURFACE_DEFENCE][side], NUM_MAP_X_SECTORS, NUM_MAP_Z_SECTORS);
	}
}




//
// DEBUG OUTPUT FUNCTIONS
//












































































































































































































































































































































































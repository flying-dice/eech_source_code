

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
}





void overload_city_building_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_CITY_BUILDING]	= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_CITY_BUILDING]	= unpack_local_data;
}





void pack_local_city_building_data (pack_modes mode)
{

	entity
		*en;

	city_building
		*raw;

	int
		city_building_pack_size,
		city_building_count,
		city_building_alive,
		count;

	if ((mode != PACK_MODE_SERVER_SESSION) && (mode != PACK_MODE_CLIENT_SESSION))
	{

		return;
	}

	debug_log ("RLE CITY BLOCK");

	city_building_pack_size = 0;

	city_building_count = 0;

	city_building_alive = TRUE;

	count = 0;

	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_CITY_BUILDING)
		{

			count ++;

			raw = (city_building *) get_local_entity_data (en);

			if (city_building_alive == raw->fix.alive)
			{

				city_building_count ++;
			}
			else
			{

				pack_int_value (NULL, INT_TYPE_LENGTH, city_building_count);
				city_building_pack_size += NUM_LENGTH_BITS;

				pack_int_value (NULL, INT_TYPE_ALIVE, city_building_alive);
				city_building_pack_size += NUM_ALIVE_BITS;

				city_building_alive = raw->fix.alive;

				city_building_count = 1;
			}
		}

		en = get_local_entity_succ (en);
	}

	pack_int_value (NULL, INT_TYPE_LENGTH, city_building_count);
	city_building_pack_size += NUM_LENGTH_BITS;

	pack_int_value (NULL, INT_TYPE_ALIVE, city_building_alive);
	city_building_pack_size += NUM_ALIVE_BITS;

	//debug_log ("CB_PACK: packing %d city_building took %d bytes (%d bits)", count, city_building_pack_size / 8, city_building_pack_size);
}





void unpack_local_city_building_data (pack_modes mode)
{

	city_building
		*raw;

	entity
		*destroy_en,
		*en;

	int
		city_building_count,
		city_building_alive;

	if ((mode != PACK_MODE_SERVER_SESSION) && (mode != PACK_MODE_CLIENT_SESSION))
	{

		return;
	}

	//
	// RLE encoded
	//

	debug_log ("RLE CITY BLOCK");

	en = get_local_entity_list ();

	city_building_count = unpack_int_value (NULL, INT_TYPE_LENGTH);

	city_building_alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

	while (en)
	{

		destroy_en = en;

		en = get_local_entity_succ (en);

		if (get_local_entity_type (destroy_en) == ENTITY_TYPE_CITY_BUILDING)
		{

			if (city_building_count == 0)
			{

				city_building_count = unpack_int_value (NULL, INT_TYPE_LENGTH);

				city_building_alive = unpack_int_value (NULL, INT_TYPE_ALIVE);
			}

			city_building_count --;

			raw = (city_building *) get_local_entity_data (destroy_en);

			raw->fix.alive = city_building_alive;

			if (!city_building_alive)
			{
				raw->fix.object_3d_shape = get_3d_object_destroyed_object_index (raw->fix.object_3d_shape);
			}
		}
	}
}





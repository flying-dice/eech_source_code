

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
}





void overload_site_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_SITE]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_SITE]	= unpack_local_data;
}





void pack_local_site_data (pack_modes mode)
{

	entity
		*en;

	site
		*raw;

	int
		pack_size,
		count,
		alive,
		total;

	if ((mode != PACK_MODE_SERVER_SESSION) && (mode != PACK_MODE_CLIENT_SESSION))
	{

		return;
	}

	debug_log ("RLE SITE");

	pack_size = 0;

	count = 0;

	alive = TRUE;

	total = 0;

	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_SITE)
		{

			total ++;

			raw = (site *) get_local_entity_data (en);

			if (alive == raw->fix.alive)
			{

				count ++;
			}
			else
			{

				pack_int_value (NULL, INT_TYPE_LENGTH, count);
				pack_size += NUM_LENGTH_BITS;

				pack_int_value (NULL, INT_TYPE_ALIVE, alive);
				pack_size += NUM_ALIVE_BITS;

				#if DEBUG_MODULE

				debug_log ("ST_PACK: packing count %d, alive %d", count, alive);

				#endif

				alive = raw->fix.alive;

				count = 1;
			}
		}

		en = get_local_entity_succ (en);
	}

	pack_int_value (NULL, INT_TYPE_LENGTH, count);
	pack_size += NUM_LENGTH_BITS;

	pack_int_value (NULL, INT_TYPE_ALIVE, alive);
	pack_size += NUM_ALIVE_BITS;

	#if DEBUG_MODULE

	debug_log ("ST_PACK: packing count %d, alive %d", count, alive);

	#endif

	//debug_log ("ST_PACK: packing %d entities took %d bytes (%d bits)", total, pack_size / 8, pack_size);
}





void unpack_local_site_data (pack_modes mode)
{

	site
		*raw;

	entity
		*en;

	int
		count,
		alive;

	if ((mode != PACK_MODE_SERVER_SESSION) && (mode != PACK_MODE_CLIENT_SESSION))
	{

		return;
	}

	//
	// RLE encoded
	//

	debug_log ("RLE SITE");

	en = get_local_entity_list ();

	count = unpack_int_value (NULL, INT_TYPE_LENGTH);

	alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

	#if DEBUG_MODULE

	debug_log ("ST_PACK: unpacking count %d, alive %d", count, alive);

	#endif

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_SITE)
		{

			if (count == 0)
			{

				count = unpack_int_value (NULL, INT_TYPE_LENGTH);

				alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

				#if DEBUG_MODULE
			
				debug_log ("ST_PACK: unpacking count %d, alive %d", count, alive);
			
				#endif
			}

			count --;

			raw = (site *) get_local_entity_data (en);

			if (!alive)
			{
				// must be done BEFORE alive flag set, and BEFORE object shape changed
//				subtract_local_entity_importance_from_keysite (en);

				raw->fix.object_3d_shape = get_3d_object_destroyed_object_index (raw->fix.object_3d_shape);
			}

			raw->fix.alive = alive;
		}

		en = get_local_entity_succ (en);
	}
}





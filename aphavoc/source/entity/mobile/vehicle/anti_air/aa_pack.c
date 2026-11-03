

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:
		case PACK_MODE_BROWSE_SESSION:

		{

			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{

			break;
		}
	}
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:
		case PACK_MODE_BROWSE_SESSION:

		{

			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{

			break;
		}
	}
}





void overload_anti_aircraft_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_ANTI_AIRCRAFT]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_ANTI_AIRCRAFT]	= unpack_local_data;
}





void pack_local_anti_aircraft_data (pack_modes mode)
{

	entity
		*en;

	anti_aircraft
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

	debug_log ("RLE SAM & AAA");

	pack_size = 0;

	count = 0;

	alive = TRUE;

	total = 0;

	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_ANTI_AIRCRAFT)
		{

			total ++;

			raw = (anti_aircraft *) get_local_entity_data (en);

			if (alive == raw->vh.mob.alive)
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

				debug_log ("AA_PACK: packing count %d, alive %d", count, alive);

				#endif

				alive = raw->vh.mob.alive;

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

	debug_log ("AA_PACK: packing count %d, alive %d", count, alive);

	#endif

	//debug_log ("AA_PACK: packing %d anti_aircraft took %d bytes (%d bits)", total, pack_size / 8, pack_size);
}





void unpack_local_anti_aircraft_data (pack_modes mode)
{

	anti_aircraft
		*raw;

	entity
		*destroy_en,
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

	debug_log ("RLE SAM & AAA");

	en = get_local_entity_list ();

	count = unpack_int_value (NULL, INT_TYPE_LENGTH);

	alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

	#if DEBUG_MODULE

	debug_log ("AA_PACK: unpacking count %d, alive %d", count, alive);

	#endif

	while (en)
	{

		destroy_en = en;

		en = get_local_entity_succ (en);

		if (get_local_entity_type (destroy_en) == ENTITY_TYPE_ANTI_AIRCRAFT)
		{

			if (count == 0)
			{

				count = unpack_int_value (NULL, INT_TYPE_LENGTH);

				alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

				#if DEBUG_MODULE

				debug_log ("AA_PACK: unpacking count %d, alive %d", count, alive);

				#endif
			}

			count --;

			raw = (anti_aircraft *) get_local_entity_data (destroy_en);

			raw->vh.mob.alive = TRUE;

			if (!alive)
			{

				kill_local_restored_anti_aircraft_entity (destroy_en);
			}
		}
	}
}





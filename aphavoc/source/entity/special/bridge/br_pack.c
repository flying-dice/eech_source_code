

































































#include "project.h"
#include "ai/ai_misc/ai_route.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			break;
		}

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

		{
			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			//
			// always use access functions to set the data
			//

			break;
		}
	}
}





void overload_bridge_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_BRIDGE]	= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_BRIDGE]	= unpack_local_data;
}





void pack_local_bridge_data (pack_modes mode)
{
   entity
      *en;

   bridge
      *raw;

	int
		alive,
		count;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode != PACK_MODE_SERVER_SESSION)
   {
      return;
   }

	//
	// RLE bridge alive flags
	//

	count = 0;

	alive = TRUE;

	debug_log ("RLE BRIDGE");

	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_BRIDGE)
		{

			raw = (bridge *) get_local_entity_data (en);

			if (alive == raw->alive)
			{
				count ++;
			}
			else
			{

				pack_int_value (NULL, INT_TYPE_LENGTH, count);

				pack_int_value (NULL, INT_TYPE_ALIVE, alive);

				#if DEBUG_MODULE

				debug_log ("BR_PACK: packing count %d, alive %d", count, alive);

				#endif

				alive = raw->alive;

				count = 1;
			}
		}
	
		en = get_local_entity_succ (en);
	}

	pack_int_value (NULL, INT_TYPE_LENGTH, count);

	pack_int_value (NULL, INT_TYPE_ALIVE, alive);

	#if DEBUG_MODULE

	debug_log ("BR_PACK: packing count %d, alive %d", count, alive);

	#endif
}





void unpack_local_bridge_data (pack_modes mode)
{
   entity
      *en;

   bridge
      *raw;

	int
		dead_bridge_count,
		alive,
		count;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode != PACK_MODE_SERVER_SESSION)
   {
      return;
   }

	//
	// unpack alive status for each bridge
	//

	debug_log ("RLE BRIDGE");

	count = 0;

	count = unpack_int_value (NULL, INT_TYPE_LENGTH);

	alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

	#if DEBUG_MODULE

	debug_log ("BR_PACK: unpacking count %d, alive %d", count, alive);

	#endif

	en = get_local_entity_list ();

	while (en)
	{
		if (en->type == ENTITY_TYPE_BRIDGE)
		{
			if (count == 0)
			{
				count = unpack_int_value (NULL, INT_TYPE_LENGTH);

				alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

				#if DEBUG_MODULE
			
				debug_log ("BR_PACK: unpacking count %d, alive %d", count, alive);
			
				#endif
			}

			count --;

			raw = (bridge *) get_local_entity_data (en);

			raw->alive = alive;

			if (!alive)
			{
				//
				// Break road network link
				//

				dead_bridge_count = get_road_link_breaks (raw->start_node, raw->end_node);

				dead_bridge_count ++;
				
				set_road_link_breaks (raw->start_node, raw->end_node, dead_bridge_count);
			}
		}

		en = get_local_entity_succ (en);
	}
}





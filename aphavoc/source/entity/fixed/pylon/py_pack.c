

































































#include "project.h"





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





void overload_pylon_pack_functions (void)
{
   fn_pack_local_entity_data[ENTITY_TYPE_PYLON]    = pack_local_data;

   fn_unpack_local_entity_data[ENTITY_TYPE_PYLON]  = unpack_local_data;
}





void pack_local_pylon_data (pack_modes mode)
{
   entity
      *en;

   pylon
      *raw;

	int
		total,
		pack_size,
		count,
		alive;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode == PACK_MODE_BROWSE_SESSION)
   {
      return;
   }

	//
	// RLE encoded
	//

	debug_log ("RLE PYLON");

	pack_size = 0;

	count = 0;

	alive = TRUE;

	total = 0;

	en = get_local_entity_list ();

	while (en)
	{

		if (en->type == ENTITY_TYPE_PYLON)
		{

			total ++;

			raw = (pylon *) get_local_entity_data (en);

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

				debug_log ("PY_PACK: packing count %d, alive %d", count, alive);

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

	debug_log ("PY_PACK: packing count %d, alive %d", count, alive);

	#endif

	//debug_log ("PY_PACK: packing %d pylons took %d bytes (%d bits)", total, pack_size / 8, pack_size);
}





void unpack_local_pylon_data (pack_modes mode)
{
   entity
		*destroy_en,
      *en;

   pylon
      *raw;

	int
		alive,
		count;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode == PACK_MODE_BROWSE_SESSION)
   {
      return;
   }

	//
	// RLE encoded
	//

	debug_log ("RLE PYLON");

	count = unpack_int_value (NULL, INT_TYPE_LENGTH);

	alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

	#if DEBUG_MODULE

	debug_log ("PY_PACK: unpacking count %d, alive %d", count, alive);

	#endif

	en = get_local_entity_list ();

	ASSERT (en);

	while (en)
	{

		destroy_en = en;

		en = get_local_entity_succ (en);

		if (get_local_entity_type (destroy_en) == ENTITY_TYPE_PYLON)
		{

			if (count == 0)
			{

				count = unpack_int_value (NULL, INT_TYPE_LENGTH);

				alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

				#if DEBUG_MODULE
			
				debug_log ("PY_PACK: unpacking count %d, alive %d", count, alive);
			
				#endif
			}

			count --;

			raw = (pylon *) get_local_entity_data (destroy_en);

			raw->fix.alive = alive;

			if (!alive)
			{
				raw->fix.object_3d_shape = get_3d_object_destroyed_object_index (raw->fix.object_3d_shape);
			}
		}
	}
}





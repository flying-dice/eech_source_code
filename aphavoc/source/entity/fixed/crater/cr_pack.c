

































































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
			crater
				*raw;

			raw = (crater *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			//
			// pack fixed data (in exactly the same order as the data is unpacked)
			//

			pack_fixed_data (en, &raw->fix, mode);

			//
			// pack crater data
			//

			pack_float_value (en, FLOAT_TYPE_CRATER_LIFETIME, raw->crater_lifetime);

			pack_float_value (en, FLOAT_TYPE_SCALE, raw->scale);

			// update

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
			int
				index;

			crater
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (crater *) malloc_fast_mem (sizeof (crater));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (crater));

			//
			// unpack fixed data (in exactly the same order as the data was packed)
			//

			unpack_fixed_data (en, &raw->fix, mode);

			//
			// unpack crater data
			//

			raw->crater_lifetime = unpack_float_value (en, FLOAT_TYPE_CRATER_LIFETIME);

			raw->scale = unpack_float_value (en, FLOAT_TYPE_SCALE);

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->fix.position), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

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





void overload_crater_pack_functions (void)
{
   fn_pack_local_entity_data[ENTITY_TYPE_CRATER]    = pack_local_data;

   fn_unpack_local_entity_data[ENTITY_TYPE_CRATER]  = unpack_local_data;
}





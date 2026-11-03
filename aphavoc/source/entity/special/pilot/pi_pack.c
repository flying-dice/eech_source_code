

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
   ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

   switch (mode)
   {

      case PACK_MODE_SERVER_SESSION:

		{
			break;
		}

      case PACK_MODE_CLIENT_SESSION:

      {
         pilot
            *raw;

         raw = (pilot *) get_local_entity_data (en);

         pack_entity_type (get_local_entity_type (en));

         pack_entity_safe_ptr (en);

         //
         // pack data
         //

         pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);

         pack_string (en, STRING_TYPE_PILOTS_NAME, raw->pilots_name);

         pack_int_value (en, INT_TYPE_PILOT_RANK, raw->rank);

         pack_int_value (en, INT_TYPE_KILLS, raw->kills);

         pack_int_value (en, INT_TYPE_UNIQUE_ID, raw->unique_id);

			// crew_role

         pack_list_root (en, LIST_TYPE_PILOT_LOCK, &raw->pilot_lock_root);

         pack_list_link (en, LIST_TYPE_AIRCREW, &raw->aircrew_link);

         pack_list_link (en, LIST_TYPE_PILOT, &raw->pilot_link);

         pack_list_link (en, LIST_TYPE_PLAYER_TASK, &raw->player_task_link);

         pack_int_value (en, INT_TYPE_SIDE, raw->side);

			pack_int_value (en, INT_TYPE_DIFFICULTY_LEVEL, raw->difficulty_level);

         break;
      }

      case PACK_MODE_BROWSE_SESSION:

      {
         pilot
            *raw;

         raw = (pilot *) get_local_entity_data (en);

         pack_entity_type (get_local_entity_type (en));

         pack_entity_safe_ptr (en);

         //
         // pack data
         //

         pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);

         pack_string (en, STRING_TYPE_PILOTS_NAME, raw->pilots_name);

         pack_int_value (en, INT_TYPE_PILOT_RANK, raw->rank);

         pack_int_value (en, INT_TYPE_KILLS, raw->kills);

         pack_int_value (en, INT_TYPE_UNIQUE_ID, raw->unique_id);

			// crew_role

			// aircrew_link

         pack_list_link (en, LIST_TYPE_PILOT, &raw->pilot_link);

         pack_int_value (en, INT_TYPE_SIDE, raw->side);

			// difficulty level

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

		{
			break;
		}

      case PACK_MODE_CLIENT_SESSION:

      {
         int
            index;

         pilot
            *raw;

         //
         // create entity
         //

         index = unpack_entity_safe_index ();

         en = get_free_entity (index);

         set_local_entity_type (en, type);

         raw = (pilot *) malloc_fast_mem (sizeof (pilot));

         set_local_entity_data (en, raw);

         memset (raw, 0, sizeof (pilot));

         //
         // unpack data (in exactly the same order as the data was packed)
         //

         raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

         unpack_string (en, STRING_TYPE_PILOTS_NAME, raw->pilots_name);

         raw->rank = unpack_int_value (en, INT_TYPE_PILOT_RANK);

         raw->kills = unpack_int_value (en, INT_TYPE_KILLS);

         raw->unique_id = unpack_int_value (en, INT_TYPE_UNIQUE_ID);

			raw->crew_role = CREW_ROLE_PILOT;

         unpack_list_root (en, LIST_TYPE_PILOT_LOCK, &raw->pilot_lock_root);

         unpack_list_link (en, LIST_TYPE_AIRCREW, &raw->aircrew_link);

         unpack_list_link (en, LIST_TYPE_PILOT, &raw->pilot_link);

         unpack_list_link (en, LIST_TYPE_PLAYER_TASK, &raw->player_task_link);

         raw->side = unpack_int_value (en, INT_TYPE_SIDE);

			raw->difficulty_level = unpack_int_value (en, INT_TYPE_DIFFICULTY_LEVEL);

         break;
      }

      case PACK_MODE_BROWSE_SESSION:

      {
         int
            index;

         pilot
            *raw;

         //
         // create entity
         //

         index = unpack_entity_safe_index ();

         en = get_free_entity (index);

         set_local_entity_type (en, type);

         raw = (pilot *) malloc_fast_mem (sizeof (pilot));

         set_local_entity_data (en, raw);

         memset (raw, 0, sizeof (pilot));

         //
         // unpack data (in exactly the same order as the data was packed)
         //

         raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

         unpack_string (en, STRING_TYPE_PILOTS_NAME, raw->pilots_name);

         raw->rank = unpack_int_value (en, INT_TYPE_PILOT_RANK);

         raw->kills = unpack_int_value (en, INT_TYPE_KILLS);

         raw->unique_id = unpack_int_value (en, INT_TYPE_UNIQUE_ID);

			raw->crew_role = CREW_ROLE_PILOT;

			// aircrew_link

         unpack_list_link (en, LIST_TYPE_PILOT, &raw->pilot_link);

         raw->side = unpack_int_value (en, INT_TYPE_SIDE);

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





void overload_pilot_pack_functions (void)
{
   fn_pack_local_entity_data[ENTITY_TYPE_PILOT]    = pack_local_data;

   fn_unpack_local_entity_data[ENTITY_TYPE_PILOT]  = unpack_local_data;
}





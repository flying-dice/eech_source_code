

































































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
			cargo
				*raw;

			raw = (cargo *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			//
			// pack aircraft data
			//

			pack_mobile_data (en, &raw->mob, mode);

			//
			// pack cargo data
			//

			if (mode == PACK_MODE_SERVER_SESSION)
			{

				pack_list_root (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_root);
			}


			if (raw->cargo_link.parent)
			{

				pack_int_value (en, INT_TYPE_VALID, TRUE);

				pack_list_link (en, LIST_TYPE_CARGO, &raw->cargo_link);
			}
			else
			{

				pack_int_value (en, INT_TYPE_VALID, FALSE);
			}


			if (raw->movement_dependent_link.parent)
			{

				pack_int_value (en, INT_TYPE_VALID, TRUE);

				pack_list_link (en, LIST_TYPE_MOVEMENT_DEPENDENT, &raw->movement_dependent_link);
			}
			else
			{

				pack_int_value (en, INT_TYPE_VALID, FALSE);
			}


			// terrain_info

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

			cargo
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (cargo *) malloc_fast_mem (sizeof (cargo));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (cargo));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			unpack_mobile_data (en, &raw->mob, mode);

			//
			// cargo
			//

			if (mode == PACK_MODE_SERVER_SESSION)
			{

				unpack_list_root (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_root);
			}


			if (unpack_int_value (en, INT_TYPE_VALID))
			{

				unpack_list_link (en, LIST_TYPE_CARGO, &raw->cargo_link);
			}


			if (unpack_int_value (en, INT_TYPE_VALID))
			{

				unpack_list_link (en, LIST_TYPE_MOVEMENT_DEPENDENT, &raw->movement_dependent_link);
			}


			// terrain_info

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->mob.position), NULL);

			//insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

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





void overload_cargo_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_CARGO]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_CARGO]	= unpack_local_data;
}





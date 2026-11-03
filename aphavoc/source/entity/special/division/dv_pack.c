

































































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
			division
				*raw;

			raw = (division *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			//
			// pack data (in exactly the same order as the data is unpacked)
			//

			pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);

			pack_int_value (en, INT_TYPE_DIVISION_ID, raw->division_id);

			pack_list_root (en, LIST_TYPE_DIVISION, &raw->division_root);

			pack_list_link (en, LIST_TYPE_DIVISION, &raw->division_link);

			pack_list_link (en, LIST_TYPE_DIVISION_HEADQUARTERS, &raw->division_headquarters_link);

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

			division
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (division *) malloc_fast_mem (sizeof (division));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (division));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			raw->division_id = unpack_int_value (en, INT_TYPE_DIVISION_ID);

			unpack_list_root (en, LIST_TYPE_DIVISION, &raw->division_root);

			unpack_list_link (en, LIST_TYPE_DIVISION, &raw->division_link);

			unpack_list_link (en, LIST_TYPE_DIVISION_HEADQUARTERS, &raw->division_headquarters_link);

			set_local_division_name (en, raw->division_name);

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





void overload_division_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_DIVISION]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_DIVISION]	= unpack_local_data;
}





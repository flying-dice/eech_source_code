

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:

/*
		{
			object
				*raw;

			raw = get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			pack_fixed_data (en, &raw->fix, mode);

			pack_vec3d (en, VEC3D_TYPE_OBJECT_SCALING, &raw->object_scaling);

			break;
		}
		*/

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

		/*
		{
			int
				index;

			object
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = malloc_fast_mem (sizeof (object));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (object));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			//
			// fixed
			//

			unpack_fixed_data (en, &raw->fix, mode);

			//
			// object
			//

			unpack_vec3d (en, VEC3D_TYPE_OBJECT_SCALING, &raw->object_scaling);

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->fix.position), NULL);

			break;
		}
		*/

		case PACK_MODE_CLIENT_SESSION:
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





void overload_object_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_OBJECT]	= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_OBJECT]	= unpack_local_data;
}





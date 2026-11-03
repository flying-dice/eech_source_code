

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	person
		*raw;

	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{

			raw = (person *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			// pack vehicle data

			pack_vehicle_data (en, &raw->vh, mode);

			// pack animation state

			pack_float_value (en, FLOAT_TYPE_PERSON_ANIMATION_STATE, raw->person_animation_state);

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
			//
			// create entity
			//
			int
				index;

			person
				*raw;


			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (person *) malloc_fast_mem (sizeof (person));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (person));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			// vehicle data

			unpack_vehicle_data (en, &raw->vh, mode);

			// animation state

			raw->person_animation_state = unpack_float_value (en, FLOAT_TYPE_PERSON_ANIMATION_STATE);

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_VIEW, get_camera_entity (), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->vh.mob.position), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

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





void overload_person_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_PERSON]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_PERSON]		= unpack_local_data;
}





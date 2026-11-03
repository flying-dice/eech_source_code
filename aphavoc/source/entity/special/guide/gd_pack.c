

































































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
			guide
				*raw;

			raw = (guide *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			//
			// pack data (in exactly the same order as the data is unpacked)
			//

			pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);

			pack_list_root (en, LIST_TYPE_FOLLOWER, &raw->follower_root);

			pack_list_link (en, LIST_TYPE_CURRENT_WAYPOINT, &raw->current_waypoint_link);

			pack_list_link (en, LIST_TYPE_GUIDE, &raw->guide_link);

			pack_list_link (en, LIST_TYPE_GUIDE_STACK, &raw->guide_stack_link);

			// update link

			pack_int_value (en, INT_TYPE_GUIDE_POSITION_TYPE, raw->guide_position_type);

			if (guide_position_type_virtual [raw->guide_position_type])
			{
				pack_vec3d (en, VEC3D_TYPE_RELATIVE_POSITION, &raw->position);

				pack_entity_safe_ptr (raw->position_entity);
			}
			else
			{
				pack_vec3d (en, VEC3D_TYPE_POSITION, &raw->position);
			}

			pack_int_value (en, INT_TYPE_VALID_GUIDE_MEMBERS, raw->valid_guide_members);

			pack_int_value (en, INT_TYPE_CHECK_WAYPOINT_ACTION, raw->check_waypoint_action);

			pack_float_value (en, FLOAT_TYPE_VELOCITY, raw->velocity);

			//
			// guide criteria
			//
			{
				int
					loop,
					valid;

				for (loop = 0; loop < NUM_GUIDE_CRITERIA_TYPES; loop ++)
				{
					valid = raw->criteria [loop].valid;
	
					pack_int_value (en, INT_TYPE_VALID, valid);

					if (valid)
					{
						pack_float_value (en, FLOAT_TYPE_GUIDE_CRITERIA_VALUE, raw->criteria [loop].value);
					}
				}
			}

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

			guide
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (guide *) malloc_fast_mem (sizeof (guide));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (guide));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			unpack_list_root (en, LIST_TYPE_FOLLOWER, &raw->follower_root);

			unpack_list_link (en, LIST_TYPE_CURRENT_WAYPOINT, &raw->current_waypoint_link);

			unpack_list_link (en, LIST_TYPE_GUIDE, &raw->guide_link);

			unpack_list_link (en, LIST_TYPE_GUIDE_STACK, &raw->guide_stack_link);

			// update link

			raw->guide_position_type = (guide_position_types) unpack_int_value (en, INT_TYPE_GUIDE_POSITION_TYPE);

			if (guide_position_type_virtual [raw->guide_position_type])
			{
				unpack_vec3d (en, VEC3D_TYPE_RELATIVE_POSITION, &raw->position);

				index = unpack_entity_safe_index ();

				raw->position_entity = get_local_entity_ptr (index);
			}
			else
			{
				unpack_vec3d (en, VEC3D_TYPE_POSITION, &raw->position);
			}

			raw->valid_guide_members = unpack_int_value (en, INT_TYPE_VALID_GUIDE_MEMBERS);

			raw->check_waypoint_action = unpack_int_value (en, INT_TYPE_CHECK_WAYPOINT_ACTION);

			raw->velocity = unpack_float_value (en, FLOAT_TYPE_VELOCITY);

			//
			// guide criteria
			//
			{
				int
					loop,
					valid;

				for (loop = 0; loop < NUM_GUIDE_CRITERIA_TYPES; loop ++)
				{
					valid = unpack_int_value (en, INT_TYPE_VALID);

					raw->criteria [loop].valid = valid;

					if (valid)
					{
						raw->criteria [loop].value = unpack_float_value (en, FLOAT_TYPE_GUIDE_CRITERIA_VALUE);
					}
				}
			}

			//
			// link into system
			//

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





void overload_guide_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_GUIDE]         = pack_local_data;
	
	fn_unpack_local_entity_data[ENTITY_TYPE_GUIDE]       = unpack_local_data;
}







































































#include "project.h"





#define DEBUG_MODULE 0





void pack_fixed_data (entity *en, fixed *raw, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			pack_vec3d (en, VEC3D_TYPE_POSITION, &raw->position);


			if (mode == PACK_MODE_SERVER_SESSION)
			{

				if (raw->task_dependent_root.first_child)
				{

					pack_int_value (en, INT_TYPE_VALID, TRUE);

					pack_list_root (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_root);
				}
				else
				{

					pack_int_value (en, INT_TYPE_VALID, FALSE);
				}
			}



			if (raw->target_root.first_child)
			{

				pack_int_value (en, INT_TYPE_VALID, TRUE);

				pack_list_root (en, LIST_TYPE_TARGET, &raw->target_root);
			}
			else
			{

				pack_int_value (en, INT_TYPE_VALID, FALSE);
			}


			// gunship_target_link

			// sector_link

			pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);

			pack_int_value (en, INT_TYPE_OBJECT_3D_SHAPE, raw->object_3d_shape);

			pack_int_value (en, INT_TYPE_ALIVE, raw->alive);

			pack_int_value (en, INT_TYPE_SIDE, raw->side);

			// gunship_radar_los_clear

			// object_drawn_once_this_frame

			pack_float_value (en, FLOAT_TYPE_HEADING, raw->heading);

			if (raw->heading == 0.0)
			{

				debug_log ("FX_PACK: heading = zero");
			}

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			//
			// cannot update entity at this level as the update may not apply to all entity types below
			//

			break;
		}
	}
}





void unpack_fixed_data (entity *en, fixed *raw, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			unpack_vec3d (en, VEC3D_TYPE_POSITION, &raw->position);


			if (mode == PACK_MODE_SERVER_SESSION)
			{
				if (unpack_int_value (en, INT_TYPE_VALID))
				{

					unpack_list_root (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_root);
				}
			}



			if (unpack_int_value (en, INT_TYPE_VALID))
			{

				unpack_list_root (en, LIST_TYPE_TARGET, &raw->target_root);
			}


			// gunship_target_link

			// sector_link

			raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			raw->object_3d_shape = unpack_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);

			raw->alive = unpack_int_value (en, INT_TYPE_ALIVE);

			raw->side = unpack_int_value (en, INT_TYPE_SIDE);

			// gunship_radar_los_clear

			// object_drawn_once_this_frame

			raw->heading = unpack_float_value (en, FLOAT_TYPE_HEADING);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			//
			// cannot update entity at this level as the update may not apply to all entity types below
			//

			break;
		}
	}
}





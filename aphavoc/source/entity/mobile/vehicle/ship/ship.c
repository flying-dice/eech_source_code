

































































#include "project.h"





#define DEBUG_MODULE 0

#define MAX_META_SMOKE_LIST_ATTACHMENT_POINTS 2





static struct ATTACHED_META_SMOKE_LIST_TABLE
{

	meta_smoke_list_types							type;
	object_3d_sub_object_index_numbers			attachment_point[MAX_META_SMOKE_LIST_ATTACHMENT_POINTS];

} attached_meta_smoke_list_table[] =
	{

		//
		// SHIP WAKES ( REAR )
		//

		{
			META_SMOKE_LIST_TYPE_REAR_SHIP_WAKE,
			{
				OBJECT_3D_SUB_OBJECT_EFFECT_SHIP_WAKE_BACK,
				-1
			}
		},

		//
		// SHIP WAKES ( FRONT )
		//

		{
			META_SMOKE_LIST_TYPE_FRONT_SHIP_WAKE,
			{
				OBJECT_3D_SUB_OBJECT_EFFECT_SHIP_WAKE_FRONT,
				-1
			}
		},

		//
		// TERMINATOR
		//

		{
			(meta_smoke_list_types) -1,
			{
				-1,
				-1
			}
		}
	};





void attach_ship_meta_smoke_lists (entity *en)
{
	int
		item,
		point;

	object_3d_sub_object_index_numbers
		attachment_point;

	meta_smoke_list_types
		type;

	item = 0;

	while (TRUE)
	{
		type = attached_meta_smoke_list_table [item].type;

		if (type == -1)
		{
			break;
		}
		else
		{
			for (point = 0; point < MAX_META_SMOKE_LIST_ATTACHMENT_POINTS; point ++)
			{
				attachment_point = attached_meta_smoke_list_table [item].attachment_point [point];

				if (attachment_point == -1)
				{
					break;
				}


			}
		}

		item ++;
	}
}





void pack_ship_meta_smoke_lists (entity *en, pack_modes mode)
{
	object_3d_instance
		*inst3d;

	int
		item,
		loop,
		count,
		point,
		packed_indices;

	entity
		*special_effect;

	object_3d_sub_object_index_numbers
		attachment_point;

	meta_smoke_list_types
		type;

	if (mode == PACK_MODE_SERVER_SESSION)
	{
		//
		// no need to pack indices as the server can use "ENTITY_INDEX_DONT_CARE"s when unpacking
		//

		return;
	}

	if (get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		inst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);

		if (inst3d)
		{






			item = 0;

			special_effect = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);

			while (TRUE)
			{
				type = attached_meta_smoke_list_table [item].type;

				if (type == -1)
				{
					break;
				}
				else
				{
					count = 0;

					for (point = 0; point < MAX_META_SMOKE_LIST_ATTACHMENT_POINTS; point ++)
					{
						attachment_point = attached_meta_smoke_list_table [item].attachment_point [point];

						if (attachment_point == -1)
						{
							break;
						}
						else
						{
							count += count_sub_object_type_depth (inst3d, attachment_point);
						}
					}

					count *= count_entities_in_meta_smoke_list (type);

					if (count > 0)
					{
						packed_indices = 0;

						for (loop = 0 ; loop < count ; loop ++)
						{
							while (TRUE)
							{
								ASSERT (special_effect);

								if (special_effect->type == ENTITY_TYPE_SMOKE_LIST)
								{
									if (effect_database [get_local_entity_int_value (special_effect, INT_TYPE_ENTITY_SUB_TYPE)].constructed_locally)
									{
										pack_entity_safe_index (get_local_entity_index (special_effect));
	





	
										packed_indices ++;

										special_effect = get_local_entity_child_succ (special_effect, LIST_TYPE_SPECIAL_EFFECT);

										break;
									}
								}
	
								special_effect = get_local_entity_child_succ (special_effect, LIST_TYPE_SPECIAL_EFFECT);
							}
						}

						//
						// check to make sure that only vehicle wake smoke lists are being sent across ( and also the right amount of them )
						//

						ASSERT (packed_indices == count);
					}
				}

				item ++;
			}
		}
	}
}





void unpack_ship_meta_smoke_lists (entity *en, pack_modes mode)
{
	object_3d_instance
		*inst3d;

	int
		item,
		loop,
		count,
		point,
		*entity_index_list;

	meta_smoke_list_types
		type;

	object_3d_sub_object_index_numbers
		attachment_point;

	if (get_local_entity_int_value (en, INT_TYPE_ALIVE))
	{
		inst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);

		if (inst3d)
		{






			item = 0;

			while (TRUE)
			{
				type = attached_meta_smoke_list_table [item].type;

				if (type == -1)
				{
					break;
				}
				else
				{				
					for (point = 0; point < MAX_META_SMOKE_LIST_ATTACHMENT_POINTS; point ++)
					{
						attachment_point = attached_meta_smoke_list_table [item].attachment_point [point];

						if (attachment_point == -1)
						{
							break;
						}
						else
						{				
							count = count_sub_object_type_depth (inst3d, attachment_point);

							if (count >0)
							{
								count *= count_entities_in_meta_smoke_list (type);						

								entity_index_list = (int *) malloc_fast_mem (sizeof (int) * count);

								for (loop = 0; loop < count; loop ++)
								{
									if (mode == PACK_MODE_SERVER_SESSION)
									{
										entity_index_list [loop] = ENTITY_INDEX_DONT_CARE;
									}
									else
									{
										entity_index_list [loop] = unpack_entity_safe_index ();
									}






								}

								//attach_local_meta_smoke_lists_to_object (en, type, attachment_point, entity_index_list);



								free_mem (entity_index_list);
							}
						}
					}
				}

				item ++;
			}
		}
	}
}








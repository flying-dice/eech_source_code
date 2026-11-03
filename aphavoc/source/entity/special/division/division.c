

































































#include "project.h"





#define DEBUG_MODULE	0





struct DIVISION_ID_DATA
{
	int
		valid,
		count,
		next,
		*number_list;
};

typedef struct DIVISION_ID_DATA division_id_data;





static division_id_data
	division_id_database [NUM_ENTITY_SIDES][NUM_ENTITY_SUB_TYPE_DIVISIONS];





entity *create_new_division (entity_sub_types type, entity_sides side, entity *parent, entity *hq, int local_only)
{
	entity
		*new_entity;

	int
		id;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (parent);

	id = get_next_free_division_id (side, type);

	if (local_only)
	{
		new_entity = create_local_entity
		(
			ENTITY_TYPE_DIVISION,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_DIVISION, parent),
			ENTITY_ATTR_PARENT (LIST_TYPE_DIVISION_HEADQUARTERS, hq),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, type),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_DIVISION_ID, id),
			ENTITY_ATTR_END
		);
	}
	else
	{
		new_entity = create_client_server_entity
		(
			ENTITY_TYPE_DIVISION,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_DIVISION, parent),
			ENTITY_ATTR_PARENT (LIST_TYPE_DIVISION_HEADQUARTERS, hq),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, type),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_DIVISION_ID, id),
			ENTITY_ATTR_END
		);
	}

	return new_entity;
}





entity *add_group_to_division (entity *group, entity *specified_division)
{
	entity_sides
		side;

	int
		count,
		max_count,
		group_type,
		keysite_type,
		division_type,
		main_division_type;

	entity
		*en,
		*force,
		*keysite,
		*division;

	vec3d
		*group_pos;
		
	ASSERT (group);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (!get_local_entity_parent (group, LIST_TYPE_DIVISION));

	group_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);
	
	group_pos = get_local_entity_vec3d_ptr (group, VEC3D_TYPE_POSITION);
	
	if (specified_division)
	{
		division = specified_division;
	}
	else
	{
		side = (entity_sides) get_local_entity_int_value (group, INT_TYPE_SIDE);
	
		force = get_local_force_entity (side);
	
		ASSERT (force);
	
		division_type = group_database [group_type].default_group_division;
	
		ASSERT (entity_sub_type_division_valid (division_type));
	
		main_division_type = division_database [division_type].default_group_division;
	
		ASSERT (entity_sub_type_division_valid (main_division_type));
	
		//
		// Find the keysite the company should be attached to
		//

		keysite = NULL;
	
		if (get_local_entity_int_value (group, INT_TYPE_GROUP_LIST_TYPE) == LIST_TYPE_KEYSITE_GROUP)
		{
			keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);

			if (!keysite)
			{
				keysite = get_closest_keysite (NUM_ENTITY_SUB_TYPE_KEYSITES, side, group_pos, 1.0 * KILOMETRE, NULL, TRUE, NULL);
			}
	
			ASSERT (keysite);
	
			keysite_type = get_local_entity_int_value (keysite, INT_TYPE_ENTITY_SUB_TYPE);
	
			if ((keysite_type == ENTITY_SUB_TYPE_KEYSITE_AIRBASE) || (keysite_type == ENTITY_SUB_TYPE_KEYSITE_ANCHORAGE))
			{
				//
				// Use keysite landed at
				//
			}
			else
			{
				//
				// Find nearest keysite (below)
				//

				keysite = NULL;
			}
		}
		else
		{
			if (group_database [group_type].movement_type == MOVEMENT_TYPE_SEA)
			{
				//
				// Find nearest carrier
				//
			
				keysite = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_ANCHORAGE, side, group_pos, 0.0, NULL, TRUE, NULL);
			}
		}

		if (!keysite)
		{
			//
			// Find nearest airbase
			//
			
			keysite = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_AIRBASE, side, group_pos, 0.0, NULL, TRUE, NULL);

			if (!keysite)
			{
				//
				// Find nearest military base
				//
				
				keysite = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_MILITARY_BASE, side, group_pos, 0.0, NULL, TRUE, NULL);

				if (!keysite)
				{
					//
					// Find nearest FARP  
					//
				
					keysite = get_closest_keysite (ENTITY_SUB_TYPE_KEYSITE_FARP, side, group_pos, 0.0, NULL, TRUE, NULL);
				}
			}
		}
	
		ASSERT (keysite);
	
		//
		// Check all companies at this keysite to see if they have vacancies....
		//
	
		max_count = group_database [group_type].maximum_groups_per_division;
	
		division = get_local_entity_first_child (keysite, LIST_TYPE_DIVISION_HEADQUARTERS);
	
		while (division)
		{
			if (get_local_entity_int_value (division, INT_TYPE_ENTITY_SUB_TYPE) == division_type)
			{
				if (max_count == 0)
				{
					break;
				}
	
				count = 0;
	
				en = get_local_entity_first_child (division, LIST_TYPE_DIVISION);
	
				while (en)
				{
					ASSERT (get_local_entity_type (en) == ENTITY_TYPE_GROUP);
	
					count ++;
	
					en = get_local_entity_child_succ (en, LIST_TYPE_DIVISION);
				}
	
				if (count < max_count)
				{
					break;
				}
			}
	
			division = get_local_entity_child_succ (division, LIST_TYPE_DIVISION_HEADQUARTERS);
		}
	
		if (division)
		{
			//
			// found an existing division with space available
			//
		}
		else
		{
			//
			// create a new division
			//
	
			entity
				*main_division;
	
			max_count = division_database [division_type].maximum_groups_per_division;
	
			main_division = get_local_entity_first_child (force, LIST_TYPE_DIVISION);
	
			while (main_division)
			{
				if (get_local_entity_int_value (main_division, INT_TYPE_ENTITY_SUB_TYPE) == main_division_type)
				{
					//
					// check division for vacancies
					//
	
					if (max_count == 0)
					{
						break;
					}
	
					count = 0;
	
					en = get_local_entity_first_child (main_division, LIST_TYPE_DIVISION);
	
					while (en)
					{
						ASSERT (get_local_entity_type (en) == ENTITY_TYPE_DIVISION);
	
						if (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE) == division_type)
						{
							count ++;
						}
						
						en = get_local_entity_child_succ (en, LIST_TYPE_DIVISION);
					}
	
					if (count < max_count)
					{
						break;
					}
				}
	
				main_division = get_local_entity_child_succ (main_division, LIST_TYPE_DIVISION);
			}
	
			if (main_division)
			{
				//
				// found an existing division with space available
				//
			}
			else
			{
				//
				// create a new division
				//
	
				main_division = create_new_division (main_division_type, side, force, NULL, FALSE);
			}
	
			ASSERT (main_division);
	
			division = create_new_division (division_type, side, main_division, keysite, FALSE);
		}
	}

	ASSERT (division);

	set_client_server_entity_parent (group, LIST_TYPE_DIVISION, division);

	return division;
}



















































































































































































	




void initialise_division_database (void)
{
	int
		side,
		type;

	for (side = 0; side < NUM_ENTITY_SIDES; side ++)
	{
		for (type = 0; type < NUM_ENTITY_SUB_TYPE_DIVISIONS; type ++)
		{
			memset (&division_id_database [side][type], 0, sizeof (division_id_data));
		}
	}
}





void deinitialise_division_database (void)
{
	int
		side,
		type;

	for (side = 0; side < NUM_ENTITY_SIDES; side ++)
	{
		for (type = 0; type < NUM_ENTITY_SUB_TYPE_DIVISIONS; type ++)
		{
			if (division_id_database [side][type].valid)
			{










				debug_assert (division_id_database [side][type].count > 0);

				free_mem (division_id_database [side][type].number_list);

				memset (&division_id_database [side][type], 0, sizeof (division_id_data));
			}
		}
	}
}





























































































void add_division_id_data (int side, int type, int count, int *number_list)
{
	debug_assert (division_id_database [side][type].count == 0);

	debug_assert (!division_id_database [side][type].number_list);

	debug_assert (count > 0);

	debug_assert (number_list);

	division_id_database [side][type].valid = TRUE;

	division_id_database [side][type].count = count;

	division_id_database [side][type].next = 0;

	division_id_database [side][type].number_list = (int *) malloc_heap_mem (sizeof (int) * count);

	memcpy (division_id_database [side][type].number_list, number_list, sizeof (int) * count);
}





int get_next_free_division_id (int side, int type)
{
	int
		id,
		index,
		count;

	if (!division_id_database [side][type].valid)
	{
		debug_fatal ("DIVISION: Side %s Type %d not valid", entity_side_short_names [side], type);
	}

	index = division_id_database [side][type].next;

	count = division_id_database [side][type].count;

	debug_assert (count > 0);

	debug_assert (index < count);

	id = division_id_database [side][type].number_list [index];

	index ++;

	if (index == count)
	{
		debug_filtered_log ("DIVISION: WARNING Side %s - %s wrapped (count = %d)",
											entity_side_short_names [side], 
											entity_sub_type_division_names [type],
											count);

		index = 0;
	}

	division_id_database [side][type].next = index;

	return id;
}








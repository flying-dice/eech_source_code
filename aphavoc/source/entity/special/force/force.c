

































































#include "project.h"





#define DEBUG_MODULE		0





int
	force_hardware_update;








entity *get_local_force_entity (entity_sides side)
{

	entity
		*force;

	if (!get_session_entity ())
	{

		return NULL;
	}

	force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (force)
	{

		debug_assert (get_local_entity_type (force) == ENTITY_TYPE_FORCE);

		if (get_local_entity_int_value (force, INT_TYPE_SIDE) == side)
		{

			return force;
		}

		force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
	}

	return NULL;
}




/*
void add_force_campaign_critiera (entity *force_en, campaign_trigger criteria_type, int value1, int value2, int value3, int value4,
											campaign_result_types result, int experience_points, int rank_points, int *experience_variable, int *rank_variable)
{

	force
		*force_raw;

	campaign_trigger_type
		*new_campaign_trigger;

	force_raw = (force *) get_local_entity_data (force_en);

	new_campaign_criteria = (campaign_criteria_type *) malloc_heap_mem (sizeof (campaign_criteria_type));

	memset (new_campaign_criteria, 0, sizeof (campaign_criteria_type));

	new_campaign_criteria->criteria_type = criteria_type;

	new_campaign_criteria->valid = TRUE;

	new_campaign_criteria->value1 = value1;
	new_campaign_criteria->value2 = value2;
	new_campaign_criteria->value3 = value3;
	new_campaign_criteria->value4 = value4;

	new_campaign_criteria->result = result;

	new_campaign_criteria->experience_points = experience_points;
	new_campaign_criteria->experience_variable = experience_variable;

	new_campaign_criteria->rank_points = rank_points;
	new_campaign_criteria->rank_variable = rank_variable;

	new_campaign_criteria->next = force_raw->campaign_criteria;

	force_raw->campaign_criteria = new_campaign_criteria;







}
*/




/*
campaign_criteria_type *get_force_campaign_criteria (entity *force_en, campaign_criteria criteria_type, int sub_type)
{

	force
		*force_raw;
		
	campaign_criteria_type
		*this_criteria;

	force_raw = (force *) get_local_entity_data (force_en);

	this_criteria = force_raw->campaign_criteria;

	while (this_criteria)
	{

		if (this_criteria->criteria_type == criteria_type)
		{

			if ((sub_type == -1) || (this_criteria->type == sub_type))
			{

				return this_criteria;
			}
		}

		this_criteria = this_criteria->next;
	}

	return NULL;
}
*/





void add_to_force_info (entity *force_en, entity *new_en)
{

	force
		*raw;

	int
		index;

	if (!force_en)
	{

		return;
	}

	if (!get_force_hardware_update ())
	{

		return;
	}
		
	raw = (force *) get_local_entity_data (force_en);

	index = get_local_entity_int_value (new_en, INT_TYPE_FORCE_INFO_CATAGORY);

	raw->force_info_current_hardware [index] ++;

	if (get_game_status () == GAME_STATUS_INITIALISED)
	{

		raw->force_info_reserve_hardware [index] --;
	}
}





void remove_from_force_info (entity *force_en, entity *new_en)
{

	force
		*raw;

	int
		index;

	add_mobile_to_force_losses_stats (force_en, new_en);

	if (get_comms_model () == COMMS_MODEL_CLIENT)
	{

		return;
	}
		
	raw = (force *) get_local_entity_data (force_en);

	index = get_local_entity_int_value (new_en, INT_TYPE_FORCE_INFO_CATAGORY);

	raw->force_info_current_hardware [index] --;
}






void replace_into_force_info (entity *force_en, entity *new_en)
{

	force
		*raw;

	int
		index;

	if (get_comms_model () == COMMS_MODEL_CLIENT)
	{

		return;
	}
		
	raw = (force *) get_local_entity_data (force_en);

	index = get_local_entity_int_value (new_en, INT_TYPE_FORCE_INFO_CATAGORY);

	raw->force_info_reserve_hardware [index] ++;

	debug_log ("FORCE: adding %s (%d) back into the reserves", get_local_entity_type_name (new_en), get_local_entity_index (new_en));
}






void add_mobile_to_force_kills_stats (entity *en, entity *victim)
{
	force
		*raw;

	int
		sub_type;

	entity
		*group;

	ASSERT (en);

	ASSERT (victim);

	raw = (force *) get_local_entity_data (en);

	group = get_local_entity_parent (victim, LIST_TYPE_MEMBER);

	ASSERT (group);

	sub_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

	raw->kills [sub_type] ++;
}





void add_mobile_to_force_losses_stats (entity *en, entity *victim)
{
	force
		*raw;

	int
		sub_type;

	entity
		*group;

	ASSERT (en);

	ASSERT (victim);

	raw = (force *) get_local_entity_data (en);

	group = get_local_entity_parent (victim, LIST_TYPE_MEMBER);

	ASSERT (group);

	sub_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

	raw->losses [sub_type] ++;
}





void add_group_type_to_force_info (entity *en, entity_sub_types group_type)
{
	force
		*raw;
		
	ASSERT (en);

	raw = (force *) get_local_entity_data (en);

	raw->group_count [group_type] ++;
}





void remove_group_type_from_force_info (entity *en, entity_sub_types group_type)
{
	force
		*raw;
		
	ASSERT (en);

	raw = (force *) get_local_entity_data (en);

	ASSERT (raw->group_count [group_type] > 0);

	raw->group_count [group_type] --;
}





int get_local_force_entity_group_count (entity *en, entity_sub_types group_type)
{
	force
		*raw;
		
	ASSERT (en);

	raw = (force *) get_local_entity_data (en);

	return (raw->group_count [group_type]);
}





































































































































































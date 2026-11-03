

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
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





void overload_force_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_FORCE]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_FORCE]	= unpack_local_data;
}





void pack_local_force_data (entity *en, pack_modes mode)
{

	int
		count,
		loop;

	force
		*raw;

	campaign_criteria_type
		*campaign_criteria;

	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	ASSERT (en);

	raw = (force *) get_local_entity_data (en);
	
	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:

		{
         pack_string (en, STRING_TYPE_FORCE_NAME, raw->force_name);

			// keysite_force
	
			// pilot	root
	
			pack_list_root (en, LIST_TYPE_DIVISION, &raw->division_root);
	
			pack_list_root (en, LIST_TYPE_CAMPAIGN_OBJECTIVE, &raw->campaign_objective_root);
	
			// air_registry_root
			// ground_registry_root
			// sea_registry_root

//			pack_list_root (en, LIST_TYPE_INDEPENDENT_GROUP, &raw->independent_group_root);
	
			// force_link
		
			// update link
		
			// task generation //////////////////////////////////////////////
		
			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_TASKS; loop ++)
			{
				pack_int_value (en, INT_TYPE_VALID, raw->task_generation [loop].valid);

				pack_int_value (en, INT_TYPE_TASK_GENERATION, raw->task_generation [loop].created);
			}

		
			// campaign criteria ////////////////////////////////////////////
		
			count = 0;
		
			campaign_criteria = raw->campaign_criteria;
		
			while (campaign_criteria)
			{
				count ++;
		
				campaign_criteria = campaign_criteria->next;
			}
		
			pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT, count);
		
			campaign_criteria = raw->campaign_criteria;
		
			while (campaign_criteria)
			{
		
				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_TYPE, campaign_criteria->criteria_type);
		
				pack_int_value (en, INT_TYPE_VALID, campaign_criteria->valid);
		
				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_RESULT, campaign_criteria->result);
		
				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value1);
		
				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value2);
		
				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value3);
		
				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value4);
		
				campaign_criteria = campaign_criteria->next;
			}

		
			// force_info_criteria
		
			for (loop = 0; loop < NUM_FORCE_INFO_CATAGORIES; loop ++)
			{
				pack_int_value (en, INT_TYPE_VALUE, raw->force_info_current_hardware [loop]);
		
				pack_int_value (en, INT_TYPE_VALUE, raw->force_info_reserve_hardware [loop]);
			}

	
			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_GROUPS; loop ++)
			{
				pack_int_value (en, INT_TYPE_VALUE, raw->kills [loop]);
				pack_int_value (en, INT_TYPE_VALUE, raw->losses [loop]);
//				pack_int_value (en, INT_TYPE_VALUE, raw->group_count [loop]);
			}

			pack_float_value (en, FLOAT_TYPE_SLEEP, raw->sleep);
		
			pack_int_value (en, INT_TYPE_FORCE_ATTITUDE, raw->force_attitude);

			// sector_count
		
			pack_int_value (en, INT_TYPE_COLOUR, raw->colour);
		
			pack_int_value (en, INT_TYPE_SIDE, raw->side);
	
			break;
		}


		case PACK_MODE_CLIENT_SESSION:

		{

         pack_string (en, STRING_TYPE_FORCE_NAME, raw->force_name);

			// keysite_force
	
			pack_list_root (en, LIST_TYPE_PILOT, &raw->pilot_root);

			pack_list_root (en, LIST_TYPE_DIVISION, &raw->division_root);

			pack_list_root (en, LIST_TYPE_CAMPAIGN_OBJECTIVE, &raw->campaign_objective_root);

			//	air_registry_root
			//	ground_registry_root
			//	sea_registry_root

//			pack_list_root (en, LIST_TYPE_INDEPENDENT_GROUP, &raw->independent_group_root);
	
			pack_list_link (en, LIST_TYPE_FORCE, &raw->force_link);

			// update link

			// task generation //////////////////////////////////////////////

			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_TASKS; loop ++)
			{
				pack_int_value (en, INT_TYPE_VALID, raw->task_generation [loop].valid);

				pack_int_value (en, INT_TYPE_TASK_GENERATION, raw->task_generation [loop].created);
			}


			// campaign criteria ////////////////////////////////////////////

			count = 0;

			campaign_criteria = raw->campaign_criteria;

			while (campaign_criteria)
			{
				count ++;

				campaign_criteria = campaign_criteria->next;
			}

			pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT, count);

			campaign_criteria = raw->campaign_criteria;

			while (campaign_criteria)
			{

				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_TYPE, campaign_criteria->criteria_type);

				pack_int_value (en, INT_TYPE_VALID, campaign_criteria->valid);

				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_RESULT, campaign_criteria->result);

				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value1);

				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value2);

				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value3);

				pack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE, campaign_criteria->value4);

				campaign_criteria = campaign_criteria->next;
			}


			// force_info_criteria

			for (loop = 0; loop < NUM_FORCE_INFO_CATAGORIES; loop ++)
			{
				pack_int_value (en, INT_TYPE_VALUE, raw->force_info_current_hardware [loop]);

				pack_int_value (en, INT_TYPE_VALUE, raw->force_info_reserve_hardware [loop]);
			}


			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_GROUPS; loop ++)
			{
				pack_int_value (en, INT_TYPE_VALUE, raw->kills [loop]);
				pack_int_value (en, INT_TYPE_VALUE, raw->losses [loop]);
//				pack_int_value (en, INT_TYPE_VALUE, raw->group_count [loop]);
			}

			pack_float_value (en, FLOAT_TYPE_SLEEP, raw->sleep);

			pack_int_value (en, INT_TYPE_FORCE_ATTITUDE, raw->force_attitude);

			pack_int_value (en, INT_TYPE_FORCE_SECTOR_COUNT, raw->sector_count);

			pack_int_value (en, INT_TYPE_COLOUR, raw->colour);

			pack_int_value (en, INT_TYPE_SIDE, raw->side);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
         pack_string (en, STRING_TYPE_FORCE_NAME, raw->force_name);

			// keysite_force_root

			pack_list_root (en, LIST_TYPE_PILOT, &raw->pilot_root);

			pack_list_link (en, LIST_TYPE_FORCE, &raw->force_link);

			// update link

			// task_generation

			// campaign_criteria

			// force_info_catagories

			// sleep

			pack_int_value (en, INT_TYPE_FORCE_ATTITUDE, raw->force_attitude);

			pack_int_value (en, INT_TYPE_COLOUR, raw->colour);

			pack_int_value (en, INT_TYPE_SIDE, raw->side);

			break;
		}
	}
}





void unpack_local_force_data (entity *en, pack_modes mode)
{

	int
		count,
		loop;

	force
		*raw;

	campaign_criteria_type
		*last_campaign_criteria,
		*campaign_criteria;

	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	ASSERT (en);

	raw = (force *) get_local_entity_data (en);
		
	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:

		{
         unpack_string (en, STRING_TYPE_FORCE_NAME, raw->force_name);

			// keysite_force
		
			// pilot root
		
			unpack_list_root (en, LIST_TYPE_DIVISION, &raw->division_root);
	
			unpack_list_root (en, LIST_TYPE_CAMPAIGN_OBJECTIVE, &raw->campaign_objective_root);
	
			// air_registry_root
			// ground_registry_root
			// sea_registry_root
	
//			unpack_list_root (en, LIST_TYPE_INDEPENDENT_GROUP, &raw->independent_group_root);
	
			// force_link
		
			// update link
		
			// task generation //////////////////////////////////////////////
		
			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_TASKS; loop ++)
			{
				raw->task_generation [loop].valid = unpack_int_value (en, INT_TYPE_VALID);

				raw->task_generation [loop].created = unpack_int_value (en, INT_TYPE_TASK_GENERATION);
			}

		
			// campaign criteria ////////////////////////////////////////////
		
			count = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT);
	
			last_campaign_criteria = NULL;
	
			while (count)
			{
	
				campaign_criteria = (campaign_criteria_type *) malloc_heap_mem (sizeof (campaign_criteria_type));
	
				memset (campaign_criteria, 0, sizeof (campaign_criteria_type));
		
				campaign_criteria->criteria_type = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_TYPE);
		
				campaign_criteria->valid = unpack_int_value (en, INT_TYPE_VALID);
		
				campaign_criteria->result = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_RESULT);
		
				campaign_criteria->value1 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);
		
				campaign_criteria->value2 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);
		
				campaign_criteria->value3 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);
		
				campaign_criteria->value4 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);
		
				campaign_criteria->next = last_campaign_criteria;
	
				raw->campaign_criteria = campaign_criteria;
	
				last_campaign_criteria = campaign_criteria;
	
				count --;
			}

		
			// force_info_criteria
		
			for (loop = 0; loop < NUM_FORCE_INFO_CATAGORIES; loop ++)
			{
				raw->force_info_current_hardware [loop] = unpack_int_value (en, INT_TYPE_VALUE);
		
				raw->force_info_reserve_hardware [loop] = unpack_int_value (en, INT_TYPE_VALUE);
			}

	
			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_GROUPS; loop ++)
			{
				raw->kills [loop] = unpack_int_value (en, INT_TYPE_VALUE);
				raw->losses [loop] = unpack_int_value (en, INT_TYPE_VALUE);
//				raw->group_count [loop] = unpack_int_value (en, INT_TYPE_VALUE);
			}

			raw->sleep = unpack_float_value (en, FLOAT_TYPE_SLEEP);
		
			raw->force_attitude = unpack_int_value (en, INT_TYPE_FORCE_ATTITUDE);
		
			// sector_count
		
			raw->colour = unpack_int_value (en, INT_TYPE_COLOUR);
		
			raw->side = unpack_int_value (en, INT_TYPE_SIDE);
	
			break;
		}

		case PACK_MODE_CLIENT_SESSION:

		{
			//
			// create entity
			//

			debug_assert (get_free_entity (get_local_entity_safe_index (en)));

			set_local_entity_type (en, ENTITY_TYPE_FORCE);

			raw = (force *) malloc_fast_mem (sizeof (force));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (force));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

         unpack_string (en, STRING_TYPE_FORCE_NAME, raw->force_name);

			// keysite_force
	
			unpack_list_root (en, LIST_TYPE_PILOT, &raw->pilot_root);

			unpack_list_root (en, LIST_TYPE_DIVISION, &raw->division_root);

			unpack_list_root (en, LIST_TYPE_CAMPAIGN_OBJECTIVE, &raw->campaign_objective_root);

			//	air_registry_root
			//	ground_registry_root
			//	sea_registry_root

//			unpack_list_root (en, LIST_TYPE_INDEPENDENT_GROUP, &raw->independent_group_root);
	
			unpack_list_link (en, LIST_TYPE_FORCE, &raw->force_link);

			// update_link


			// task_generation
			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_TASKS; loop ++)
			{
				raw->task_generation [loop].valid = unpack_int_value (en, INT_TYPE_VALID);

				raw->task_generation [loop].created = unpack_int_value (en, INT_TYPE_TASK_GENERATION);
			}



			// campaign criteria 
			raw->campaign_criteria = NULL;

			count = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_COUNT);

			while (count)
			{

				count --;

				campaign_criteria = (campaign_criteria_type *) malloc_heap_mem (sizeof (campaign_criteria_type));

				memset (campaign_criteria, 0, sizeof (campaign_criteria_type));

				campaign_criteria->next = raw->campaign_criteria;

				raw->campaign_criteria = campaign_criteria;

				campaign_criteria->criteria_type = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_TYPE);

				campaign_criteria->valid = unpack_int_value (en, INT_TYPE_VALID);

				campaign_criteria->result = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_RESULT);

				campaign_criteria->value1 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);

				campaign_criteria->value2 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);

				campaign_criteria->value3 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);

				campaign_criteria->value4 = unpack_int_value (en, INT_TYPE_CAMPAIGN_CRITERIA_VALUE);
			}



			// force_info_criteria
			for (loop = 0; loop < NUM_FORCE_INFO_CATAGORIES; loop ++)
			{
				raw->force_info_current_hardware [loop] = unpack_int_value (en, INT_TYPE_VALUE);

				raw->force_info_reserve_hardware [loop] = unpack_int_value (en, INT_TYPE_VALUE);
			}
			// force_info_criteria

			for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_GROUPS; loop ++)
			{
				raw->kills [loop] = unpack_int_value (en, INT_TYPE_VALUE);
				raw->losses [loop] = unpack_int_value (en, INT_TYPE_VALUE);
//				raw->group_count [loop] = unpack_int_value (en, INT_TYPE_VALUE);
			}

			raw->sleep = unpack_float_value (en, FLOAT_TYPE_SLEEP);

			raw->force_attitude = unpack_int_value (en, INT_TYPE_FORCE_ATTITUDE);

			raw->sector_count = unpack_int_value (en, INT_TYPE_FORCE_SECTOR_COUNT);

			raw->colour = unpack_int_value (en, INT_TYPE_COLOUR);

			raw->side = unpack_int_value (en, INT_TYPE_SIDE);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			//
			// create entity
			//

			debug_assert (get_free_entity (get_local_entity_safe_index (en)));

			set_local_entity_type (en, ENTITY_TYPE_FORCE);

			raw = (force *) malloc_fast_mem (sizeof (force));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (force));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

         unpack_string (en, STRING_TYPE_FORCE_NAME, raw->force_name);

			// keysite_force_root

			unpack_list_root (en, LIST_TYPE_PILOT, &raw->pilot_root);

			unpack_list_link (en, LIST_TYPE_FORCE, &raw->force_link);

			// update_link

			// task_generation

			// campaign_criteria

			// force_info_catagories

			// sleep

			raw->force_attitude = unpack_int_value (en, INT_TYPE_FORCE_ATTITUDE);

			raw->colour = unpack_int_value (en, INT_TYPE_COLOUR);

			raw->side = unpack_int_value (en, INT_TYPE_SIDE);

			break;
		}
	}
}






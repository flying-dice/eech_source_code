

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	mobile
		*raw;







	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ALIVE:

		{
			raw->alive = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_KILL_NEXT_UPDATE:

		{
			raw->kill_next_update = value;

			break;
		}









		case INT_TYPE_SIDE:

		{
			raw->side = value;

			break;
		}
		/*

		case INT_TYPE_TIMER:

		{
			raw->timer = value;

			break;
		}
		*/









		case INT_TYPE_UPDATED:
		case INT_TYPE_MOVED:
		case INT_TYPE_ROTATED:

		{
			int
				mask,
				index,
				offset;

			//
			// Coded around the entity system to avoid spinning through the update list twice
			// (which Vtune shows up as very slow). Update_entities is declared in update.c
			// You may only set the value to TRUE. It is set to FALSE by the memset in update.c
			//

			ASSERT (value == TRUE);

			index = get_local_entity_index (en) >> 5;

			offset = get_local_entity_index (en) & 0x1f;

			mask = 1 << offset;

			if (type == INT_TYPE_UPDATED)
				updated_entities [index] |= mask;
			else if (type == INT_TYPE_MOVED)
				moved_entities [index] |= mask;
			else
				rotated_entities [index] |= mask;

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX:

		{
			raw->gunship_radar_target_index = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}























































static int get_local_int_value (entity *en, int_types type)
{
	mobile
		*raw;

	int
		value;

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ALIVE:

		{
			value = raw->alive;

			break;
		}

		case INT_TYPE_IDENTIFY_MOBILE:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_INSIDE_HANGAR:

		{
			value = FALSE;

			if (get_local_entity_int_value (en, INT_TYPE_LANDED))
			{
				entity
					*landing;

				int
					formation_position;

				landing = get_local_group_member_landing_entity_from_keysite (en);

				if (landing)
				{
					formation_position = get_local_entity_int_value (en, INT_TYPE_FORMATION_POSITION);

					if (get_local_entity_int_value (landing, INT_TYPE_INSIDE_HANGAR) & (1 << formation_position))
					{
						value = TRUE;
					}
				}
			}

			break;
		}

		case INT_TYPE_KILL_NEXT_UPDATE:

		{
			value = raw->kill_next_update;

			break;
		}









		case INT_TYPE_SIDE:

		{
			value = raw->side;

			break;
		}









		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE);

			break;
		}
		/*

		case INT_TYPE_TIMER:

		{
			value = raw->timer;

			break;
		}
		*/







		case INT_TYPE_UPDATED:
		case INT_TYPE_MOVED:
		case INT_TYPE_ROTATED:

		{
			int
				mask,
				index,
				offset;

			index = get_local_entity_index (en) >> 5;

			offset = get_local_entity_index (en) & 0x1f;

			mask = 1 << offset;

			if (type == INT_TYPE_UPDATED)
				value = updated_entities [index] &= mask;
			else if (type == INT_TYPE_MOVED)
				value = moved_entities [index] &= mask;
			else
				value = rotated_entities [index] &= mask;

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX:

		{
			value = raw->gunship_radar_target_index;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}

	return (value);
}










































































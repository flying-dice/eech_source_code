

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	fixed_wing
		*raw;







	raw = (fixed_wing *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_OPERATIONAL_STATE:

		{
			int
				state1,
				state2;

			state1 = get_local_entity_int_value (en, INT_TYPE_MOBILE_MOVING);

			raw->ac.operational_state = value;

			state2 = get_local_entity_int_value (en, INT_TYPE_MOBILE_MOVING);

			if ((state1 == FALSE) && (state2 == TRUE))
			{
				//
				// wasn't moving, but is now
				//


			}
			else if ((state1 == TRUE) && (state2 == FALSE))
			{
				//
				// was moving, but isn't anymore
				//




			}

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
	fixed_wing
		*raw;

	int
		value;

	raw = (fixed_wing *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_IDENTIFY_FIXED_WING:

		{
			value = 1;

			break;
		}

		case INT_TYPE_RADAR_ON:

		{
			//
			// fixed wing aircraft must have an airborne radar and a target for the radar to be on
			//

			value = FALSE;

			if (aircraft_database[raw->ac.mob.sub_type].threat_type == THREAT_TYPE_AIRBORNE_RADAR)
			{
				if (raw->ac.mob.target_link.parent != NULL)
				{
					value = TRUE;
				}
			}

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			if (get_local_entity_int_value (en, INT_TYPE_LANDED))
			{
				value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_GROUNDED_AIRCRAFT | TASK_TARGET_TYPE_FIXED_WING);
			}
			else
			{
				value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_AIRBORNE_AIRCRAFT | TASK_TARGET_TYPE_FIXED_WING);
			}

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{
			value = aircraft_database[raw->ac.mob.sub_type].armor_level;

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


























































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	fixed_wing
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

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

				resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ENGINE_LOOPING1);
			}
			else if ((state1 == TRUE) && (state2 == FALSE))
			{
				//
				// was moving, but isn't anymore
				//

				pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ENGINE_LOOPING1, 0.5);

				pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ENGINE_LOOPING2, 0.5);
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





static void set_remote_int_value (entity *en, int_types type, int value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_INT_VALUE, en, type, value);
}





static void set_server_int_value (entity *en, int_types type, int value)
{
	validate_client_server_local_fn ();

	set_local_int_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_int_value (en, type, value);
}





static void set_client_int_value (entity *en, int_types type, int value)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_int_value (en, type, value);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_int_value (en, type, value);
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





void overload_fixed_wing_int_value_functions (void)
{
	fn_get_local_entity_int_value				[ENTITY_TYPE_FIXED_WING][INT_TYPE_IDENTIFY_FIXED_WING]			= get_local_int_value;

	fn_set_local_entity_int_value				[ENTITY_TYPE_FIXED_WING][INT_TYPE_OPERATIONAL_STATE]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_FIXED_WING][INT_TYPE_OPERATIONAL_STATE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_FIXED_WING][INT_TYPE_OPERATIONAL_STATE][COMMS_MODEL_CLIENT]	= set_client_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_FIXED_WING][INT_TYPE_RADAR_ON]			  				= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_FIXED_WING][INT_TYPE_TASK_TARGET_TYPE]				= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_FIXED_WING][INT_TYPE_ARMOR_LEVEL]		= get_local_int_value;
}





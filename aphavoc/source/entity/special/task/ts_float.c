

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	task
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_EXPIRE_TIMER:

		{
			raw->expire_timer = value;

			break;
		}

		case FLOAT_TYPE_RATING:

		{
			raw->rating = value;

			break;
		}

		case FLOAT_TYPE_START_TIME:

		{
			raw->start_time = value;

			break;
		}

		case FLOAT_TYPE_STOP_TIMER:

		{
			raw->stop_timer = value;

			break;
		}

		case FLOAT_TYPE_TASK_PRIORITY:

		{
			raw->task_priority = value;

			break;
		}

		case FLOAT_TYPE_TASK_USER_DATA:

		{
			raw->task_user_data = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}





static void set_remote_float_value (entity *en, float_types type, float value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}





static void set_server_float_value (entity *en, float_types type, float value)
{
	validate_client_server_local_fn ();

	set_local_float_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_float_value (en, type, value);
}





static void set_client_float_value (entity *en, float_types type, float value)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_float_value (en, type, value);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_float_value (en, type, value);
	}
}





static float get_local_float_value (entity *en, float_types type)
{
	task
		*raw;

	float
		value;

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ELAPSED_MISSION_TIME:

		{
			if (raw->task_state == TASK_STATE_ASSIGNED)
			{
				ASSERT (get_session_entity ());

				value = get_local_entity_float_value (get_session_entity (), FLOAT_TYPE_ELAPSED_TIME) - raw->start_time;
			}
			else
			{
				value = 0.0;
			}

			break;
		}

		case FLOAT_TYPE_EXPIRE_TIMER:

		{
			value = raw->expire_timer;

			break;
		}

		case FLOAT_TYPE_RATING:

		{
			value = raw->rating;

			break;
		}

		case FLOAT_TYPE_START_TIME:

		{
			value = raw->start_time;

			break;
		}

		case FLOAT_TYPE_STOP_TIMER:

		{
			value = raw->stop_timer;

			break;
		}

		case FLOAT_TYPE_TASK_DEFAULT_PRIORITY:

		{
			value = task_database [raw->sub_type].task_priority;

			break;
		}

		case FLOAT_TYPE_TASK_PRIORITY:

		{
			value = raw->task_priority;

			break;
		}

		case FLOAT_TYPE_TASK_USER_DATA:

		{
			value = raw->task_user_data;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}





void overload_task_float_value_functions (void)
{

	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_ELAPSED_MISSION_TIME]				= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_TASK][FLOAT_TYPE_EXPIRE_TIMER]							= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_EXPIRE_TIMER]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_EXPIRE_TIMER][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_EXPIRE_TIMER][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_EXPIRE_TIMER]							= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_TASK][FLOAT_TYPE_RATING]									= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_RATING]									= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_RATING][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_RATING][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_RATING]									= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_TASK][FLOAT_TYPE_START_TIME]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_START_TIME]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_START_TIME][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_START_TIME][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_START_TIME]								= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_TASK][FLOAT_TYPE_STOP_TIMER]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_STOP_TIMER]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_STOP_TIMER][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_STOP_TIMER][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_STOP_TIMER]								= get_local_float_value;

	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_DEFAULT_PRIORITY]					= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_PRIORITY]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_PRIORITY]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_PRIORITY][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_PRIORITY][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_PRIORITY]								= get_local_float_value;

	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_USER_DATA]							= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_USER_DATA]							= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_USER_DATA][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_USER_DATA][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_TASK][FLOAT_TYPE_TASK_USER_DATA]							= get_local_float_value;
}








































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	task
		*raw;







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






















































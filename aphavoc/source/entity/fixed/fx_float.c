

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	fixed
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_HEADING:

		{
			raw->heading = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_remote_float_value (entity *en, float_types type, float value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value);
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_server_float_value (entity *en, float_types type, float value)
{
	validate_client_server_local_fn ();

	set_local_float_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_float_value (en, type, value);
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

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

#endif





#if GET_LOCAL_REQUIRED

static float get_local_float_value (entity *en, float_types type)
{
	fixed
		*raw;

	float
		value;

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			value = raw->position.y;

			break;
		}

		case FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE:

		{
			value = fixed_3d_object_database[raw->object_3d_shape].importance;

			break;
		}

		case FLOAT_TYPE_FIXED_STRUCTURE_HEIGHT:

		{
			object_3d_bounds
				*bounding_box;

			if (raw->object_3d_shape != OBJECT_3D_INVALID_OBJECT_INDEX)
			{
				bounding_box = get_object_3d_bounding_box (raw->object_3d_shape);

				value = bounding_box->ymax;
			}

			break;
		}


		case FLOAT_TYPE_HEADING:

		{
			value = raw->heading;

			break;
		}

		case FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE:

		{
			value = object_3d_information_database[raw->object_3d_shape].maximum_distance;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK:

		{
			//
			// Target priority is object importance - scaled down so any mobile threats have higher priority
			//
			
			value = get_local_entity_float_value (en, FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE);

			value *= 0.25;

			break;
		}

		case FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK:

		{
			//
			// Target priority is object importance - scaled down so any mobile threats have higher priority
			//
			
			value = get_local_entity_float_value (en, FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE);

			value *= 0.25;

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

#endif





void overload_fixed_float_value_functions (entity_types type)
{
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_ALTITUDE]							= get_local_float_value;

	fn_get_local_entity_float_value				[type][FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE]		= get_local_float_value;

	fn_get_local_entity_float_value				[type][FLOAT_TYPE_FIXED_STRUCTURE_HEIGHT]			= get_local_float_value;

	fn_set_local_entity_raw_float_value			[type][FLOAT_TYPE_HEADING]								= set_local_float_value;
	fn_set_local_entity_float_value				[type][FLOAT_TYPE_HEADING]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_HEADING][COMMS_MODEL_SERVER]	= set_server_float_value;
	fn_set_client_server_entity_float_value	[type][FLOAT_TYPE_HEADING][COMMS_MODEL_CLIENT]	= set_client_float_value;
	fn_get_local_entity_float_value				[type][FLOAT_TYPE_HEADING]								= get_local_float_value;

	fn_get_local_entity_float_value				[type][FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE]	= get_local_float_value;

	fn_get_local_entity_float_value				[type][FLOAT_TYPE_TARGET_PRIORITY_AIR_ATTACK]		= get_local_float_value;

	fn_get_local_entity_float_value				[type][FLOAT_TYPE_TARGET_PRIORITY_GROUND_ATTACK]	= get_local_float_value;
}





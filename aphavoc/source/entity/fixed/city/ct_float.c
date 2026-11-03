

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	city
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_FLOAT_VALUE, en, type, value);

	#endif

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

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
	city
		*raw;

	float
		value;

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE:

		{
			env_3d
				*env;

			float
				yonder,
				hither;

			env = get_3d_active_environment ();

			ASSERT (env);

			get_3d_view_distances (env, &yonder, &hither);

			value = yonder;

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





void overload_city_float_value_functions (void)
{
	fn_set_local_entity_raw_float_value			[ENTITY_TYPE_CITY][FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE]								= set_local_float_value;
	fn_set_local_entity_float_value				[ENTITY_TYPE_CITY][FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE]								= set_local_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_CITY][FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE][COMMS_MODEL_SERVER]		= set_server_float_value;
	fn_set_client_server_entity_float_value	[ENTITY_TYPE_CITY][FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE][COMMS_MODEL_CLIENT]		= set_client_float_value;
	fn_get_local_entity_float_value				[ENTITY_TYPE_CITY][FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE]								= get_local_float_value;
}





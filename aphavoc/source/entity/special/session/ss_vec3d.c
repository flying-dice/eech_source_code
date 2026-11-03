

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	session
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_WEATHER_POSITION:

		{
			raw->weather_position = *v;

			break;
		}

		case VEC3D_TYPE_WEATHER_VELOCITY:

		{
			raw->weather_velocity = *v;

			break;
		}

		case VEC3D_TYPE_WIND_DIRECTION_VECTOR:

		{
			raw->wind_direction_vector = *v;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_POSITION:

		{
			raw->wind_effect_position = *v;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_VELOCITY:

		{
			raw->wind_effect_velocity = *v;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}





static void set_remote_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_VEC3D, en, type, v);
}





static void set_server_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	ASSERT (v);

	validate_client_server_local_fn ();

	set_local_vec3d (en, type, v);

	validate_client_server_remote_fn ();

	set_remote_vec3d (en, type, v);
}





static void set_client_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	ASSERT (v);

	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_vec3d (en, type, v);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_vec3d (en, type, v);
	}
}





static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	session
		*raw;

	ASSERT (v);

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_WEATHER_POSITION:

		{
			*v = raw->weather_position;

			break;
		}

		case VEC3D_TYPE_WEATHER_VELOCITY:

		{
			*v = raw->weather_velocity;

			break;
		}

		case VEC3D_TYPE_WIND_DIRECTION_VECTOR:

		{
			*v = raw->wind_direction_vector;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_POSITION:

		{
			*v = raw->wind_effect_position;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_VELOCITY:

		{
			*v = raw->wind_effect_velocity;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}





static vec3d *get_local_vec3d_ptr (entity *en, vec3d_types type)
{
	session
		*raw;

	vec3d
		*v;

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_WEATHER_POSITION:

		{
			v = &raw->weather_position;

			break;
		}

		case VEC3D_TYPE_WEATHER_VELOCITY:

		{
			v = &raw->weather_velocity;

			break;
		}

		case VEC3D_TYPE_WIND_DIRECTION_VECTOR:

		{
			v = &raw->wind_direction_vector;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_POSITION:

		{
			v = &raw->wind_effect_position;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_VELOCITY:

		{
			v = &raw->wind_effect_velocity;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}

	return (v);
}





void overload_session_vec3d_functions (void)
{
	fn_set_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_POSITION]								= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_POSITION][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_POSITION][COMMS_MODEL_CLIENT]	= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_POSITION]							 	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_POSITION]								= get_local_vec3d_ptr;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_VELOCITY]								= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_VELOCITY][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_VELOCITY][COMMS_MODEL_CLIENT]	= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_VELOCITY]							 	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_SESSION][VEC3D_TYPE_WEATHER_VELOCITY]								= get_local_vec3d_ptr;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_DIRECTION_VECTOR]									= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_DIRECTION_VECTOR][COMMS_MODEL_SERVER]		= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_DIRECTION_VECTOR][COMMS_MODEL_CLIENT]		= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_DIRECTION_VECTOR]							 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_DIRECTION_VECTOR]									= get_local_vec3d_ptr;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_POSITION]							= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_POSITION][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_POSITION][COMMS_MODEL_CLIENT]	= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_POSITION]						 	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_POSITION]							= get_local_vec3d_ptr;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_VELOCITY]							= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_VELOCITY][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_VELOCITY][COMMS_MODEL_CLIENT]	= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_VELOCITY]						 	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_SESSION][VEC3D_TYPE_WIND_EFFECT_VELOCITY]							= get_local_vec3d_ptr;
}





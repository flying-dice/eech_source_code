

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	city
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_remote_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_VEC3D, en, type, v);
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_server_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	ASSERT (v);

	validate_client_server_local_fn ();

	set_local_vec3d (en, type, v);

	validate_client_server_remote_fn ();

	set_remote_vec3d (en, type, v);
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

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

#endif





#if GET_LOCAL_REQUIRED

static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	city
		*raw;

	ASSERT (v);

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_BOUNDING_BOX_MAX:

		{
			*v = raw->bounding_box_max;

			break;
		}

		case VEC3D_TYPE_BOUNDING_BOX_MIN:

		{
			*v = raw->bounding_box_min;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}

#endif





#if GET_LOCAL_REQUIRED

static vec3d *get_local_vec3d_ptr (entity *en, vec3d_types type)
{
	city
		*raw;

	vec3d
		*v;

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_BOUNDING_BOX_MAX:

		{
			v = &raw->bounding_box_max;

			break;
		}

		case VEC3D_TYPE_BOUNDING_BOX_MIN:

		{
			v = &raw->bounding_box_min;

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

#endif





void overload_city_vec3d_functions (void)
{
	fn_get_local_entity_vec3d				[ENTITY_TYPE_CITY][VEC3D_TYPE_BOUNDING_BOX_MAX]	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_CITY][VEC3D_TYPE_BOUNDING_BOX_MAX]	= get_local_vec3d_ptr;

	fn_get_local_entity_vec3d				[ENTITY_TYPE_CITY][VEC3D_TYPE_BOUNDING_BOX_MIN]	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_CITY][VEC3D_TYPE_BOUNDING_BOX_MIN]	= get_local_vec3d_ptr;
}





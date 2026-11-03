

































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	scenic
		*raw;

	ASSERT (v);







	raw = get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_OBJECT_SCALING:

		{

			raw->object_scaling = *v;

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
	scenic
		*raw;

	ASSERT (v);

	raw = get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_OBJECT_SCALING:

		{
			*v = raw->object_scaling;

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
	scenic
		*raw;

	vec3d
		*v;

	raw = get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_OBJECT_SCALING:

		{
			v = &raw->object_scaling;

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





void overload_scenic_vec3d_functions (void)
{
	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_SCENIC][VEC3D_TYPE_OBJECT_SCALING]									= set_local_vec3d;
	fn_set_local_entity_vec3d				[ENTITY_TYPE_SCENIC][VEC3D_TYPE_OBJECT_SCALING]									= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SCENIC][VEC3D_TYPE_OBJECT_SCALING][COMMS_MODEL_SERVER]			= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_SCENIC][VEC3D_TYPE_OBJECT_SCALING][COMMS_MODEL_CLIENT]			= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_SCENIC][VEC3D_TYPE_OBJECT_SCALING]							 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_SCENIC][VEC3D_TYPE_OBJECT_SCALING]									= get_local_vec3d_ptr;
}
*/






































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	mobile
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{

			raw->position = *v;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	mobile
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			raw->motion_vector = *v;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{

			entity
				*old_sector,
				*new_sector;

			raw->position = *v;

			old_sector = get_local_entity_parent (en, LIST_TYPE_SECTOR);

			new_sector = get_local_sector_entity (v);

			if (old_sector != new_sector)
			{
				delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, new_sector, NULL);
			}

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
	mobile
		*raw;

	ASSERT (v);

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			*v = raw->motion_vector;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->position;

			break;
		}

		case VEC3D_TYPE_XV:

		{
			*v = raw->xv;

			break;
		}

		case VEC3D_TYPE_YV:

		{
			*v = raw->yv;

			break;
		}

		case VEC3D_TYPE_ZV:

		{
			*v = raw->zv;

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
	mobile
		*raw;

	vec3d
		*v;

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			v = &raw->motion_vector;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->position;

			break;
		}

		case VEC3D_TYPE_XV:

		{
			v = &raw->xv;

			break;
		}

		case VEC3D_TYPE_YV:

		{
			v = &raw->yv;

			break;
		}

		case VEC3D_TYPE_ZV:

		{
			v = &raw->zv;

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





void overload_mobile_vec3d_functions (entity_types type)
{
	fn_set_local_entity_raw_vec3d			[type][VEC3D_TYPE_MOTION_VECTOR]								= set_local_vec3d;
	fn_set_local_entity_vec3d				[type][VEC3D_TYPE_MOTION_VECTOR]								= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[type][VEC3D_TYPE_MOTION_VECTOR][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[type][VEC3D_TYPE_MOTION_VECTOR][COMMS_MODEL_CLIENT]	= set_client_vec3d;
	fn_get_local_entity_vec3d				[type][VEC3D_TYPE_MOTION_VECTOR]								= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[type][VEC3D_TYPE_MOTION_VECTOR]								= get_local_vec3d_ptr;

	fn_set_local_entity_raw_vec3d			[type][VEC3D_TYPE_POSITION]									= set_local_raw_vec3d;
	fn_set_local_entity_vec3d				[type][VEC3D_TYPE_POSITION]									= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[type][VEC3D_TYPE_POSITION][COMMS_MODEL_SERVER]			= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[type][VEC3D_TYPE_POSITION][COMMS_MODEL_CLIENT]			= set_client_vec3d;
	fn_get_local_entity_vec3d				[type][VEC3D_TYPE_POSITION]							 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[type][VEC3D_TYPE_POSITION]									= get_local_vec3d_ptr;

	fn_get_local_entity_vec3d				[type][VEC3D_TYPE_XV]									 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[type][VEC3D_TYPE_XV]											= get_local_vec3d_ptr;

	fn_get_local_entity_vec3d				[type][VEC3D_TYPE_YV]									 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[type][VEC3D_TYPE_YV]											= get_local_vec3d_ptr;

	fn_get_local_entity_vec3d				[type][VEC3D_TYPE_ZV]									 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[type][VEC3D_TYPE_ZV]											= get_local_vec3d_ptr;
}





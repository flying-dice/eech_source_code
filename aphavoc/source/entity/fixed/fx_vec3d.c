

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	fixed
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (fixed *) get_local_entity_data (en);

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
	fixed
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			int
				old_x_sec,
				old_z_sec,
				new_x_sec,
				new_z_sec;

			entity
				*parent;

			get_x_sector (old_x_sec, raw->position.x);
			get_z_sector (old_z_sec, raw->position.z);

			get_x_sector (new_x_sec, v->x);
			get_z_sector (new_z_sec, v->z);

			if ((old_x_sec != new_x_sec) || (old_z_sec != new_z_sec))
			{
				delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

				raw->position = *v;

				parent = get_local_raw_sector_entity (new_x_sec, new_z_sec);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);
			}
			else
			{
				raw->position = *v;
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
	fixed
		*raw;

	ASSERT (v);

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->position;

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
	fixed
		*raw;

	vec3d
		*v;

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->position;

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





void overload_fixed_vec3d_functions (entity_types type)
{

	//
	// VEC3D_TYPE_POSITION
	//


	fn_set_local_entity_raw_vec3d			[type][VEC3D_TYPE_POSITION]									= set_local_raw_vec3d;

	fn_set_local_entity_vec3d				[type][VEC3D_TYPE_POSITION]									= set_local_vec3d;

	fn_set_client_server_entity_vec3d	[type][VEC3D_TYPE_POSITION][COMMS_MODEL_SERVER]			= set_server_vec3d;

	fn_set_client_server_entity_vec3d	[type][VEC3D_TYPE_POSITION][COMMS_MODEL_CLIENT]			= set_client_vec3d;

	fn_get_local_entity_vec3d				[type][VEC3D_TYPE_POSITION]							 		= get_local_vec3d;

	fn_get_local_entity_vec3d_ptr			[type][VEC3D_TYPE_POSITION]									= get_local_vec3d_ptr;
}





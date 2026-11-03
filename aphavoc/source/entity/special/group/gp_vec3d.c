

































































#include "project.h"





#define DEBUG_MODULE 0





static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	group
		*raw;

	ASSERT (v);

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			#ifdef DEBUG
			
			debug_fatal ("Invalid call to get group position");

			#endif

			break;
		}

		case VEC3D_TYPE_LAST_KNOWN_POSITION:

		{
			#ifdef DEBUG
			
			debug_fatal ("Invalid call to get group last known position. Use ptr not vec3d copy");

			#endif

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
	group
		*raw;

	vec3d
		*v;

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			entity
				*member;

			member = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_GROUP_LEADER);

			if (member)
			{
				v = get_local_entity_vec3d_ptr (member, type);
			}
			else
			{
				v = NULL;
			}

			break;
		}

		case VEC3D_TYPE_LAST_KNOWN_POSITION:

		{
			v = &raw->last_known_position;

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






static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	group
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_LAST_KNOWN_POSITION:

		{
			raw->last_known_position = *v;

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





void overload_group_vec3d_functions (void)
{
	fn_get_local_entity_vec3d				[ENTITY_TYPE_GROUP][VEC3D_TYPE_POSITION]								= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_GROUP][VEC3D_TYPE_POSITION]								= get_local_vec3d_ptr;

	fn_get_local_entity_vec3d				[ENTITY_TYPE_GROUP][VEC3D_TYPE_LAST_KNOWN_POSITION]								= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_GROUP][VEC3D_TYPE_LAST_KNOWN_POSITION]								= get_local_vec3d_ptr;

	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_GROUP][VEC3D_TYPE_LAST_KNOWN_POSITION]								= set_local_vec3d;
	fn_set_local_entity_vec3d				[ENTITY_TYPE_GROUP][VEC3D_TYPE_LAST_KNOWN_POSITION]								= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_GROUP][VEC3D_TYPE_LAST_KNOWN_POSITION][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_GROUP][VEC3D_TYPE_LAST_KNOWN_POSITION][COMMS_MODEL_CLIENT]	= set_client_vec3d;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	task
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (task *) get_local_entity_data (en);

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

	task
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{

			// is this needed for mobile landing ?
			//
			// if parent == keysite then mobile landing entity
			//
/*
			entity
				*parent;

			parent = get_local_entity_parent (en, LIST_TYPE_GENERIC_TASK);

			if (parent)
			{

				if (parent->type == ENTITY_TYPE_KEYSITE)
				{

					notify_local_entity (ENTITY_MESSAGE_SET_ENTITY_POSITION, parent, en, v, 0.0, 0.0);
				}
			}
*/
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
	task
		*raw;

	ASSERT (v);

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->position;

			break;
		}

		case VEC3D_TYPE_START_POSITION:

		{
			*v = raw->route_nodes [0];

			break;
		}

		case VEC3D_TYPE_STOP_POSITION:

		{
			*v = raw->route_nodes [raw->route_length - 1];

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
	task
		*raw;

	vec3d
		*v;

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->position;

			break;
		}

		case VEC3D_TYPE_START_POSITION:

		{
			v = &raw->route_nodes [0];

			break;
		}

		case VEC3D_TYPE_STOP_POSITION:

		{
			v = &raw->route_nodes [raw->route_length - 1];

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





void overload_task_vec3d_functions (void)
{

	//
	// VEC3D_TYPE_POSITION
	//


	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_TASK][VEC3D_TYPE_POSITION]											= set_local_raw_vec3d;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_TASK][VEC3D_TYPE_POSITION]											= set_local_vec3d;

	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_TASK][VEC3D_TYPE_POSITION][COMMS_MODEL_SERVER]					= set_server_vec3d;

	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_TASK][VEC3D_TYPE_POSITION][COMMS_MODEL_CLIENT]					= set_client_vec3d;

	fn_get_local_entity_vec3d				[ENTITY_TYPE_TASK][VEC3D_TYPE_POSITION]									 		= get_local_vec3d;

	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_TASK][VEC3D_TYPE_POSITION]											= get_local_vec3d_ptr;


	//
	// VEC3D_TYPE_START_POSITION
	//


	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_TASK][VEC3D_TYPE_START_POSITION]									= set_local_raw_vec3d;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_TASK][VEC3D_TYPE_START_POSITION]									= set_local_vec3d;

	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_TASK][VEC3D_TYPE_START_POSITION][COMMS_MODEL_SERVER]			= set_server_vec3d;

	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_TASK][VEC3D_TYPE_START_POSITION][COMMS_MODEL_CLIENT]			= set_client_vec3d;

	fn_get_local_entity_vec3d				[ENTITY_TYPE_TASK][VEC3D_TYPE_START_POSITION]							 		= get_local_vec3d;

	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_TASK][VEC3D_TYPE_START_POSITION]									= get_local_vec3d_ptr;


	//
	// VEC3D_TYPE_STOP_POSITION
	//


	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_TASK][VEC3D_TYPE_STOP_POSITION]										= set_local_raw_vec3d;

	fn_set_local_entity_vec3d				[ENTITY_TYPE_TASK][VEC3D_TYPE_STOP_POSITION]										= set_local_vec3d;

	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_TASK][VEC3D_TYPE_STOP_POSITION][COMMS_MODEL_SERVER]			= set_server_vec3d;

	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_TASK][VEC3D_TYPE_STOP_POSITION][COMMS_MODEL_CLIENT]			= set_client_vec3d;

	fn_get_local_entity_vec3d				[ENTITY_TYPE_TASK][VEC3D_TYPE_STOP_POSITION]							 			= get_local_vec3d;

	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_TASK][VEC3D_TYPE_STOP_POSITION]										= get_local_vec3d_ptr;
}





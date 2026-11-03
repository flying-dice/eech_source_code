

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	guide
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			raw->position = *v;

			break;
		}

		case VEC3D_TYPE_RELATIVE_POSITION:

		{
			raw->position = *v;			// No - this isn't a cut'n'paste error !!

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
	guide
		*raw;

	ASSERT (v);

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_GUIDE_POSITION:

		{
			get_local_guide_position (en, v);

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->position;			

			break;
		}

		case VEC3D_TYPE_RELATIVE_POSITION:

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
	guide
		*raw;

	vec3d
		*v;

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->position;

		  	break;
		}

		case VEC3D_TYPE_RELATIVE_POSITION:

		{
			v = &raw->position;			// No - this isn't a cut'n'paste error !!

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





void overload_guide_vec3d_functions (void)
{
	fn_get_local_entity_vec3d				[ENTITY_TYPE_GUIDE][VEC3D_TYPE_GUIDE_POSITION]					 				= get_local_vec3d;

	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_GUIDE][VEC3D_TYPE_POSITION]											= set_local_vec3d;
	fn_set_local_entity_vec3d				[ENTITY_TYPE_GUIDE][VEC3D_TYPE_POSITION]											= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_GUIDE][VEC3D_TYPE_POSITION][COMMS_MODEL_SERVER]				= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_GUIDE][VEC3D_TYPE_POSITION][COMMS_MODEL_CLIENT]				= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_GUIDE][VEC3D_TYPE_POSITION]							 				= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_GUIDE][VEC3D_TYPE_POSITION]											= get_local_vec3d_ptr;

	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_GUIDE][VEC3D_TYPE_RELATIVE_POSITION]								= set_local_vec3d;
	fn_set_local_entity_vec3d				[ENTITY_TYPE_GUIDE][VEC3D_TYPE_RELATIVE_POSITION]								= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_GUIDE][VEC3D_TYPE_RELATIVE_POSITION][COMMS_MODEL_SERVER]	= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_GUIDE][VEC3D_TYPE_RELATIVE_POSITION][COMMS_MODEL_CLIENT]	= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_GUIDE][VEC3D_TYPE_RELATIVE_POSITION]							 	= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_GUIDE][VEC3D_TYPE_RELATIVE_POSITION]								= get_local_vec3d_ptr;
}





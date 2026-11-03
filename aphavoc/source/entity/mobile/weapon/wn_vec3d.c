

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	weapon
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			raw->mob.position = *v;

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
	weapon
		*raw;

	ASSERT (v);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_VEC3D, en, type, v);

	#endif

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			entity
				*special_effect,
				*old_sector,
				*new_sector;

			raw->mob.position = *v;

			old_sector = get_local_entity_parent (en, LIST_TYPE_SECTOR);

			new_sector = get_local_sector_entity (v);

			if (old_sector != new_sector)
			{
				delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, new_sector, NULL);
			}

			//
			// notify special effects of move
			//

			special_effect = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);

			while (special_effect)
			{
				#if DEBUG_MODULE

				debug_log ("WEAPON VEC3D: sending set entity position %f, %f, %f", raw->mob.position.x, raw->mob.position.y, raw->mob.position.z);

				#endif

				notify_local_entity (ENTITY_MESSAGE_SET_ENTITY_POSITION, special_effect, en, &raw->mob.position, 0.0, 0.0);

				special_effect = get_local_entity_child_succ (special_effect, LIST_TYPE_SPECIAL_EFFECT);
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
	weapon
		*raw;

	ASSERT (v);

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->mob.position;

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
	weapon
		*raw;

	vec3d
		*v;

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->mob.position;

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





void overload_weapon_vec3d_functions (void)
{
	fn_set_local_entity_raw_vec3d			[ENTITY_TYPE_WEAPON][VEC3D_TYPE_POSITION]									= set_local_raw_vec3d;
	fn_set_local_entity_vec3d				[ENTITY_TYPE_WEAPON][VEC3D_TYPE_POSITION]									= set_local_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_WEAPON][VEC3D_TYPE_POSITION][COMMS_MODEL_SERVER]		= set_server_vec3d;
	fn_set_client_server_entity_vec3d	[ENTITY_TYPE_WEAPON][VEC3D_TYPE_POSITION][COMMS_MODEL_CLIENT]		= set_client_vec3d;
	fn_get_local_entity_vec3d				[ENTITY_TYPE_WEAPON][VEC3D_TYPE_POSITION]							 		= get_local_vec3d;
	fn_get_local_entity_vec3d_ptr			[ENTITY_TYPE_WEAPON][VEC3D_TYPE_POSITION]									= get_local_vec3d_ptr;
}





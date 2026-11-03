

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	task
		*raw;

	ASSERT (v);







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


































































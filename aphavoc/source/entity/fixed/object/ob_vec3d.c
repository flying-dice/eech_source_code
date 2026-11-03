

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	object
		*raw;

	ASSERT (v);







	raw = (object *) get_local_entity_data (en);

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





























































static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	object
		*raw;

	ASSERT (v);

	raw = (object *) get_local_entity_data (en);

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
	object
		*raw;

	vec3d
		*v;

	raw = (object *) get_local_entity_data (en);

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



















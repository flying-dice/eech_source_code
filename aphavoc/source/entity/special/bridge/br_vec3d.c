

































































#include "project.h"





#define DEBUG_MODULE 0





static vec3d *get_local_vec3d_ptr (entity *en, vec3d_types type)
{
	vec3d
		*v;

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			entity
				*segment;

			segment = get_local_entity_first_child (en, LIST_TYPE_SEGMENT);

			ASSERT (segment);

			v = get_local_entity_vec3d_ptr (segment, type);

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














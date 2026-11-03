

































































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






			break;
		}

		case VEC3D_TYPE_LAST_KNOWN_POSITION:

		{






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

















































































































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	helicopter
		*raw;

	ASSERT (v);







	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_COVER_POSITION:

		{
			raw->cover_position = *v;

			break;
		}

		case VEC3D_TYPE_WAIT_POSITION:

		{
			raw->wait_position = *v;

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
	helicopter
		*raw;

	ASSERT (v);

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_COVER_POSITION:

		{
			*v = raw->cover_position;

			break;
		}

		case VEC3D_TYPE_WAIT_POSITION:

		{
			*v = raw->wait_position;

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
	helicopter
		*raw;

	vec3d
		*v;

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_COVER_POSITION:

		{
			v = &raw->cover_position;

			break;
		}

		case VEC3D_TYPE_WAIT_POSITION:

		{
			v = &raw->wait_position;

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

































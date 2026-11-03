

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	city
		*raw;

	ASSERT (v);







	raw = get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}

#endif





#if SET_CLIENT_SERVER_REQUIRED














#endif





#if SET_CLIENT_SERVER_REQUIRED














#endif





#if SET_CLIENT_SERVER_REQUIRED



















#endif





#if GET_LOCAL_REQUIRED

static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	city
		*raw;

	ASSERT (v);

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_BOUNDING_BOX_MAX:

		{
			*v = raw->bounding_box_max;

			break;
		}

		case VEC3D_TYPE_BOUNDING_BOX_MIN:

		{
			*v = raw->bounding_box_min;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}

#endif





#if GET_LOCAL_REQUIRED

static vec3d *get_local_vec3d_ptr (entity *en, vec3d_types type)
{
	city
		*raw;

	vec3d
		*v;

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_BOUNDING_BOX_MAX:

		{
			v = &raw->bounding_box_max;

			break;
		}

		case VEC3D_TYPE_BOUNDING_BOX_MIN:

		{
			v = &raw->bounding_box_min;

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

#endif


















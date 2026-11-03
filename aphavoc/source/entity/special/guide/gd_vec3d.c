

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	guide
		*raw;

	ASSERT (v);







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




























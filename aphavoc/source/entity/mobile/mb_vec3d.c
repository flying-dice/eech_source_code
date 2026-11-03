

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	mobile
		*raw;

	ASSERT (v);







	raw = (mobile *) get_local_entity_data (en);

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
	mobile
		*raw;

	ASSERT (v);







	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			raw->motion_vector = *v;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{

			entity
				*old_sector,
				*new_sector;

			raw->position = *v;

			old_sector = get_local_entity_parent (en, LIST_TYPE_SECTOR);

			new_sector = get_local_sector_entity (v);

			if (old_sector != new_sector)
			{
				delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, new_sector, NULL);
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





























































static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	mobile
		*raw;

	ASSERT (v);

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			*v = raw->motion_vector;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->position;

			break;
		}

		case VEC3D_TYPE_XV:

		{
			*v = raw->xv;

			break;
		}

		case VEC3D_TYPE_YV:

		{
			*v = raw->yv;

			break;
		}

		case VEC3D_TYPE_ZV:

		{
			*v = raw->zv;

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
	mobile
		*raw;

	vec3d
		*v;

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			v = &raw->motion_vector;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->position;

			break;
		}

		case VEC3D_TYPE_XV:

		{
			v = &raw->xv;

			break;
		}

		case VEC3D_TYPE_YV:

		{
			v = &raw->yv;

			break;
		}

		case VEC3D_TYPE_ZV:

		{
			v = &raw->zv;

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



































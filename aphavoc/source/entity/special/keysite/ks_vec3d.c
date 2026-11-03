

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	keysite
		*raw;

	ASSERT (v);







	raw = (keysite *) get_local_entity_data (en);

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
	keysite
		*raw;

	ASSERT (v);







	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			int
				old_x_sec,
				old_z_sec,
				new_x_sec,
				new_z_sec;

			entity
				*parent;

			get_x_sector (old_x_sec, raw->position.x);
			get_z_sector (old_z_sec, raw->position.z);

			get_x_sector (new_x_sec, v->x);
			get_z_sector (new_z_sec, v->z);

			if ((old_x_sec != new_x_sec) || (old_z_sec != new_z_sec))
			{
				delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

				raw->position = *v;

				parent = get_local_raw_sector_entity (new_x_sec, new_z_sec);

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);
			}
			else
			{
				raw->position = *v;
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
	keysite
		*raw;

	ASSERT (v);

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

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
	keysite
		*raw;

	vec3d
		*v;

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->position;

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






























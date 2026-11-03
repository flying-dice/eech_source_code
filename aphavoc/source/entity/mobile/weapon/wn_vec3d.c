

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	weapon
		*raw;

	ASSERT (v);







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



















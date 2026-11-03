

































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	cargo
		*raw;







	raw = (cargo *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}
}
*/




static void *get_local_ptr_value (entity *en, ptr_types type)
{
	cargo
		*raw;

	void
		*ptr;

	raw = (cargo *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_TERRAIN_FACE_NORMAL:

		{
			ptr = get_3d_terrain_point_data_normal (&raw->terrain_info);

			break;
		}

		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}

	return (ptr);
}














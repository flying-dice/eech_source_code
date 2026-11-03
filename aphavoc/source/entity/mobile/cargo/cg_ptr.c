

































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	cargo
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

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





void overload_cargo_ptr_value_functions (void)
{
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_CARGO][PTR_TYPE_TERRAIN_FACE_NORMAL]							= get_local_ptr_value;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	vehicle
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_SOUND_EFFECT_DATA:

		{
			raw->sound_effect_data = (system_sound_effect *) ptr;

			break;
		}

		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}
}





static void *get_local_ptr_value (entity *en, ptr_types type)
{
	vehicle
		*raw;

	void
		*ptr;

	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_INSTANCE_3D_OBJECT:

		{
			ptr = raw->inst3d;

			break;
		}

		case PTR_TYPE_SOUND_EFFECT_DATA:

		{
			ptr = raw->sound_effect_data;

			break;
		}

		case PTR_TYPE_TERRAIN_FACE_NORMAL:

		{
			ptr = get_3d_terrain_point_data_normal (&raw->terrain_info);

			break;
		}

		case PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY:

		{
			ptr = raw->weapon_package_status_array;

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





void overload_vehicle_ptr_value_functions (entity_types type)
{
	fn_get_local_entity_ptr_value	[type][PTR_TYPE_INSTANCE_3D_OBJECT]				= get_local_ptr_value;

	fn_set_local_entity_ptr_value	[type][PTR_TYPE_SOUND_EFFECT_DATA]				= set_local_ptr_value;
	fn_get_local_entity_ptr_value	[type][PTR_TYPE_SOUND_EFFECT_DATA]				= get_local_ptr_value;

	fn_get_local_entity_ptr_value	[type][PTR_TYPE_TERRAIN_FACE_NORMAL]			= get_local_ptr_value;

	fn_get_local_entity_ptr_value	[type][PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY]	= get_local_ptr_value;
}





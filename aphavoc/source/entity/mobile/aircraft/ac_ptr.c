

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	aircraft
		*raw;







	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{










		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}
}





static void *get_local_ptr_value (entity *en, ptr_types type)
{
	aircraft
		*raw;

	void
		*ptr;

	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{

























		case PTR_TYPE_TERRAIN_INFO:

		{
			ptr = &raw->terrain_info;

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


























































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	vehicle
		*raw;







	raw = (vehicle *) get_local_entity_data (en);

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
	vehicle
		*raw;

	void
		*ptr;

	raw = (vehicle *) get_local_entity_data (en);

	switch (type)
	{

























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





















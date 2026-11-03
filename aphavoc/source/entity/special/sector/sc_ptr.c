

































































#define DEBUG_MODULE 0





#include "project.h"





static void *get_local_ptr_value (entity *en, ptr_types type)
{
	sector
		*raw;

	void
		*ptr;

	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}

	return (ptr);
}





void overload_sector_ptr_value_functions (void)
{
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	pylon
		*raw;







	raw = (pylon *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_SUCC:

		{
			raw->succ = (entity *) ptr;

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
	pylon
		*raw;

	void
		*ptr;

	raw = (pylon *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_SUCC:

		{
			ptr = raw->succ;

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

















































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	pylon
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

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





void overload_pylon_ptr_value_functions (void)
{
	fn_set_local_entity_ptr_value[ENTITY_TYPE_PYLON][PTR_TYPE_SUCC] = set_local_ptr_value;
	fn_get_local_entity_ptr_value[ENTITY_TYPE_PYLON][PTR_TYPE_SUCC] = get_local_ptr_value;
}





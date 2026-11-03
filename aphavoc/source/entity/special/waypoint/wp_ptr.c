

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	waypoint
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_POSITION_ENTITY:

		{
			if (ptr)
			{
				debug_fatal ("WP_PTR: REMOVE");
			}
			raw->position_entity = (entity *) ptr;

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
	waypoint
		*raw;

	void
		*ptr;

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_POSITION_ENTITY:

		{
			debug_fatal ("WP_PTR: REMOVE");
			ptr = raw->position_entity;

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





void overload_waypoint_ptr_value_functions (void)
{


	//
	// INT_TYPE_ENTITY
	//


	fn_set_local_entity_ptr_value				[ENTITY_TYPE_WAYPOINT][PTR_TYPE_POSITION_ENTITY]					= set_local_ptr_value;

	fn_get_local_entity_ptr_value				[ENTITY_TYPE_WAYPOINT][PTR_TYPE_POSITION_ENTITY]					= get_local_ptr_value;
}





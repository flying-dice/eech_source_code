

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	waypoint
		*raw;







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























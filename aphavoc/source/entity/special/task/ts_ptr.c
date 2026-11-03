

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	task
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_RETURN_KEYSITE:

		{
			raw->return_keysite = (entity *) ptr;

			break;
		}

		case PTR_TYPE_ROUTE_DEPENDENTS:

		{
			raw->route_dependents = (entity **) ptr;

			break;
		}

		case PTR_TYPE_ROUTE_FORMATION_TYPES:

		{
			raw->route_formation_types = (formation_types *) ptr;

			break;
		}

		case PTR_TYPE_ROUTE_NODE:

		{
			raw->route_nodes = (vec3d *) ptr;

			break;
		}

		case PTR_TYPE_ROUTE_WAYPOINT_TYPES:

		{
			raw->route_waypoint_types = (entity_sub_types *) ptr;

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
	task
		*raw;

	void
		*ptr;

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_RETURN_KEYSITE:

		{
			ptr = raw->return_keysite;

			break;
		}

		case PTR_TYPE_ROUTE_DEPENDENTS:

		{
			ptr = raw->route_dependents;

			break;
		}

		case PTR_TYPE_ROUTE_FORMATION_TYPES:

		{
			ptr = raw->route_formation_types;

			break;
		}

		case PTR_TYPE_ROUTE_NODE:

		{
			ptr = raw->route_nodes;

			break;
		}

		case PTR_TYPE_ROUTE_WAYPOINT_TYPES:

		{
			ptr = raw->route_waypoint_types;

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





void overload_task_ptr_value_functions (void)
{
	fn_set_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_RETURN_KEYSITE]									= set_local_ptr_value;
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_RETURN_KEYSITE]									= get_local_ptr_value;

	fn_set_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_DEPENDENTS]								= set_local_ptr_value;
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_DEPENDENTS]								= get_local_ptr_value;

	fn_set_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_FORMATION_TYPES]							= set_local_ptr_value;
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_FORMATION_TYPES]							= get_local_ptr_value;

	fn_set_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_NODE]										= set_local_ptr_value;
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_NODE]										= get_local_ptr_value;

	fn_set_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_WAYPOINT_TYPES]							= set_local_ptr_value;
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_TASK][PTR_TYPE_ROUTE_WAYPOINT_TYPES]							= get_local_ptr_value;
}





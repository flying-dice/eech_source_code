

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	landing
		*raw;







	raw = (landing *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_FREE_LANDING_SITES:

		{
			
			raw->free_landing_sites = value;

			break;
		}

		case INT_TYPE_INSIDE_HANGAR:

		{
			raw->inside_hangar = value;

			break;
		}

		case INT_TYPE_LANDED_LOCK:

		{
			raw->landed_lock = value;

			break;
		}

		case INT_TYPE_LANDING_LOCK:

		{
			raw->landing_lock = value;

			break;
		}

		case INT_TYPE_RESERVED_LANDING_SITES:

		{
			raw->reserved_landing_sites = value;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			raw->route_node = value;

			break;
		}

		case INT_TYPE_TOTAL_LANDING_SITES:

		{
			raw->total_landing_sites = value;

			break;
		}

		case INT_TYPE_TAKEOFF_LOCK:

		{
			raw->takeoff_lock = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}























































static int get_local_int_value (entity *en, int_types type)
{
	landing
		*raw;

	int
		value;

	raw = (landing *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_FREE_LANDING_SITES:

		{
			value = raw->free_landing_sites;

			break;
		}

		case INT_TYPE_INSIDE_HANGAR:

		{
			value = raw->inside_hangar;

			break;
		}

		case INT_TYPE_LANDED_LOCK:

		{
			value = raw->landed_lock;

			break;
		}

		case INT_TYPE_LANDING_LOCK:

		{
			value = raw->landing_lock;

			break;
		}

		case INT_TYPE_RESERVED_LANDING_SITES:

		{
			value = raw->reserved_landing_sites;

			break;
		}

		case INT_TYPE_ROUTE_NODE:

		{
			debug_fatal ("ld_int: can't use route node");

			value = raw->route_node;

			break;
		}

		case INT_TYPE_TOTAL_LANDING_SITES:

		{
			value = raw->total_landing_sites;

			break;
		}

		case INT_TYPE_TAKEOFF_LOCK:

		{
			value = raw->takeoff_lock;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}

	return (value);
}


































































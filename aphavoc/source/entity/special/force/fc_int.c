

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	force
		*raw;







	raw = (force *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_COLOUR:

		{
			raw->colour = value;

			break;
		}

		case INT_TYPE_FORCE_ATTITUDE:

		{
			raw->force_attitude = value;

			break;
		}

		case INT_TYPE_FORCE_SECTOR_COUNT:

		{
			raw->sector_count = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

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
	force
		*raw;

	int
		value;

	raw = (force *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_COLOUR:

		{
			value = raw->colour;

			break;
		}

		case INT_TYPE_FORCE_ATTITUDE:

		{
			value = raw->force_attitude;

			break;
		}

		case INT_TYPE_FORCE_SECTOR_COUNT:

		{
			value = raw->sector_count;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

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




































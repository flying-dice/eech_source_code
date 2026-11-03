

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	sector
		*raw;







	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_KEYSITE_COUNT:

		{
			raw->keysite_count = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;









			
			break;
		}

		case INT_TYPE_X_SECTOR:

		{
			raw->x_sector = value;

			break;
		}

		case INT_TYPE_Z_SECTOR:

		{
			raw->z_sector = value;

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
	sector
		*raw;

	int
		value;

	raw = (sector *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_KEYSITE_COUNT:

		{
			value = raw->keysite_count;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

			break;
		}

		case INT_TYPE_SECTOR_SIDE:

		{
			if (raw->sector_side [ENTITY_SIDE_BLUE_FORCE] > raw->sector_side [ENTITY_SIDE_RED_FORCE])
			{
				value = ENTITY_SIDE_BLUE_FORCE;
			}
			else
			{
				value = ENTITY_SIDE_RED_FORCE;
			}

			break;
		}

		case INT_TYPE_X_SECTOR:

		{
			value = raw->x_sector;

			break;
		}

		case INT_TYPE_Z_SECTOR:

		{
			value = raw->z_sector;

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













































































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	segment
		*raw;







	raw = (segment *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_BRIDGE_SEGMENT_TYPE:

		{
			raw->bridge_segment_type = value;

			break;
		}

		case INT_TYPE_DAMAGE_LEVEL:

		{
			raw->damage_level = value;

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
	segment
		*raw;

	int
		value;

	raw = (segment *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_BRIDGE_SEGMENT_TYPE:

		{
			value = raw->bridge_segment_type;

			break;
		}

		case INT_TYPE_DAMAGE_LEVEL:

		{
			value = raw->damage_level;

			break;
		}

		case INT_TYPE_EXPLOSIVE_POWER:

		{
			value = EXPLOSIVE_POWER_LOW;

			break;
		}

		case INT_TYPE_EXPLOSIVE_QUALITY:

		{
			value = EXPLOSIVE_QUALITY_DUST;

			break;
		}

		case INT_TYPE_TARGET_SYMBOL_TYPE:

		{
			value = TARGET_SYMBOL_STRUCTURE;

			break;
		}

		case INT_TYPE_TARGET_TYPE:

		{
			value = segment_database[raw->bridge_segment_type].target_type;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = TASK_TARGET_TYPE_BRIDGE;

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{
			value = segment_database[raw->bridge_segment_type].armor_level;

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




































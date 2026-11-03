

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	group
		*raw;







	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			raw->supplies.ammo_supply_level = value;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			raw->assist_timer = value;

			if ((value != 0.0) && (!raw->update_link.parent))
			{

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
			}

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			raw->supplies.fuel_supply_level = value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			raw->sleep = value;

			if ((value != 0.0) && (!raw->update_link.parent))
			{

				insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
			}

			break;
		}

		case FLOAT_TYPE_LAST_SEEN_TIME:

		{
			raw->last_seen_time = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}























































static float get_local_float_value (entity *en, float_types type)
{
	group
		*raw;

	float
		value;

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			value = raw->supplies.ammo_supply_level;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			value = raw->assist_timer;

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			value = raw->supplies.fuel_supply_level;

			break;
		}

		case FLOAT_TYPE_REARMING_TIME:

		{
			value = group_database [raw->sub_type].rearming_time;

			break;
		}

		case FLOAT_TYPE_RECON_DISTANCE:

		{
			value = 100.0;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			value = raw->sleep;

			break;
		}

		case FLOAT_TYPE_LAST_SEEN_TIME:

		{
			value = raw->last_seen_time;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}














































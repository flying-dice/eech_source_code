

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	keysite
		*raw;







	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			raw->supplies.ammo_supply_level = value;

			break;
		}

		case FLOAT_TYPE_ASSIGN_TIMER:

		{
			raw->assign_timer = value;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			raw->assist_timer = value;

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			raw->supplies.fuel_supply_level = value;

			break;
		}

		case FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH:

		{
			raw->keysite_maximum_strength = value;

			break;
		}

		case FLOAT_TYPE_KEYSITE_STRENGTH:

		{
			raw->keysite_strength = value;

			break;
		}

		case FLOAT_TYPE_REPAIR_TIMER:

		{
			raw->repair_timer = value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			raw->sleep = value;

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
	keysite
		*raw;

	float
		value;

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AMMO_SUPPLY_LEVEL:

		{
			value = raw->supplies.ammo_supply_level;

			break;
		}

		case FLOAT_TYPE_ASSIGN_TIMER:

		{
			value = raw->assign_timer;

			break;
		}

		case FLOAT_TYPE_ASSIST_TIMER:

		{
			value = raw->assist_timer;

			break;
		}

		case FLOAT_TYPE_EFFICIENCY:

		{
			if (raw->keysite_maximum_strength > 0.0)
			{
				ASSERT (raw->keysite_strength <= raw->keysite_maximum_strength);

				value = raw->keysite_strength / raw->keysite_maximum_strength;
			}
			else
			{
				value = 1.0;
			}

			break;
		}

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			value = raw->supplies.fuel_supply_level;

			break;
		}

		case FLOAT_TYPE_MINIMUM_EFFICIENCY:

		{
			value = keysite_database [raw->sub_type].minimum_efficiency;

			break;
		}

		case FLOAT_TYPE_KEYSITE_IMPORTANCE:

		{
			value = keysite_database [raw->sub_type].importance;

			break;
		}

		case FLOAT_TYPE_KEYSITE_MAXIMUM_STRENGTH:

		{
			value = raw->keysite_maximum_strength;

			break;
		}

		case FLOAT_TYPE_KEYSITE_STRENGTH:

		{
			value = raw->keysite_strength;

			break;
		}

		case FLOAT_TYPE_RECON_DISTANCE:

		{
			value = keysite_database [raw->sub_type].recon_distance;

			break;
		}

		case FLOAT_TYPE_REPAIR_TIMER:

		{
			value = raw->repair_timer;

			break;
		}

		case FLOAT_TYPE_SECTOR_SIDE_MAX_VALUE:

		{
			value = keysite_database [raw->sub_type].sector_side_max_value;

			break;
		}

		case FLOAT_TYPE_SLEEP:

		{
			value = raw->sleep;

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





































































































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	helicopter
		*raw;







	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{

			raw->fuel_supply_level = value;

			break;
		}

		case FLOAT_TYPE_INVULNERABLE_TIMER:

		{
			raw->invulnerable_timer = value;

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
	helicopter
		*raw;

	float
		value;

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_FUEL_SUPPLY_LEVEL:

		{
			value = raw->fuel_supply_level;

			break;
		}

		case FLOAT_TYPE_INVULNERABLE_TIMER:

		{
			value = raw->invulnerable_timer;

			break;
		}









































		case FLOAT_TYPE_RADAR_STEALTH_FACTOR:

		{
			value = NO_RADAR_STEALTH_FACTOR;

			if (raw->ac.mob.sub_type == ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE)
				if (get_local_entity_weapon_system_ready_state (en) == WEAPON_SYSTEM_READY_CLOSED)
					value -= NO_RADAR_STEALTH_FACTOR * 0.4;

			if (!get_comanche_stub_wings_attached (en))
				value -= NO_RADAR_STEALTH_FACTOR * 0.35;
				
			if (get_local_entity_undercarriage_state (en) == AIRCRAFT_UNDERCARRIAGE_UP)
				value -= NO_RADAR_STEALTH_FACTOR * 0.1;

			ASSERT (value > 0.0);

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












































































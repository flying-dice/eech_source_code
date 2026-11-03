

































































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

		case FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE:

		{
			raw->main_rotor_blade_coning_angle = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_PITCH:

		{
			raw->main_rotor_pitch = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_ROLL:

		{
			raw->main_rotor_roll = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_RPM:

		{
			raw->main_rotor_rpm = value;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER:

		{
			raw->main_rotor_spin_up_timer = value;

			break;
		}

		case FLOAT_TYPE_TAIL_ROTOR_RPM:

		{
			raw->tail_rotor_rpm = value;

			break;
		}

		
		// Jabberwock 050310 MP Cannon bug
		case FLOAT_TYPE_PLAYER_WEAPON_HEADING:

		{
			raw->player_weapon_heading = value;

			break;
		}

		case FLOAT_TYPE_PLAYER_WEAPON_PITCH:

		{
			raw->player_weapon_pitch = value;

			break;
		}
		// Jabberwock 050310 ends

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

		case FLOAT_TYPE_MAIN_ROTOR_BLADE_CONING_ANGLE:

		{
			value = raw->main_rotor_blade_coning_angle;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_PITCH:

		{
			value = raw->main_rotor_pitch;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_ROLL:

		{
			value = raw->main_rotor_roll;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_RPM:

		{
			value = raw->main_rotor_rpm;

			break;
		}

		case FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER:

		{
			value = raw->main_rotor_spin_up_timer;

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

		case FLOAT_TYPE_TAIL_ROTOR_RPM:

		{
			value = raw->tail_rotor_rpm;

			break;
		}

		
		// Jabberwock 050310 MP Cannon bug
		case FLOAT_TYPE_PLAYER_WEAPON_HEADING:

		{
			value = raw->player_weapon_heading;

			break;
		}

		case FLOAT_TYPE_PLAYER_WEAPON_PITCH:

		{
			value = raw->player_weapon_pitch;

			break;
		}
		// Jabberwock 050310 ends

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}












































































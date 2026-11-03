

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	session
		*raw;







	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_AUTO_ASSIGN_GUNSHIP:

		{
			raw->auto_assign_gunship = value;

			break;
		}

		case INT_TYPE_CAMPAIGN_MEDAL:

		{
			raw->campaign_medal = value;

			break;
		}

		case INT_TYPE_CAMPAIGN_REQUIRES_APACHE_HAVOC:

		{
			raw->campaign_requires_apache_havoc = value;

			break;
		}

		case INT_TYPE_CHEATS_ENABLED:

		{
			raw->cheats_enabled = value;

			break;
		}

		case INT_TYPE_INFINITE_FUEL:

		{
			raw->infinite_fuel = value;

			break;
		}

		case INT_TYPE_INFINITE_WEAPONS:

		{
			raw->infinite_weapons = value;

			break;
		}

		case INT_TYPE_INVULNERABLE_FROM_COLLISIONS:

		{
			raw->invulnerable_from_collisions = value;

			break;
		}

		case INT_TYPE_INVULNERABLE_FROM_WEAPONS:

		{
			raw->invulnerable_from_weapons = value;

			break;
		}

		case INT_TYPE_LOCAL_WEATHER_MODEL:

		{
			raw->local_weather_model = value;

			break;
		}

		case INT_TYPE_SESSION_COMPLETE:

		{
			raw->session_complete = value;

			break;
		}

		case INT_TYPE_SKIP_NIGHT_TIME:

		{
			raw->skip_night_time = value;

			break;
		}

		case INT_TYPE_SUPPRESS_AI_FIRE:

		{
			raw->suppress_ai_fire = value;

			break;
		}

		case INT_TYPE_TARGET_WEATHER_MODE:

		{
			raw->target_weather_mode = (weathermodes) value;

			break;
		}

		case INT_TYPE_VERSION_NUMBER:

		{
			raw->version_number = value;

			break;
		}

		case INT_TYPE_WEATHER_INCREASING:

		{
			raw->weather_increasing = value;

			break;
		}

		case INT_TYPE_WEATHER_MODE:

		{
			raw->weather_mode = (weathermodes) value;

			break;
		}

		case INT_TYPE_WIND_INCREASING:

		{
			raw->wind_increasing = value;

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
	session
		*raw;

	int
		value;

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_AUTO_ASSIGN_GUNSHIP:

		{
			value = raw->auto_assign_gunship;

			break;
		}

		case INT_TYPE_CAMPAIGN_MEDAL:

		{
			value = raw->campaign_medal;

			break;
		}

		case INT_TYPE_CAMPAIGN_REQUIRES_APACHE_HAVOC:

		{
			value = raw->campaign_requires_apache_havoc;

			break;
		}

		case INT_TYPE_CHEATS_ENABLED:

		{
			if (get_connection_list_head ())
			{
				value = FALSE;
			}
			else if (command_line_cheats_on)
			{
				value = TRUE;
			}
			else
			{
				value = raw->cheats_enabled;
			}

			break;
		}

		case INT_TYPE_DAY:

		{
			calculate_session_time_of_day (en, &value);

			value += 1;

			break;
		}

		case INT_TYPE_DAY_SEGMENT_TYPE:

		{
			value = raw->day_segment_type;

			break;
		}

		case INT_TYPE_INFINITE_FUEL:

		{
			value = (raw->infinite_fuel /*&& get_local_entity_int_value (en, INT_TYPE_CHEATS_ENABLED)*/);

			break;
		}

		case INT_TYPE_INFINITE_WEAPONS:

		{
			value = (raw->infinite_weapons /*&& get_local_entity_int_value (en, INT_TYPE_CHEATS_ENABLED)*/);

			break;
		}

		case INT_TYPE_INVULNERABLE_FROM_COLLISIONS:

		{
			value = (raw->invulnerable_from_collisions /*&& get_local_entity_int_value (en, INT_TYPE_CHEATS_ENABLED)*/);

			break;
		}

		case INT_TYPE_INVULNERABLE_FROM_WEAPONS:

		{
			value = (raw->invulnerable_from_weapons /*&& get_local_entity_int_value (en, INT_TYPE_CHEATS_ENABLED)*/);

			break;
		}

		case INT_TYPE_LOCAL_WEATHER_MODEL:

		{
			value = raw->local_weather_model;

			break;
		}

		case INT_TYPE_SESSION_COMPLETE:

		{
			value = raw->session_complete;

			break;
		}

		case INT_TYPE_SKIP_NIGHT_TIME:

		{
			value = raw->skip_night_time;

			break;
		}

		case INT_TYPE_SUPPRESS_AI_FIRE:

		{
			value = raw->suppress_ai_fire;

			break;
		}

		case INT_TYPE_TARGET_WEATHER_MODE:

		{
			value = raw->target_weather_mode;

			break;
		}

		case INT_TYPE_VERSION_NUMBER:

		{
			value = raw->version_number;

			break;
		}

		case INT_TYPE_WEATHER_INCREASING:

		{
			value = raw->weather_increasing;

			break;
		}

		case INT_TYPE_WEATHER_MODE:

		{
			value = raw->weather_mode;

			break;
		}

		case INT_TYPE_WIND_INCREASING:

		{
			value = raw->wind_increasing;

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






















































































































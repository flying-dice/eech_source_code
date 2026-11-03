

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	session
		*raw;







	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ELAPSED_TIME:

		{
			raw->elapsed_time = value;

			break;
		}

		case FLOAT_TYPE_FOG_OF_WAR_MAXIMUM_VALUE:

		{
			raw->fog_of_war_maximum_value = value;

			break;
		}

		case FLOAT_TYPE_LIGHTNING_TIMER:

		{
			raw->lightning_timer = value;

			break;
		}

		case FLOAT_TYPE_POPULATION_X_MIN:

		{
			raw->population_x_min = value;

			break;
		}

		case FLOAT_TYPE_POPULATION_X_MAX:

		{
			raw->population_x_max = value;

			break;
		}

		case FLOAT_TYPE_POPULATION_Z_MIN:

		{
			raw->population_z_min = value;

			break;
		}

		case FLOAT_TYPE_POPULATION_Z_MAX:

		{
			raw->population_z_max = value;

			break;
		}

		case FLOAT_TYPE_START_TIME:

		{
			raw->start_time = value;

			raw->day_segment_type = get_day_segment_type (value);

			break;
		}

		case FLOAT_TYPE_TIME_OF_DAY_ACCELERATION:

		{
			raw->time_of_day_acceleration = value;

			break;
		}

		case FLOAT_TYPE_WEATHER_MODE_TRANSITIONAL_PERIOD:

		{
			raw->weather_mode_transitional_period = value;

			break;
		}

		case FLOAT_TYPE_WEATHER_MODE_TRANSITIONAL_STATUS:

		{
			raw->weather_mode_transitional_status = value;

			break;
		}

		case FLOAT_TYPE_WEATHER_RADIUS:

		{
			raw->weather_radius = value;

			break;
		}

		case FLOAT_TYPE_WIND_EFFECT_RADIUS:

		{
			raw->wind_effect_radius = value;

			break;
		}

		case FLOAT_TYPE_WIND_GUSTING_VALUE:

		{
			raw->wind_gusting_value = value;

			break;
		}

		case FLOAT_TYPE_WIND_MAXIMUM_SPEED:

		{
			raw->wind_maximum_speed = value;

			break;
		}

		case FLOAT_TYPE_WIND_MINIMUM_SPEED:

		{
			raw->wind_minimum_speed = value;

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
	session
		*raw;

	float
		value;

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ELAPSED_TIME:

		{
			value = raw->elapsed_time;

			break;
		}

		case FLOAT_TYPE_FOG_OF_WAR_MAXIMUM_VALUE:

		{
			value = raw->fog_of_war_maximum_value;

			break;
		}

		case FLOAT_TYPE_LIGHTNING_TIMER:

		{
			value = raw->lightning_timer;

			break;
		}

		case FLOAT_TYPE_POPULATION_X_MIN:

		{
			value = raw->population_x_min;

			break;
		}

		case FLOAT_TYPE_POPULATION_X_MAX:

		{
			value = raw->population_x_max;

			break;
		}

		case FLOAT_TYPE_POPULATION_Z_MIN:

		{
			value = raw->population_z_min;

			break;
		}

		case FLOAT_TYPE_POPULATION_Z_MAX:

		{
			value = raw->population_z_max;

			break;
		}

		case FLOAT_TYPE_START_TIME:

		{
			value = raw->start_time;

			break;
		}

		case FLOAT_TYPE_TIME_OF_DAY:

		{
			value = calculate_session_time_of_day (en, NULL);

			break;
		}

		case FLOAT_TYPE_TIME_OF_DAY_ACCELERATION:

		{
			value = raw->time_of_day_acceleration;

			break;
		}

		case FLOAT_TYPE_WEATHER_MODE_TRANSITIONAL_PERIOD:

		{
			value = raw->weather_mode_transitional_period;

			break;
		}

		case FLOAT_TYPE_WEATHER_MODE_TRANSITIONAL_STATUS:

		{
			value = raw->weather_mode_transitional_status;

			break;
		}

		case FLOAT_TYPE_WEATHER_RADIUS:

		{
			value = raw->weather_radius;

			break;
		}

		case FLOAT_TYPE_WIND_EFFECT_RADIUS:

		{
			value = raw->wind_effect_radius;

			break;
		}

		case FLOAT_TYPE_WIND_GUSTING_VALUE:

		{
			value = raw->wind_gusting_value;

			break;
		}

		case FLOAT_TYPE_WIND_MAXIMUM_SPEED:

		{
			value = raw->wind_maximum_speed;

			break;
		}

		case FLOAT_TYPE_WIND_MINIMUM_SPEED:

		{
			value = raw->wind_minimum_speed;

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














































































































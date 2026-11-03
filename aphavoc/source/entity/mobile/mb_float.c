

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_float_value (entity *en, float_types type, float value)
{
	mobile
		*raw;







	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_HEADING:

		{

			matrix3x3
				attitude;

			float
				pitch,
				roll;

			pitch = get_pitch_from_attitude_matrix (raw->attitude);

			roll = get_roll_from_attitude_matrix (raw->attitude);

			get_3d_transformation_matrix (attitude, value, pitch, roll);

			set_local_entity_attitude_matrix (en, attitude);

			break;
		}

		case FLOAT_TYPE_HIGH_VELOCITY:
		case FLOAT_TYPE_LOW_VELOCITY:
		case FLOAT_TYPE_MEDIUM_VELOCITY:
		case FLOAT_TYPE_VELOCITY:
		case FLOAT_TYPE_VERY_HIGH_VELOCITY:

		{
			int update_smoke = FALSE;
			
			if(get_local_entity_int_value (en, INT_TYPE_IDENTIFY_AIRCRAFT))
				if (raw->velocity > AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD && value <= AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD ||
						raw->velocity <= AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD && value > AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD)
					update_smoke = TRUE;
			
			raw->velocity = value;

			if (update_smoke)
				switch_aircraft_damaged_smoke(en);

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
	mobile
		*raw;

	float
		value;

	raw = (mobile *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_ALTITUDE:

		{
			value = raw->position.y;

			break;
		}

		case FLOAT_TYPE_HEADING:

		{
			value = get_heading_from_attitude_matrix (raw->attitude);

			break;
		}

		case FLOAT_TYPE_HIGH_VELOCITY:
		case FLOAT_TYPE_LOW_VELOCITY:
		case FLOAT_TYPE_MEDIUM_VELOCITY:
		case FLOAT_TYPE_VELOCITY:
		case FLOAT_TYPE_VERY_HIGH_VELOCITY:

		{
			value = raw->velocity;

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

















































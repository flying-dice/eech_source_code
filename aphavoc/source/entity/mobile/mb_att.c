

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_attitude_matrix (entity *en, matrix3x3 attitude)
{
	mobile
		*raw;

	ASSERT (attitude);







	raw = (mobile *) get_local_entity_data (en);

	memcpy (raw->attitude, attitude, sizeof (matrix3x3));
}





























































static void get_local_attitude_matrix (entity *en, matrix3x3 attitude)
{
	mobile
		*raw;

	ASSERT (attitude);

	raw = (mobile *) get_local_entity_data (en);

	memcpy (attitude, raw->attitude, sizeof (matrix3x3));
}





static matrix3x3 *get_local_attitude_matrix_ptr (entity *en)
{
	mobile
		*raw;

	raw = (mobile *) get_local_entity_data (en);

	return (&raw->attitude);
}





static void set_local_attitude_angles (entity *en, float heading, float pitch, float roll)
{
	mobile
		*raw;







	raw = (mobile *) get_local_entity_data (en);

	get_3d_transformation_matrix (raw->attitude, heading, pitch, roll);
}























































static void get_local_attitude_angles (entity *en, float *heading, float *pitch, float *roll)
{
	mobile
		*raw;

	ASSERT (heading);

	ASSERT (pitch);

	ASSERT (roll);

	raw = (mobile *) get_local_entity_data (en);

	*heading = get_heading_from_attitude_matrix (raw->attitude);

	*pitch = get_pitch_from_attitude_matrix (raw->attitude);

	*roll = get_roll_from_attitude_matrix (raw->attitude);
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

		case FLOAT_TYPE_HEADING:

		{
			value = get_heading_from_attitude_matrix (raw->attitude);

			break;
		}

		case FLOAT_TYPE_PITCH:

		{
			value = get_pitch_from_attitude_matrix (raw->attitude);

			break;
		}

		case FLOAT_TYPE_ROLL:

		{
			value = get_roll_from_attitude_matrix (raw->attitude);

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




























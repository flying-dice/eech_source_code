

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_attitude_matrix (entity *en, matrix3x3 attitude)
{
	mobile
		*raw;

	ASSERT (attitude);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_ATTITUDE_MATRIX, en, attitude);

	#endif

	raw = (mobile *) get_local_entity_data (en);

	memcpy (raw->attitude, attitude, sizeof (matrix3x3));
}





static void set_remote_attitude_matrix (entity *en, matrix3x3 attitude)
{
	ASSERT (attitude);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_ATTITUDE_MATRIX, en, attitude);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_ATTITUDE_MATRIX, en, attitude);
}





static void set_server_attitude_matrix (entity *en, matrix3x3 attitude)
{
	ASSERT (attitude);

	validate_client_server_local_fn ();

	set_local_attitude_matrix (en, attitude);

	validate_client_server_remote_fn ();

	set_remote_attitude_matrix (en, attitude);
}





static void set_client_attitude_matrix (entity *en, matrix3x3 attitude)
{
	ASSERT (attitude);

	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_attitude_matrix (en, attitude);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_attitude_matrix (en, attitude);
	}
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

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_ATTITUDE_ANGLES, en, heading, pitch, roll);

	#endif

	raw = (mobile *) get_local_entity_data (en);

	get_3d_transformation_matrix (raw->attitude, heading, pitch, roll);
}





static void set_remote_attitude_angles (entity *en, float heading, float pitch, float roll)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_ATTITUDE_ANGLES, en, heading, pitch, roll);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_ATTITUDE_ANGLES, en, heading, pitch, roll);
}





static void set_server_attitude_angles (entity *en, float heading, float pitch, float roll)
{
	validate_client_server_local_fn ();

	set_local_attitude_angles (en, heading, pitch, roll);

	validate_client_server_remote_fn ();

	set_remote_attitude_angles (en, heading, pitch, roll);
}





static void set_client_attitude_angles (entity *en, float heading, float pitch, float roll)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_attitude_angles (en, heading, pitch, roll);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_attitude_angles (en, heading, pitch, roll);
	}
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





void overload_mobile_attitude_functions (entity_types type)
{
	fn_set_local_entity_raw_attitude_matrix	 	[type]		  				 	= set_local_attitude_matrix;
	fn_set_local_entity_attitude_matrix				[type]		  				 	= set_local_attitude_matrix;
	fn_set_client_server_entity_attitude_matrix	[type][COMMS_MODEL_SERVER]	= set_server_attitude_matrix;
	fn_set_client_server_entity_attitude_matrix	[type][COMMS_MODEL_CLIENT]	= set_client_attitude_matrix;
	fn_get_local_entity_attitude_matrix				[type]						 	= get_local_attitude_matrix;
	fn_get_local_entity_attitude_matrix_ptr		[type]						 	= get_local_attitude_matrix_ptr;

	fn_set_local_entity_raw_attitude_angles		[type]	  			  		 	= set_local_attitude_angles;
	fn_set_local_entity_attitude_angles				[type]		  				 	= set_local_attitude_angles;
	fn_set_client_server_entity_attitude_angles	[type][COMMS_MODEL_SERVER]	= set_server_attitude_angles;
	fn_set_client_server_entity_attitude_angles	[type][COMMS_MODEL_CLIENT]	= set_client_attitude_angles;
	fn_get_local_entity_attitude_angles				[type]						 	= get_local_attitude_angles;
	fn_get_local_entity_float_value					[type][FLOAT_TYPE_HEADING]	= get_local_float_value;
	fn_get_local_entity_float_value					[type][FLOAT_TYPE_PITCH]	= get_local_float_value;
	fn_get_local_entity_float_value					[type][FLOAT_TYPE_ROLL]		= get_local_float_value;
}





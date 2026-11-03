

































































#include "project.h"





#define DEBUG_MODULE 0





void (*fn_set_local_entity_raw_attitude_matrix[NUM_ENTITY_TYPES]) (entity *en, matrix3x3 attitude);

void (*fn_set_local_entity_attitude_matrix[NUM_ENTITY_TYPES]) (entity *en, matrix3x3 attitude);

void (*fn_set_client_server_entity_attitude_matrix[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, matrix3x3 attitude);

void (*fn_get_local_entity_attitude_matrix[NUM_ENTITY_TYPES]) (entity *en, matrix3x3 attitude);

matrix3x3 *(*fn_get_local_entity_attitude_matrix_ptr[NUM_ENTITY_TYPES]) (entity *en);





void (*fn_set_local_entity_raw_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float heading, float pitch, float roll);

void (*fn_set_local_entity_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float heading, float pitch, float roll);

void (*fn_set_client_server_entity_attitude_angles[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, float heading, float pitch, float roll);

void (*fn_get_local_entity_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float *heading, float *pitch, float *roll);





static void default_set_entity_attitude_matrix (entity *en, matrix3x3 attitude)
{
	ASSERT (attitude);
}





static void default_get_entity_attitude_matrix (entity *en, matrix3x3 attitude)
{
	ASSERT (attitude);

	get_identity_matrix3x3 (attitude);
}





static matrix3x3 *default_get_entity_attitude_matrix_ptr (entity *en)
{
	return (NULL);
}





static void default_set_entity_attitude_angles (entity *en, float heading, float pitch, float roll)
{
}





static void default_get_entity_attitude_angles (entity *en, float *heading, float *pitch, float *roll)
{
	ASSERT (heading);

	ASSERT (pitch);

	ASSERT (roll);

	*heading = 0.0;

	*pitch = 0.0;

	*roll = 0.0;
}





void initialise_entity_attitude_default_functions (void)
{
	int
		i;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		//
		// matrix
		//

		fn_set_local_entity_raw_attitude_matrix[i]									= default_set_entity_attitude_matrix;

		fn_set_local_entity_attitude_matrix[i]											= default_set_entity_attitude_matrix;

		fn_set_client_server_entity_attitude_matrix[i][COMMS_MODEL_SERVER]	= default_set_entity_attitude_matrix;

		fn_set_client_server_entity_attitude_matrix[i][COMMS_MODEL_CLIENT]	= default_set_entity_attitude_matrix;

		fn_get_local_entity_attitude_matrix[i]					  						= default_get_entity_attitude_matrix;

		fn_get_local_entity_attitude_matrix_ptr[i]									= default_get_entity_attitude_matrix_ptr;

		//
		// angles
		//

		fn_set_local_entity_raw_attitude_angles[i]	  								= default_set_entity_attitude_angles;

		fn_set_local_entity_attitude_angles[i]											= default_set_entity_attitude_angles;

		fn_set_client_server_entity_attitude_angles[i][COMMS_MODEL_SERVER]	= default_set_entity_attitude_angles;

		fn_set_client_server_entity_attitude_angles[i][COMMS_MODEL_CLIENT]	= default_set_entity_attitude_angles;

		fn_get_local_entity_attitude_angles[i]					  						= default_get_entity_attitude_angles;
	}
}





void pack_attitude_matrix (entity *en, matrix3x3 attitude)
{
	float
		heading,
		pitch,
		roll;

	ASSERT (attitude);

	heading = get_heading_from_attitude_matrix (attitude);

	pack_float_value (en, FLOAT_TYPE_HEADING, heading);

	pitch = get_pitch_from_attitude_matrix (attitude);

	pack_float_value (en, FLOAT_TYPE_PITCH, pitch);

	roll = get_roll_from_attitude_matrix (attitude);

	pack_float_value (en, FLOAT_TYPE_ROLL, roll);
}





void unpack_attitude_matrix (entity *en, matrix3x3 attitude)
{
	float
		heading,
		pitch,
		roll;

	ASSERT (attitude);

	heading = unpack_float_value (en, FLOAT_TYPE_HEADING);

	pitch = unpack_float_value (en, FLOAT_TYPE_PITCH);

	roll = unpack_float_value (en, FLOAT_TYPE_ROLL);

	get_3d_transformation_matrix (attitude, heading, pitch, roll);
}





void pack_attitude_angles (entity *en, float heading, float pitch, float roll)
{
	pack_float_value (en, FLOAT_TYPE_HEADING, heading);

	pack_float_value (en, FLOAT_TYPE_PITCH, pitch);

	pack_float_value (en, FLOAT_TYPE_ROLL, roll);
}





void unpack_attitude_angles (entity *en, float *heading, float *pitch, float *roll)
{
	ASSERT (heading);

	ASSERT (pitch);

	ASSERT (roll);

	*heading = unpack_float_value (en, FLOAT_TYPE_HEADING);

	*pitch = unpack_float_value (en, FLOAT_TYPE_PITCH);

	*roll = unpack_float_value (en, FLOAT_TYPE_ROLL);
}





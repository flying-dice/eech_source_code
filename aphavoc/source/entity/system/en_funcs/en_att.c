

































































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














static void default_get_entity_attitude_matrix (entity *en, matrix3x3 attitude)
{
	ASSERT (attitude);

	get_identity_matrix3x3 (attitude);
}





static matrix3x3 *default_get_entity_attitude_matrix_ptr (entity *en)
{
	return (NULL);
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
































































































































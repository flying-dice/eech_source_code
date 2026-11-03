

































































extern void (*fn_set_local_entity_raw_attitude_matrix[NUM_ENTITY_TYPES]) (entity *en, matrix3x3 attitude);

#define set_local_entity_raw_attitude_matrix(EN,ATTITUDE) (fn_set_local_entity_raw_attitude_matrix[get_local_entity_type ((EN))] ((EN), (ATTITUDE)))





extern void (*fn_set_local_entity_attitude_matrix[NUM_ENTITY_TYPES]) (entity *en, matrix3x3 attitude);

#define set_local_entity_attitude_matrix(EN,ATTITUDE) (fn_set_local_entity_attitude_matrix[get_local_entity_type ((EN))] ((EN), (ATTITUDE)))





extern void (*fn_set_client_server_entity_attitude_matrix[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, matrix3x3 attitude);

#define set_client_server_entity_attitude_matrix(EN,ATTITUDE) (fn_set_client_server_entity_attitude_matrix[get_local_entity_type ((EN))][get_comms_model ()] ((EN), (ATTITUDE)))





extern void (*fn_get_local_entity_attitude_matrix[NUM_ENTITY_TYPES]) (entity *en, matrix3x3 attitude);

#define get_local_entity_attitude_matrix(EN,ATTITUDE) (fn_get_local_entity_attitude_matrix[get_local_entity_type ((EN))] ((EN), (ATTITUDE)))





extern matrix3x3 *(*fn_get_local_entity_attitude_matrix_ptr[NUM_ENTITY_TYPES]) (entity *en);

#define get_local_entity_attitude_matrix_ptr(EN) (fn_get_local_entity_attitude_matrix_ptr[get_local_entity_type ((EN))] ((EN)))





extern void (*fn_set_local_entity_raw_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float heading, float pitch, float roll);

#define set_local_entity_raw_attitude_angles(EN,H,P,R) (fn_set_local_entity_raw_attitude_angles[get_local_entity_type ((EN))] ((EN), (H), (P), (R)))





extern void (*fn_set_local_entity_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float heading, float pitch, float roll);

#define set_local_entity_attitude_angles(EN,H,P,R) (fn_set_local_entity_attitude_angles[get_local_entity_type ((EN))] ((EN), (H), (P), (R)))





extern void (*fn_set_client_server_entity_attitude_angles[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, float heading, float pitch, float roll);

#define set_client_server_entity_attitude_angles(EN,H,P,R) (fn_set_client_server_entity_attitude_angles[get_local_entity_type ((EN))][get_comms_model ()] ((EN), (H), (P), (R)))





extern void (*fn_get_local_entity_attitude_angles[NUM_ENTITY_TYPES]) (entity *en, float *heading, float *pitch, float *roll);

#define get_local_entity_attitude_angles(EN,H,P,R) (fn_get_local_entity_attitude_angles[get_local_entity_type ((EN))] ((EN), (H), (P), (R)))



















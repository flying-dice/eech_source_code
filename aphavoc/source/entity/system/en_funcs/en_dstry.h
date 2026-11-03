

































































extern void (*fn_destroy_local_entity[NUM_ENTITY_TYPES]) (entity *en);





extern void (*fn_destroy_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);

#define destroy_client_server_entity(EN) (fn_destroy_client_server_entity[get_local_entity_type ((EN))][get_comms_model ()] ((EN)))





extern void (*fn_destroy_local_entity_family[NUM_ENTITY_TYPES]) (entity *en);

#define destroy_local_entity_family(EN) (fn_destroy_local_entity_family[get_local_entity_type ((EN))] ((EN)))





extern void (*fn_destroy_client_server_entity_family[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);

#define destroy_client_server_entity_family(EN) (fn_destroy_client_server_entity_family[get_local_entity_type ((EN))][get_comms_model ()] ((EN)))





extern void (*fn_kill_local_entity[NUM_ENTITY_TYPES]) (entity *en);

#define kill_local_entity(EN) (fn_kill_local_entity[get_local_entity_type ((EN))] ((EN)))





extern void (*fn_kill_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);

#define kill_client_server_entity(EN) (fn_kill_client_server_entity[get_local_entity_type ((EN))][get_comms_model ()] ((EN)))





extern void initialise_entity_destroy_default_functions (void);

extern void destroy_local_entity (entity *en);

extern void destroy_local_entities (void);

extern void destroy_local_entity_children (entity *en, list_types list);

extern void destroy_client_server_entity_children (entity *en, list_types list);

extern void destroy_local_only_entities (void);





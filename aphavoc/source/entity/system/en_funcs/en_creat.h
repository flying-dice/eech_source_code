

































































extern entity *(*fn_create_local_entity[NUM_ENTITY_TYPES]) (entity_types type, int index, char *pargs);





extern entity *(*fn_create_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity_types type, int index, char *pargs);





extern int
	force_local_entity_create_stack_attributes;

#define set_force_local_entity_create_stack_attributes(FLAG) (force_local_entity_create_stack_attributes = (FLAG))





extern void initialise_entity_create_default_functions (void);

extern entity *create_local_entity (entity_types type, int index, ...);

entity *create_client_server_entity (entity_types type, int index, ...);

extern void create_local_only_entities (pack_modes pack_mode);





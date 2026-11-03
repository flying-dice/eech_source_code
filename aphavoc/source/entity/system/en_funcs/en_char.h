

































































enum CHAR_TYPES
{
	CHAR_TYPE_TAG,
	CHAR_TYPE_VALUE,
	NUM_CHAR_TYPES
};

typedef enum CHAR_TYPES char_types;





#define NUM_CHAR_TYPE_PACK_BITS (2)





struct CHAR_TYPE_DATA
{
	char
		*name;

	int
		debug_pack;
};

typedef struct CHAR_TYPE_DATA char_type_data;





extern char_type_data
	char_type_database[NUM_CHAR_TYPES];

#define get_char_type_name(CHAR_TYPE) (char_type_database[(CHAR_TYPE)].name)





extern const char
	*overload_invalid_char_type_message,
	*debug_fatal_invalid_char_type_message;

#ifdef DEBUG

#define validate_char_type(CHAR_TYPE) ((((CHAR_TYPE) >= 0) && ((CHAR_TYPE) < NUM_CHAR_TYPES)) ? ((CHAR_TYPE)) : (process_assert (overload_invalid_char_type_message, __FILE__, __LINE__), 0))

#else

#define validate_char_type(CHAR_TYPE) ((CHAR_TYPE))

#endif

#define debug_fatal_invalid_char_type(EN,CHAR_TYPE) (debug_fatal (debug_fatal_invalid_char_type_message, get_local_entity_type_name ((EN)), get_local_entity_index ((EN)), get_char_type_name ((CHAR_TYPE)), __FILE__, __LINE__))





extern void (*fn_set_local_entity_raw_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type, char value);

#define set_local_entity_raw_char_value(EN,CHAR_TYPE,VALUE) (fn_set_local_entity_raw_char_value[get_local_entity_type ((EN))][validate_char_type ((CHAR_TYPE))] ((EN), (CHAR_TYPE), (VALUE)))





extern void (*fn_set_local_entity_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type, char value);

#define set_local_entity_char_value(EN,CHAR_TYPE,VALUE) (fn_set_local_entity_char_value[get_local_entity_type ((EN))][validate_char_type ((CHAR_TYPE))] ((EN), (CHAR_TYPE), (VALUE)))





extern void (*fn_set_client_server_entity_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, char_types type, char value);

#define set_client_server_entity_char_value(EN,CHAR_TYPE,VALUE) (fn_set_client_server_entity_char_value[get_local_entity_type ((EN))][validate_char_type ((CHAR_TYPE))][get_comms_model ()] ((EN), (CHAR_TYPE), (VALUE)))





extern char (*fn_get_local_entity_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type);

#define get_local_entity_char_value(EN,CHAR_TYPE) (fn_get_local_entity_char_value[get_local_entity_type ((EN))][validate_char_type ((CHAR_TYPE))] ((EN), (CHAR_TYPE)))



















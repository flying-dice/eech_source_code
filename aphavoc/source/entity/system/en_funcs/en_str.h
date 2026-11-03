

































































enum STRING_TYPES
{
	STRING_TYPE_DIVISION_NAME,
	STRING_TYPE_EASY_DISPLAY_NAME,
	STRING_TYPE_FORCE_NAME,
	STRING_TYPE_FULL_NAME,
	STRING_TYPE_GROUP_CALLSIGN,
	STRING_TYPE_GROUP_SHORT_NAME,
	STRING_TYPE_HUD_NAME,
	STRING_TYPE_KEYSITE_NAME,
	STRING_TYPE_LONG_DISPLAY_NAME,
	STRING_TYPE_PILOT_MESSAGE,
	STRING_TYPE_PILOTS_NAME,
	STRING_TYPE_PLAYERS_NAME,
	STRING_TYPE_SHORT_DISPLAY_NAME,
	NUM_STRING_TYPES
};

typedef enum STRING_TYPES string_types;





#define NUM_STRING_TYPE_PACK_BITS (4)





// string max lengths exclude terminator
//
// max length is limited to 255
//
// read only strings have zero max length

#define STRING_TYPE_MAX_LENGTH							(255)

#define STRING_TYPE_DIVISION_NAME_MAX_LENGTH			(48)
#define STRING_TYPE_EASY_DISPLAY_NAME_MAX_LENGTH	(0)
#define STRING_TYPE_FULL_NAME_MAX_LENGTH 				(0)
#define STRING_TYPE_FORCE_NAME_MAX_LENGTH				(256)
#define STRING_TYPE_HUD_NAME_MAX_LENGTH	 			(0)
#define STRING_TYPE_KEYSITE_NAME_MAX_LENGTH			(256)
#define STRING_TYPE_LONG_DISPLAY_NAME_MAX_LENGTH	(0)
#define STRING_TYPE_MESSAGE_LENGTH	 					(160)
#define STRING_TYPE_PLAYERS_NAME_MAX_LENGTH 			(256)
#define STRING_TYPE_SHORT_DISPLAY_NAME_MAX_LENGTH	(0)





struct STRING_TYPE_DATA
{
	char
		*name;

	int
		max_length,
		debug_pack;

	char
		*default_value;
};

typedef struct STRING_TYPE_DATA string_type_data;





extern string_type_data
	string_type_database[NUM_STRING_TYPES];

#define get_string_type_name(STRING_TYPE) (string_type_database[(STRING_TYPE)].name)

#define get_string_type_max_length(STRING_TYPE) (string_type_database[(STRING_TYPE)].max_length)





extern const char
	*overload_invalid_string_type_message,
	*debug_fatal_invalid_string_type_message;

#ifdef DEBUG

#define validate_string_type(STRING_TYPE) ((((STRING_TYPE) >= 0) && ((STRING_TYPE) < NUM_STRING_TYPES)) ? ((STRING_TYPE)) : (process_assert (overload_invalid_string_type_message, __FILE__, __LINE__), 0))

#else

#define validate_string_type(STRING_TYPE) ((STRING_TYPE))

#endif

#define debug_fatal_invalid_string_type(EN,STRING_TYPE) (debug_fatal (debug_fatal_invalid_string_type_message, get_local_entity_type_name ((EN)), get_local_entity_index ((EN)), get_string_type_name ((STRING_TYPE)), __FILE__, __LINE__))





extern void (*fn_set_local_entity_raw_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type, const char *s);

#define set_local_entity_raw_string(EN,STRING_TYPE,VALUE) (fn_set_local_entity_raw_string[get_local_entity_type ((EN))][validate_string_type ((STRING_TYPE))] ((EN), (STRING_TYPE), (VALUE)))





extern void (*fn_set_local_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type, const char *s);

#define set_local_entity_string(EN,STRING_TYPE,VALUE) (fn_set_local_entity_string[get_local_entity_type ((EN))][validate_string_type ((STRING_TYPE))] ((EN), (STRING_TYPE), (VALUE)))





extern void (*fn_set_client_server_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, string_types type, const char *s);

#define set_client_server_entity_string(EN,STRING_TYPE,VALUE) (fn_set_client_server_entity_string[get_local_entity_type ((EN))][validate_string_type ((STRING_TYPE))][get_comms_model ()] ((EN), (STRING_TYPE), (VALUE)))





extern const char *(*fn_get_local_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type);

#define get_local_entity_string(EN,STRING_TYPE) (fn_get_local_entity_string[get_local_entity_type ((EN))][validate_string_type ((STRING_TYPE))] ((EN), (STRING_TYPE)))





extern void initialise_entity_string_default_functions (void);

extern void pack_string (entity *en, string_types type, const char *s);

extern void unpack_string (entity *en, string_types type, char *s);

extern void pack_string_type (string_types type);

extern string_types unpack_string_type (void);





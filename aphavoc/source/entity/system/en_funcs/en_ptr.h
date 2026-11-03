

































































enum PTR_TYPES
{
	PTR_TYPE_EXTERNAL_VIEW_ENTITY,
	PTR_TYPE_GROUP_LEADER,
	PTR_TYPE_INSTANCE_3D_OBJECT,
	PTR_TYPE_POSITION_ENTITY,
	PTR_TYPE_PROTOTYPE_1,
	PTR_TYPE_PROTOTYPE_2,
	PTR_TYPE_PROTOTYPE_3,
	PTR_TYPE_PROTOTYPE_4,
	PTR_TYPE_RETURN_KEYSITE,
	PTR_TYPE_ROUTE_DEPENDENTS,
	PTR_TYPE_ROUTE_FORMATION_TYPES,
	PTR_TYPE_ROUTE_NODE,
	PTR_TYPE_ROUTE_WAYPOINT_TYPES,
	PTR_TYPE_SUCC,
	PTR_TYPE_SOUND_EFFECT_DATA,
	PTR_TYPE_TASK_LEADER,
	PTR_TYPE_TASK_OBJECTIVE,
	PTR_TYPE_TERRAIN_FACE_NORMAL,
	PTR_TYPE_TERRAIN_INFO,
	PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY,
	NUM_PTR_TYPES
};

typedef enum PTR_TYPES ptr_types;





struct PTR_TYPE_DATA
{
	char
		*name;
};

typedef struct PTR_TYPE_DATA ptr_type_data;





extern ptr_type_data
	ptr_type_database[NUM_PTR_TYPES];

#define get_ptr_type_name(PTR_TYPE) (ptr_type_database[(PTR_TYPE)].name)





extern const char
	*overload_invalid_ptr_type_message,
	*debug_fatal_invalid_ptr_type_message;

#ifdef DEBUG

#define validate_ptr_type(PTR_TYPE) ((((PTR_TYPE) >= 0) && ((PTR_TYPE) < NUM_PTR_TYPES)) ? ((PTR_TYPE)) : (process_assert (overload_invalid_ptr_type_message, __FILE__, __LINE__), 0))

#else

#define validate_ptr_type(PTR_TYPE) ((PTR_TYPE))

#endif

#define debug_fatal_invalid_ptr_type(EN,PTR_TYPE) (debug_fatal (debug_fatal_invalid_ptr_type_message, get_local_entity_type_name ((EN)), get_local_entity_index ((EN)), get_ptr_type_name ((PTR_TYPE)), __FILE__, __LINE__))





extern void (*fn_set_local_entity_ptr_value[NUM_ENTITY_TYPES][NUM_PTR_TYPES]) (entity *en, ptr_types type, void *ptr);

#define set_local_entity_ptr_value(EN,PTR_TYPE,VALUE) (fn_set_local_entity_ptr_value[get_local_entity_type ((EN))][validate_ptr_type ((PTR_TYPE))] ((EN), (PTR_TYPE), (VALUE)))





extern void *(*fn_get_local_entity_ptr_value[NUM_ENTITY_TYPES][NUM_PTR_TYPES]) (entity *en, ptr_types type);

#define get_local_entity_ptr_value(EN,PTR_TYPE) (fn_get_local_entity_ptr_value[get_local_entity_type ((EN))][validate_ptr_type ((PTR_TYPE))] ((EN), (PTR_TYPE)))





extern void initialise_entity_ptr_value_default_functions (void);





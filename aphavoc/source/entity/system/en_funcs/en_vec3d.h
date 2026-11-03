

































































enum VEC3D_TYPES
{
	VEC3D_TYPE_BOUNDING_BOX_MAX,
	VEC3D_TYPE_BOUNDING_BOX_MIN,
	VEC3D_TYPE_COVER_POSITION,
	VEC3D_TYPE_EO_TRACKING_POINT,
	VEC3D_TYPE_GUIDE_POSITION,
	VEC3D_TYPE_INITIAL_VELOCITY,
	VEC3D_TYPE_MOTION_VECTOR,
	VEC3D_TYPE_OBJECT_SCALING,
	VEC3D_TYPE_POSITION,
	VEC3D_TYPE_RELATIVE_POSITION,
	VEC3D_TYPE_START_POSITION,
	VEC3D_TYPE_STOP_POSITION,
	VEC3D_TYPE_WAIT_POSITION,
	VEC3D_TYPE_WEAPON_TO_TARGET_VECTOR,
	VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR,
	VEC3D_TYPE_WEAPON_VECTOR,
	VEC3D_TYPE_WEATHER_POSITION,
	VEC3D_TYPE_WEATHER_VELOCITY,
	VEC3D_TYPE_WIND_DIRECTION_VECTOR,
	VEC3D_TYPE_WIND_EFFECT_POSITION,
	VEC3D_TYPE_WIND_EFFECT_VELOCITY,
	VEC3D_TYPE_XV,
	VEC3D_TYPE_YV,
	VEC3D_TYPE_ZV,
	VEC3D_TYPE_LAST_KNOWN_POSITION,
	NUM_VEC3D_TYPES
};

typedef enum VEC3D_TYPES vec3d_types;





#define NUM_VEC3D_TYPE_PACK_BITS (5)





enum VEC3D_PACK_TYPES
{
	VEC3D_PACK_TYPE_UNPACKED,
	VEC3D_PACK_TYPE_UNIT_VECTOR,
	VEC3D_PACK_TYPE_POSITION,
	VEC3D_PACK_TYPE_RELATIVE_POSITION,
	VEC3D_PACK_TYPE_INITIAL_VELOCITY,
	VEC3D_PACK_TYPE_MOTION_VECTOR,
	NUM_VEC3D_PACK_TYPES
};

typedef enum VEC3D_PACK_TYPES vec3d_pack_types;





struct VEC3D_TYPE_DATA
{
	char
		*name;

	vec3d_pack_types
		pack_type;

	int
		debug_pack;
};

typedef struct VEC3D_TYPE_DATA vec3d_type_data;





extern vec3d_type_data
	vec3d_type_database[NUM_VEC3D_TYPES];

#define get_vec3d_type_name(VEC3D_TYPE) (vec3d_type_database[(VEC3D_TYPE)].name)





extern const char
	*overload_invalid_vec3d_type_message,
	*debug_fatal_invalid_vec3d_type_message;

#ifdef DEBUG

#define validate_vec3d_type(VEC3D_TYPE) ((((VEC3D_TYPE) >= 0) && ((VEC3D_TYPE) < NUM_VEC3D_TYPES)) ? ((VEC3D_TYPE)) : (process_assert (overload_invalid_vec3d_type_message, __FILE__, __LINE__), 0))

#else

#define validate_vec3d_type(VEC3D_TYPE) ((VEC3D_TYPE))

#endif

#define debug_fatal_invalid_vec3d_type(EN,VEC3D_TYPE) (debug_fatal (debug_fatal_invalid_vec3d_type_message, get_local_entity_type_name ((EN)), get_local_entity_index ((EN)), get_vec3d_type_name ((VEC3D_TYPE)), __FILE__, __LINE__))





extern void (*fn_set_local_entity_raw_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type, vec3d *v);

#define set_local_entity_raw_vec3d(EN,VEC3D_TYPE,V) (fn_set_local_entity_raw_vec3d[get_local_entity_type ((EN))][validate_vec3d_type ((VEC3D_TYPE))] ((EN), (VEC3D_TYPE), (V)))





extern void (*fn_set_local_entity_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type, vec3d *v);

#define set_local_entity_vec3d(EN,VEC3D_TYPE,V) (fn_set_local_entity_vec3d[get_local_entity_type ((EN))][validate_vec3d_type ((VEC3D_TYPE))] ((EN), (VEC3D_TYPE), (V)))





extern void (*fn_set_client_server_entity_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, vec3d_types type, vec3d *v);

#define set_client_server_entity_vec3d(EN,VEC3D_TYPE,V) (fn_set_client_server_entity_vec3d[get_local_entity_type ((EN))][validate_vec3d_type ((VEC3D_TYPE))][get_comms_model ()] ((EN), (VEC3D_TYPE), (V)))





extern void (*fn_get_local_entity_vec3d[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type, vec3d *v);

#define get_local_entity_vec3d(EN,VEC3D_TYPE,V) (fn_get_local_entity_vec3d[get_local_entity_type ((EN))][validate_vec3d_type ((VEC3D_TYPE))] ((EN), (VEC3D_TYPE), (V)))





extern vec3d *(*fn_get_local_entity_vec3d_ptr[NUM_ENTITY_TYPES][NUM_VEC3D_TYPES]) (entity *en, vec3d_types type);

#define get_local_entity_vec3d_ptr(EN,VEC3D_TYPE) (fn_get_local_entity_vec3d_ptr[get_local_entity_type ((EN))][validate_vec3d_type ((VEC3D_TYPE))] ((EN), (VEC3D_TYPE)))





extern void initialise_entity_vec3d_default_functions (void);

extern void pack_vec3d (entity *en, vec3d_types type, vec3d *v);

extern void unpack_vec3d (entity *en, vec3d_types type, vec3d *v);

extern void pack_vec3d_type (vec3d_types type);

extern vec3d_types unpack_vec3d_type (void);

extern float get_local_entity_vec3d_magnitude (entity *en, vec3d_types type);





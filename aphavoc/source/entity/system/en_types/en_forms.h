

































































enum FORMATION_TYPES
{

	FORMATION_NONE,
	FORMATION_1,
	FORMATION_ROW_LEFT,
	FORMATION_ROW_RIGHT,
	FORMATION_ECHELON_LEFT,
	FORMATION_ECHELON_RIGHT,
	FORMATION_COLUMN,
	FORMATION_WEDGE,
//VJ 060305 three new formations for maverick
   	FORMATION_DIAMOND,
   	FORMATION_STAGGERED_TRAIL_LEFT,
   	FORMATION_STAGGERED_TRAIL_RIGHT,
   
   FORMATION_80M_ROAD_NODE_16_TANKS,
	FORMATION_INFANTRY_COLUMN,

	FORMATION_INVALID,
	NUM_FORMATION_TYPES
};

typedef enum FORMATION_TYPES formation_types;





enum FORMATION_COMPONENT_TYPES
{

	FORMATION_COMPONENT_HELICOPTER_LIGHT_ASSAULT_GROUP,
	FORMATION_COMPONENT_HELICOPTER_HEAVY_ASSAULT_GROUP,
	FORMATION_COMPONENT_HELICOPTER_MARINE_ASSAULT_GROUP_A,
	FORMATION_COMPONENT_HELICOPTER_MARINE_ASSAULT_GROUP_B,

	FORMATION_COMPONENT_HELICOPTER_LIGHT_ATTACK_GROUP_A,
	FORMATION_COMPONENT_HELICOPTER_LIGHT_ATTACK_GROUP_B,
	FORMATION_COMPONENT_HELICOPTER_HEAVY_ATTACK_GROUP_A,
	FORMATION_COMPONENT_HELICOPTER_HEAVY_ATTACK_GROUP_B,
	FORMATION_COMPONENT_HELICOPTER_MARINE_ATTACK_GROUP_A,
	FORMATION_COMPONENT_HELICOPTER_MARINE_ATTACK_GROUP_B,
	FORMATION_COMPONENT_HELICOPTER_MARINE_ATTACK_GROUP_C,
	FORMATION_COMPONENT_HELICOPTER_MARINE_ATTACK_GROUP_D,

	FORMATION_COMPONENT_HELICOPTER_LIGHT_RECON_GROUP,

	FORMATION_COMPONENT_HELICOPTER_LIGHT_RECON_ATTACK_GROUP,
	FORMATION_COMPONENT_HELICOPTER_HEAVY_RECON_ATTACK_GROUP,
	FORMATION_COMPONENT_HELICOPTER_HUNTER_KILLER_GROUP,

	FORMATION_COMPONENT_HELICOPTER_MEDIUM_LIFT_GROUP,
	FORMATION_COMPONENT_HELICOPTER_HEAVY_LIFT_GROUP_A,
	FORMATION_COMPONENT_HELICOPTER_HEAVY_LIFT_GROUP_B,

	FORMATION_COMPONENT_FIXED_WING_CLOSE_AIR_SUPPORT_GROUP,
	FORMATION_COMPONENT_FIXED_WING_MULTI_ROLE_GROUP,
	FORMATION_COMPONENT_FIXED_WING_CARRIER_BORNE_STRIKE_GROUP,
	FORMATION_COMPONENT_FIXED_WING_CARRIER_BORNE_INTERCEPTOR_GROUP,
	FORMATION_COMPONENT_FIXED_WING_MEDIUM_LIFT_GROUP,
	FORMATION_COMPONENT_FIXED_WING_HEAVY_LIFT_GROUP,
	FORMATION_COMPONENT_PRIMARY_FRONTLINE_GROUP,
	FORMATION_COMPONENT_SECONDARY_FRONTLINE_GROUP,
	FORMATION_COMPONENT_HEAVY_SAM_AAA_GROUP,
	FORMATION_COMPONENT_LIGHT_SAM_AAA_GROUP,
	FORMATION_COMPONENT_ARTILLERY_GROUP,
	FORMATION_COMPONENT_MLRS_GROUP,
	FORMATION_COMPONENT_INFANTRY,
	FORMATION_COMPONENT_INFANTRY_SAM_STANDING,
	FORMATION_COMPONENT_INFANTRY_SAM_KNEELING,
	FORMATION_COMPONENT_AIRCRAFT_CARRIER_GROUP,
	FORMATION_COMPONENT_FRIGATE_GROUP,
	FORMATION_COMPONENT_LANDING_GROUP,
	FORMATION_COMPONENT_LIGHT_FIRING_POINT,
	FORMATION_COMPONENT_MEDIUM_FIRING_POINT,
	FORMATION_COMPONENT_HEAVY_FIRING_POINT,

	NUM_FORMATION_COMPONENT_TYPES
};

typedef enum FORMATION_COMPONENT_TYPES formation_component_types;





struct FORMATION_TYPE
{

	formation_types
		formation;

	char
		*name;

	short int
		number_in_formation;

	vec3d
		*sites;

	struct FORMATION_TYPE
		*next;
};

typedef struct FORMATION_TYPE formation_type;





struct FORMATION_VEHICLE_COMPONENTS
{

	formation_component_types
		formation_component;

	formation_types
		formation;

	int
		count,
		group_type,
		*components;

	struct FORMATION_VEHICLE_COMPONENTS
		*next;
};

typedef struct FORMATION_VEHICLE_COMPONENTS formation_vehicle_components;





extern const char
	*formation_names [],
	*formation_display_names [],
	*formation_component_names [];

extern formation_type
	*formation_database;

extern formation_vehicle_components
	*formation_component_database;





extern void initialise_formation_database (void);

extern void initialise_formation_database_table (void);

extern int add_formation_to_database (int formation_count, vec3d *site_positions);

extern int get_formation_database_count (void);

extern void deinitialise_formation_database (void);

extern formation_type *get_formation (formation_types formation);

extern short int get_number_in_formation (formation_types formation);

extern const char *get_formation_name (formation_types formation);

extern float get_formation_member_radius (formation_types form, int member_count);

extern void get_formation_position (formation_types form, int index, vec3d *vec);





extern void initialise_formation_component_database (void);

extern void deinitialise_formation_component_database (void);

extern formation_vehicle_components *get_formation_components (formation_component_types formation_component);





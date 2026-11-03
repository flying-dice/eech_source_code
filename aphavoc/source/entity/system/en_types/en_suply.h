

































































#define MAX_REARMING_TIME_SCALING_FACTOR			5

#define MAX_REFUELING_TIME_SCALING_FACTOR			5

#define FUEL_SUPPLY_LEVEL_USAGE 					10.0

#define AMMO_SUPPLY_LEVEL_USAGE 					10.0

#define KEYSITE_MINIMUM_FUEL_SUPPLY_LEVEL 	10.0

#define KEYSITE_MINIMUM_AMMO_SUPPLY_LEVEL 	10.0

#define FUEL_USAGE_ACCELERATOR					1.0

#define FUEL_RESTOCK_ACCELERATOR					100.0

#define AMMO_USAGE_ACCELERATOR					1.0

#define AMMO_RESTOCK_ACCELERATOR					100.0

#define KEYSITE_SUPPLY_REQUEST_THRESHOLD		75.0





#define DEBUG_SUPPLY									0





enum RESUPPLY_SOURCE_TYPE
{

	RESUPPLY_SOURCE_NONE,
	RESUPPLY_SOURCE_KEYSITE,
	RESUPPLY_SOURCE_GROUP,
	NUM_RESUPPLY_SOURCE_TYPES
};

typedef enum RESUPPLY_SOURCE_TYPE resupply_source_type;





struct SUPPLY_TYPE
{

	float
		air_to_ground_ammo_supply_level,
		air_to_air_ammo_supply_level,
		ammo_supply_level,
		fuel_supply_level;
};

typedef struct SUPPLY_TYPE supply_type;





extern float get_local_entity_rearming_sleep_time (entity *group, int weapon_type);

extern float get_local_entity_refueling_sleep_time (entity *group);





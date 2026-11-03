

































































struct SECTOR
{
	list_root
		sector_root,
		sector_task_root,
		special_effect_root;

	unsigned int
		keysite_count	: 8,
		x_sector			: NUM_SECTOR_BITS,
		z_sector			: NUM_SECTOR_BITS,
		side				: NUM_SIDE_BITS;

	float
		tallest_structure_height,
		sector_side [NUM_ENTITY_SIDES],
		fog_of_war [NUM_ENTITY_SIDES],
		importance_level [NUM_ENTITY_SIDES],
		distance_to_friendly_base [NUM_ENTITY_SIDES],
		surface_to_air_defence_level [NUM_ENTITY_SIDES],
		surface_to_surface_defence_level [NUM_ENTITY_SIDES];
};

typedef struct SECTOR sector;





extern entity
	**entity_sector_map;





#ifdef DEBUG

#define get_local_raw_sector_entity(X_SEC,Z_SEC) ((entity_sector_map [(X_SEC) + ((Z_SEC) * (NUM_MAP_X_SECTORS))]) ? (entity_sector_map[(X_SEC) + ((Z_SEC) * (NUM_MAP_X_SECTORS))]) : (process_assert (null_sector_entity_map_pointer_message, __FILE__, __LINE__), (entity*)NULL))

#else

#define get_local_raw_sector_entity(X_SEC,Z_SEC) (entity_sector_map [(X_SEC) + ((Z_SEC) * (NUM_MAP_X_SECTORS))])

#endif





extern const char
	*null_sector_entity_map_pointer_message;





extern entity *get_local_sector_entity (vec3d *pos);

extern float get_local_sector_entity_enemy_surface_to_air_defence_level (entity *sector_en, entity_sides side);

extern float get_local_sector_entity_enemy_surface_to_surface_defence_level (entity *sector_en, entity_sides side);

extern float get_sector_importance_level (int sx, int sz, entity_sides side);

extern float get_sector_distance_to_friendly_base (int sx, int sz, entity_sides side);

extern void add_mobile_values_to_sector (entity *sector_en, entity *mobile_en);

extern void remove_mobile_values_from_sector (entity *sector_en, entity *mobile_en);

extern float get_local_sector_side_ratio (int sx, int sz, entity_sides side);

extern void set_sector_fog_of_war_value (entity *en, entity *sector_en);

extern float get_sector_fog_of_war_value (entity *en, entity_sides side);

extern void update_sector_fog_of_war (void);

extern void update_client_server_sector_fog_of_war (void);

extern void update_sector_side_count (void);

extern void update_client_server_sector_side_count (void);



extern int get_sector_task_type_count (entity *en, entity_sub_types task_type, entity_sides side);





// JB 030311 VC rename source files
#include "sc_seccreat.h"

#include "sc_dstry.h"



// JB 030311 VC rename source files














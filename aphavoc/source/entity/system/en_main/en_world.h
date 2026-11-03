

































































struct WORLD_MAP_DATA
{
	int
		sector_side_length,			// must be 2^n value
		sector_side_length_mask,
		num_map_x_sectors,
		num_map_z_sectors,
		num_map_sectors,
		min_map_x_sector,
		min_map_z_sector,
		max_map_x_sector,
		max_map_z_sector;

	float
		min_map_x,
		min_map_y,
		min_map_z,
		mid_map_x,
		mid_map_y,
		mid_map_z,
		max_map_x,
		max_map_y,
		max_map_z;
};

typedef struct WORLD_MAP_DATA world_map_data;





extern world_map_data
	world_map;





#define SECTOR_SIDE_LENGTH			(world_map.sector_side_length)
#define SECTOR_SIDE_LENGTH_MASK	(world_map.sector_side_length_mask)

#define NUM_MAP_X_SECTORS			(world_map.num_map_x_sectors)
#define NUM_MAP_Z_SECTORS			(world_map.num_map_z_sectors)

#define NUM_MAP_SECTORS				(world_map.num_map_sectors)

#define MIN_MAP_X_SECTOR			(world_map.min_map_x_sector)
#define MIN_MAP_Z_SECTOR			(world_map.min_map_z_sector)

#define MAX_MAP_X_SECTOR			(world_map.max_map_x_sector)
#define MAX_MAP_Z_SECTOR			(world_map.max_map_z_sector)

#define MIN_MAP_X						(world_map.min_map_x)
#define MIN_MAP_Y						(world_map.min_map_y)
#define MIN_MAP_Z						(world_map.min_map_z)

#define MID_MAP_X						(world_map.mid_map_x)
#define MID_MAP_Y						(world_map.mid_map_y)
#define MID_MAP_Z						(world_map.mid_map_z)

#define MAX_MAP_X						(world_map.max_map_x)
#define MAX_MAP_Y						(world_map.max_map_y)
#define MAX_MAP_Z						(world_map.max_map_z)





#define MAP_PERIMETER_SIZE	(5.0 * KILOMETRE)





#define get_x_sector(X_SEC,X) {convert_float_to_int ((X), &(X_SEC)); (X_SEC) /= SECTOR_SIDE_LENGTH;}

#define get_z_sector(Z_SEC,Z) {convert_float_to_int ((Z), &(Z_SEC)); (Z_SEC) /= SECTOR_SIDE_LENGTH;}





#define point_inside_map_area(POS)	\
(												\
	((POS)->x >= MIN_MAP_X) &&			\
	((POS)->x <= MAX_MAP_X) &&			\
	((POS)->z >= MIN_MAP_Z) &&			\
	((POS)->z <= MAX_MAP_Z)				\
)





#define point_inside_map_volume(POS)	\
(											 		\
	((POS)->x >= MIN_MAP_X) &&		  		\
	((POS)->x <= MAX_MAP_X) &&		  		\
	((POS)->y >= MIN_MAP_Y) &&		  		\
	((POS)->y <= MAX_MAP_Y) &&		  		\
	((POS)->z >= MIN_MAP_Z) &&		  		\
	((POS)->z <= MAX_MAP_Z)			  		\
)





#define point_inside_adjusted_map_area(POS)						\
(											 										\
	((POS)->x >= (MIN_MAP_X + MAP_PERIMETER_SIZE)) &&		  		\
	((POS)->x <= (MAX_MAP_X - MAP_PERIMETER_SIZE)) &&		  		\
	((POS)->z >= (MIN_MAP_Z + MAP_PERIMETER_SIZE)) &&		  		\
	((POS)->z <= (MAX_MAP_Z - MAP_PERIMETER_SIZE))			  		\
)





#define point_inside_adjusted_map_volume(POS)						\
(											 										\
	((POS)->x >= (MIN_MAP_X + MAP_PERIMETER_SIZE)) &&		  		\
	((POS)->x <= (MAX_MAP_X - MAP_PERIMETER_SIZE)) &&		  		\
	((POS)->y >= (MIN_MAP_Y)) &&									  		\
	((POS)->y <= (MAX_MAP_Y)) &&		 							 		\
	((POS)->z >= (MIN_MAP_Z + MAP_PERIMETER_SIZE)) &&		  		\
	((POS)->z <= (MAX_MAP_Z - MAP_PERIMETER_SIZE))			  		\
)





#define point_above_ground(POS)																		\
(																												\
	((POS)->y >= get_terrain_3d_point_sector_max_elevation ((POS)->x, (POS)->z)) ||	\
	((POS)->y >= get_3d_terrain_elevation ((POS)->x, (POS)->z))								\
)

#define point_below_ground(POS)																		\
(																							\
	(point_inside_map_area(POS) == FALSE) || 										\
	((POS)->y < get_terrain_3d_point_sector_max_elevation ((POS)->x, (POS)->z)) &&	\
	((POS)->y < get_3d_terrain_elevation ((POS)->x, (POS)->z))								\
)

#define point_above_floor(POS) ((POS)->y >= MIN_MAP_Y)

#define point_below_ceiling(POS) ((POS)->y <= MAX_MAP_Y)





#define sector_inside_map_area(X_SEC,Z_SEC)	\
(															\
	((X_SEC) >= MIN_MAP_X_SECTOR) &&				\
	((X_SEC) <= MAX_MAP_X_SECTOR) &&				\
	((Z_SEC) >= MIN_MAP_Z_SECTOR) &&				\
	((Z_SEC) <= MAX_MAP_Z_SECTOR)					\
)





extern void set_entity_world_map_size (int num_map_x_sectors, int num_map_z_sectors, int sector_side_length);

extern int bound_position_to_map_area (vec3d *position);

extern int bound_position_to_adjusted_map_area (vec3d *position);

extern int bound_position_to_map_volume (vec3d *position);
extern int bound_double_position_to_map_volume (double_vec3d *position);
extern int bound_position_to_adjusted_map_volume (vec3d *position);

extern int bound_double_position_to_adjusted_map_volume (double_vec3d *position);





#define get_entity_world_map_sector_side_length() (world_map.sector_side_length)
#define get_entity_world_map_sector_side_length_mask() (world_map.sector_side_length_mask)

#define get_entity_world_map_num_map_x_sectors() (world_map.num_map_x_sectors)
#define get_entity_world_map_num_map_z_sectors() (world_map.num_map_z_sectors)

#define get_entity_world_map_num_map_sectors() (world_map.num_map_sectors)

#define get_entity_world_map_min_map_x_sector() (world_map.min_map_x_sector)
#define get_entity_world_map_min_map_z_sector() (world_map.min_map_z_sector)

#define get_entity_world_map_max_map_x_sector() (world_map.max_map_x_sector)
#define get_entity_world_map_max_map_z_sector() (world_map.max_map_z_sector)

#define get_entity_world_map_min_map_x() (world_map.min_map_x)
#define get_entity_world_map_min_map_y() (world_map.min_map_y)
#define get_entity_world_map_min_map_z() (world_map.min_map_z)

#define get_entity_world_map_mid_map_x() (world_map.mid_map_x)
#define get_entity_world_map_mid_map_y() (world_map.mid_map_y)
#define get_entity_world_map_mid_map_z() (world_map.mid_map_z)

#define get_entity_world_map_max_map_x() (world_map.max_map_x)
#define get_entity_world_map_max_map_y() (world_map.max_map_y)
#define get_entity_world_map_max_map_z() (world_map.max_map_z)





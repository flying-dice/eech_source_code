

































































struct TERRAIN_DATABASE_TYPE
{
	terrain_classes
		terr_class;

	unsigned int
		draw_crater							: 1,
		raised								: 1,
		suitable_for_ground_vehicle	: 1;
};

typedef struct TERRAIN_DATABASE_TYPE terrain_database_type;





extern terrain_classes get_terrain_type_class (terrain_types type);

extern int get_terrain_type_draw_crater (terrain_types type);

extern int get_terrain_type_raised (terrain_types type);

extern int get_terrain_type_suitable_for_ground_vehicle (terrain_types type);

extern void initialise_terrain_database ();







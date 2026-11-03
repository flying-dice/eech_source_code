

































































struct FIXED_DATA
{
	object_3d_index_numbers
		default_3d_shape;

	int
		align_with_terrain,
		collision_test_fixed,
		ground_radar_clutter;
};

typedef struct FIXED_DATA fixed_data;





extern const fixed_data
	fixed_database[NUM_ENTITY_SUB_TYPE_FIXED];







































































struct CRATER_DATA
{
	object_3d_index_numbers
		default_3d_shape;

	float
		min_scale,
		max_scale,
		lifetime;
};

typedef struct CRATER_DATA crater_data;





extern const crater_data
	crater_database[NUM_CRATER_TYPES];







































































struct BRIDGE_DATA
{

	bridge_segment_types
		start_section_type,
		end_section_type;

	float
		default_leg_height,
		min_suitable_span,
		max_suitable_span,
		min_suitable_depth,
		max_suitable_depth,
		max_suitable_slope;

	unsigned short int
		num_elements_in_mid_section_pattern		: 15,
		valid_bridge_type								: 1;

	bridge_segment_types
		*mid_section_type;
};

typedef struct BRIDGE_DATA bridge_data;





extern bridge_data
	bridge_database[NUM_ENTITY_SUB_TYPE_BRIDGES];





extern void initialise_bridge_database();

extern void deinitialise_bridge_database();





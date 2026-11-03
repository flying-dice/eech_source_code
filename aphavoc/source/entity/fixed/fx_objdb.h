

































































struct FIXED_3D_OBJECT_DATA
{
	float
		importance;

	unsigned int
		explosive_quality	:	3,
		explosive_power	:	2,
		hardness				:	2,
		target_type			:	1,
		target_symbol		:	4,
		initial_damage		:	12,
		valid_entry			:	1,
		keysite_object		:	1;
};

typedef struct FIXED_3D_OBJECT_DATA fixed_3d_object_data;





extern fixed_3d_object_data
	fixed_3d_object_database[OBJECT_3D_LAST];





extern void initialise_fixed_3d_object_database (void);

extern void report_non_entries_in_fixed_3d_object_database (void);





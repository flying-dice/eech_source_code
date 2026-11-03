

































































struct DIVISION_DATA
{
	char
		*full_name;

	entity_sub_types
		default_group_division;

	int
		maximum_groups_per_division;
};

typedef struct DIVISION_DATA division_data;





extern const division_data
	division_database[NUM_ENTITY_SUB_TYPE_DIVISIONS];





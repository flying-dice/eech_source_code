

































































struct GROUP_DATA
{
	const char
		*full_name,
		*group_short_name;

	group_category_types
		group_category;

	list_types
		registry_list_type,
		group_list_type;

	movement_types
		movement_type;

	entity_sub_types
		default_landing_type;

	entity_types
		default_entity_type;

	entity_sub_types
		default_blue_force_sub_type,
		default_red_force_sub_type;

	formation_types
		default_group_formation;

	entity_sub_types
		default_group_division;

	int
		maximum_groups_per_division;

	const char
		*platoon_name;

	int
		platoon_short_name_type,
		map_layer_type,
		map_icon;

	float
		rearming_time;

	unsigned int
		frontline_flag				: NUM_FRONTLINE_BITS,			
		local_only_group			: NUM_LOCAL_ONLY_GROUP_BITS,
		default_engage_enemy		: NUM_ENGAGE_ENEMY_BITS,
		amalgamate					: 1,
		platoon_id_type			: 3,
		maximum_member_count		: 5,
		minimum_idle_count		: 4,
		resupply_source			: NUM_RESUPPLY_SOURCE_BITS;

	ai_statistics
		ai_stats;
};

typedef struct GROUP_DATA group_data;




//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern group_data
	group_database[NUM_ENTITY_SUB_TYPE_GROUPS];





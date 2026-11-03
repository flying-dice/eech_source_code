

































































struct FORCE
{
	char
		force_name [STRING_TYPE_FORCE_NAME_MAX_LENGTH + 1];

	list_root
		keysite_force_root,
		independent_group_root,
		pilot_root,
		division_root,
		campaign_objective_root,

		air_registry_root,
		ground_registry_root,
		sea_registry_root;

	list_link
		force_link,
		update_link;

	task_generation_type
		task_generation [NUM_ENTITY_SUB_TYPE_TASKS];

	campaign_criteria_type
		*campaign_criteria;

	int
		force_info_current_hardware [NUM_FORCE_INFO_CATAGORIES],
		force_info_reserve_hardware [NUM_FORCE_INFO_CATAGORIES];

	int
		kills [NUM_ENTITY_SUB_TYPE_GROUPS],
		losses [NUM_ENTITY_SUB_TYPE_GROUPS],
		group_count [NUM_ENTITY_SUB_TYPE_GROUPS];

	int
		sector_count;

	float
		sleep;

	unsigned int
		force_attitude				: NUM_FORCE_ATTITUDE_BITS,
		colour 						: NUM_COLOUR_BITS,
		side							: NUM_SIDE_BITS;
};

typedef struct FORCE force;





























extern int
	force_hardware_update;

#define set_force_hardware_update(FLAG) (force_hardware_update = (FLAG))

#define get_force_hardware_update() (force_hardware_update)





extern entity *get_local_force_entity (entity_sides side);

extern void add_to_force_info (entity *force_en, entity *new_en);

extern void remove_from_force_info (entity *force_en, entity *new_en);



extern void replace_into_force_info (entity *force_en, entity *new_en);

extern void add_mobile_to_force_kills_stats (entity *en, entity *victim);

extern void add_mobile_to_force_losses_stats (entity *en, entity *victim);

extern void add_group_type_to_force_info (entity *en, entity_sub_types group_type);

extern void remove_group_type_from_force_info (entity *en, entity_sub_types group_type);

extern int get_local_force_entity_group_count (entity *en, entity_sub_types group_type);






































































typedef struct
{
	entity_sub_types weapon_type;
	unsigned count;
} weapon_count;







extern int get_local_entity_weapon_count (entity *en, entity_sub_types weapon_sub_type);













extern void get_local_entity_weapon_load(entity* en, weapon_count weapon_load[], unsigned max_num_weapons);
	








extern int get_local_entity_weapon_hardpoint_info
(
	entity *en,
	int heading_depth,
	entity_sub_types given_weapon_sub_type,
	entity_sub_types *weapon_sub_type,
	int *number,
	int *damaged
);

extern void set_comanche_stub_wing_visibility (entity *en);

extern int get_comanche_stub_wings_attached (entity *en);





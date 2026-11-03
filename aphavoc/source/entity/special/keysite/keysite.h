

































































#define DEFAULT_KEYSITE_ASSISTANCE_REQUEST_TIMER	(ONE_MINUTE + (sfrand1 () * 20.0))

#define KEYSITE_TASK_ASSIGN_TIMER						(3.0 * ONE_MINUTE)

#define KEYSITE_UPDATE_SLEEP_TIMER						(1.0 * ONE_MINUTE)





enum KEYSITE_AIR_FORCE_CAPACITY_TYPES
{
	KEYSITE_AIR_FORCE_CAPACITY_NONE,
	KEYSITE_AIR_FORCE_CAPACITY_SMALL,
	KEYSITE_AIR_FORCE_CAPACITY_LARGE,

	NUM_KEYSITE_AIR_FORCE_CAPACITY_TYPES
};





struct KEYSITE
{

	entity_sub_types
		sub_type;

	vec3d
		position;

	list_root
		unassigned_task_root,
		assigned_task_root,
		completed_task_root,

		building_group_root,
		keysite_group_root,
		division_headquarters_root,

		task_dependent_root,
		cargo_root,
		regen_root,
		landing_site_root;

	list_link
		campaign_objective_link,
		keysite_force_link,
		movement_dependent_link,
		sector_link,
		update_link;

	float
		keysite_strength,
		keysite_maximum_strength,
		repair_timer,
		assist_timer,
		assign_timer,
		sleep;

	supply_type
		supplies;

	char
		keysite_name [STRING_TYPE_KEYSITE_NAME_MAX_LENGTH + 1];

	unsigned int
		keysite_id							:NUM_KEYSITE_ID_BITS,
		object_index						:NUM_OBJECT_INDEX_BITS,
		landing_types 						:NUM_LANDING_TYPE_BITS,
		alive									:NUM_ALIVE_BITS,
		in_use								:NUM_IN_USE_BITS,
		keysite_usable_state				:NUM_KEYSITE_USABLE_STATE_BITS,
		side									:NUM_SIDE_BITS,
		ndb_frequency						:NUM_NDB_FREQUENCY_BITS;
};

typedef struct KEYSITE keysite;







#include "ks_dbase.h"







#include "ks_int.h"

















extern entity *get_closest_keysite (entity_sub_types type, entity_sides side, vec3d *pos, float min_range, float *actual_range, int outside_of_range, entity *exclude_keysite);

extern vec3d *get_keysite_supply_position (entity *keysite);

extern void get_keysite_troop_pickup_position (entity *keysite, vec3d *pos);

extern int get_keysite_troop_dropoff_position (entity *keysite, vec3d *pos, float *heading);

extern void get_keysite_troop_outside_building_position (entity *keysite, vec3d *pos, int count);

extern void get_keysite_troop_in_building_position (entity *keysite, vec3d *pos);

extern int get_keysite_takenoff_position (entity *keysite, vec3d *pos, entity_sub_types landing_types);

extern int get_keysite_start_landing_position (entity *keysite, vec3d *pos, entity_sub_types landing_type);

extern int get_keysite_landing_sites_available (entity *en, int landing_type);

extern void update_keysite_cargo (entity *en, float cargo_count, entity_sub_types sub_type, float cargo_size);

extern void initialise_keysite_farp_enable (entity *force);

extern entity *get_fixed_structure_keysite_entity (entity *structure);

extern int get_local_keysite_member_count (entity *keysite);

extern int add_local_entity_importance_to_keysite (entity *en, entity *keysite_en);

extern int restore_local_entity_importance_to_keysite (entity *en, entity *keysite_en);

extern void subtract_local_entity_importance_from_keysite (entity *en);

extern void notify_keysite_structure_under_attack (entity *structure, entity *aggressor);

extern void destroy_keysite (entity *en, entity_sides side);

extern void capture_keysite (entity *en, entity_sides new_side);

extern void change_local_keysite_building_sides (entity *en, entity_sides new_side);

extern int repair_client_server_entity_keysite (entity *en);

extern int get_keysite_suitable_for_player (entity *keysite, entity *pilot);



extern void validate_keysite_landing_site_heights (entity *keysite);










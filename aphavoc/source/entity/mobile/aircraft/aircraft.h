

































































struct AIRCRAFT
{
	mobile
		mob;

	object_3d_index_numbers
		object_3d_shape;

	object_3d_instance
		*inst3d;

	list_root
		aircrew_root,
		task_dependent_root,
		launched_weapon_root,
		movement_dependent_root;

	list_link
		designated_target_link, // Jabberwock 031107 Designated targets
		follower_link,
		gunship_target_link,
		member_link,
		view_link,
		takeoff_queue_link,
		movement_dependent_link;

	float
		distance,
		cargo_door_state,
		loading_door_state,
		target_scan_timer,
		undercarriage_state,
		view_interest_level,
		weapon_burst_timer,
		weapon_system_ready_state,
		decoy_release_timer,
		death_timer,
		air_radar_contact_timeout,
		rudder_state,
		weapon_launch_delay,
		weapon_to_intercept_point_range;

	weapon_config_types
		weapon_config_type;

	weapon_package_status
		*weapon_package_status_array;

	entity_sub_types
		selected_weapon;

	struct SYSTEM_SOUND_EFFECT
		*sound_effect_data;

	unsigned int
		formation_position					:NUM_FORMATION_POSITION_BITS,
		operational_state						:NUM_OPERATIONAL_STATE_BITS,
		valid_sound_effect 					:NUM_VALID_SOUND_EFFECT_BITS,
		selected_weapon_system_ready	  	:NUM_SELECTED_WEAPON_SYSTEM_READY_BITS,
		weapon_and_target_vectors_valid	:NUM_WEAPON_AND_TARGET_VECTORS_VALID_BITS,
		gunship_radar_los_clear				:NUM_GUNSHIP_RADAR_LOS_CLEAR_BITS,
		lights_on								:NUM_LIGHTS_ON_BITS,
		cpg_identified							:NUM_CPG_IDENTIFIED_BITS,
		group_member_number					:NUM_GROUP_MEMBER_NUMBER_BITS,
		id_number_significant_digits		:NUM_ID_NUMBER_SIGNIFICANT_DIGIT_BITS,
		ejected									:NUM_EJECTED_BITS,
		troops_onboard						:NUM_TROOPS_ONBOARD_BITS;

	unsigned int
		weapon_salvo_timer;

	int
		damage_level,
		id_number;

	vec3d
		weapon_vector,
		weapon_to_target_vector,
		weapon_to_intercept_point_vector;

	terrain_3d_point_data
		terrain_info;
};

typedef struct AIRCRAFT aircraft;





typedef enum
{

	AIRCRAFT_DAMAGE_NONE,
	AIRCRAFT_DAMAGE_MORTAL,
	AIRCRAFT_DAMAGE_CRITICAL

} aircraft_damage_types;







#include "ac_dbase.h"

















#include "ac_tgt.h"



#include "ac_wpn.h"





#include "helicop/helicop.h"

#include "fixwing/fixwing.h"





#define AIRCRAFT_DAMAGED_SMOKE_SPEED_THRESHOLD 20.0

//extern int get_local_aircraft_maneuver_complete (entity *en, float min_angle);

extern aircraft_damage_types aircraft_critically_damaged (entity *en);

extern void assess_aircraft_damage_level (entity *en, int old_damage_level, int new_damage_level);

















extern void switch_aircraft_damaged_smoke (entity *en);









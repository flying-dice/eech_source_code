

































































#define VEHICLE_CRITICAL_DAMAGE_LEVEL 15

struct VEHICLE
{
	mobile
		mob;

	object_3d_index_numbers
		object_3d_shape;

	object_3d_instance
		*inst3d;

	list_root
		task_dependent_root,
		launched_weapon_root,
		movement_dependent_root;

	list_link
		designated_target_link, // Jabberwock 031107 Designated targets
		follower_link,
		gunship_target_link,
		member_link,
		movement_dependent_link,
		view_link,
		takeoff_queue_link;

	float
		sleep,
		distance,
		loading_door_state,
		radar_rotation_state,
		target_scan_timer,
		target_fire_timer,
		view_interest_level,
		weapon_burst_timer,
		weapon_system_ready_state,
		decoy_release_timer,
		death_timer,
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
		selected_weapon_system_ready	  	:NUM_SELECTED_WEAPON_SYSTEM_READY_BITS,
		weapon_and_target_vectors_valid	:NUM_WEAPON_AND_TARGET_VECTORS_VALID_BITS,
		gunship_radar_los_clear				:NUM_GUNSHIP_RADAR_LOS_CLEAR_BITS,
		los_to_target							:NUM_LOS_TO_TARGET_BITS,
		lights_on								:NUM_LIGHTS_ON_BITS,
		cpg_identified							:NUM_CPG_IDENTIFIED_BITS,
		group_member_number					:NUM_GROUP_MEMBER_NUMBER_BITS,
		id_number_significant_digits		:NUM_ID_NUMBER_SIGNIFICANT_DIGIT_BITS;

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

typedef struct VEHICLE vehicle;







#include "vh_dbase.h"

















#include "vh_tgt.h"



#include "vh_wpn.h"





#include "anti_air/anti_air.h"

#include "person/person.h"

#include "routed/routed.h"

#include "ship/ship.h"









extern void assess_vehicle_damage_level (entity *en, int old_damage_level);

extern int vehicle_critically_damaged (entity *en);









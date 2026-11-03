

































































struct HELICOPTER
{
	aircraft
		ac;

	int
		timer;

	float
		fuel_supply_level,
		invulnerable_timer,
		main_rotor_rpm,
		main_rotor_pitch,
		main_rotor_roll,
		main_rotor_blade_coning_angle,
		main_rotor_spin_up_timer,
		// Jabberwock 050310 MP Cannon bug
		player_weapon_heading,
		player_weapon_pitch,
		// Jabberwock 050310 ends
		tail_rotor_rpm;

	unsigned int
		helicopter_damage_flags	: NUM_HELICOPTER_DAMAGE_FLAGS_BITS,
		helicopter_weapon_damage_flags	: NUM_HELICOPTER_DAMAGE_FLAGS_BITS,
		auto_pilot					: NUM_AUTO_PILOT_BITS,
		player						: NUM_PLAYER_BITS,
		main_rotor_damaged		: 1,
		tail_rotor_damaged		: 1,
		los_to_target				: NUM_LOS_TO_TARGET_BITS,
		radar_on						: NUM_RADAR_ON_BITS,
		laser_on						: NUM_LASER_ON_BITS,
		infra_red_jammer_on		: NUM_INFRA_RED_JAMMER_ON_BITS,
		radar_jammer_on			: NUM_RADAR_JAMMER_ON_BITS,
		weapons_hold				: NUM_WEAPONS_HOLD_BITS,
		position_hold				: NUM_POSITION_HOLD_BITS,
		lock_on_after_launch		: NUM_LOCK_ON_AFTER_LAUNCH_BITS;

	list_root
		designated_target_root,	// Jabberwock 031107 Designated target
		gunship_target_root,
		padlock_root;

	vec3d
		cover_position,
		wait_position,
		eo_tracking_point;
};

typedef struct HELICOPTER helicopter;





extern entity
	*gunship_entity;

struct SECTOR;

extern struct SECTOR
	*gunship_current_sector;

#define get_gunship_entity() (gunship_entity)





#define MAIN_ROTOR_SPIN_UP_TIMER 25.0





extern void set_gunship_entity (entity *en);

extern void assign_entity_to_user (entity *en);

extern void set_raw_gunship_entity (entity *en);



extern void restore_helicopter_entity (entity *en);

extern void notify_gunship_entity_mission_completed (entity *en, entity *task);

extern void notify_gunship_entity_mission_terminated (entity *en, entity *task);









extern int get_local_entity_suitable_for_player (entity *en, entity *pilot);

extern void helicopter_assume_player_control (entity *en);

extern void helicopter_release_player_control (entity *en);

extern void transmit_player_recon_data (entity *en);

extern int get_helicopter_allowed_to_rearm (entity *en);

extern int helicopter_within_keysite_area (entity *en);

extern int player_mission_logged(void);







#include "hc_creat.h"













#include "hc_move.h"

















































































#define DEBUG_TERRAIN_CHECK 0





struct MOBILE
{
	entity_sub_types
		sub_type;

	vec3d
		position;

	union
	{
		matrix3x3
			attitude;

		struct
		{
			vec3d
				xv,
				yv,
				zv;
		};
	};

	vec3d
		motion_vector;

	float
		velocity;

	list_root
		special_effect_root,
		target_root;

	list_link
		padlock_link,
		sector_link,
		target_link,
		update_link;

	unsigned int
		alive			  		 	 			: NUM_ALIVE_BITS,
		side		 	  		 	  			: NUM_SIDE_BITS,
		kill_next_update	 	  			: NUM_KILL_NEXT_UPDATE_BITS,
		tacview_logging : 1,
		gunship_radar_target_index		: NUM_GUNSHIP_RADAR_TARGET_INDEX_BITS,
		object_drawn_once_this_frame	: NUM_OBJECT_DRAWN_ONCE_THIS_FRAME_BITS;
};

typedef struct MOBILE mobile;





#include	"mb_att.h"

#include	"mb_float.h"

#include	"mb_funcs.h"

#include	"mb_int.h"

#include	"mb_list.h"

#include	"mb_move.h"

#include	"mb_msgs.h"

#include	"mb_pack.h"

#include	"mb_tgt.h"

#include	"mb_vec3d.h"





//
// INCLUDE ORDER IS CRITICAL
//

#include "weapon/weapon.h"

#include "aircraft/aircraft.h"

#include "cargo/cargo.h"

#include "vehicle/vehicle.h"





extern float mobile_terrain_adjust_waypoint_height (entity *en, vec3d *wp_pos, float min_height);

extern void draw_mobile_entity_debug_info (entity *en);

extern void update_mobile_sprite_light_timers (void);

extern void credit_client_server_mobile_kill (entity *victim, entity *aggressor);

extern float calculate_mobile_death_timer_value (entity *en);

extern int insert_mobile_into_takeoff_route (entity *member, entity *landing_entity);

extern int mobile_check_all_members_taken_off (entity *en);

extern void play_mobile_under_attack_speech (entity *en, entity *aggressor);











































































enum GUIDE_POSITION_TYPES
{

	GUIDE_POSITION_ABSOLUTE,
	GUIDE_POSITION_WAYPOINT,
	GUIDE_POSITION_RELATIVE_WORLD_SPACE,
	GUIDE_POSITION_RELATIVE_OBJECT_SPACE,
	GUIDE_POSITION_RELATIVE_CIRCULAR,

	NUM_GUIDE_POSITION_TYPES
};

typedef enum GUIDE_POSITION_TYPES guide_position_types;

extern const int
	guide_position_type_virtual [NUM_GUIDE_POSITION_TYPES];





enum GUIDE_CRITERIA_TYPES
{

	GUIDE_CRITERIA_RADIUS,
	GUIDE_CRITERIA_HEADING,
	GUIDE_CRITERIA_ALTITUDE,	
	GUIDE_CRITERIA_TRANSMIT_DATA,	
	GUIDE_CRITERIA_LAST_TO_REACH,	
	GUIDE_CRITERIA_WEAPON_VECTOR,

	NUM_GUIDE_CRITERIA_TYPES
};

typedef enum GUIDE_CRITERIA_TYPES guide_criteria_types;

extern const char
	*guide_criteria_type_names [NUM_GUIDE_CRITERIA_TYPES];





struct GUIDE_CRITERIA_ELEMENT
{
	int
		valid;

	float
		value;
};

typedef struct GUIDE_CRITERIA_ELEMENT guide_criteria_element;





#define NUM_SEEK_COVER_COUNT_BITS 5

struct GUIDE_SEEK_COVER_RESULT_TYPE
{
	unsigned int
		search_mode			: 4,
		sector				: 1,
		count					: NUM_SEEK_COVER_COUNT_BITS,
		score					: 8;
};
	
typedef struct GUIDE_SEEK_COVER_RESULT_TYPE guide_seek_cover_result_type;





struct GUIDE
{
	entity_sub_types
		sub_type;

	list_root
		follower_root;

	list_link
		current_waypoint_link,
		guide_link,
		guide_stack_link,
		update_link;

	guide_position_types
		guide_position_type;

	vec3d
		position;

	entity
		*position_entity;

	unsigned int
		valid_guide_members;

	unsigned int
		check_waypoint_action				: 1;

	float
		velocity;

	guide_criteria_element
		criteria [NUM_GUIDE_CRITERIA_TYPES];

	//
	// Server only seek-cover data
	//
	// Not packed up - CLIENT does not need it - SERVER can simply start seeking again upon reload
	//

	guide_seek_cover_result_type
		current_seek_cover_result,
		best_seek_cover_result;

	terrain_3d_triangle
		*terrain_polygon_list;

	int
		terrain_polygon_count;
};

typedef struct GUIDE guide;





enum GUIDE_TERRAIN_FOLLOW_MODES
{
	GUIDE_TERRAIN_FOLLOW_NONE,
	GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,
	GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE,

	NUM_GUIDE_TERRAIN_FOLLOW_MODES
};

typedef enum GUIDE_TERRAIN_FOLLOW_MODES guide_terrain_follow_modes;

#define NUM_GUIDE_TERRAIN_FOLLOW_BITS	2





#include	"gd_attck.h"

#include	"gd_cover.h"



#include	"gd_dbase.h"















#include	"gd_nav.h"









#include	"ag_attck.h"

#include	"aa_attck.h"





extern entity *create_client_server_guide_entity (entity *task_en, entity *first_waypoint, unsigned int valid_members);

extern int validate_local_guide_entity (entity *en);

extern int check_guide_reached_waypoint (entity *en);

extern int check_guide_waypoint_action_reached (entity *en);

extern int set_guide_new_waypoint (entity *en, entity *wp);

extern int set_guide_next_waypoint (entity *en);

extern int set_guide_prev_waypoint (entity *en);

extern void get_local_guide_position (entity *en, vec3d *pos);

extern void attach_group_member_to_guide_entity (entity *member, entity *guide);

extern void attach_group_to_guide_entity (entity *group, entity *guide);

extern void delete_group_member_from_guide_entity (entity *member, entity *guide);

extern void delete_group_member_from_engage_guide (entity *member, entity *guide, int engage_enemy);

extern void set_client_server_guide_entity_new_position (entity *en, vec3d *position, entity *position_entity);

extern entity *get_local_group_primary_guide (entity *en);

extern entity *get_local_entity_primary_guide (entity *en);

extern void get_local_guide_entity_pointers (entity *en, entity **aggressor, entity **waypoint, entity **task, entity **objective);

extern int get_guide_required_heading (entity *en, entity *mobile, float *heading);



extern int get_guide_criteria_valid (entity *en, guide_criteria_types type);

extern float get_guide_criteria_value (entity *en, guide_criteria_types type);

extern void set_client_server_guide_criteria_valid (entity *en, guide_criteria_types type, int valid, float value);

extern void set_local_guide_criteria_valid (entity *en, guide_criteria_types type, int valid, float value);





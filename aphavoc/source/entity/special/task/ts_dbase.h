

































































struct TASK_DATA
{

	//
	// names
	//


	const char
		*full_name,
		*short_name,
		*key;

	task_category_types
		task_category;

	float
		task_priority;

	int
		difficulty_rating;

	unsigned int
		task_default_target_class	: NUM_TASK_TARGET_CLASS_BITS,
		task_default_target_source	: NUM_TASK_TARGET_SOURCE_BITS,
		task_default_target_type	: NUM_TASK_TARGET_TYPE_BITS,
		task_objective_info			: 2,
		minimum_member_count			: 4,
		rules_of_engagement			: NUM_TASK_ROE_TYPE_BITS,
		engage_enemy					: 1,
		escort_required_threshold	: 4;

	sys_colours
		waypoint_route_colour;

	unsigned int
		add_start_waypoint				: 1,
		primary_task						: 1,
		solo_task							: 1,
		persistent_task					: 1,
		visible_task						: 1,
		keysite_air_force_capacity		: 2,
		assess_landing						: 1,
		task_route_search					: 1,
		perform_debrief					: 1,
		delay_task_assignment			: 1,
		task_objective_preview			: 2,
		counts_towards_player_log		: 1,
		wait_for_end_task					: 1,
		player_reserve_factor			: 3;

	unsigned int
		task_pass_percentage_partial	: 7,
		task_pass_percentage_success	: 7,
		task_completed_score				: 16;

	movement_types
		movement_type;

	int
		landing_types;


	//
	// AI Statistics
	//


	ai_statistics
		ai_stats;
};

typedef struct TASK_DATA task_data;





//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern task_data
	task_database[NUM_ENTITY_SUB_TYPE_TASKS];





#define ESCORT_NEVER 15

#define ESCORT_CRITICAL 6






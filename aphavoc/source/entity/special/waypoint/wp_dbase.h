

































































struct WAYPOINT_DATA
{

	//
	// names
	//


	const char
		*full_name;

	int
		verbose_operational_state	: NUM_VERBOSE_OPERATIONAL_STATE_BITS;

	unsigned int
		mobile_follow_waypoint_offset		: 1,
		waypoint_action_message				: 9,
		waypoint_reached_message			: 9,
		waypoint_reached_return_value		: 4,
		planner_moveable						: 1,
		objective_waypoint					: 1,
		player_skip_waypoint					: 1,
		check_waypoint_action				: 1;

	entity_sub_types
		guide_sub_type;

	int
		map_icon;

	// FIXED WING

	float
		fw_minimum_previous_waypoint_distance,
		fw_action_radius,
		fw_reached_radius,
		fw_velocity;

	unsigned int
		fw_criteria_last_to_reach	: 1,
		fw_criteria_transmit_recon	: 1,
		fw_position_type				: NUM_POSITION_TYPE_BITS,
		fw_movement_type				: NUM_MOVEMENT_TYPE_BITS;

	// HELICOPTER

	float
		hc_minimum_previous_waypoint_distance,
		hc_action_radius,
		hc_reached_radius,
		hc_velocity;

	unsigned int
		hc_criteria_last_to_reach	: 1,
		hc_criteria_transmit_recon	: 1,
		hc_position_type				: NUM_POSITION_TYPE_BITS,
		hc_movement_type				: NUM_MOVEMENT_TYPE_BITS;

	// GROUND

	float
		rv_minimum_previous_waypoint_distance,
		rv_action_radius,
		rv_reached_radius,
		rv_velocity;

	unsigned int
		rv_criteria_last_to_reach	: 1,
		rv_criteria_transmit_recon	: 1,
		rv_position_type				: NUM_POSITION_TYPE_BITS,
		rv_movement_type				: NUM_MOVEMENT_TYPE_BITS;

	// SEA

	float
		sh_minimum_previous_waypoint_distance,
		sh_action_radius,
		sh_reached_radius,
		sh_velocity;

	unsigned int
		sh_criteria_last_to_reach	: 1,
		sh_criteria_transmit_recon	: 1,
		sh_position_type				: NUM_POSITION_TYPE_BITS,
		sh_movement_type				: NUM_MOVEMENT_TYPE_BITS;
};

typedef struct WAYPOINT_DATA waypoint_data;





//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern waypoint_data
	waypoint_database[NUM_ENTITY_SUB_TYPE_WAYPOINTS];





extern float get_waypoint_database_action_radius_value (entity_sub_types waypoint_type, entity_types mobile_type);

extern float get_local_entity_waypoint_database_action_radius_value (entity *waypoint, entity *mobile);

extern float get_waypoint_database_reached_radius_value (entity_sub_types waypoint_type, entity_types mobile_type);

extern float get_local_entity_waypoint_database_reached_radius_value (entity *waypoint, entity *mobile);

extern float get_waypoint_database_velocity_value (entity_sub_types waypoint_type, entity_types mobile_type);

extern int get_waypoint_database_transmit_recon_flag (entity_sub_types waypoint_type, entity_types mobile_type);

extern int get_waypoint_database_last_to_reach_flag (entity_sub_types waypoint_type, entity_types mobile_type);

//extern int get_waypoint_database_waypoint_criteria_value (entity_sub_types waypoint_type, entity_types mobile_type);

extern int get_waypoint_database_position_type_value (entity_sub_types waypoint_type, entity_types mobile_type);

extern int get_waypoint_database_movement_type_value (entity_sub_types waypoint_type, entity_types mobile_type);

extern float get_waypoint_database_minimum_previous_waypoint_distance (entity_sub_types waypoint_type, entity_types mobile_type);





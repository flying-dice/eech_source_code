

































































#define HELICOPTER_TERRAIN_FOLLOW_LOOKAHEAD_TIME				2.0





extern void helicopter_movement (entity *en);

extern void helicopter_death_movement (entity *en);

extern void helicopter_impact_movement (entity *en);

extern void update_player_helicopter_waypoint_distance (entity *en);

extern void get_waypoint_display_position (entity *en, entity *wp, vec3d *wp_pos);

extern void debug_visuals (entity *en, vec3d *tg_pos, float lifetime);





extern float
	min_coning_angle,
	max_coning_angle;





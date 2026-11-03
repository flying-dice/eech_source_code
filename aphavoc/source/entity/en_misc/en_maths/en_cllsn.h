

































































#define LARGEST_3D_OBJECT_RADIUS	(150.0f)





extern int
	draw_collision_boxes;

extern float
	collision_box_lifetime;





extern terrain_classes get_ground_impact_point (vec3d *above_ground, vec3d *below_ground, vec3d *ground_impact_point);

extern entity *get_line_of_sight_collision_entity
(
	entity *los_source,
	entity *los_target,
	vec3d *source_old_position,
	vec3d *source_new_position,
	vec3d *source_intercept_point,
	vec3d *face_normal
);

extern int get_line_of_sight_collision_tree
(
	vec3d *source_old_position,
	vec3d *source_new_position,
	vec3d *source_intercept_point,
	vec3d *face_normal,
	int ignore_forest_edge_trees,
	int skip_tree_top_test
);

































































#ifndef _CROUTE_H
#define _CROUTE_H





struct ROUTE_NODE
{

	entity_sub_types
		type;

	int
		formation;

	entity
		*dependent;

	vec3d
		position;

	struct ROUTE_NODE
		*next,
		*prev;
};

typedef struct ROUTE_NODE route_node;





extern int
	assign_create_waypoint_route;

#define set_assign_create_waypoint_route(FLAG) (assign_create_waypoint_route = (FLAG))

#define get_assign_create_waypoint_route() (assign_create_waypoint_route)





extern int create_generic_waypoint_route (entity *group, entity *task_en, entity *return_keysite, vec3d *client_start, vec3d *client_stop, unsigned int *waypoint_indices, int indices_count);



extern void initialise_route_generator (void);

extern int generate_biased_vec3d_route (route_node *points, route_node **route, int side, movement_types movement_type);

extern void destroy_fast_route (route_node *route);





extern route_node *temp_create_generic_waypoint_route (entity *group, entity *task_en, entity *return_keysite, unsigned int *waypoint_indices, int indices_count);





#endif // _CROUTE_H



































































struct ROUTE_WAYPOINT_POSITION
{

	int
		number_of_positions;

	vec3d
		position;

	vec3d
		*offsets;
};

typedef struct ROUTE_WAYPOINT_POSITION route_waypoint_position;





extern void destroy_routegen_waypoint_routes ( void );

extern void parse_waypoint_routes_from_object ( int object_index, int number_of_matching_slots, vec3d *slots );





extern int
	number_of_route_waypoint_positions;

extern route_waypoint_position
	*route_waypoint_positions;






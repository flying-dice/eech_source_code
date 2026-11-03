

































































struct WAYPOINT
{
	entity_sub_types
		sub_type;

	list_root
		current_waypoint_root;

	list_link
		task_dependent_link,
		waypoint_link;

	float
		flight_time,
		altitude,
		heading;

	unsigned int
		waypoint_action_notified				: NUM_WAYPOINT_REACHED_BITS,
		position_type								: NUM_POSITION_TYPE_BITS,
		route_node									: NUM_ROUTE_NODE_BITS,
		tag											: NUM_TAG_BITS,
		waypoint_formation						: NUM_WAYPOINT_FORMATION_BITS;

	vec3d
		position;

	entity
		*position_entity;
};

typedef struct WAYPOINT waypoint;





#include	"wp_char.h"



#include	"wp_dbase.h"



























extern entity *get_local_entity_first_waypoint (entity *en);

extern entity *get_local_entity_current_waypoint (entity *en);

extern entity *get_local_entity_last_waypoint (entity *en);

extern entity *get_local_entity_next_waypoint (entity *en);

extern entity *get_local_entity_prev_waypoint (entity *en);





extern void get_local_waypoint_formation_position (int formation_position, entity *wp, vec3d *pos);



extern void get_local_waypoint_simple_formation_position (int formation_position, vec3d *origin, int type, vec3d *pos);





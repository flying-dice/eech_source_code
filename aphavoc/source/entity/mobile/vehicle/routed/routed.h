

































































struct ROUTED_VEHICLE
{
	vehicle
		vh;

	float
		desired_velocity;

	unsigned int
		//timer,
		sub_waypoint_count		: NUM_SUB_WAYPOINT_COUNT_BITS,
		waypoint_next_index		: NUM_WAYPOINT_INDEX_BITS,
		waypoint_this_index		: NUM_WAYPOINT_INDEX_BITS;

	struct NODE_LINK_DATA
		*sub_route;
};

typedef struct ROUTED_VEHICLE routed_vehicle;







#include "rv_creat.h"











//#include "rv_list.h"

#include "rv_move.h"

















extern void stop_vehicles_on_route (int start_node, int end_node);





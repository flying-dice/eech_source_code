

































































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





#include "rv_anim.h"

#include "rv_creat.h"

#include "rv_draw.h"

#include "rv_dstry.h"

#include "rv_float.h"

#include "rv_funcs.h"

#include "rv_int.h"

//#include "rv_list.h"

#include "rv_move.h"

#include "rv_msgs.h"

#include "rv_pack.h"

#include "rv_updt.h"





extern void attach_routed_vehicle_meta_smoke_lists (entity *en);

extern void pack_routed_vehicle_meta_smoke_lists (entity *en, pack_modes mode);

extern void unpack_routed_vehicle_meta_smoke_lists (entity *en, pack_modes mode);

extern void stop_vehicles_on_route (int start_node, int end_node);







































































struct CARGO
{
	mobile
		mob;

	list_root
		task_dependent_root;

	list_link
		cargo_link,
		movement_dependent_link;

	terrain_3d_point_data
		terrain_info;
};

typedef struct CARGO cargo;





#define CARGO_AMMO_SIZE 10

#define CARGO_FUEL_SIZE 10





#include "cg_creat.h"

#include "cg_draw.h"

#include "cg_dstry.h"

#include "cg_funcs.h"

#include "cg_int.h"

#include "cg_list.h"

#include "cg_move.h"

#include "cg_msgs.h"

#include "cg_pack.h"

#include "cg_ptr.h"

#include "cg_updt.h"





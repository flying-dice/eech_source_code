

































































struct SHIP_VEHICLE
{
	vehicle
		vh;
};

typedef struct SHIP_VEHICLE ship_vehicle;





#include "sh_anim.h"

#include "sh_creat.h"

#include "sh_draw.h"

#include "sh_dstry.h"

#include "sh_float.h"

#include "sh_funcs.h"

#include "sh_int.h"

#include "sh_move.h"

#include "sh_msgs.h"

#include "sh_pack.h"

#include "sh_updt.h"





extern void attach_ship_meta_smoke_lists (entity *en);

extern void pack_ship_meta_smoke_lists (entity *en, pack_modes mode);

extern void unpack_ship_meta_smoke_lists (entity *en, pack_modes mode);





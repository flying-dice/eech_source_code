

































































struct FIXED_WING
{
	aircraft
		ac;

	float
		afterburner_state,
		airbrake_state,
		flap_state,
		propellor_rpm,
		fixed_wing_toggle,
		turn_speed_multi;
};

typedef struct FIXED_WING fixed_wing;





#include "fw_anim.h"

#include "fw_creat.h"

#include "fw_draw.h"

#include "fw_dstry.h"

#include "fw_float.h"

#include "fw_funcs.h"

#include "fw_int.h"

#include "fw_list.h"

#include "fw_move.h"

#include "fw_msgs.h"

#include "fw_pack.h"

#include "fw_tgt.h"

#include "fw_updt.h"





extern void update_fixed_wing_crate_dropping (entity *en);





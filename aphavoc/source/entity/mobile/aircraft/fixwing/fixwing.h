

































































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







#include "fw_creat.h"













#include "fw_move.h"













extern void update_fixed_wing_crate_dropping (entity *en);





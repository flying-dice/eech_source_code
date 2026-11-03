

































































struct SITE_UPDATABLE
{
	fixed
		fix;

	float
		loading_door_timer,
		loading_door_state,
		radar_rotation_state,
		ground_crew_animation;

	int
		damage_level;

	list_link
		member_link,
		update_link;
};

typedef struct SITE_UPDATABLE site_updatable;





#include "su_anim.h"

#include "su_creat.h"

#include "su_draw.h"

#include "su_dstry.h"

#include "su_float.h"

#include "su_funcs.h"

#include "su_int.h"

#include "su_list.h"

#include "su_msgs.h"

#include "su_pack.h"

#include "su_updt.h"





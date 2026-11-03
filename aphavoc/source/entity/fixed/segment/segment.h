

































































struct SEGMENT
{
	fixed
		fix;

	list_link
		segment_link,
		update_link;

	int
		bridge_segment_type,
		damage_level;

	float
		altitude;
};

typedef struct SEGMENT segment;





extern void repair_local_segment_entity (entity *en);





#include "sg_creat.h"

#include "sg_dbase.h"

#include "sg_draw.h"

#include "sg_dstry.h"

#include "sg_float.h"

#include "sg_funcs.h"

#include "sg_int.h"

#include "sg_list.h"

#include "sg_msgs.h"

#include "sg_pack.h"

#include "sg_updt.h"





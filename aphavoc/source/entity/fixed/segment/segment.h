

































































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







#include "sg_dbase.h"























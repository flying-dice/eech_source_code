

































































#include "project.h"





#define DEBUG_MODULE 0





void repair_local_segment_entity (entity *en)
{
	segment
		*raw;

	ASSERT (en);

	raw = (segment *) get_local_entity_data (en);

	//
	// reset alive flag
	//

	raw->fix.alive = TRUE;

	//
	// reset object
	//

	raw->fix.object_3d_shape = segment_database [raw->bridge_segment_type].normal_object;

	//
	// reset position
	//

	raw->fix.position.y = raw->altitude;
}







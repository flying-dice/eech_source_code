

































































#include "project.h"





#define DEBUG_MODULE 0





float
	entity_movement_delta_time = 1.0;

int
	entity_movement_iterations = 1;





void set_entity_movement_frame_rate (int frame_rate)
{

	ASSERT ((frame_rate >= 1) && (frame_rate <= 100));

	entity_movement_iterations = (int) (get_delta_time () * frame_rate + 1.0);

	entity_movement_delta_time = get_delta_time () / entity_movement_iterations;






}







































































#include "project.h"





#define DEBUG_MODULE 0

































































static int response_to_set_entity_position (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	float
		velocity,
		heading;

	vec3d
		*position;

	entity
		*building;

	position = va_arg (pargs, vec3d *);

	heading = (float) va_arg (pargs, double);

	velocity = (float) va_arg (pargs, double);








	//
	// move landing entities
	//

	building = get_local_entity_first_child (receiver, LIST_TYPE_MEMBER);

	while (building)
	{

		notify_local_entity (ENTITY_MESSAGE_SET_ENTITY_POSITION, building, receiver, position, (double) heading, (double) velocity);

		building = get_local_entity_child_succ (building, LIST_TYPE_MEMBER);
	}

	//
	// Use full set_position function to keep sector data correct.
	//

	set_local_entity_vec3d (receiver, VEC3D_TYPE_POSITION, position);

	return (TRUE);
}




























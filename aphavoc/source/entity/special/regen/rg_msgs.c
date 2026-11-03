

































































#include "project.h"





#define DEBUG_MODULE 0





#if DEBUG_MODULE

static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





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

	#if DEBUG_MODULE

	debug_log ("RG_MSGS: setting position %f, %f, %f, heading %f",
					position->x, position->y, position->z, heading);

	#endif

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





void overload_regen_message_responses (void)
{
	overload_fixed_message_responses (ENTITY_TYPE_REGEN);

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_REGEN][ENTITY_MESSAGE_LINK_CHILD]							= response_to_link_child;

	message_responses[ENTITY_TYPE_REGEN][ENTITY_MESSAGE_UNLINK_CHILD]						= response_to_unlink_child;

	message_responses[ENTITY_TYPE_REGEN][ENTITY_MESSAGE_LINK_PARENT]						= response_to_link_parent;

	message_responses[ENTITY_TYPE_REGEN][ENTITY_MESSAGE_UNLINK_PARENT]						= response_to_unlink_parent;

	#endif

	message_responses[ENTITY_TYPE_REGEN][ENTITY_MESSAGE_SET_ENTITY_POSITION]					= response_to_set_entity_position;
}





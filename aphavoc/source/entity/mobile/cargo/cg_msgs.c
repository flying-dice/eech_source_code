

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	return (TRUE);
}





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

	vec3d
		*position;

	float
		velocity,
		heading;

	cargo
		*raw;

	raw = (cargo *) get_local_entity_data (receiver);

	position = va_arg (pargs, vec3d *);

	heading = (float) va_arg (pargs, double);

	velocity = (float) va_arg (pargs, double);

	//
	// should this notify children of movement (for mobile landing entities) ?
	//

	set_local_entity_vec3d (receiver, VEC3D_TYPE_POSITION, position);

	return (TRUE);
}





void overload_cargo_message_responses (void)
{
	overload_aircraft_message_responses (ENTITY_TYPE_CARGO);

	message_responses[ENTITY_TYPE_CARGO][ENTITY_MESSAGE_COLLISION]					= response_to_collision;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_CARGO][ENTITY_MESSAGE_LINK_CHILD]					= response_to_link_child;

	message_responses[ENTITY_TYPE_CARGO][ENTITY_MESSAGE_UNLINK_CHILD]					= response_to_unlink_child;

	message_responses[ENTITY_TYPE_CARGO][ENTITY_MESSAGE_LINK_PARENT]					= response_to_link_parent;

	message_responses[ENTITY_TYPE_CARGO][ENTITY_MESSAGE_UNLINK_PARENT]				= response_to_unlink_parent;

	#endif

	message_responses[ENTITY_TYPE_CARGO][ENTITY_MESSAGE_SET_ENTITY_POSITION]		= response_to_set_entity_position;
}







































































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





void overload_crater_message_responses (void)
{
	overload_fixed_message_responses (ENTITY_TYPE_CRATER);

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_PYLON][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	message_responses[ENTITY_TYPE_PYLON][ENTITY_MESSAGE_UNLINK_CHILD]		= response_to_unlink_child;

	message_responses[ENTITY_TYPE_PYLON][ENTITY_MESSAGE_LINK_PARENT]		= response_to_link_parent;

	message_responses[ENTITY_TYPE_PYLON][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif
}





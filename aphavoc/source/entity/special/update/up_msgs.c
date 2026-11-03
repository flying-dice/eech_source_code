

































































#include "project.h"





#define DEBUG_MODULE 0





#if DEBUG_MODULE

static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		type;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	type = va_arg (pargs, list_types);

	if (type == LIST_TYPE_UPDATE)
	{
		if (sender == get_update_succ ())
		{
			set_update_succ (get_local_entity_child_succ (sender, LIST_TYPE_UPDATE));
		}
	}

	return (TRUE);
}





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





void overload_update_message_responses (void)
{
	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	#endif

	message_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_LINK_PARENT]		= response_to_link_parent;

	message_responses[ENTITY_TYPE_UPDATE][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif
}





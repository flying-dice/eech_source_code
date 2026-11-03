

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	#if DEBUG_MODULE

	list_types
		list_type;

	debug_log_entity_message (message, receiver, sender, pargs);

	list_type = va_arg (pargs, list_types);

	if (list_type == LIST_TYPE_AIR_TASK)
	{

		debug_log ("SS_MSGS: adding task %d to air task list", sender);
	}
	else if (list_type == LIST_TYPE_ASSIGNED_AIR_TASK)
	{

		debug_log ("SS_MSGS: adding task %d to assigned air task list", sender);
	}

	#endif

	return (TRUE);
}





static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	#if DEBUG_MODULE

	list_types
		list_type;

	debug_log_entity_message (message, receiver, sender, pargs);

	list_type = va_arg (pargs, list_types);

	if (list_type == LIST_TYPE_AIR_TASK)
	{

		debug_log ("SS_MSGS: removing task %d to air task list", sender);
	}
	else if (list_type == LIST_TYPE_ASSIGNED_AIR_TASK)
	{

		debug_log ("SS_MSGS: removing task %d to assigned air task list", sender);
	}

	#endif

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





void overload_session_message_responses (void)
{
	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_SESSION][ENTITY_MESSAGE_LINK_PARENT]	= response_to_link_parent;

	message_responses[ENTITY_TYPE_SESSION][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif

	message_responses[ENTITY_TYPE_SESSION][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	message_responses[ENTITY_TYPE_SESSION][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;
}





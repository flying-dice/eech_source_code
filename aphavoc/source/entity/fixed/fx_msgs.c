

































































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





static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	list_type = va_arg (pargs, list_types);

	switch (list_type)
	{

		case LIST_TYPE_GUNSHIP_TARGET:

		{
			set_local_entity_int_value (receiver, INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR, TRUE);

			break;
		}
	}

	return (TRUE);
}





static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	list_type = va_arg (pargs, list_types);

	switch (list_type)
	{

		case LIST_TYPE_GUNSHIP_TARGET:

		{
			set_local_entity_int_value (receiver, INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR, FALSE);

			break;
		}
	}

	return (TRUE);
}





void overload_fixed_message_responses (entity_types type)
{
	#if DEBUG_MODULE

	message_responses[type][ENTITY_MESSAGE_LINK_CHILD] 	= response_to_link_child;

	message_responses[type][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;

	#endif

	message_responses[type][ENTITY_MESSAGE_LINK_PARENT]	= response_to_link_parent;

	message_responses[type][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;
}





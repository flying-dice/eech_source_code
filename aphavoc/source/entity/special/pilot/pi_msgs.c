

































































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

	if (list_type == LIST_TYPE_AIRCREW)
	{
		//
		// notify group screen to update when player assumes control of helicopter
		//
		
		notify_campaign_screen (CAMPAIGN_SCREEN_GROUP_ADD_MEMBER, sender);
	}
	else if (list_type == LIST_TYPE_PILOT)
	{
		campaign_screen_update_chat_page_pilot (receiver);
	}
	
	return (TRUE);
}





#if DEBUG_MODULE

static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





void overload_pilot_message_responses (void)
{
	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_PILOT][ENTITY_MESSAGE_LINK_CHILD]					= response_to_link_child;

	message_responses[ENTITY_TYPE_PILOT][ENTITY_MESSAGE_UNLINK_CHILD]					= response_to_unlink_child;

	message_responses[ENTITY_TYPE_PILOT][ENTITY_MESSAGE_UNLINK_PARENT]				= response_to_unlink_parent;

	#endif

	message_responses[ENTITY_TYPE_PILOT][ENTITY_MESSAGE_LINK_PARENT]					= response_to_link_parent;
}





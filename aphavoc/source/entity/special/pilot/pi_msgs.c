

































































#include "project.h"





#define DEBUG_MODULE 0



































static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;







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







































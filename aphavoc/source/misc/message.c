

































































#include "project.h"





#define DEBUG_MODULE	0





const message_text_info_type
	message_text_info [NUM_MESSAGE_TEXT_TYPES] =
	{
			//send string		//echo		//Skip Queue	// display time
		{	TRUE,					TRUE, 		FALSE,			5.0	},			// MESSAGE_TEXT_SYSTEM_NEW_PILOT,
		{	FALSE,				TRUE, 		FALSE,			5.0	},			// MESSAGE_TEXT_SYSTEM_ENTITY_KILLED,
		{	FALSE,				TRUE, 		FALSE,			4.0	},			// MESSAGE_TEXT_SYSTEM_ENTITY_UNDER_ATTACK,
		{	TRUE,					TRUE, 		FALSE,			8.0	},			// MESSAGE_TEXT_SYSTEM_RECON_DATA,
		{	FALSE,				TRUE, 		FALSE,			4.0	},			// MESSAGE_TEXT_SYSTEM_TASK_COMPLETED,
		{	FALSE,				TRUE, 		FALSE,			10.0	},			// MESSAGE_TEXT_SYSTEM_KEYSITE_CAPTURED,
		{	FALSE,				TRUE, 		FALSE,			10.0	},			// MESSAGE_TEXT_SYSTEM_KEYSITE_DESTROYED,
		{	FALSE,				TRUE,		FALSE,			5.0	},			// MESSAGE_WINGMAN_STATUS_REPORT,
		{	TRUE,					TRUE, 		TRUE,				8.0	},			// MESSAGE_TEXT_PILOT_STRING,
		{	TRUE,					TRUE, 		TRUE,				8.0	},			// MESSAGE_TEXT_WINGMAN_STRING,
		{	TRUE,					TRUE, 		FALSE,				2.0	},			// MESSAGE_TEXT_SHORT_WINGMAN_STRING,
		{	FALSE,					TRUE, 		TRUE,				5.0	},			// MESSAGE_TEXT_LOCAL_MESSAGE
	};









void display_in_flight_messages ()
{
	if (get_incoming_in_flight_message_flag () || get_outgoing_in_flight_message_flag ())
	{
		display_in_flight_incoming_messages ();

		display_in_flight_outgoing_messages ();
	}
}





int message_text_type_send_string (message_text_types type)
{
	ASSERT ((type >= 0) && (type < NUM_MESSAGE_TEXT_TYPES));

	return message_text_info [type].send_string;
}






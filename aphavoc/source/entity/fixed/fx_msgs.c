

































































#include "project.h"





#define DEBUG_MODULE 0



































static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;







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
























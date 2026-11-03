

































































#include "project.h"





//
// 2 levels of debug
//

#define DEBUG_MODULE 0











































































































































































































static int response_to_waypoint_reverse_convoy_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	routed_vehicle
		*raw;

	raw = (routed_vehicle *) get_local_entity_data (receiver);







	//
	// Reverse order of formation to make them fill the landed formation safely.
	// Otherwise they will fill from the centre out.
	//	NB. only do when first member reached this waypoint
	//
/*
	if (raw->vh.formation_position == 0)
	{

		entity
			*group;

		group = raw->vh.member_link.parent;

		if (group)
		{

			entity
				*member;

			int
				max_formation_position;

			max_formation_position = get_local_group_member_max_formation_position (group);

			member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

			while (member)
			{

				set_client_server_entity_int_value (member, INT_TYPE_FORMATION_POSITION, max_formation_position - get_local_entity_int_value (member, INT_TYPE_FORMATION_POSITION));

				member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
			}
		}
	}
*/
	return (TRUE);
}
































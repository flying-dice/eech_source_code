

































































#include "project.h"






//
// 2 levels of debug
//


#define DEBUG_MODULE 0

#define FATAL 0






























































































static int response_to_landing_site_request (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	formation_type
		*formation_data;

	unsigned int
		landed_lock,
		landing_lock,
		member_number,
		formation_landing,
		formation_position;

	entity
		*member,
		*guide,
		*keysite,
		*fw_landing,
		*incoming_group,
		*landing_task,
		*holding_task,
		*landing_route_first_wp;

	landing
		*raw;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);







	incoming_group = get_local_entity_parent (sender, LIST_TYPE_MEMBER);

	member = get_local_entity_first_child (incoming_group, LIST_TYPE_MEMBER);

	//
	// Check for lock
	//







	landing_task = get_local_landing_entity_task (receiver, ENTITY_SUB_TYPE_TASK_LANDING);

	holding_task = get_local_landing_entity_task (receiver, ENTITY_SUB_TYPE_TASK_LANDING_HOLDING);

	landing_route_first_wp = get_local_entity_first_child (landing_task, LIST_TYPE_WAYPOINT);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (landing_route_first_wp, INT_TYPE_WAYPOINT_FORMATION));

	while (member)
	{

		member_number = get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER);

		//
		// unreserve landing site
		//

		notify_local_entity (ENTITY_MESSAGE_UNRESERVE_LANDING_SITE, receiver, member, 1);

		//
		// Calculate formation position
		//

		formation_position = 0;

		landed_lock = get_local_entity_int_value (receiver, INT_TYPE_LANDED_LOCK);

		while ((formation_position < (unsigned int)raw->total_landing_sites) && ((1 << formation_position) & (raw->landed_lock)))
		{

			formation_position ++;
		}

		if (get_local_entity_int_value (member, INT_TYPE_FORMATION_POSITION) != formation_position)
		{

			set_client_server_entity_int_value (member, INT_TYPE_FORMATION_POSITION, formation_position);
		}







		formation_landing = formation_position % formation_data->number_in_formation;







		landing_lock = get_local_entity_int_value (receiver, INT_TYPE_LANDING_LOCK);

		keysite = get_local_entity_parent (receiver, LIST_TYPE_LANDING_SITE);

		fw_landing = get_local_entity_landing_entity (keysite, ENTITY_SUB_TYPE_LANDING_FIXED_WING);

		if ((get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT) &&
			(get_local_entity_int_value (fw_landing, INT_TYPE_LANDING_LOCK)))
		{

			debug_log ("LD_MSGS: can't land TRANSPORT %s (%d) atm, something else is landing on the FW route", get_local_entity_string (member, STRING_TYPE_FULL_NAME), get_local_entity_index (member));
		}

		// See if the member can land.... or if its a TRANSPORT check if the fw route is clear as well...

		//if (landing_lock & (1 << formation_landing))
		if ((landing_lock & (1 << formation_landing)) ||
			((get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT) &&
			(get_local_entity_int_value (fw_landing, INT_TYPE_LANDING_LOCK))))
		{
		
			//
			// Place member on holding route
			//

			#if FATAL

			ASSERT (get_local_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE) != OPERATIONAL_STATE_LANDING_HOLDING);

			#endif

			set_client_server_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_LANDING_HOLDING);







			guide = create_client_server_guide_entity (holding_task, NULL, (1 << member_number));

			attach_group_to_guide_entity (incoming_group, guide);

			attach_group_member_to_guide_entity (member, guide);

			//
			// Lock landing site
			//

			notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_SITE, receiver, member);
	

			//
			// SPECIAL_EFFECT_HOOK FOR PREPARING FOR LANDING-HOLDING PATTERN
			//





































			//
			//

		}
		else
		{
			
			//
			// Place member on landing route
			//

			#if FATAL

			ASSERT (get_local_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE) != OPERATIONAL_STATE_LANDING);

			#endif

			set_client_server_entity_int_value (member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_LANDING);

			//
			//
			//
	






			guide = create_client_server_guide_entity (landing_task, NULL, (1 << member_number));

			attach_group_to_guide_entity (incoming_group, guide);

			attach_group_member_to_guide_entity (member, guide);

			notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_ROUTE, receiver, member);
		
			//
			// Lock landing site and route
			//

			notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_SITE, receiver, member);
	

			//
			// SPECIAL_EFFECT_HOOK FOR LANDING GRANTED
			//







































			//
			//

		}
	
		//
		// Set member's internal wp parameters
		//
	
		member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
	}

	return (FALSE);
}





static int response_to_lock_landing_route (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	formation_type
		*formation_data;

	entity
		*wp;

	landing
		*raw;

	unsigned int
		formation_landing;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	formation_landing = get_local_entity_int_value (sender, INT_TYPE_FORMATION_POSITION);

	wp = get_local_landing_entity_route (receiver, ENTITY_SUB_TYPE_TASK_LANDING);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	formation_landing %= formation_data->number_in_formation;










































	#if FATAL

	ASSERT (!(raw->landing_lock & (1 << formation_landing)));

	#endif

	raw->landing_lock |= 1 << formation_landing;


	//
	// SPECIAL_EFFECT_HOOK FOR PREPARING FOR LANDING
	//



	//
	//






























												
	return (TRUE);
}





static int response_to_lock_landing_site (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	landing
		*raw;

	unsigned int
		formation_landing;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	formation_landing = get_local_entity_int_value (sender, INT_TYPE_FORMATION_POSITION);

	#if FATAL

	ASSERT (!(raw->landed_lock & (1 << formation_landing)));

	#endif
	
















	raw->landed_lock |= 1 << formation_landing;

	raw->free_landing_sites --;

























	#if FATAL

	ASSERT (raw->free_landing_sites >= 0);

	#endif


	//
	// SPECIAL_EFFECT_HOOK FOR REQUESTING LANDING SITE
	//



	//
	//































	return (TRUE);
}





static int response_to_lock_takeoff_route (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	formation_type
		*formation_data;

	entity
		*wp;

	landing
		*raw;

	unsigned int
		formation_takeoff;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	formation_takeoff = get_local_entity_int_value (sender, INT_TYPE_FORMATION_POSITION);

	wp = get_local_landing_entity_route (receiver, ENTITY_SUB_TYPE_TASK_TAKEOFF);

	//
	// get the formation at the end of the takeoff route, i.e. at the taken off waypoint
	//
	
	while (get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT))
	{

		wp = get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT);
	}

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	formation_takeoff %= formation_data->number_in_formation;

	if (!(raw->takeoff_lock & (1 << formation_takeoff)))
	{
	

















	
		raw->takeoff_lock |= 1 << formation_takeoff;


		//
		// SPECIAL_EFFECT_HOOK FOR PREPARING FOR TAKEOFF
		//


		{
			//
			// open hanger doors if there is one,
			// 	search through buildings looking for the nearest (within a range)
			//

			entity
				*group,
				*hanger,
				*keysite,
				*building;

			float
				sqr_range;

			vec3d
				*en_pos,
				*building_pos;

			keysite = get_local_entity_parent (receiver, LIST_TYPE_LANDING_SITE);

			group = get_local_entity_first_child (keysite, LIST_TYPE_BUILDING_GROUP);

			building = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

			en_pos = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);

			hanger = NULL;

			while (building)
			{

				building_pos = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);

				sqr_range = get_sqr_3d_range (en_pos, building_pos);

				if (sqr_range < 2.0)
				{

					//
					// early out.
					//

					hanger = building;

					break;
				}

				building = get_local_entity_child_succ (building, LIST_TYPE_MEMBER);
			}

			if (hanger)
			{

				open_client_server_entity_loading_doors (hanger);

				set_local_entity_float_value (hanger, FLOAT_TYPE_LOADING_DOOR_TIMER, 30.0);
			}
		}


		//
		// SPECIAL_EFFECT_HOOK FOR TAKEOFF GRANTED
		//


		// takeoff speech played in mb_msgs.c
		

		//
		//






























		return (TRUE);
	}








	//
	// SPECIAL_EFFECT_HOOK FOR DENIED TAKEOFF
	//










































	//
	//


	return (FALSE);
}





static int response_to_reserve_landing_site (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	landing
		*raw;

	int
		count;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	count = va_arg (pargs, int);
	


















	raw->reserved_landing_sites += count;

























	#if FATAL

	ASSERT (raw->reserved_landing_sites <= raw->total_landing_sites);

	#endif


	//
	// SPECIAL_EFFECT_HOOK FOR RESERVE LANDING SITE
	//


































	//
	//


	return (TRUE);
}





























































static int response_to_task_terminated (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	formation_type
		*formation_data;

	unsigned int
		exit,
		landing_lock,
		member_number,
		formation_landing;

	entity
		*guide,
		*keysite,
		*landing_task,
		*holding_task,
		*holding_member,
		*next_group,
		*incoming_group,
		*next_landing_member,
//		*landing_route_last_wp,
		*landing_route_first_wp;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);







	incoming_group = get_local_entity_parent (sender, LIST_TYPE_MEMBER);

	//
	// Unlock landing route 
	//

//	if (get_local_entity_int_value (sender, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
	{

		notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_ROUTE, receiver, sender);
	}

	//
	// Clear landing route for this member
	//

	landing_task = get_local_landing_entity_task (receiver, ENTITY_SUB_TYPE_TASK_LANDING);

	landing_route_first_wp = get_local_entity_first_child (landing_task, LIST_TYPE_WAYPOINT);

//	set_local_landing_route_clear (landing_task, sender);

	//
	// Land any members that are holding and landing site is free
	// NB. if TRANSPORT flip back to FW landing
	//

	if (get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT)
	{

		debug_log ("LD_MSGS: TRANSPORT %s %d terminated landing... switching to FW landing en", get_local_entity_string (sender, STRING_TYPE_FULL_NAME), get_local_entity_index (sender));

		keysite = get_local_entity_parent (receiver, LIST_TYPE_LANDING_SITE);

		if (get_local_entity_landing_entity (keysite, ENTITY_SUB_TYPE_LANDING_FIXED_WING))
		{
	
			receiver = get_local_entity_landing_entity (keysite, ENTITY_SUB_TYPE_LANDING_FIXED_WING);
	
			landing_task = get_local_landing_entity_task (receiver, ENTITY_SUB_TYPE_TASK_LANDING);
	
			landing_route_first_wp = get_local_entity_first_child (landing_task, LIST_TYPE_WAYPOINT);
		}
	}

	holding_task = get_local_landing_entity_task (receiver, ENTITY_SUB_TYPE_TASK_LANDING_HOLDING);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (landing_route_first_wp, INT_TYPE_WAYPOINT_FORMATION));

	next_landing_member = NULL;

	exit = FALSE;

	landing_lock = get_local_entity_int_value (receiver, INT_TYPE_LANDING_LOCK);

	guide = get_local_entity_first_child (holding_task, LIST_TYPE_GUIDE);

	if (guide)
	{
		//
		// find last entity in the list
		//
		
		while (get_local_entity_child_succ (guide, LIST_TYPE_GUIDE))
		{
			guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE);
		}

		//
		// spin backwards through the list
		//

		while ((guide) && (!exit))
		{

			holding_member = get_local_entity_first_child (guide, LIST_TYPE_FOLLOWER);

			while (holding_member)
			{

				formation_landing = get_local_entity_int_value (holding_member, INT_TYPE_FORMATION_POSITION);

				formation_landing %= formation_data->number_in_formation;

				if (!(landing_lock & (1 << formation_landing)))
				{

					next_landing_member = holding_member;

					exit = TRUE;

					break;
				}













				holding_member = get_local_entity_child_succ (holding_member, LIST_TYPE_FOLLOWER);
			}

			guide = get_local_entity_child_pred (guide, LIST_TYPE_GUIDE);
		}
	}

	if (next_landing_member)
	{
		//
		// remove member from holding route
		//

		guide = get_local_entity_parent (next_landing_member, LIST_TYPE_FOLLOWER);

		ASSERT (get_local_entity_parent (guide, LIST_TYPE_GUIDE) == holding_task);

		destroy_client_server_entity (guide);

		//
		// Place member on landing route
		//

		next_group = get_local_entity_parent (next_landing_member, LIST_TYPE_MEMBER);






	
		//
		// Lock landing route and set operational state
		//

		member_number = get_local_entity_int_value (next_landing_member, INT_TYPE_GROUP_MEMBER_NUMBER);

		guide = create_client_server_guide_entity (landing_task, NULL, (1 << member_number));

		attach_group_to_guide_entity (next_group, guide);

		attach_group_member_to_guide_entity (next_landing_member, guide);

		notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_ROUTE, receiver, next_landing_member);

		#if FATAL

		ASSERT (get_local_entity_int_value (next_landing_member, INT_TYPE_OPERATIONAL_STATE) != OPERATIONAL_STATE_LANDING);

		#endif

		set_client_server_entity_int_value (next_landing_member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_LANDING);
	}
	else
	{

		if (get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_LANDING_FIXED_WING)
		{

			//
			// check TRANSPORT holding pattern
			//

			if (!get_local_entity_int_value (receiver, INT_TYPE_LANDING_LOCK))
			{

				entity
					*keysite,
					*transport_landing;

				debug_log ("LD_MSGS: no more FIXED WING landing, checking TRANSPORT");

				keysite = get_local_entity_parent (receiver, LIST_TYPE_LANDING_SITE);

				transport_landing = get_local_entity_landing_entity (keysite, ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT);

				if (transport_landing)
				{
	
					holding_task = get_local_landing_entity_task (transport_landing, ENTITY_SUB_TYPE_TASK_LANDING_HOLDING);
	
					ASSERT (holding_task);
	
					guide = get_local_entity_first_child (holding_task, LIST_TYPE_GUIDE);
	
					if (guide)
					{
		
						holding_member = get_local_entity_first_child (guide, LIST_TYPE_FOLLOWER);
		
						if (holding_member)
						{

							debug_log ("LD_MSGS: got a TRANSPORT %s", get_local_entity_string (holding_member, STRING_TYPE_FULL_NAME), get_local_entity_index (holding_member));
	
							landing_task = get_local_landing_entity_task (transport_landing, ENTITY_SUB_TYPE_TASK_LANDING);
					
							destroy_client_server_entity (guide);
					
							//
							// Place member on landing route
							//
					
							next_group = get_local_entity_parent (holding_member, LIST_TYPE_MEMBER);
					
							//#if DEBUG_MODULE
				
							debug_log ("LD_MSGS:%d, inserting TRANSPORT landing holding %s (%d: AI) into 1st landing route waypoint", get_local_entity_index (transport_landing), get_local_entity_string (holding_member, STRING_TYPE_FULL_NAME), get_local_entity_index (holding_member));
					
							//#endif
						
							//
							// Lock landing route and set operational state
							//
				
							member_number = get_local_entity_int_value (holding_member, INT_TYPE_GROUP_MEMBER_NUMBER);
				
							guide = create_client_server_guide_entity (landing_task, NULL, (1 << member_number));
				
							attach_group_to_guide_entity (next_group, guide);
				
							attach_group_member_to_guide_entity (holding_member, guide);
				
							notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_ROUTE, transport_landing, holding_member);

							set_client_server_entity_int_value (holding_member, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_LANDING);
						}
					}
				}
			}
		}
	}

	//
	// Set group idle if all landed
	//
	{
		entity
			*landed_member;

		int
			count = 0,
			member_count;

		member_count = get_local_entity_int_value (incoming_group, INT_TYPE_MEMBER_COUNT);

		landed_member = get_local_entity_first_child (incoming_group, LIST_TYPE_MEMBER);

		while (landed_member)
		{

			if (get_local_entity_int_value (landed_member, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_LANDED)
			{

				count ++;
			}

			landed_member = get_local_entity_child_succ (landed_member, LIST_TYPE_MEMBER);
		}

		if (count == member_count)
		{







			set_local_entity_float_value (incoming_group, FLOAT_TYPE_SLEEP, 15 * ONE_MINUTE);
		}


		//

	}

	return (TRUE);
}





static int response_to_unlock_landing_route (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	formation_type
		*formation_data;

	entity
		*wp;

	landing
		*raw;

	unsigned int
		formation_landing;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	formation_landing = get_local_entity_int_value (sender, INT_TYPE_FORMATION_POSITION);

	wp = get_local_landing_entity_route (receiver, ENTITY_SUB_TYPE_TASK_LANDING);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	formation_landing %= formation_data->number_in_formation;










































	#if FATAL

	ASSERT (raw->landing_lock & (1 << formation_landing));

	#endif

	formation_landing = ~(1 << formation_landing);

	raw->landing_lock &= formation_landing;


	//
	// SPECIAL_EFFECT_HOOK FOR LANDING COMPLETE
	//



	//
	//































	return (TRUE);
}





static int response_to_unlock_landing_site (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	landing
		*raw;

	unsigned int
		formation_landing;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	formation_landing = get_local_entity_int_value (sender, INT_TYPE_FORMATION_POSITION);

	#if FATAL

	ASSERT (raw->landed_lock & (1 << formation_landing));

	#endif



















	formation_landing = ~(1 << formation_landing);

	raw->landed_lock &= formation_landing;

	raw->free_landing_sites ++;

























	#if FATAL

	ASSERT (raw->free_landing_sites <= raw->total_landing_sites);

	#endif


	//
	// SPECIAL_EFFECT_HOOK FOR PREPARING FOR TAKEOFF
	//



	//
	//































	return (TRUE);
}





static int response_to_unlock_takeoff_route (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	formation_type
		*formation_data;

	entity
		*wp,
		*member;

	landing
		*raw;

	unsigned int
		formation_takeoff;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	formation_takeoff = get_local_entity_int_value (sender, INT_TYPE_FORMATION_POSITION);

	wp = get_local_landing_entity_route (receiver, ENTITY_SUB_TYPE_TASK_TAKEOFF);

	// test, get the formation at the end of the takeoff route, i.e. at the taken off waypoint
	while (get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT))
	{

		wp = get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT);
	}
	// end test

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	formation_takeoff %= formation_data->number_in_formation;









































	#if FATAL

	ASSERT (raw->takeoff_lock & (1 << formation_takeoff));

	#endif

	formation_takeoff = ~(1 << formation_takeoff);

	raw->takeoff_lock &= formation_takeoff;


	//
	// SPECIAL_EFFECT_HOOK FOR TAKEOFF COMPLETE
	//



	//
	//































	//
	// spin through takeoff queue (in reverse order)
	//

	member = get_local_entity_first_child (receiver, LIST_TYPE_TAKEOFF_QUEUE);

	if (member)
	{
		//
		// find last entity in the list
		//
		
		while (get_local_entity_child_succ (member, LIST_TYPE_TAKEOFF_QUEUE))
		{
			member = get_local_entity_child_succ (member, LIST_TYPE_TAKEOFF_QUEUE);
		}

		//
		// spin backwards through the list
		//

		while (member)
		{
			if (notify_local_entity (ENTITY_MESSAGE_LOCK_TAKEOFF_ROUTE, receiver, member))
			{
				//#if DEBUG_MODULE

				debug_log ("LD_MSGS: %s (%d) finished taking off - inserting %s (%d) into takeoff route",
											get_local_entity_string (sender, STRING_TYPE_FULL_NAME),
											get_local_entity_index (sender),
											get_local_entity_string (member, STRING_TYPE_FULL_NAME),
											get_local_entity_index (member));

				//#endif

				insert_mobile_into_takeoff_route (member, receiver);

				delete_local_entity_from_parents_child_list (member, LIST_TYPE_TAKEOFF_QUEUE);

				break;
			}
			else
			{
				//#if DEBUG_MODULE

				debug_log ("LD_MSGS: %s (%d) finished taking off - CAN'T insert %s (%d) into takeoff route",
											get_local_entity_string (sender, STRING_TYPE_FULL_NAME),
											get_local_entity_index (sender),
											get_local_entity_string (member, STRING_TYPE_FULL_NAME),
											get_local_entity_index (member));

				//#endif
			}

			member = get_local_entity_child_pred (member, LIST_TYPE_TAKEOFF_QUEUE);
		}
	}

	return (TRUE);
}





static int response_to_unreserve_landing_site (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	landing
		*raw;

	int
		count;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (landing *) get_local_entity_data (receiver);

	count = va_arg (pargs, int);










































	raw->reserved_landing_sites -= count;

	#if FATAL

	ASSERT (raw->reserved_landing_sites >= 0);

	#endif


	//
	// SPECIAL_EFFECT_HOOK FOR NEGOTIATE LANDING POSITION
	//



	//
	//































	return (TRUE);
}





static int response_to_waypoint_taken_off_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

//	entity
//		*task;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);







	// get takeoff group





	//
	// Unlock takeoff
	//

	notify_local_entity (ENTITY_MESSAGE_UNLOCK_TAKEOFF_ROUTE, receiver, sender);

	return (TRUE);
}

































































































































































































































































































































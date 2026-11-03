

































































#include "project.h"





#define DEBUG_MODULE 					0

#define DEBUG_MODULE_MESSAGE_TEXT	0

























































































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

			set_local_entity_float_value (receiver, FLOAT_TYPE_AIR_RADAR_CONTACT_TIMEOUT, AIR_RADAR_CONTACT_TIMEOUT);

			// show icon on map if gunship scans it.
/*
			if (get_local_entity_int_value (receiver, INT_TYPE_SIDE) == get_global_gunship_side ())
			{

				set_planner_icon_update (receiver, FALSE);
			}
*/
			break;
		}

		case LIST_TYPE_TARGET:

		{
			entity
				*group;

			//
			// Notify the group that it has been targeted
			//
			
			if (get_comms_model () == COMMS_MODEL_SERVER)
			{
				if (get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_MOBILE))
				{
					group = get_local_entity_parent (sender, LIST_TYPE_MEMBER);

					if (group)
					{
						notify_local_entity (ENTITY_MESSAGE_ENTITY_TARGETED, group, receiver, sender);
					}
				}
			}

			//
			// if targetting the player then insert into the gunship target list if not already there
			//

			if (sender == get_gunship_entity ())
			{
				if (!get_local_entity_parent (receiver, LIST_TYPE_GUNSHIP_TARGET))
				{
					if (get_local_entity_int_value (receiver, INT_TYPE_TARGET_TYPE) != TARGET_TYPE_INVALID)
					{
						if (get_local_entity_int_value (receiver, INT_TYPE_THREAT_TYPE) != THREAT_TYPE_INVALID)
						{
							insert_local_entity_into_parents_child_list (receiver, LIST_TYPE_GUNSHIP_TARGET, sender, NULL);
						}
					}
				}
			}

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

			set_local_entity_float_value (receiver, FLOAT_TYPE_AIR_RADAR_CONTACT_TIMEOUT, AIR_RADAR_CONTACT_TIMEOUT_INVALID);

			break;
		}

		case LIST_TYPE_UPDATE:

		{
			set_local_entity_float_value (receiver, FLOAT_TYPE_VIEW_INTEREST_LEVEL, 0.0);

			break;
		}
	}

	return (TRUE);
}





static int response_to_task_terminated (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity
		*guide,
		*landing_en,
		*task;

	aircraft
		*raw;

	raw = (aircraft *) get_local_entity_data (receiver);







	debug_assert (raw->member_link.parent);

	//
	// Check if aircraft was on a landing/takeoff/holding route
	//

	guide = get_local_entity_parent (receiver, LIST_TYPE_FOLLOWER);

	ASSERT (guide);

	task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

	ASSERT (task);

	landing_en = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);

	switch (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE))
	{

		case ENTITY_SUB_TYPE_TASK_LANDING:
		{
			ASSERT (landing_en);







			if (get_local_entity_int_value (receiver, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
			{

				notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_ROUTE, landing_en, receiver);
			}

			notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_SITE, landing_en, receiver);

			break;
		}

		case ENTITY_SUB_TYPE_TASK_TAKEOFF:
		{
			ASSERT (landing_en);







			notify_local_entity (ENTITY_MESSAGE_UNLOCK_TAKEOFF_ROUTE, landing_en, receiver);

			break;
		}

		case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
		{
			ASSERT (landing_en);







			notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_SITE, landing_en, receiver);

			break;
		}

		case ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING:
		{
			ASSERT (landing_en);







			break;
		}
	}

	return (TRUE);
}





























































static int response_to_guide_cover_position_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*aggressor;
		
	float
		silence_timer;
		
	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_GUIDE);

	ASSERT (get_local_entity_int_value (receiver, INT_TYPE_GROUP_LEADER));

	if (get_local_entity_int_value (receiver, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
	{
		//
		// AI group leader notifies sender to fire straight away (PLAYER group leader must use radio message)
		//

		attack_guide_proceed_to_fire_positon (sender);

		return (TRUE);
	}

	aggressor = get_local_entity_first_child (sender, LIST_TYPE_FOLLOWER);

	ASSERT (aggressor);

	ASSERT (get_local_entity_int_value (aggressor, INT_TYPE_IDENTIFY_AIRCRAFT));

	if (!get_local_entity_int_value (aggressor, INT_TYPE_WEAPONS_HOLD))
	{
		//
		// If the aggressor is in weapons free mode, then he does not have to wait for the player's notification
		//

		attack_guide_proceed_to_fire_positon (sender);

		return (TRUE);
	}

	//
	// Play Speech
	//

	silence_timer = 15.0 + (frand1 () * 12.0);

	play_client_server_wingman_message
	(
		aggressor,
		0.8,
		1.5,
		SPEECH_CATEGORY_IN_POSITION,
		silence_timer,
		SPEECH_INTRODUCTION_ALWAYS,
		SPEECH_ARRAY_WINGMAN_MESSAGES,
		SPEECH_WINGMAN_IN_POSITION
	);

	return (TRUE);
}




































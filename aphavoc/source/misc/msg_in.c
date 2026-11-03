

































































#include "project.h"

#include "ai/taskgen/taskgen.h"
#include "ai/highlevl/reaction.h"





#define DEBUG_MODULE	0

#define MESSAGE_LIST_INC	100





static float
	current_message_timer = 0.0;

static message_log_type
	*message_log = NULL;

static int
	num_messages = 0,
	max_messages = 0,
	current_message = -1,
	group_wingman_cycle_forwards = FALSE;

static rgb_colour
	message_side_colour [NUM_ENTITY_SIDES];





static void clear_in_flight_received_message (void);

static void process_message_attack_my_target (entity *en, entity *wingman);

static void process_message_help_me (entity *en, entity *wingman);

static void process_message_return_to_base (entity *en);

static void process_message_weapons_hold (entity *en, entity *wingman, int state, int play_speech);

static void process_message_hold_position (entity *en, entity *wingman);

static void process_message_rejoin_formation (entity *en, entity *wingman);

static void process_message_bob_up (entity *en, entity *wingman);

static void process_message_change_formation (entity *en, entity *group, int new_formation);

static void process_message_request_airstrike (entity *en);

static void process_message_request_artillery (entity *en);

static void process_message_request_assistance (entity *en);

static void set_new_display_message (int index);

void process_message_attack_pfz(entity *en, entity *wingman);


















































































































static int message_intended_for_recipient (entity *sender, entity *target, message_text_types type)
{
	if (!get_pilot_entity ())
	{
		//
		// can't display messages until player is properly in the game
		//

		return FALSE;
	}

	switch (type)
	{
		case MESSAGE_TEXT_SYSTEM_NEW_PILOT:
		case MESSAGE_TEXT_SYSTEM_ENTITY_KILLED:
		case MESSAGE_TEXT_SYSTEM_KEYSITE_CAPTURED:
		case MESSAGE_TEXT_SYSTEM_KEYSITE_DESTROYED:
		case MESSAGE_TEXT_LOCAL_MESSAGE:
		{
			//
			// all pilots display these messages
			//

			return TRUE;
		}

		case MESSAGE_TEXT_SYSTEM_RECON_DATA:
		case MESSAGE_TEXT_SYSTEM_ENTITY_UNDER_ATTACK:
		{
			ASSERT (target);

			if (get_local_entity_int_value (target, INT_TYPE_SIDE) == get_local_entity_int_value (get_pilot_entity (), INT_TYPE_SIDE))
			{
				//
				// message broadcast to all pilots on this side
				//

				return TRUE;
			}

			break;
		}

		case MESSAGE_TEXT_SYSTEM_TASK_COMPLETED:
		{
			//
			// all pilots on this side display these messages - but only if the task was a primary task
			//

			ASSERT (target);

			if (task_database [get_local_entity_int_value (target, INT_TYPE_ENTITY_SUB_TYPE)].primary_task)
			{
				if (get_local_entity_int_value (target, INT_TYPE_SIDE) == get_local_entity_int_value (get_pilot_entity (), INT_TYPE_SIDE))
				{
					return TRUE;
				}
			}

			break;
		}

		case MESSAGE_TEXT_PILOT_STRING:
		{
			ASSERT (sender);

			ASSERT (target);

			if (target == get_pilot_entity ())
			{
				//
				// personal message for this pilot
				//

				return TRUE;
			}

			if (sender == get_pilot_entity ())
			{
				//
				// Echo Message sent by this pilot
				//

				return TRUE;
			}

			if (target == get_session_entity ())
			{
				//
				// message broadcast to all pilots
				//

				return TRUE;
			}

			ASSERT (target);

			if ((get_local_entity_type (target) == ENTITY_TYPE_FORCE) && (get_pilot_entity () != sender))
			{
				//
				// message broadcast to all pilots on this side (but don't display it if you sent it!!)
				//

				if (get_local_entity_int_value (target, INT_TYPE_SIDE) == get_local_entity_int_value (get_pilot_entity (), INT_TYPE_SIDE))
				{
					return TRUE;
				}
			}

			break;
		}

		case MESSAGE_TEXT_WINGMAN_STRING:
		case MESSAGE_TEXT_SHORT_WINGMAN_STRING:
		{
			entity
				*parent;

			ASSERT (sender);
			ASSERT (target);

			parent = get_local_entity_parent (get_pilot_entity (), LIST_TYPE_AIRCREW);

			if (target == parent)
			{
				//
				// personal message for this pilot
				//

				return TRUE;
			}

			break;
		}

		case MESSAGE_WINGMAN_STATUS_REPORT:
		{
			if (target == get_pilot_entity())
				return TRUE;

			break;
		}

		default:
		{
			debug_fatal ("MESSAGE: Unknown message type %d", type);
		}
	}

	return FALSE;
}






















































































































































































































































































































































































































































































void set_incoming_message (entity *sender, entity *target, message_text_types type, char *string)
{
	message_log_type
		*temp,
		*new_message;

	entity
		*subject;

	//
	// check that this player SHOULD display the message
	//

	if (!message_intended_for_recipient (sender, target, type))
	{
		return;
	}

	//
	// Need more log space ?
	//

	if (num_messages >= max_messages)
	{
		max_messages += MESSAGE_LIST_INC;

		temp = (message_log_type *) malloc_heap_mem (sizeof (message_log_type) * max_messages);

		if (num_messages > 0)
		{
			ASSERT (message_log);

			memcpy (temp, message_log, sizeof (message_log_type) * num_messages);

			free_mem (message_log);
		}

		message_log = temp;
	}

	//
	// add to log
	//

	new_message = &message_log [num_messages];

	new_message->type = type;

	if (get_session_entity ())
	{
		new_message->day = get_local_entity_int_value (get_session_entity (), INT_TYPE_DAY);

		new_message->time_of_day = get_local_entity_float_value (get_session_entity (), FLOAT_TYPE_TIME_OF_DAY);
	}
	else
	{
		new_message->day = 0;

		new_message->time_of_day = 0.0;
	}

	subject = format_incoming_message (new_message, sender, target, string);

	num_messages ++;

	// Jabberwock 031129 Chat server log
	if ((type == MESSAGE_TEXT_PILOT_STRING) && (get_comms_model () == COMMS_MODEL_SERVER))
	{
		server_log ("%s: %s", get_local_entity_string (sender, STRING_TYPE_PILOTS_NAME), string);
	}

	if ((current_message == -1) || (message_text_info [type].skip_queue))
	{
		set_new_display_message (num_messages - 1);
	}

	notify_campaign_screen (CAMPAIGN_SCREEN_NEW_MESSAGE, subject);

	set_message_objective_view_interest_level (sender, target, type);
}




















void process_radio_message (entity *en, message_categories type, int value)
{
	entity
		*group,
		*wingman;

	switch (type)
	{
		case MESSAGE_WINGMAN_ATTACK_MY_TARGET:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_attack_my_target (en, wingman);

			break;
		}
		case MESSAGE_WINGMAN_ATTACK_PFZ:
		{
			wingman = get_local_entity_safe_ptr (value);
			process_message_attack_pfz (en, wingman);

			break;
		}
		case MESSAGE_WINGMAN_RETURN_TO_BASE:
		{
			process_message_return_to_base (en);

			break;
		}
		case MESSAGE_WINGMAN_REQUEST_STATUS:
		{
			wingman = get_local_entity_safe_ptr (value);
			if (wingman == get_gunship_entity())
			{
				debug_log("request message from me!");
			}

			break;
		}
		case MESSAGE_WINGMAN_HELP_ME:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_help_me (en, wingman);

			break;
		}
		case MESSAGE_WINGMAN_WEAPONS_HOLD:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_weapons_hold (en, wingman, TRUE, TRUE);

			break;
		}
		case MESSAGE_WINGMAN_WEAPONS_FREE:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_weapons_hold (en, wingman, FALSE, TRUE);

			break;
		}
		case MESSAGE_WINGMAN_BOB_UP:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_bob_up (en, wingman);

			break;
		}
		case MESSAGE_WINGMAN_HOLD_POSITION:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_hold_position (en, wingman);

			break;
		}
		case MESSAGE_WINGMAN_REJOIN_FORMATION:
		{
			wingman = get_local_entity_safe_ptr (value);

			process_message_rejoin_formation (en, wingman);

			break;
		}
		case MESSAGE_WINGMAN_REQUEST_TARGET_LIST:
		{
			wingman = get_local_entity_safe_ptr (value);

			transmit_entity_comms_message (ENTITY_COMMS_REQUEST_TARGET_LIST, en, wingman);

			break;
		}
		case MESSAGE_GROUP_ROW_LEFT:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_ROW_LEFT);

			break;
		}
		case MESSAGE_GROUP_ROW_RIGHT:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_ROW_RIGHT);

			break;
		}
		case MESSAGE_GROUP_ECHELON_LEFT:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_ECHELON_LEFT);

			break;
		}
		case MESSAGE_GROUP_ECHELON_RIGHT:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_ECHELON_RIGHT);

			break;
		}
		case MESSAGE_GROUP_COLUMN:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_COLUMN);

			break;
		}
		case MESSAGE_GROUP_WEDGE:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_WEDGE);

			break;
	    }
//VJ 060305 three new formations for maverick ====>
		case MESSAGE_GROUP_DIAMOND:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_DIAMOND);

			break;
        }
		case MESSAGE_GROUP_STAGGERED_TRAIL_LEFT:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_STAGGERED_TRAIL_LEFT);

			break;
         }
		case MESSAGE_GROUP_STAGGERED_TRAIL_RIGHT:
		{
			group = get_local_entity_safe_ptr (value);

			process_message_change_formation (en, group, FORMATION_STAGGERED_TRAIL_RIGHT);

			break;

		}
//VJ 060305 three new formations for maverick <====
		case MESSAGE_LOCAL_BASE_REQUEST_AIRSTRIKE:
		{
			process_message_request_airstrike (en);

			break;
		}
		case MESSAGE_LOCAL_BASE_REQUEST_ARTILLERY:
		{
			process_message_request_artillery (en);

			break;
		}
		case MESSAGE_LOCAL_BASE_REQUEST_ASSISTANCE:
		{
			process_message_request_assistance (en);

			break;
		}
		case MESSAGE_LOCAL_BASE_TRANSMIT_RECON:
		{
			transmit_player_recon_data (en);

			break;
		}
// Jabberwock 031119 CamCom MP compatible
		case MESSAGE_LOCAL_BASE_CAMCOM_MESSAGE:
		{
			create_reaction_to_map_click (en);

			break;
		}
// Jabberwock 031119 ends
	}
}



































































































































































































































//
// MESSAGE SPECIFIC FUNCTIONS
//





//
//	ATTACK MY TARGET
//


void process_message_attack_my_target (entity *en, entity *wingman)
{
	entity
		*group,
		*target;

	unsigned int
		valid_members,
		member_number;

	ASSERT (en);

	ASSERT (wingman);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	target = get_local_entity_parent (en, LIST_TYPE_TARGET);

	if (target)
	{
		group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

		if (group)
		{
			if (get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
			{
//				if (get_local_entity_int_value (target, INT_TYPE_SIDE) != get_local_entity_int_value (group, INT_TYPE_SIDE))
				{
					if (get_local_entity_type (wingman) == ENTITY_TYPE_GROUP)
					{
						valid_members = TASK_ASSIGN_ALL_MEMBERS;
					}
					else
					{
						member_number = get_local_entity_int_value (wingman, INT_TYPE_GROUP_MEMBER_NUMBER);

						valid_members = (1 << member_number);
					}

					//
					// call to "attack my target" also cancels "weapons hold"
					//

					process_message_weapons_hold (en, wingman, FALSE, FALSE);

					//
					// Engage
					//

					engage_specific_target (group, target, valid_members, TRUE);

					play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_ENGAGING_TARGET, 1.0, 10.0);

					return;
				}
			}
		}
	}

	play_client_server_radio_message_negative_response (wingman, SPEECH_WINGMAN_UNABLE_TO_COMPLY, 1.0, 10.0);
}





void process_message_attack_pfz(entity *en, entity *wingman)
{
	entity
		*group;

	ASSERT (en);
	ASSERT (wingman);
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);
	ASSERT (get_local_entity_type(wingman) == ENTITY_TYPE_HELICOPTER);
	ASSERT (get_local_entity_int_value(wingman, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_AIRCRAFT_AH64D_APACHE_LONGBOW);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	if (group && get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
	{
		if (get_local_entity_int_value (wingman, INT_TYPE_WEAPONS_HOLD))
			play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ACKNOWLEDGE, 1.0, 10.0);
		else
			play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_ENGAGING_TARGET, 1.0, 10.0);

		return;
	}

	play_client_server_radio_message_negative_response (wingman, SPEECH_WINGMAN_UNABLE_TO_COMPLY, 1.0, 10.0);
}


//
// Help Me
//


void process_message_help_me (entity *en, entity *wingman)
{
	entity
		*group,
		*target,
		*new_task;

	unsigned int
		valid_members,
		member_number;

	int
		task_assigned;

	ASSERT (en);

	ASSERT (wingman);

	new_task = NULL;

	task_assigned = FALSE;

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (group)
	{
		if (get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
		{
			target = get_local_entity_first_child (en, LIST_TYPE_TARGET);

			while (target)
			{
				if ((get_local_entity_int_value (target, INT_TYPE_IDENTIFY_AIRCRAFT)) ||
						(get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE)))
				{
					new_task = create_engage_task (group, target, en, TRUE);

					if (new_task)
					{
						if (get_local_entity_type (wingman) == ENTITY_TYPE_GROUP)
						{
							valid_members = TASK_ASSIGN_ALL_MEMBERS;
						}
						else
						{
							member_number = get_local_entity_int_value (wingman, INT_TYPE_GROUP_MEMBER_NUMBER);

							valid_members = (1 << member_number);
						}

						if (assign_task_to_group (group, new_task, TASK_ASSIGN_NO_MEMBERS))
						{
							task_assigned = TRUE;
						}
						else
						{
							debug_log ("MESSAGE: can't assign my persuer %s to my wingmen", get_local_entity_string (target, STRING_TYPE_FULL_NAME));
						}
					}
				}

				target = get_local_entity_child_succ (target, LIST_TYPE_TARGET);
			}
		}

		if (task_assigned)
		{
			play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_COMING_TO_ASSIST, 1.0, 8.0);

			//
			// call to "help me" also cancels "weapons hold"
			//

			process_message_weapons_hold (en, wingman, FALSE, FALSE);
		}
		else
		{
			play_client_server_radio_message_negative_response (wingman, SPEECH_WINGMAN_UNABLE_TO_COMPLY, 1.0, 8.0);
		}
	}
}






//
// RETURN TO BASE
//


void process_message_return_to_base (entity *en)
{
	entity
		*group;

	ASSERT (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	ASSERT (group);

	if (get_local_group_primary_guide (group))
	{
		group_return_to_base (group);

		play_client_server_radio_message_affirmative_response (group, SPEECH_WINGMAN_ROGER_RETURNING_TO_BASE, 0.2, 20.0);
	}
}






//
// WEAPONS FREE / WEAPONS HOLD
//


void process_message_weapons_hold (entity *en, entity *wingman, int state, int play_speech)
{
	entity
		*group,
		*guide,
		*task,
		*member;

	unsigned int
		member_number,
		valid_members;

	ASSERT (wingman);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	valid_members = 0;

	group = NULL;

	if (get_local_entity_type (wingman) == ENTITY_TYPE_GROUP)
	{
		group = wingman;

		member = get_local_entity_first_child (wingman, LIST_TYPE_MEMBER);

		while (member)
		{
			if (get_local_entity_int_value (member, INT_TYPE_WEAPONS_HOLD) != state)
			{
				set_client_server_entity_int_value (member, INT_TYPE_WEAPONS_HOLD, state);

				//
				// If setting weapons free, see if member should be assigned to existing engage tasks
				//

				if (!state)
				{
					entity* target;

					if (get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
					{
						member_number = (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER));

						// add engage task for all PFZ targets
						for (target = get_local_entity_first_child (member, LIST_TYPE_DESIGNATED_TARGET);
							 target;
							 target = get_local_entity_child_succ (target, LIST_TYPE_DESIGNATED_TARGET))
						{
							engage_specific_target(group, target, member_number, FALSE);
						}
					}

					guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);

					if (guide)
					{
						task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

						ASSERT (task);

						if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_ENGAGE)
						{
							valid_members |= member_number;
						}
					}
				}
			}

			member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
		}
	}
	else
	{
		member = wingman;

		group = get_local_entity_parent (member, LIST_TYPE_MEMBER);

		set_client_server_entity_int_value (member, INT_TYPE_WEAPONS_HOLD, state);

		//
		// If setting weapons free, see if member should be assigned to existing engage tasks
		//

		if (!state)
		{
			entity* target;

			member_number = (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER));

			if (get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY))
			{
				// add engage task for all PFZ targets
				for (target = get_local_entity_first_child (member, LIST_TYPE_DESIGNATED_TARGET);
					 target;
					 target = get_local_entity_child_succ (target, LIST_TYPE_DESIGNATED_TARGET))
				{
					engage_specific_target(group, target, member_number, FALSE);
				}
			}

			guide = get_local_entity_parent (member, LIST_TYPE_FOLLOWER);

			if (guide)
			{
				task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

				ASSERT (task);

				if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_ENGAGE)
					valid_members |= member_number;
			}
		}
	}

	if (play_speech)
	{
		if (state)
		{
			play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_WEAPONS_HOLD, 0.6, 6.0);
		}
		else
		{
			play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_WEAPONS_FREE, 0.6, 6.0);
		}
	}

	//
	// re-assign "forgotten" engage tasks
	//

	if (valid_members)
	{
		ASSERT (group);

		assign_engage_tasks_to_group (group, valid_members);
	}
}






//
// HOLD POSITION
//


void process_message_hold_position (entity *en, entity *wingman)
{
	entity
		*member;

	ASSERT (wingman);

	if (get_local_entity_type (wingman) == ENTITY_TYPE_GROUP)
	{
		member = get_local_entity_first_child (wingman, LIST_TYPE_MEMBER);

		while (member)
		{
			process_message_hold_position (en, member);

			member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
		}
	}
	else
	{
		if ((!get_local_entity_int_value (wingman, INT_TYPE_GROUP_LEADER)) && (get_local_entity_primary_task (wingman)))
		{
			set_client_server_entity_int_value (wingman, INT_TYPE_POSITION_HOLD, TRUE);

			set_client_server_entity_vec3d (wingman, VEC3D_TYPE_WAIT_POSITION, get_local_entity_vec3d_ptr (wingman, VEC3D_TYPE_POSITION));

			play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_MAINTAINING_POSITION, 0.6, 10.0);
		}
		else
		{
			play_client_server_radio_message_negative_response (wingman, SPEECH_WINGMAN_UNABLE_TO_COMPLY, 0.6, 8.0);
		}
	}
}






//
// REJOIN FORMATION
//


void process_message_rejoin_formation (entity *en, entity *wingman)
{
	entity
		*member;

	ASSERT (wingman);

	if (get_local_entity_type (wingman) == ENTITY_TYPE_GROUP)
	{
		member = get_local_entity_first_child (wingman, LIST_TYPE_MEMBER);

		while (member)
		{
			process_message_rejoin_formation (en, member);

			member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
		}
	}
	else
	{
		//
		// Cancel Position Hold
		//

		set_client_server_entity_int_value (wingman, INT_TYPE_POSITION_HOLD, FALSE);

		//
		// Play Speech
		//

		play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_REJOINING_FORMATION, 0.5, 7.0);

		//
		// Abort any engage tasks
		//

		terminate_entity_current_engage_task (wingman);
	}
}






//
// BOB-UP
//


void process_message_bob_up (entity *en, entity *wingman)
{
	entity
		*guide,
		*follower;

	ASSERT (wingman);

	if (get_local_entity_type (wingman) == ENTITY_TYPE_GROUP)
	{
		guide = get_local_entity_first_child (wingman, LIST_TYPE_GUIDE_STACK);

		while (guide)
		{
			if (get_local_entity_int_value (guide, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_TAKE_COVER)
			{
				attack_guide_proceed_to_fire_positon (guide);

				follower = get_local_entity_first_child (guide, LIST_TYPE_FOLLOWER);

				if (follower)
				{
					play_client_server_radio_message_affirmative_response (follower, SPEECH_WINGMAN_ROGER_BOBBING_UP, 1.0, 4.0);
				}
			}

			guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE_STACK);
		}
	}
	else
	{
		guide = get_local_entity_parent (wingman, LIST_TYPE_FOLLOWER);

		if (guide)
		{
			if (get_local_entity_int_value (guide, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_TAKE_COVER)
			{
				attack_guide_proceed_to_fire_positon (guide);

				play_client_server_radio_message_affirmative_response (wingman, SPEECH_WINGMAN_ROGER_BOBBING_UP, 1.0, 4.0);
			}
		}
	}
}





void process_message_change_formation (entity *en, entity *group, int new_formation)
{
	ASSERT (group);

	ASSERT (get_local_entity_type (group) == ENTITY_TYPE_GROUP);

	set_client_server_entity_int_value (group, INT_TYPE_GROUP_FORMATION, new_formation);

	play_client_server_radio_message_affirmative_response (group, SPEECH_WINGMAN_ROGER_MOVING_INTO_ORDERED_POSITION, 0.1, 5.0);
}


//
// Request Airstrike
//


void process_message_request_airstrike (entity *en)
{
	entity
		*target,
		*task_objective,
		*group,
		*assigned_group,
		*new_task;

	entity_sides
		side;

	ASSERT (en);

	new_task = NULL;

	target = get_local_entity_parent (en, LIST_TYPE_TARGET);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (target)
	{
		side = (entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE);

		if (get_local_entity_int_value (target, INT_TYPE_SIDE) != side)
		{
			switch (get_local_entity_type (target))
			{
				case ENTITY_TYPE_SITE:
				case ENTITY_TYPE_SITE_UPDATABLE:
				{
					entity
						*building_group;

					//
					// create deep strike
					//

					building_group = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if ((building_group) && (get_local_entity_type (building_group) == ENTITY_TYPE_GROUP))
					{
						task_objective = get_local_entity_parent (building_group, LIST_TYPE_BUILDING_GROUP);

						if ((task_objective) && (get_local_entity_type (task_objective) == ENTITY_TYPE_KEYSITE))
						{
							if (get_local_entity_float_value (task_objective, FLOAT_TYPE_EFFICIENCY) >= get_local_entity_float_value (task_objective, FLOAT_TYPE_MINIMUM_EFFICIENCY))
							{
								if (!entity_is_object_of_task (task_objective, ENTITY_SUB_TYPE_TASK_GROUND_STRIKE, side))
								{
									new_task = create_ground_strike_task (side, task_objective, group, TRUE, task_database [ENTITY_SUB_TYPE_TASK_GROUND_STRIKE].task_priority, NULL, NULL);
								}
							}
						}
					}

					break;
				}
				case ENTITY_TYPE_ANTI_AIRCRAFT:
				{
					//
					// create SEAD
					//

					task_objective = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if (task_objective)
					{
						if (!entity_is_object_of_task (task_objective, ENTITY_SUB_TYPE_TASK_SEAD, side))
						{
							new_task = create_sead_task (side, task_objective, group, TRUE, task_database [ENTITY_SUB_TYPE_TASK_SEAD].task_priority, NULL, NULL);
						}
					}

					break;
				}
				case ENTITY_TYPE_ROUTED_VEHICLE:
				{
					//
					// create CAS
					//

					task_objective = get_local_entity_parent (target, LIST_TYPE_MEMBER);

					if (task_objective)
					{
						if (!entity_is_object_of_task (task_objective, ENTITY_SUB_TYPE_TASK_CLOSE_AIR_SUPPORT, side))
						{
							new_task = create_close_air_support_task (side, task_objective, group, TRUE, task_database [ENTITY_SUB_TYPE_TASK_CLOSE_AIR_SUPPORT].task_priority, NULL, NULL);
						}
					}

					break;
				}
			}

			if (new_task)
			{
				assigned_group = get_suitable_registered_group (new_task, NULL);

				if (assigned_group)
				{
					if (assign_primary_task_to_group (assigned_group, new_task))
					{
						//
						// Affirmative response speech...
						//

						play_client_server_speech
						(
							get_session_entity (),
							en,
							(entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE),
							ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE,
							SOUND_LOCALITY_RADIO,
							0.0,
							1.0,
							20.0,
							SPEECH_ORIGINATOR_FORWARD_AIR_CONTROLLER,
							SPEECH_CATEGORY_RADIO_RESPONSE,
							-1.0,
							SPEECH_ARRAY_FAC_MESSAGES, SPEECH_FAC_INTRODUCTION,
							SPEECH_ARRAY_FAC_MESSAGES, SPEECH_FAC_REQUEST_ACCEPTED,
							-1
						);

						return;
					}
				}
			}

			//
			// Negative response speech...
			//

			play_client_server_speech
			(
				get_session_entity (),
				en,
				(entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE),
				ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE,
				SOUND_LOCALITY_RADIO,
				0.0,
				1.0,
				20.0,
				SPEECH_ORIGINATOR_FORWARD_AIR_CONTROLLER,
				SPEECH_CATEGORY_RADIO_RESPONSE,
				-1.0,
				SPEECH_ARRAY_FAC_MESSAGES, SPEECH_FAC_INTRODUCTION,
				SPEECH_ARRAY_FAC_MESSAGES, SPEECH_FAC_REQUEST_DENIED,
				-1
			);
		}
	}
}






//
// Request Artillery
//


#define ARTILLERY_ATTACK_RADIUS	(1.0 * KILOMETRE)

void process_message_request_artillery (entity *en)
{
	entity
		*target,
		*force,
		*group,
		*member,
		*best_group,
		*new_task;

	entity_sides
		side;

	vec3d
		*group_pos,
		*target_pos;

	float
		range,
		best_range,
		weapon_range;

	ASSERT (en);

	new_task = NULL;

	target = get_local_entity_parent (en, LIST_TYPE_TARGET);

	if (target)
	{
		side = (entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE);

		if (get_local_entity_int_value (target, INT_TYPE_SIDE) != side)
		{
			//
			// Find closest friendly artillery group
			//

			target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

			force = get_local_force_entity (side);

			best_group = NULL;

			best_range = FLT_MAX;

			group = get_local_entity_first_child (force, LIST_TYPE_GROUND_REGISTRY);

			while (group)
			{
				if (
						(get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_GROUP_SELF_PROPELLED_ARTILLERY) ||
						(get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_GROUP_SELF_PROPELLED_MLRS)
					)
				{
					if (get_local_entity_int_value (group, INT_TYPE_GROUP_MODE) == GROUP_MODE_IDLE)
					{
						group_pos = get_local_entity_vec3d_ptr (group, VEC3D_TYPE_POSITION);

						range = get_2d_range (target_pos, group_pos);

						debug_log ("MSG_IN: Artillery Range = %f", range);

						if (range < best_range)
						{
							//
							// Check range is within groups weapon range
							//

							member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

							while (member)
							{
								weapon_range = get_local_entity_max_weapon_range (member);

								if (weapon_range > 0.0)
								{
									debug_log ("MSG_IN: Artillery Weapon Range = %f", weapon_range);

									if (weapon_range > range)
									{
										best_range = range;

										best_group = group;
									}

									break;
								}

								member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
							}
						}
					}
				}

				group = get_local_entity_child_succ (group, LIST_TYPE_GROUND_REGISTRY);
			}

			//
			// found group ?
			//

			if (best_group)
			{
				group = best_group;

				if (engage_targets_in_area (group, target_pos, ARTILLERY_ATTACK_RADIUS, get_local_entity_int_value (target, INT_TYPE_TASK_TARGET_TYPE), TRUE))
				{
					debug_log ("MSG_IN: Created Artillery Strike - range %f", best_range);

					//
					// Affirmative response speech...
					//

					play_client_server_speech
					(
						get_session_entity (),
						en,
						(entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE),
						ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE,
						SOUND_LOCALITY_RADIO,
						0.0,
						1.0,
						20.0,
						SPEECH_ORIGINATOR_GROUND_CONTROLLER,
						SPEECH_CATEGORY_RADIO_RESPONSE,
						-1.0,
						SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_INTRODUCTION,
						SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_FIRE_MISSION_RECEIVED,
						-1
					);

					return;
				}

				//
				// Negative response speech... (group available but can't engage)
				//

				play_client_server_speech
				(
					get_session_entity (),
					en,
					(entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE),
					ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE,
					SOUND_LOCALITY_RADIO,
					0.0,
					1.0,
					20.0,
					SPEECH_ORIGINATOR_GROUND_CONTROLLER,
					SPEECH_CATEGORY_RADIO_RESPONSE,
					-1.0,
					SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_INTRODUCTION,
					SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_ARTILLERY_SHIFTING_POSITION,
					-1
				);

				return;
			}

			//
			// Negative response speech... (no group available)
			//

			play_client_server_speech
			(
				get_session_entity (),
				en,
				(entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE),
				ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE,
				SOUND_LOCALITY_RADIO,
				0.0,
				1.0,
				20.0,
				SPEECH_ORIGINATOR_GROUND_CONTROLLER,
				SPEECH_CATEGORY_RADIO_RESPONSE,
				-1.0,
				SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_INTRODUCTION,
				SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_ARTILLERY_BUSY,
				-1
			);
		}
	}
}






//
// Request Assistance
//


void process_message_request_assistance (entity *en)
{
	entity
		*force,
		*group,
		*aggressor;

	ASSERT (en);

	aggressor = get_local_entity_first_child (en, LIST_TYPE_TARGET);

	//
	// check that we actually have at least one pursuer before spinning through the group lists...
	//

	if (aggressor)
	{
		group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

		ASSERT (group);

		force = get_local_force_entity ((entity_sides) get_local_entity_int_value (en, INT_TYPE_SIDE));

		ASSERT (force);

		while (aggressor)
		{
			if (
					(get_local_entity_int_value (aggressor, INT_TYPE_IDENTIFY_AIRCRAFT)) ||
					(get_local_entity_int_value (aggressor, INT_TYPE_IDENTIFY_VEHICLE))
				)
			{
				notify_local_entity (ENTITY_MESSAGE_REQUEST_ASSISTANCE, force, en, aggressor);
			}

			aggressor = get_local_entity_child_succ (aggressor, LIST_TYPE_TARGET);
		}
	}
}

























































































































































































































































































































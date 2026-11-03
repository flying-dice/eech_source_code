

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*aggressor;

	int
		total_damage_level;

	float
		damage_modifier;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	damage_modifier = va_arg (pargs, double);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	if (sender)
	{
		if (get_local_entity_type (sender) == ENTITY_TYPE_WEAPON)
		{
			aggressor = get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON);
		}
		else
		{
			aggressor = sender;
		}

		if (aggressor)
		{
			notify_keysite_structure_under_attack (receiver, aggressor);
		}
	}
	else
	{
		aggressor = NULL;
	}

	damage_client_server_entity (receiver, sender, damage_modifier);

	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

	if (total_damage_level <= 0)
	{
		if (aggressor)
		{
			if (get_local_entity_float_value (receiver, FLOAT_TYPE_FIXED_OBJECT_IMPORTANCE) >= 0.5)
			{
				if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_LOCAL)
				{
					inc_player_log_kills (get_global_gunship_side (), get_current_player_log (), receiver);
				}
				else if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) == ENTITY_PLAYER_REMOTE)
				{
					transmit_entity_comms_message (ENTITY_COMMS_PLAYER_KILL, aggressor, receiver);
				}
			}
		}

		kill_client_server_entity (receiver);
	}

	return (TRUE);
}





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





void overload_site_message_responses (void)
{
	overload_fixed_message_responses (ENTITY_TYPE_SITE);

	message_responses[ENTITY_TYPE_SITE][ENTITY_MESSAGE_COLLISION]		= response_to_collision;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_SITE][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	message_responses[ENTITY_TYPE_SITE][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;

	message_responses[ENTITY_TYPE_SITE][ENTITY_MESSAGE_LINK_PARENT]	= response_to_link_parent;

	message_responses[ENTITY_TYPE_SITE][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif
}





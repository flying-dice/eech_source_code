

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	int
		total_damage_level;

	float
		damage_modifier;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	damage_modifier = va_arg (pargs, double);

	damage_client_server_entity (receiver, sender, damage_modifier);

	total_damage_level = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);

	if (total_damage_level <= 0)
	{
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





void overload_object_message_responses (void)
{
	overload_fixed_message_responses (ENTITY_TYPE_OBJECT);

	message_responses[ENTITY_TYPE_OBJECT][ENTITY_MESSAGE_COLLISION]			= response_to_collision;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_OBJECT][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	message_responses[ENTITY_TYPE_OBJECT][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;

	message_responses[ENTITY_TYPE_OBJECT][ENTITY_MESSAGE_LINK_PARENT]		= response_to_link_parent;

	message_responses[ENTITY_TYPE_OBJECT][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif
}





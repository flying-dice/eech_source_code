

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_string (entity *en, string_types type, char *s)
{
	weapon
		*raw;

	ASSERT (s);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_STRING, en, type, s);

	#endif

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_remote_string (entity *en, string_types type, char *s)
{
	ASSERT (s);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_STRING, en, type, s);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_STRING, en, type, s);
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_server_string (entity *en, string_types type, char *s)
{
	ASSERT (s);

	validate_client_server_local_fn ();

	set_local_string (en, type, s);

	validate_client_server_remote_fn ();

	set_remote_string (en, type, s);
}

#endif





#if SET_CLIENT_SERVER_REQUIRED

static void set_client_string (entity *en, string_types type, char *s)
{
	ASSERT (s);

	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_string (en, type, s);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_string (en, type, s);
	}
}

#endif





#if GET_LOCAL_REQUIRED

static const char *get_local_string (entity *en, string_types type)
{
	weapon
		*raw;

	const char
		*s;

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = weapon_database[raw->mob.sub_type].full_name;

			break;
		}

		case STRING_TYPE_HUD_NAME:

		{
			s = weapon_database[raw->mob.sub_type].hud_name;

			break;
		}

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}

	return (s);
}

#endif





void overload_weapon_string_functions (void)
{
	fn_get_local_entity_string		[ENTITY_TYPE_WEAPON][STRING_TYPE_FULL_NAME]	= get_local_string;

	fn_get_local_entity_string		[ENTITY_TYPE_WEAPON][STRING_TYPE_HUD_NAME]	= get_local_string;
}





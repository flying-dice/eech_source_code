

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_string (entity *en, string_types type, const char *s)
{
	keysite
		*raw;

	ASSERT (s);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_STRING, en, type, s);

	#endif

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:
		case STRING_TYPE_KEYSITE_NAME:

		{
			strncpy (raw->keysite_name, s, get_string_type_max_length (type));

			break;
		}

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}
}





static void set_remote_string (entity *en, string_types type, const char *s)
{
	ASSERT (s);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_STRING, en, type, s);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_STRING, en, type, s);
}





static void set_server_string (entity *en, string_types type, const char *s)
{
	ASSERT (s);

	validate_client_server_local_fn ();

	set_local_string (en, type, s);

	validate_client_server_remote_fn ();

	set_remote_string (en, type, s);
}





static void set_client_string (entity *en, string_types type, const char *s)
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





static const char *get_local_string (entity *en, string_types type)
{
	keysite
		*raw;

	const char
		*s;

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = keysite_database [raw->sub_type].full_name;

			break;
		}

		case STRING_TYPE_KEYSITE_NAME:

		{
			s = raw->keysite_name;

			break;
		}

		case STRING_TYPE_SHORT_DISPLAY_NAME:

		{
			s = keysite_database [raw->sub_type].short_name;

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





void overload_keysite_string_functions (void)
{
	fn_set_local_entity_raw_string		[ENTITY_TYPE_KEYSITE][STRING_TYPE_FULL_NAME]									= set_local_string;
	fn_set_local_entity_string				[ENTITY_TYPE_KEYSITE][STRING_TYPE_FULL_NAME]									= set_local_string;
	fn_set_client_server_entity_string	[ENTITY_TYPE_KEYSITE][STRING_TYPE_FULL_NAME][COMMS_MODEL_SERVER]		= set_server_string;
	fn_set_client_server_entity_string	[ENTITY_TYPE_KEYSITE][STRING_TYPE_FULL_NAME][COMMS_MODEL_CLIENT]		= set_client_string;
	fn_get_local_entity_string				[ENTITY_TYPE_KEYSITE][STRING_TYPE_FULL_NAME]									= get_local_string;

	fn_set_local_entity_raw_string		[ENTITY_TYPE_KEYSITE][STRING_TYPE_KEYSITE_NAME]								= set_local_string;
	fn_set_local_entity_string				[ENTITY_TYPE_KEYSITE][STRING_TYPE_KEYSITE_NAME]								= set_local_string;
	fn_set_client_server_entity_string	[ENTITY_TYPE_KEYSITE][STRING_TYPE_KEYSITE_NAME][COMMS_MODEL_SERVER]	= set_server_string;
	fn_set_client_server_entity_string	[ENTITY_TYPE_KEYSITE][STRING_TYPE_KEYSITE_NAME][COMMS_MODEL_CLIENT]	= set_client_string;
	fn_get_local_entity_string				[ENTITY_TYPE_KEYSITE][STRING_TYPE_KEYSITE_NAME]								= get_local_string;

	fn_get_local_entity_string				[ENTITY_TYPE_KEYSITE][STRING_TYPE_SHORT_DISPLAY_NAME]						= get_local_string;
}





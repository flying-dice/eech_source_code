

































































#include "project.h"





#define DEBUG_MODULE 0





/*

static void set_local_string (entity *en, string_types type, char *s)
{
	aircraft
		*raw;

	ASSERT (s);







	raw = get_local_entity_data (en);

	switch (type)
	{

		default:

		{
			debug_fatal_invalid_string_type (en, type);

			break;
		}
	}
}

*/





/*

static void set_remote_string (entity *en, string_types type, char *s)
{
	ASSERT (s);







	transmit_entity_comms_message (ENTITY_COMMS_STRING, en, type, s);
}

*/





/*

static void set_server_string (entity *en, string_types type, char *s)
{
	ASSERT (s);

	validate_client_server_local_fn ();

	set_local_string (en, type, s);

	validate_client_server_remote_fn ();

	set_remote_string (en, type, s);
}

*/





/*

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

*/





static const char *get_local_string (entity *en, string_types type)
{
	task
		*raw;

	const char
		*s;

	raw = (task *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_FULL_NAME:

		{
			s = waypoint_database[raw->sub_type].full_name;

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
















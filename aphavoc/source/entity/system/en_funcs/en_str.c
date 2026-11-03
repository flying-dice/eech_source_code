

































































#include "project.h"





#define DEBUG_MODULE				0

#define DEBUG_MODULE_PACK_ONE	0

#define DEBUG_MODULE_PACK_ALL	(PACK_ENTIRE_SESSION || 0)





string_type_data
	string_type_database[NUM_STRING_TYPES] =
	{
		{
			"STRING_TYPE_DIVISION_NAME",						// name
			STRING_TYPE_DIVISION_NAME_MAX_LENGTH,			// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"UNKNOWN",												// default_value
		},
		{
			"STRING_TYPE_EASY_DISPLAY_NAME",					// name
			STRING_TYPE_EASY_DISPLAY_NAME_MAX_LENGTH,		// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"UNKNOWN",												// default_value
		},
		{
			"STRING_TYPE_FORCE_NAME",							// name
			STRING_TYPE_FORCE_NAME_MAX_LENGTH,				// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"Unknown",												// default_value
		},
		{
			"STRING_TYPE_FULL_NAME",							// name
			STRING_TYPE_FULL_NAME_MAX_LENGTH,				// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"Unknown",												// default_value
		},
		{
			"STRING_TYPE_GROUP_CALLSIGN",						// name
			STRING_TYPE_FULL_NAME_MAX_LENGTH,				// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"Unknown",												// default_value
		},
		{
			"STRING_TYPE_GROUP_SHORT_NAME",					// name
			STRING_TYPE_FULL_NAME_MAX_LENGTH,				// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"Unknown",												// default_value
		},
		{
			"STRING_TYPE_HUD_NAME",								// name
			STRING_TYPE_HUD_NAME_MAX_LENGTH,					// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"Unknown", 												// default_value
		},
		{
			"STRING_TYPE_KEYSITE_NAME",						// name
			STRING_TYPE_KEYSITE_NAME_MAX_LENGTH,			// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			NULL,														// default_value
		},
		{
			"STRING_TYPE_LONG_DISPLAY_NAME",			  		// name
			STRING_TYPE_LONG_DISPLAY_NAME_MAX_LENGTH,		// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"UNKNOWN",	 											// default_value
		},
		{
			"STRING_TYPE_PILOT_MESSAGE",						// name
			STRING_TYPE_MESSAGE_LENGTH,						// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			NULL,														// default_value
		},
		{
			"STRING_TYPE_PILOTS_NAME",							// name
			STRING_TYPE_PLAYERS_NAME_MAX_LENGTH,			// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			NULL,														// default_value
		},
		{
			"STRING_TYPE_PLAYERS_NAME",						// name
			STRING_TYPE_PLAYERS_NAME_MAX_LENGTH,			// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"Anonymous",											// default_value
		},
		{
			"STRING_TYPE_SHORT_DISPLAY_NAME",			  	// name
			STRING_TYPE_SHORT_DISPLAY_NAME_MAX_LENGTH,	// max_length
			(DEBUG_MODULE_PACK_ALL || 0),						// debug_pack
			"UNKNOWN",	 											// default_value
		},
	};





const char
	*overload_invalid_string_type_message = "Overloaded entity function invoked with invalid string type",
	*debug_fatal_invalid_string_type_message = "Invalid string type (entity type = %s, index = %d, string type = %s, file = %s, line = %d)";





void (*fn_set_local_entity_raw_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type, const char *s);

void (*fn_set_local_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type, const char *s);

void (*fn_set_client_server_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, string_types type, const char *s);

const char *(*fn_get_local_entity_string[NUM_ENTITY_TYPES][NUM_STRING_TYPES]) (entity *en, string_types type);





static void default_set_entity_string (entity *en, string_types type, const char *s)
{
	ASSERT (s);
}





static const char *default_get_entity_string (entity *en, string_types type)
{
	char
		*s;

	switch (type)
	{

		default:

		{
			s = string_type_database[type].default_value;

			break;
		}
	}

	return (s);
}





void initialise_entity_string_default_functions (void)
{
	int
		i,
		j;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_STRING_TYPES; j++)
		{
			fn_set_local_entity_raw_string[i][j]									= default_set_entity_string;

			fn_set_local_entity_string[i][j]									  		= default_set_entity_string;

			fn_set_client_server_entity_string[i][j][COMMS_MODEL_SERVER]	= default_set_entity_string;

			fn_set_client_server_entity_string[i][j][COMMS_MODEL_CLIENT]	= default_set_entity_string;

			fn_get_local_entity_string[i][j]									 		= default_get_entity_string;
		}
	}
}





void pack_string (entity *en, string_types type, const char *s)
{
	ASSERT ((type >= 0) && (type < NUM_STRING_TYPES));

	ASSERT (s);

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (string_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_PACK, ENTITY_DEBUG_STRING, en, type, s);
	}

	#endif

	ASSERT (strlen (s) <= (unsigned)get_string_type_max_length (type));

	while (*s)
	{
		pack_unsigned_data ((unsigned char)*s++, 8);//DEBUG//
	}

	pack_unsigned_data (0, 8);//DEBUG//
}





void unpack_string (entity *en, string_types type, char *s)
{
	char
		buffer[STRING_TYPE_MAX_LENGTH+1],
		*p;

	ASSERT ((type >= 0) && (type < NUM_STRING_TYPES));

	ASSERT (s);

	p = buffer;

	do
	{
	 	*p = unpack_unsigned_data (8);//DEBUG//
	}
	while (*p++);

	ASSERT (strlen (buffer) <= (unsigned)get_string_type_max_length (type));

	strcpy (s, buffer);

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (string_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_UNPACK, ENTITY_DEBUG_STRING, en, type, s);
	}

	#endif
}





void pack_string_type (string_types type)
{
	ASSERT ((type >= 0) && (type < NUM_STRING_TYPES));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (string_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_PACK, ENTITY_DEBUG_STRING_TYPE, NULL, type);
	}

	#endif

	ASSERT(!(type >> NUM_STRING_TYPE_PACK_BITS));

	pack_unsigned_data (type, NUM_STRING_TYPE_PACK_BITS);
}





string_types unpack_string_type (void)
{
	string_types
		type;

	type = (string_types) unpack_unsigned_data (NUM_STRING_TYPE_PACK_BITS);

	ASSERT ((type >= 0) && (type < NUM_STRING_TYPES));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (string_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_UNPACK, ENTITY_DEBUG_STRING_TYPE, NULL, type);
	}

	#endif

	return (type);
}







































































#include "project.h"















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
















































































































































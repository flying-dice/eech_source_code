

































































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
















































































































































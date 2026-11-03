

































































#include "project.h"





#define DEBUG_MODULE				0

#define DEBUG_MODULE_PACK_ONE	0

#define DEBUG_MODULE_PACK_ALL	(PACK_ENTIRE_SESSION || 0)





char_type_data
	char_type_database[NUM_CHAR_TYPES] =
	{
		{
			"CHAR_TYPE_TAG",						// name
			(DEBUG_MODULE_PACK_ALL || 0),		// debug_pack
		},
		{
			"CHAR_TYPE_VALUE",					// name
			(DEBUG_MODULE_PACK_ALL || 0),		// debug_pack
		},
	};





const char
	*overload_invalid_char_type_message = "Overloaded entity function invoked with invalid char type",
	*debug_fatal_invalid_char_type_message = "Invalid char type (entity type = %s, index = %d, char type = %s, file = %s, line = %d)";





void (*fn_set_local_entity_raw_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type, char value);

void (*fn_set_local_entity_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type, char value);

void (*fn_set_client_server_entity_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en, char_types type, char value);

char (*fn_get_local_entity_char_value[NUM_ENTITY_TYPES][NUM_CHAR_TYPES]) (entity *en, char_types type);













static char default_get_entity_char_value (entity *en, char_types type)
{
	char
		value;

	switch (type)
	{

		case CHAR_TYPE_TAG:

		{
			value = '?';

			break;
		}

		default:

		{
			value = ' ';

			break;
		}
	}

	return (value);
}




























































































































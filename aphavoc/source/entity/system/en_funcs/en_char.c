

































































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





static void default_set_entity_char_value (entity *en, char_types type, char value)
{
}





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





void initialise_entity_char_value_default_functions (void)
{
	int
		i,
		j;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_CHAR_TYPES; j++)
		{
			fn_set_local_entity_raw_char_value[i][j]									= default_set_entity_char_value;

			fn_set_local_entity_char_value[i][j]										= default_set_entity_char_value;

			fn_set_client_server_entity_char_value[i][j][COMMS_MODEL_SERVER]	= default_set_entity_char_value;

			fn_set_client_server_entity_char_value[i][j][COMMS_MODEL_CLIENT]	= default_set_entity_char_value;

			fn_get_local_entity_char_value[i][j]										= default_get_entity_char_value;
		}
	}
}





void pack_char_value (entity *en, char_types type, char value)
{
	ASSERT ((type >= 0) && (type < NUM_CHAR_TYPES));
	ASSERT (!(value >> 7));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (char_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_PACK, ENTITY_DEBUG_CHAR_VALUE, en, type, value);
	}

	#endif

	pack_unsigned_data (value, 7);
}





char unpack_char_value (entity *en, char_types type)
{
	char
		value;

	ASSERT ((type >= 0) && (type < NUM_CHAR_TYPES));

	value = unpack_unsigned_data (7);

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (char_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_UNPACK, ENTITY_DEBUG_CHAR_VALUE, en, type, value);
	}

	#endif

	return (value);
}





void pack_char_type (char_types type)
{
	ASSERT ((type >= 0) && (type < NUM_CHAR_TYPES));
	ASSERT (!(type >> NUM_CHAR_TYPE_PACK_BITS));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (char_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_PACK, ENTITY_DEBUG_CHAR_TYPE, NULL, type);
	}

	#endif

	pack_unsigned_data (type, NUM_CHAR_TYPE_PACK_BITS);
}





char_types unpack_char_type (void)
{
	char_types
		type;

	type = (char_types) unpack_unsigned_data (NUM_CHAR_TYPE_PACK_BITS);

	ASSERT ((type >= 0) && (type < NUM_CHAR_TYPES));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (char_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_UNPACK, ENTITY_DEBUG_CHAR_TYPE, NULL, type);
	}

	#endif

	return (type);
}





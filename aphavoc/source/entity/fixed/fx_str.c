

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				0
#define SET_CLIENT_SERVER_REQUIRED	0
#define GET_LOCAL_REQUIRED				1





static const char
	*structure_target_names[] =
	{
		"UNKNOWN",		// TARGET_SYMBOL_UNKNOWN
		"VEHICLE",		// TARGET_SYMBOL_WHEELED_VEHICLE
		"VEHICLE",		// TARGET_SYMBOL_TRACKED_VEHICLE
		"AIR DEFENCE",	// TARGET_SYMBOL_AIR_DEFENCE_UNIT
		"AIRCRAFT",		// TARGET_SYMBOL_AIRCRAFT
		"HELICOPTER",	// TARGET_SYMBOL_HELICOPTER
		"SHIP",			// TARGET_SYMBOL_SHIP
		"STRUCTURE",	// TARGET_SYMBOL_STRUCTURE
		"HAZZARD",		// TARGET_SYMBOL_HAZZARD
	};





#if SET_LOCAL_REQUIRED

static void set_local_string (entity *en, string_types type, const char *s)
{
	fixed
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

#endif









































































#if GET_LOCAL_REQUIRED

static const char *get_local_string (entity *en, string_types type)
{
	fixed
		*raw;

	const char
		*s;

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case STRING_TYPE_EASY_DISPLAY_NAME:

		{
			int
				i;

			i = get_local_entity_int_value (en, INT_TYPE_TARGET_SYMBOL_TYPE);

			if ((i >= 0) && (i < NUM_TARGET_SYMBOL_TYPES))
			{
				s = structure_target_names[i];
			}
			else
			{
				s = "STRUCTURE";

				debug_colour_log (DEBUG_COLOUR_RED, "TARGET SYMBOL OUT OF RANGE IN fixed_3d_object_database");
			}

			break;
		}

		case STRING_TYPE_FULL_NAME:

		{
			s = "Structure";

			break;
		}

		case STRING_TYPE_LONG_DISPLAY_NAME:

		{
			int
				i;

			i = get_local_entity_int_value (en, INT_TYPE_TARGET_SYMBOL_TYPE);

			if ((i >= 0) && (i < NUM_TARGET_SYMBOL_TYPES))
			{
				s = structure_target_names[i];
			}
			else
			{
				s = "STRUCTURE";

				debug_colour_log (DEBUG_COLOUR_RED, "TARGET SYMBOL OUT OF RANGE IN fixed_3d_object_database");
			}

			break;
		}

		case STRING_TYPE_SHORT_DISPLAY_NAME:

		{
			int
				i;

			i = get_local_entity_int_value (en, INT_TYPE_TARGET_SYMBOL_TYPE);

			if ((i >= 0) && (i < NUM_TARGET_SYMBOL_TYPES))
			{
				s = structure_target_names[i];
			}
			else
			{
				s = "STRUCTURE";

				debug_colour_log (DEBUG_COLOUR_RED, "TARGET SYMBOL OUT OF RANGE IN fixed_3d_object_database");
			}

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




















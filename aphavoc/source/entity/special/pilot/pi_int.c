

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	pilot
		*raw;







	raw = (pilot *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CREW_ROLE:

		{
			raw->crew_role = (crew_roles) value;

			break;
		}

		case INT_TYPE_DIFFICULTY_LEVEL:

		{
			raw->difficulty_level = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_KILLS:

		{
			raw->kills = value;

			//
			// update high score table if deathmatch, and victim was a player gunship
			//

			update_pilot_high_score_table ();

			break;
		}

		case INT_TYPE_PILOT_RANK:

		{
			raw->rank = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

			break;
		}

		case INT_TYPE_UNIQUE_ID:

		{
			raw->unique_id = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}























































static int get_local_int_value (entity *en, int_types type)
{
	pilot
		*raw;

	int
		value;

	raw = (pilot *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_CREW_ROLE:

		{
			value = raw->crew_role;

			break;
		}

		case INT_TYPE_DIFFICULTY_LEVEL:

		{
			value = raw->difficulty_level;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_KILLS:

		{
			value = raw->kills;

			break;
		}

		case INT_TYPE_PILOT_RANK:

		{
			value = raw->rank;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

			break;
		}

		case INT_TYPE_UNIQUE_ID:

		{
			value = raw->unique_id;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}

	return (value);
}






















































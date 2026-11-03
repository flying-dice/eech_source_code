

































































#include "project.h"





#define DEBUG_MODULE 				0

#define KEYSITE_ICON_FLASH_RATE 	1.0





int
	keysite_icon_timer_flag;





static void set_local_int_value (entity *en, int_types type, int value)
{
	keysite
		*raw;







	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ALIVE:

		{
			raw->alive = value;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			raw->sub_type = value;

			break;
		}

		case INT_TYPE_IN_USE:

		{
			raw->in_use = value;

			break;
		}

		case INT_TYPE_KEYSITE_ID:

		{
			raw->keysite_id = value;

			break;
		}

		case INT_TYPE_KEYSITE_USABLE_STATE:

		{
			raw->keysite_usable_state = value;

			break;
		}

		case INT_TYPE_LANDING_TYPES:

		{
			raw->landing_types = value;

			break;
		}









		case INT_TYPE_OBJECT_INDEX:

		{
			raw->object_index = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

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
	keysite
		*raw;

	int
		value;

	raw = (keysite *) get_local_entity_data (en);

	switch (type)
	{









		case INT_TYPE_ALIVE:

		{
			value = raw->alive;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_HIDDEN_BY_FOG_OF_WAR:

		{
			value = keysite_database [raw->sub_type].hidden_by_fog_of_war;

			break;
		}

		case INT_TYPE_IN_USE:

		{
			value = raw->in_use;

			break;
		}

		case INT_TYPE_KEYSITE_ID:

		{
			value = raw->keysite_id;

			break;
		}

		case INT_TYPE_KEYSITE_USABLE_STATE:

		{
			value = raw->keysite_usable_state;

			break;
		}

		case INT_TYPE_LANDING_TYPES:

		{
			value = raw->landing_types;

			break;
		}

















		case INT_TYPE_OBJECT_INDEX:

		{
			value = raw->object_index;

			break;
		}

		case INT_TYPE_POTENTIAL_CAMPAIGN_OBJECTIVE:

		{
			value = keysite_database [raw->sub_type].campaign_objective;

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

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










































































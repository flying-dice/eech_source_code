

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	weapon
		*raw;







	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_WEAPON_BURST_SIZE:

		{
			raw->burst_size = value;

			break;
		}

		case INT_TYPE_WEAPON_KILL_CODE:

		{
			raw->kill_code = value;

			break;
		}

		case INT_TYPE_LOCK_ON_AFTER_LAUNCH:

		{
			raw->loal_mode = value;

			break;
		}

		case INT_TYPE_WEAPON_MISSILE_PHASE:

		{
			raw->missile_phase = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}





/*
static void set_remote_int_value (entity *en, int_types type, int value)
{






	transmit_entity_comms_message (ENTITY_COMMS_INT_VALUE, en, type, value);
}
*/





/*
static void set_server_int_value (entity *en, int_types type, int value)
{
	validate_client_server_local_fn ();

	set_local_int_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_int_value (en, type, value);
}
*/





/*
static void set_client_int_value (entity *en, int_types type, int value)
{
	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		set_remote_int_value (en, type, value);
	}
	else
	{
		validate_client_server_local_fn ();

		set_local_int_value (en, type, value);
	}
}
*/
















































































































































































































































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{


















	return (TRUE);
}





static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{



















	return (TRUE);
}



































static int response_to_task_completed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	switch (get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE))
	{

		case ENTITY_SUB_TYPE_TASK_SUPPLY:
		{

			//#if DEBUG_MODULE || DEBUG_SUPPLY

			debug_log ("KS_MSGS: SUPPLY_INFO: keysite %s (%d) resupplied : ammo = %f, fuel = %f",
							get_local_entity_string (receiver, STRING_TYPE_FULL_NAME),
							get_local_entity_index (receiver),
							get_local_entity_float_value (receiver, FLOAT_TYPE_AMMO_SUPPLY_LEVEL),
							get_local_entity_float_value (receiver, FLOAT_TYPE_FUEL_SUPPLY_LEVEL));

			//#endif
		
			break;
		}

		default :
		{

			break;
		}
	}

	return (TRUE);
}





static int response_to_waypoint_drop_off_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	keysite
		*raw;

	entity
		*cargo;

	raw = (keysite *) get_local_entity_data (receiver);

	cargo = va_arg (pargs, entity *);












	switch (get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE))
	{

		case ENTITY_SUB_TYPE_CARGO_AMMO:
		{

			set_client_server_entity_float_value (receiver, FLOAT_TYPE_AMMO_SUPPLY_LEVEL, 100.0);

			break;
		}

		case ENTITY_SUB_TYPE_CARGO_FUEL:
		{

			set_client_server_entity_float_value (receiver, FLOAT_TYPE_FUEL_SUPPLY_LEVEL, 100.0);

			break;
		}
	}

	return (TRUE);
}





static int response_to_waypoint_pick_up_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity
		*cargo;

	keysite
		*raw;

	raw = (keysite *) get_local_entity_data (receiver);

	cargo = va_arg (pargs, entity *);












	switch (get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE))
	{

		case ENTITY_SUB_TYPE_CARGO_AMMO:
		{

			raw->supplies.ammo_supply_level -= CARGO_AMMO_SIZE;

			break;
		}

		case ENTITY_SUB_TYPE_CARGO_FUEL:
		{

			raw->supplies.fuel_supply_level -= CARGO_FUEL_SIZE;

			break;
		}
	}

	return (TRUE);
}























































































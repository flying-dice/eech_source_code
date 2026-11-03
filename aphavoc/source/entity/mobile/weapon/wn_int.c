

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	weapon
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

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
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

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





static int get_local_int_value (entity *en, int_types type)
{
	weapon
		*raw;

	int
		value;

	raw = (weapon *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_DEFAULT_3D_SHAPE:

		{
			value = weapon_database[raw->mob.sub_type].default_3d_shape;

			break;
		}

		case INT_TYPE_ARMOR_PENETRATION_CAPABILITY:

		{
			value = weapon_database[raw->mob.sub_type].armor_penetration_capability;

			break;
		}

		case INT_TYPE_IDENTIFY_WEAPON:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_DAMAGE_CAPABILITY:

		{
			value = weapon_database[raw->mob.sub_type].damage_capability;

			break;
		}

		case INT_TYPE_THREAT_TYPE:

		{
			value = weapon_database[raw->mob.sub_type].threat_type;

			break;
		}

		case INT_TYPE_VIEWABLE_WEAPON:

		{
			value = weapon_database[raw->mob.sub_type].viewable_weapon;

			break;
		}

		case INT_TYPE_LOCK_ON_AFTER_LAUNCH:

		{
			value = raw->loal_mode;

			break;
		}

		case INT_TYPE_WEAPON_MISSILE_PHASE:

		{
			value = raw->missile_phase;

			break;
		}

		case INT_TYPE_WEAPON_BURST_SIZE:

		{
			value = raw->burst_size;

			break;
		}

		case INT_TYPE_WEAPON_DECOY_TYPE:

		{
			value = weapon_database[raw->mob.sub_type].decoy_type;

			break;
		}

		case INT_TYPE_WEAPON_GUIDANCE_TYPE:

		{
			value = weapon_database[raw->mob.sub_type].guidance_type;

			break;
		}

		case INT_TYPE_WEAPON_KILL_CODE:

		{
			value = raw->kill_code;

			break;
		}

		case INT_TYPE_WEAPON_SMOKE_TRAIL_TYPE:

		{
			value = weapon_database[raw->mob.sub_type].smoke_trail_type;

			break;
		}

		case INT_TYPE_WEAPON_WARHEAD_TYPE:

		{
			value = weapon_database[raw->mob.sub_type].warhead_type;

			break;
		}

		case INT_TYPE_WEAPON_ROCKET:

		{
			value = (weapon_database[raw->mob.sub_type].boost_time > 0);

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{
			value = 1;

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





void overload_weapon_int_value_functions (void)
{
	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_DEFAULT_3D_SHAPE]				= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_ARMOR_PENETRATION_CAPABILITY]		= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_IDENTIFY_WEAPON]				= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_DAMAGE_CAPABILITY]		= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_THREAT_TYPE]					= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_VIEWABLE_WEAPON]				= get_local_int_value;

	fn_set_local_entity_raw_int_value	[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_BURST_SIZE]			= set_local_int_value;
	fn_set_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_BURST_SIZE]			= set_local_int_value;
	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_BURST_SIZE]			= get_local_int_value;

	fn_set_local_entity_raw_int_value	[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_MISSILE_PHASE]			= set_local_int_value;
	fn_set_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_MISSILE_PHASE]			= set_local_int_value;
	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_MISSILE_PHASE]			= get_local_int_value;

	fn_set_local_entity_raw_int_value	[ENTITY_TYPE_WEAPON][INT_TYPE_LOCK_ON_AFTER_LAUNCH]			= set_local_int_value;
	fn_set_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_LOCK_ON_AFTER_LAUNCH]			= set_local_int_value;
	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_LOCK_ON_AFTER_LAUNCH]			= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_DECOY_TYPE]			= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_GUIDANCE_TYPE]		= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_KILL_CODE]				= get_local_int_value;
	fn_set_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_KILL_CODE]				= set_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_SMOKE_TRAIL_TYPE]	= get_local_int_value;

	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_WARHEAD_TYPE]			= get_local_int_value;
	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_WEAPON_ROCKET]			= get_local_int_value;
	fn_get_local_entity_int_value			[ENTITY_TYPE_WEAPON][INT_TYPE_ARMOR_LEVEL]						= get_local_int_value;
}





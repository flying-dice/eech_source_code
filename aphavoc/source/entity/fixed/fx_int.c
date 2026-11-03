

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	fixed
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (fixed *) get_local_entity_data (en);

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

		case INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR:

		{
			raw->gunship_radar_los_clear = value;

			break;
		}

		case INT_TYPE_OBJECT_3D_SHAPE:

		{
			raw->object_3d_shape = value;

			break;
		}

		case INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME:

		{
			raw->object_drawn_once_this_frame = value;

			break;
		}

		case INT_TYPE_SIDE:

		{
			raw->side = value;

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX:

		{
			raw->gunship_radar_target_index = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}





static void set_remote_int_value (entity *en, int_types type, int value)
{
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	transmit_entity_comms_message (ENTITY_COMMS_INT_VALUE, en, type, value);
}





static void set_server_int_value (entity *en, int_types type, int value)
{
	validate_client_server_local_fn ();

	set_local_int_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_int_value (en, type, value);
}





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





static int get_local_int_value (entity *en, int_types type)
{
	fixed
		*raw;

	int
		value;

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_ALIGN_WITH_TERRAIN:

		{
			value = fixed_database[raw->sub_type].align_with_terrain;

			break;
		}

		case INT_TYPE_ALIVE:

		{
			value = raw->alive;

			break;
		}

		case INT_TYPE_COLLISION_TEST_FIXED:

		{
			value = fixed_database[raw->sub_type].collision_test_fixed;

			break;
		}

		case INT_TYPE_DEFAULT_3D_SHAPE:

		{
			value = fixed_database[raw->sub_type].default_3d_shape;

			break;
		}

		case INT_TYPE_ENTITY_SUB_TYPE:

		{
			value = raw->sub_type;

			break;
		}

		case INT_TYPE_EXPLOSIVE_POWER:

		{
			value = fixed_3d_object_database [raw->object_3d_shape].explosive_power;

			break;
		}

		case INT_TYPE_EXPLOSIVE_QUALITY:

		{
			value = fixed_3d_object_database [raw->object_3d_shape].explosive_quality;

			break;
		}

		case INT_TYPE_GROUND_RADAR_CLUTTER:

		{
			value = fixed_database[raw->sub_type].ground_radar_clutter;

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR:

		{
			value = raw->gunship_radar_los_clear;

			break;
		}

		case INT_TYPE_IDENTIFY_FIXED:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_INITIAL_DAMAGE_LEVEL:

		{
			value = fixed_3d_object_database [raw->object_3d_shape].initial_damage;

			break;
		}

		case INT_TYPE_MOVEMENT_TYPE:

		{
			value = MOVEMENT_TYPE_ALL;

			break;
		}

		case INT_TYPE_OBJECT_3D_SHAPE:

		{
			value = raw->object_3d_shape;

			break;
		}

		case INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME:

		{
			value = raw->object_drawn_once_this_frame;

			break;
		}

		case INT_TYPE_POINTS_VALUE:

		{
			float
				t;

			t = fixed_3d_object_database [raw->object_3d_shape].importance * 500.0;

			convert_float_to_int (t, &value);

			break;
		}

		case INT_TYPE_SIDE:

		{
			value = raw->side;

			break;
		}

		case INT_TYPE_TARGET_PRIORITY_TYPE:

		{
			value = TARGET_PRIORITY_LOW;

			break;
		}

		case INT_TYPE_TARGET_SYMBOL_TYPE:

		{
			value = fixed_3d_object_database [raw->object_3d_shape].target_symbol;

			break;
		}

		case INT_TYPE_TARGET_TYPE:

		{
			value = fixed_3d_object_database [raw->object_3d_shape].target_type;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			value = TASK_TARGET_TYPE_NONE;

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{

			switch (fixed_3d_object_database [raw->object_3d_shape].hardness)
			{
				case 2:
				{
					value = 40;

					break;
				}
				case 1:
				{
					value = 15;

					break;
				}
				default:
				{
					value = 3;

					break;
				}
			}

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX:

		{
			value = raw->gunship_radar_target_index;

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





void overload_fixed_int_value_functions (entity_types type)
{
	fn_get_local_entity_int_value				[type][INT_TYPE_ALIGN_WITH_TERRAIN]							= get_local_int_value;

	fn_set_local_entity_raw_int_value		[type][INT_TYPE_ALIVE]											= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_ALIVE]						 					= set_local_int_value;
	fn_set_client_server_entity_int_value	[type][INT_TYPE_ALIVE][COMMS_MODEL_SERVER] 	  			= set_server_int_value;
	fn_set_client_server_entity_int_value	[type][INT_TYPE_ALIVE][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_ALIVE]											= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_COLLISION_TEST_FIXED]	  					= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_DEFAULT_3D_SHAPE]							= get_local_int_value;

	fn_set_local_entity_raw_int_value  		[type][INT_TYPE_ENTITY_SUB_TYPE]							  	= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_ENTITY_SUB_TYPE]							 	= set_local_int_value;
	fn_set_client_server_entity_int_value	[type][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[type][INT_TYPE_ENTITY_SUB_TYPE][COMMS_MODEL_CLIENT] 	= set_client_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_ENTITY_SUB_TYPE]				 				= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_EXPLOSIVE_POWER]								= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_EXPLOSIVE_QUALITY]							= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_GROUND_RADAR_CLUTTER]						= get_local_int_value;

	fn_set_local_entity_raw_int_value  		[type][INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR]					= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR]					= set_local_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR]					= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_IDENTIFY_FIXED]								= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_INITIAL_DAMAGE_LEVEL]						= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_MOVEMENT_TYPE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value  		[type][INT_TYPE_OBJECT_3D_SHAPE]								= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_OBJECT_3D_SHAPE]								= set_local_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_OBJECT_3D_SHAPE]								= get_local_int_value;

	fn_set_local_entity_raw_int_value  		[type][INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME]			= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME]			= set_local_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME]			= get_local_int_value;

	fn_set_local_entity_raw_int_value  		[type][INT_TYPE_SIDE]				 							= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_SIDE]						 					= set_local_int_value;
	fn_set_client_server_entity_int_value	[type][INT_TYPE_SIDE][COMMS_MODEL_SERVER]					= set_server_int_value;
	fn_set_client_server_entity_int_value	[type][INT_TYPE_SIDE][COMMS_MODEL_CLIENT]					= set_client_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_SIDE]											= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_TARGET_PRIORITY_TYPE]						= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_TARGET_SYMBOL_TYPE]							= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_TARGET_TYPE]									= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_TASK_TARGET_TYPE]							= get_local_int_value;

	fn_get_local_entity_int_value				[type][INT_TYPE_ARMOR_LEVEL]					= get_local_int_value;

	fn_set_local_entity_raw_int_value  			[type][INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX]								= set_local_int_value;
	fn_set_local_entity_int_value				[type][INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX]								= set_local_int_value;
	fn_get_local_entity_int_value				[type][INT_TYPE_GUNSHIP_RADAR_TARGET_INDEX]								= get_local_int_value;
}







































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	fixed
		*raw;







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























































static int get_local_int_value (entity *en, int_types type)
{
	fixed
		*raw;

	int
		value;

	raw = (fixed *) get_local_entity_data (en);

	switch (type)
	{







		}

		case INT_TYPE_ALIVE:

		{
			value = raw->alive;

			break;
		}







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












































































































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_int_value (entity *en, int_types type, int value)
{
	aircraft
		*raw;







	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_SELECTED_WEAPON:

		{
			raw->selected_weapon = value;

			break;
		}

		case INT_TYPE_WEAPON_CONFIG_TYPE:

		{
			raw->weapon_config_type = (weapon_config_types) value;

			break;
		}

		default:

		{
			debug_fatal_invalid_int_type (en, type);

			break;
		}
	}
}





static void set_local_int_value (entity *en, int_types type, int value)
{
	aircraft
		*raw;







	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{









		case INT_TYPE_DAMAGE_LEVEL:

		{
			float old_damage = raw->damage_level;

			raw->damage_level = value;

			assess_aircraft_damage_level (en, old_damage, value);

			break;
		}

		case INT_TYPE_EJECTED:

		{
			raw->ejected = value;

			break;
		}

		case INT_TYPE_TROOPS_ONBOARD:

		{
			raw->troops_onboard = value;

			ASSERT (raw->inst3d);

			set_sub_object_type_visible_status(raw->inst3d, OBJECT_3D_SUB_OBJECT_TROOPS_ONBOARD, !!value);

			break;
		}

		case INT_TYPE_FORMATION_POSITION:

		{
			raw->formation_position = value;

			break;
		}

		case INT_TYPE_GROUP_MEMBER_NUMBER:

		{
			ASSERT (value >= 0);

			ASSERT (value < (1 << NUM_GROUP_MEMBER_NUMBER_BITS));

			raw->group_member_number = value;

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR:

		{
			raw->gunship_radar_los_clear = value;

			break;
		}

		case INT_TYPE_ID_NUMBER:

		{
			raw->id_number = value;

			break;
		}

		case INT_TYPE_ID_NUMBER_SIGNIFICANT_DIGITS:

		{
			raw->id_number_significant_digits = value;

			break;
		}









		case INT_TYPE_OBJECT_3D_SHAPE:

		{
			raw->object_3d_shape = value;

			break;
		}

		case INT_TYPE_OPERATIONAL_STATE:

		{

			raw->operational_state = value;

			break;
		}

		case INT_TYPE_SELECTED_WEAPON:

		{
			//
			// stop continuous sound effect
			//

//			if (raw->selected_weapon != value)
			{

			}

			//
			// play weapon selected speech
			//

			if (get_comms_model () == COMMS_MODEL_SERVER)
			{

			}

			raw->selected_weapon = value;

			raw->weapon_burst_timer = 0.0;
			raw->weapon_salvo_timer = 0.0;

			//















					hud_mode = HUD_MODE_WEAPON;
				}
			}

			break;
		}

		case INT_TYPE_SELECTED_WEAPON_SYSTEM_READY:

		{
			raw->selected_weapon_system_ready = value;

			break;
		}

		case INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID:

		{
			raw->weapon_and_target_vectors_valid = value;

			break;
		}

		case INT_TYPE_WEAPON_CONFIG_TYPE:

		{
			raw->weapon_config_type = (weapon_config_types) value;

			load_local_entity_weapon_config (en);

			if ((get_comms_model () == COMMS_MODEL_CLIENT) && (en == get_gunship_entity ()) && get_local_entity_int_value (en, INT_TYPE_GUNSHIP_TYPE) < NUM_GUNSHIP_TYPES)
			{
				//
				// needed because of the clients lag between selecting a new config, and it actually being changed
				//

				weapon_loading_update_currently_selected_weapons (en);
			}

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
	aircraft
		*raw;

	int
		value;

	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_AIR_TO_AIR_WEAPON_CONFIG_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].air_to_air_weapon_config_type;

			break;
		}

		case INT_TYPE_AIR_TO_SURFACE_WEAPON_CONFIG_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].air_to_surface_weapon_config_type;

			break;
		}

		case INT_TYPE_AIRBORNE_AIRCRAFT:

		{

			switch (raw->operational_state)
			{

				case OPERATIONAL_STATE_ASLEEP:
				case OPERATIONAL_STATE_DEAD:
				case OPERATIONAL_STATE_LANDED:
				//case OPERATIONAL_STATE_REPAIRING:
				//case OPERATIONAL_STATE_REFUELING:
				//case OPERATIONAL_STATE_REARMING:
				case OPERATIONAL_STATE_STOPPED:
				case OPERATIONAL_STATE_TAXIING:
				case OPERATIONAL_STATE_UNKNOWN:
				case OPERATIONAL_STATE_WAITING:
				{

					value = FALSE;

					break;
				}

				case OPERATIONAL_STATE_DYING:
				case OPERATIONAL_STATE_LANDING:
				case OPERATIONAL_STATE_LANDING_HOLDING:
				case OPERATIONAL_STATE_NAVIGATING:
				case OPERATIONAL_STATE_TAKEOFF:
				case OPERATIONAL_STATE_TAKEOFF_HOLDING:
				{

					value = TRUE;

					break;
				}

				default:
				{

					debug_fatal ("AC_INT: unknown operational state");
				}
			}

			break;
		}

		case INT_TYPE_COLLISION_TEST_MOBILE:

		{
			value = TRUE;

			break;
		}









		case INT_TYPE_DAMAGE_LEVEL:

		{
			value = raw->damage_level;

			break;
		}









		case INT_TYPE_DEFAULT_WEAPON_CONFIG_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].default_weapon_config_type;

			break;
		}

		case INT_TYPE_DEFAULT_WEAPON_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].default_weapon_type;

			break;
		}









		case INT_TYPE_EJECTED:

		{
			value = raw->ejected;

			break;
		}

		case INT_TYPE_TROOPS_ONBOARD:

		{
			value = raw->troops_onboard;

			break;
		}

		case INT_TYPE_ENGAGE_ENEMY:

		{
			entity
				*group;

			value = FALSE;

			switch (raw->operational_state)
			{
				case OPERATIONAL_STATE_NAVIGATING:
				{
					group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

					if (group)
					{
						value = get_local_entity_int_value (group, INT_TYPE_ENGAGE_ENEMY);
					}

					break;
				}
			}

			break;
		}

		case INT_TYPE_EXPLOSIVE_POWER:

		{
			value = aircraft_database[raw->mob.sub_type].explosive_power;

			break;
		}

		case INT_TYPE_EXPLOSIVE_QUALITY:

		{
			value = aircraft_database[raw->mob.sub_type].explosive_quality;

			break;
		}

		case INT_TYPE_FORCE:

		{
			value = aircraft_database[raw->mob.sub_type].force;

			break;
		}

		case INT_TYPE_FORCE_INFO_CATAGORY:

		{
			value = aircraft_database[raw->mob.sub_type].force_info_catagory;

			break;
		}

		case INT_TYPE_FORMATION_POSITION:

		{
			value = raw->formation_position;

			break;
		}

		case INT_TYPE_GROUP_LEADER:

		{
			if ((get_local_entity_parent (en, LIST_TYPE_MEMBER)) && (get_local_entity_child_pred (en, LIST_TYPE_MEMBER) == NULL))
			{
				value = TRUE;
			}
			else
			{
				value = FALSE;
			}

			break;
		}

		case INT_TYPE_GROUP_MEMBER_ID:

		{
			value = raw->group_member_number + 1;

			break;
		}

		case INT_TYPE_GROUP_MEMBER_NUMBER:

		{
			value = raw->group_member_number;

			break;
		}

		case INT_TYPE_GROUP_MEMBER_STATE:

		{
			entity
				*task;

			//
			// First check for being on takeoff queue
			//

			if (get_local_entity_parent (en, LIST_TYPE_TAKEOFF_QUEUE))
			{
				value = GROUP_MEMBER_STATE_WAITING;

				break;
			}

			//
			// Check Task
			//

			task = get_local_entity_current_task (en);

			if (task)
			{
				if (task == get_local_entity_primary_task (en))
				{
					//
					// Member is on primary task
					//

					if (get_local_entity_int_value (task, INT_TYPE_TASK_STATE) == TASK_STATE_COMPLETED)
					{
						value = GROUP_MEMBER_STATE_RETURNING;
					}
					else
					{
						value = GROUP_MEMBER_STATE_NAVIGATING;
					}
				}
				else
				{
					//
					// Not A Primary Task
					//

					switch (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE))
					{
						case ENTITY_SUB_TYPE_TASK_ENGAGE:
						{
							value = GROUP_MEMBER_STATE_ATTACKING;

							break;
						}
						case ENTITY_SUB_TYPE_TASK_LANDING:
						case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
						{
							value = GROUP_MEMBER_STATE_LANDING;

							break;
						}
						case ENTITY_SUB_TYPE_TASK_TAKEOFF:
						case ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING:
						{
							value = GROUP_MEMBER_STATE_TAKING_OFF;

							break;
						}
						default:
						{
							debug_fatal ("AC_INT: Invalid Task Type %d For Get Group Member State",
												get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE));
						}
					}
				}
			}
			else
			{
				value = GROUP_MEMBER_STATE_IDLE;
			}

			break;
		}

		case INT_TYPE_GUNSHIP_RADAR_LOS_CLEAR:

		{
			value = raw->gunship_radar_los_clear;

			break;
		}

		case INT_TYPE_GUNSHIP_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].gunship_type;

			break;
		}

		case INT_TYPE_IDENTIFY_AIRCRAFT:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_ID_NUMBER:

		{
			value = raw->id_number;

			break;
		}

		case INT_TYPE_ID_NUMBER_SIGNIFICANT_DIGITS:

		{
			value = raw->id_number_significant_digits;

			break;
		}

		case INT_TYPE_INITIAL_DAMAGE_LEVEL:

		{
			value = aircraft_database[raw->mob.sub_type].initial_damage_level;

			break;
		}

		case INT_TYPE_LANDED:

		{

			switch (raw->operational_state)
			{

				case OPERATIONAL_STATE_LANDED:
				//case OPERATIONAL_STATE_REPAIRING:
				//case OPERATIONAL_STATE_REFUELING:
				//case OPERATIONAL_STATE_REARMING:
				{

					value = TRUE;

					break;
				}

				case OPERATIONAL_STATE_UNKNOWN:
				case OPERATIONAL_STATE_DEAD:
				case OPERATIONAL_STATE_DYING:
				case OPERATIONAL_STATE_LANDING:
				case OPERATIONAL_STATE_LANDING_HOLDING:
				case OPERATIONAL_STATE_NAVIGATING:
				case OPERATIONAL_STATE_STOPPED:
				case OPERATIONAL_STATE_TAKEOFF:
				case OPERATIONAL_STATE_TAKEOFF_HOLDING:
				case OPERATIONAL_STATE_TAXIING:
				case OPERATIONAL_STATE_WAITING:
				{

					value = FALSE;

					break;
				}

				default:
				{

					debug_fatal ("AC_INT: unknown operational state");
				}
			}

			break;
		}

















		case INT_TYPE_MAX_WEAPON_CONFIG_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].max_weapon_config_type;

			break;
		}

		case INT_TYPE_MIN_WEAPON_CONFIG_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].min_weapon_config_type;

			break;
		}

		case INT_TYPE_MOBILE_MOVING:

		{

			switch (raw->operational_state)
			{

				case OPERATIONAL_STATE_LANDED:
				case OPERATIONAL_STATE_STOPPED:
				case OPERATIONAL_STATE_DEAD:
				//case OPERATIONAL_STATE_REPAIRING:
				//case OPERATIONAL_STATE_REFUELING:
				//case OPERATIONAL_STATE_REARMING:
				case OPERATIONAL_STATE_UNKNOWN:
				{

					value = FALSE;

					break;
				}

				case OPERATIONAL_STATE_DYING:
				case OPERATIONAL_STATE_LANDING:
				case OPERATIONAL_STATE_LANDING_HOLDING:
				case OPERATIONAL_STATE_NAVIGATING:
				case OPERATIONAL_STATE_TAKEOFF:
				case OPERATIONAL_STATE_TAKEOFF_HOLDING:
				case OPERATIONAL_STATE_TAXIING:
				case OPERATIONAL_STATE_WAITING:
				{

					value = TRUE;

					break;
				}

				default:
				{

					debug_fatal ("VH_INT: unknown operational state");
				}
			}

			break;
		}

		case INT_TYPE_MOVEMENT_TYPE:

		{
			value = MOVEMENT_TYPE_AIR;

			break;
		}

		case INT_TYPE_OBJECT_3D_SHAPE:

		{
			value = raw->object_3d_shape;

			break;
		}

		case INT_TYPE_OFFENSIVE_CAPABILITY:

		{
			value = aircraft_database[raw->mob.sub_type].offensive_capability;

			break;
		}

		case INT_TYPE_OPERATIONAL_STATE:

		{
			value = raw->operational_state;

			break;
		}

		case INT_TYPE_PLAYER_CONTROLLABLE:

		{
			value = aircraft_database[raw->mob.sub_type].player_controllable;

			break;
		}

		case INT_TYPE_POINTS_VALUE:

		{
			value = aircraft_database[raw->mob.sub_type].points_value;

			break;
		}

		case INT_TYPE_SCOUT_WEAPON_CONFIG_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].scout_weapon_config_type;

			break;
		}

		case INT_TYPE_SELECTED_WEAPON:

		{
			value = raw->selected_weapon;

			break;
		}

		case INT_TYPE_SELECTED_WEAPON_SYSTEM_READY:

		{
			value = raw->selected_weapon_system_ready;

			break;
		}

		case INT_TYPE_TARGET_PRIORITY_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].target_priority_type;

			break;
		}









		case INT_TYPE_TARGET_TYPE:

		{
			value = TARGET_TYPE_AIR;

			break;
		}

		case INT_TYPE_TASK_LEADER:

		{
			entity
				*guide;

			value = FALSE;

			guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

			if (guide)
			{
				if (get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER) == en)
				{
					value = TRUE;
				}
			}

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			if (get_local_entity_int_value (en, INT_TYPE_LANDED))
			{
				value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_GROUNDED_AIRCRAFT);
			}
			else
			{
				value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_AIRBORNE_AIRCRAFT);
			}

			break;
		}

		case INT_TYPE_THREAT_TYPE:

		{
			value = aircraft_database[raw->mob.sub_type].threat_type;

			break;
		}

































		case INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID:

		{
			value = raw->weapon_and_target_vectors_valid;

			break;
		}

		case INT_TYPE_WEAPON_CONFIG_TYPE:

		{
			value = raw->weapon_config_type;

			break;
		}

		case INT_TYPE_WEAPON_SYSTEM_READY:

		{

			//
			// WARNING! This only returns a valid value for aircraft that have weapon
			//          systems to be made ready in the current weapons configuration.
			//


			if (get_local_entity_float_value (en, FLOAT_TYPE_WEAPON_SYSTEM_READY_STATE) == AIRCRAFT_WEAPON_SYSTEM_READY_OPEN_FLOAT_VALUE)
			{
				value = TRUE;
			}
			else
			{
				value = FALSE;
			}

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














































































































































































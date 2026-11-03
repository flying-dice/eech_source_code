

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
	helicopter
		*raw;

	ASSERT (en);
	
	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_INT_VALUE, en, type, value);

	#endif

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_AUTO_PILOT:

		{
			raw->auto_pilot = value;

			break;
		}

		case INT_TYPE_HELICOPTER_DAMAGE_FLAGS:

		{
			raw->helicopter_damage_flags = value;

			break;
		}

		case INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS:

		{
			raw->helicopter_weapon_damage_flags = value;

			break;
		}

		case INT_TYPE_INFRA_RED_JAMMER_ON:

		{
			raw->infra_red_jammer_on = value;

			break;
		}

		case INT_TYPE_LASER_ON:

		{
			raw->laser_on = value;

			break;
		}

		case INT_TYPE_LOCK_ON_AFTER_LAUNCH:

		{
			raw->lock_on_after_launch = value;

			break;
		}

		case INT_TYPE_LOS_TO_TARGET:

		{
			raw->los_to_target = value;

			break;
		}

		case INT_TYPE_MAIN_ROTOR_DAMAGED:

		{
			if ((value) && (!raw->main_rotor_damaged))
			{
				raw->main_rotor_damaged = value;
				damage_helicopter_main_rotors (en, -2);
			}

			break;
		}

		case INT_TYPE_OPERATIONAL_STATE:

		{
			raw->ac.operational_state = value;

			break;
		}

		case INT_TYPE_PLAYER:

		{

			int
				old_value,
				multiplayer_group,
				create_stack_attributes;

			entity
				*group;

			old_value = raw->player;

			//
			// never set player's entity to 'remote', set 'local' instead
			//

			if (en == get_gunship_entity ())
			{
				if (value == ENTITY_PLAYER_REMOTE)
				{
					value = ENTITY_PLAYER_LOCAL;
				}
			}

			//
			// Set value
			//

			raw->player = value;

			if (get_comms_model () == COMMS_MODEL_SERVER)
			{
				create_stack_attributes = force_local_entity_create_stack_attributes;

				if (get_comms_data_flow () != COMMS_DATA_FLOW_TX)
				{
					set_force_local_entity_create_stack_attributes (TRUE);
				}
	
				//
				// Prepare Helicopter Entity
				//

				if ((old_value == ENTITY_PLAYER_AI) && (value != ENTITY_PLAYER_AI))
				{
					helicopter_assume_player_control (en);
				}
				else if ((old_value != ENTITY_PLAYER_AI) && (value == ENTITY_PLAYER_AI))
				{
					helicopter_release_player_control (en);
				}

				//
				// Maintain Multiplayer group flag
				//
			
				group = raw->ac.member_link.parent;

				if (group)
				{

					entity
						*member;

					multiplayer_group = FALSE;

					member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

					while (member)
					{

						if (get_local_entity_int_value (member, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
						{

							multiplayer_group = TRUE;

							break;
						}

						member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
					}

					set_client_server_entity_int_value (group, INT_TYPE_MULTIPLAYER_GROUP, multiplayer_group);
				}

				set_force_local_entity_create_stack_attributes (create_stack_attributes);
			}

			break;
		}

		case INT_TYPE_POSITION_HOLD:

		{
			raw->position_hold = value;

			break;
		}

		case INT_TYPE_RADAR_JAMMER_ON:

		{
			raw->radar_jammer_on = value;

			break;
		}

		case INT_TYPE_RADAR_ON:

		{
			raw->radar_on = value;

			break;
		}

		case INT_TYPE_TAIL_ROTOR_DAMAGED:

		{
			if ((value) && (!raw->tail_rotor_damaged))
			{
				damage_helicopter_tail_rotors (en);
			}

			raw->tail_rotor_damaged = value;

			break;
		}

		case INT_TYPE_TIMER:

		{
			raw->timer = value;

			break;
		}

		case INT_TYPE_WEAPONS_HOLD:

		{
			raw->weapons_hold = value;

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
	ASSERT (en);

	transmit_entity_comms_message (ENTITY_COMMS_INT_VALUE, en, type, value);
}





static void set_server_int_value (entity *en, int_types type, int value)
{
	ASSERT (en);

	validate_client_server_local_fn ();

	set_local_int_value (en, type, value);

	validate_client_server_remote_fn ();

	set_remote_int_value (en, type, value);
}





static void set_client_int_value (entity *en, int_types type, int value)
{
	ASSERT (en);

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
	helicopter
		*raw;

	int
		value;

	ASSERT (en);

	raw = (helicopter *) get_local_entity_data (en);

	switch (type)
	{

		case INT_TYPE_AUTO_PILOT:

		{
			value = raw->auto_pilot;

			break;
		}

		case INT_TYPE_HELICOPTER_DAMAGE_FLAGS:

		{
			value = raw->helicopter_damage_flags;

			break;
		}

		case INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS:

		{
			value = raw->helicopter_weapon_damage_flags;

			break;
		}

		case INT_TYPE_IDENTIFY_HELICOPTER:

		{
			value = TRUE;

			break;
		}

		case INT_TYPE_INVULNERABLE_FROM_COLLISIONS:

		{

			if ((raw->invulnerable_timer > 0.0) ||
				(get_local_entity_int_value (get_session_entity (), INT_TYPE_INVULNERABLE_FROM_COLLISIONS)))
			{

				value = TRUE;
			}
			else
			{

				value = FALSE;
			}

			break;
		}

		case INT_TYPE_INVULNERABLE_FROM_WEAPONS:

		{

			if ((raw->invulnerable_timer > 0.0) ||
				(get_local_entity_int_value (get_session_entity (), INT_TYPE_INVULNERABLE_FROM_WEAPONS)))
			{

				value = TRUE;
			}
			else
			{

				value = FALSE;
			}

			break;
		}

		case INT_TYPE_INFRA_RED_JAMMER_ON:

		{
			value = raw->infra_red_jammer_on;

			break;
		}

		case INT_TYPE_LANDED_ON_CARRIER:

		{

			entity
				*group,
				*keysite;

			value = FALSE;

			if (get_local_entity_int_value (en, INT_TYPE_LANDED))
			{
				if (get_local_entity_int_value (en, INT_TYPE_ALIVE))
				{
					group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	
					ASSERT (group);
	
					keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);
	
					ASSERT (keysite);

					ASSERT (get_local_entity_type (keysite) == ENTITY_TYPE_KEYSITE);
					
					if (get_local_entity_int_value (keysite, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_KEYSITE_ANCHORAGE)
					{
						value = TRUE;
					}
				}
			}

			break;
		}

		case INT_TYPE_LASER_ON:

		{
			value = raw->laser_on;

			break;
		}

		case INT_TYPE_LOCK_ON_AFTER_LAUNCH:

		{
			value = raw->lock_on_after_launch;

			break;
		}

		case INT_TYPE_LOS_TO_TARGET:

		{
			value = raw->los_to_target;

			break;
		}

		case INT_TYPE_MAIN_ROTOR_DAMAGED:

		{
			value = raw->main_rotor_damaged;

			break;
		}

		case INT_TYPE_PLAYER:

		{
			value = raw->player;

			break;
		}

		case INT_TYPE_POSITION_HOLD:

		{
			value = raw->position_hold;

			break;
		}

		case INT_TYPE_RADAR_JAMMER_ON:

		{
			value = raw->radar_jammer_on;

			break;
		}

		case INT_TYPE_RADAR_ON:

		{
			if (raw->player == ENTITY_PLAYER_AI)
			{
				//
				// AI helicopters must have an airborne radar and a target for the radar to be on
				//

				value = FALSE;

				if (aircraft_database[raw->ac.mob.sub_type].threat_type == THREAT_TYPE_AIRBORNE_RADAR)
				{
					if (raw->ac.mob.target_link.parent != NULL)
					{
						value = TRUE;
					}
				}
			}
			else
			{
				value = raw->radar_on;
			}

			break;
		}

		case INT_TYPE_TAIL_ROTOR_DAMAGED:

		{
			value = raw->tail_rotor_damaged;

			break;
		}

		case INT_TYPE_TASK_TARGET_TYPE:

		{
			if (get_local_entity_int_value (en, INT_TYPE_LANDED))
			{
				value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_GROUNDED_AIRCRAFT | TASK_TARGET_TYPE_HELICOPTER);
			}
			else
			{
				value = (TASK_TARGET_TYPE_ANY | TASK_TARGET_TYPE_MOBILE | TASK_TARGET_TYPE_AIRBORNE_AIRCRAFT | TASK_TARGET_TYPE_HELICOPTER);
			}
			
			break;
		}

		case INT_TYPE_TIMER:

		{
			value = raw->timer;

			break;
		}

		case INT_TYPE_ARMOR_LEVEL:

		{
			value = aircraft_database[raw->ac.mob.sub_type].armor_level;

			break;
		}

		case INT_TYPE_WEAPONS_HOLD:

		{
			value = raw->weapons_hold;

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





void overload_helicopter_int_value_functions (void)
{
	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_AUTO_PILOT]												= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_AUTO_PILOT]												= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_AUTO_PILOT][COMMS_MODEL_SERVER]						= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_AUTO_PILOT][COMMS_MODEL_CLIENT]						= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_AUTO_PILOT]												= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_DAMAGE_FLAGS]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_DAMAGE_FLAGS]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_DAMAGE_FLAGS][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_DAMAGE_FLAGS][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_DAMAGE_FLAGS]				  				= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS]								= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS]								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS][COMMS_MODEL_SERVER]	= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS][COMMS_MODEL_CLIENT]	= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_HELICOPTER_WEAPON_DAMAGE_FLAGS]				  				= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_IDENTIFY_HELICOPTER]		 							= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_INVULNERABLE_FROM_COLLISIONS]		 							= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_INVULNERABLE_FROM_WEAPONS]		 							= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_INFRA_RED_JAMMER_ON]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_INFRA_RED_JAMMER_ON]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_INFRA_RED_JAMMER_ON][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_INFRA_RED_JAMMER_ON][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_INFRA_RED_JAMMER_ON]									= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LANDED_ON_CARRIER]										= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_LASER_ON]													= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LASER_ON]			  										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_LASER_ON][COMMS_MODEL_SERVER]						= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_LASER_ON][COMMS_MODEL_CLIENT]  						= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LASER_ON]			  										= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOCK_ON_AFTER_LAUNCH]									= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOCK_ON_AFTER_LAUNCH]									= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOCK_ON_AFTER_LAUNCH][COMMS_MODEL_SERVER]		= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOCK_ON_AFTER_LAUNCH][COMMS_MODEL_CLIENT]		= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOCK_ON_AFTER_LAUNCH]									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOS_TO_TARGET]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOS_TO_TARGET]			  								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOS_TO_TARGET][COMMS_MODEL_SERVER]					= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOS_TO_TARGET][COMMS_MODEL_CLIENT]  				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_LOS_TO_TARGET]			  								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_MAIN_ROTOR_DAMAGED]										= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_MAIN_ROTOR_DAMAGED]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_MAIN_ROTOR_DAMAGED][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_MAIN_ROTOR_DAMAGED][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_MAIN_ROTOR_DAMAGED]										= get_local_int_value;

	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_OPERATIONAL_STATE]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_OPERATIONAL_STATE][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_OPERATIONAL_STATE][COMMS_MODEL_CLIENT]			= set_client_int_value;

	fn_set_local_entity_raw_int_value  		[ENTITY_TYPE_HELICOPTER][INT_TYPE_PLAYER]							 							= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_PLAYER]						  								= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_PLAYER][COMMS_MODEL_SERVER]							= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_PLAYER][COMMS_MODEL_CLIENT]							= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_PLAYER]														= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_POSITION_HOLD]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_POSITION_HOLD]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_POSITION_HOLD][COMMS_MODEL_SERVER]					= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_POSITION_HOLD][COMMS_MODEL_CLIENT]					= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_POSITION_HOLD]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_JAMMER_ON]											= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_JAMMER_ON]											= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_JAMMER_ON][COMMS_MODEL_SERVER]				= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_JAMMER_ON][COMMS_MODEL_CLIENT]				= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_JAMMER_ON]											= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_ON]													= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_ON]			  										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_ON][COMMS_MODEL_SERVER]						= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_ON][COMMS_MODEL_CLIENT]  						= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_RADAR_ON]			  										= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_TAIL_ROTOR_DAMAGED]										= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_TAIL_ROTOR_DAMAGED]										= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_TAIL_ROTOR_DAMAGED][COMMS_MODEL_SERVER]			= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_TAIL_ROTOR_DAMAGED][COMMS_MODEL_CLIENT]			= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_TAIL_ROTOR_DAMAGED]										= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_TASK_TARGET_TYPE]	 									= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_TIMER]														= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_TIMER]														= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_TIMER][COMMS_MODEL_SERVER]							= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_TIMER][COMMS_MODEL_CLIENT]							= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_TIMER]														= get_local_int_value;

	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_ARMOR_LEVEL] 								= get_local_int_value;

	fn_set_local_entity_raw_int_value		[ENTITY_TYPE_HELICOPTER][INT_TYPE_WEAPONS_HOLD]												= set_local_int_value;
	fn_set_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_WEAPONS_HOLD]												= set_local_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_WEAPONS_HOLD][COMMS_MODEL_SERVER]					= set_server_int_value;
	fn_set_client_server_entity_int_value	[ENTITY_TYPE_HELICOPTER][INT_TYPE_WEAPONS_HOLD][COMMS_MODEL_CLIENT]					= set_client_int_value;
	fn_get_local_entity_int_value				[ENTITY_TYPE_HELICOPTER][INT_TYPE_WEAPONS_HOLD]												= get_local_int_value;
}





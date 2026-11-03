

































































#include "project.h"





#define DEBUG_MODULE 0










//
// LOADING DOORS
//


//#define AIRCRAFT_LOADING_DOORS_CLOSED_FLOAT_VALUE 						0.0

#define AIRCRAFT_LOADING_DOORS_OPENING_FLOAT_VALUE 						1.0

//#define AIRCRAFT_LOADING_DOORS_OPEN_FLOAT_VALUE							2.0

#define AIRCRAFT_LOADING_DOORS_CLOSING_FLOAT_VALUE							3.0





void animate_aircraft_loading_doors( entity *en )
{
	aircraft
		*raw;

	//
	// articulate the doors on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (aircraft *) get_local_entity_data( en );

	animate_entity_reversable_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_LOADING_DOOR, raw->loading_door_state );
}





void update_aircraft_loading_doors( entity *en )
{
	aircraft
		*raw;

	int
		door_state;

	//
	// updates the doors status ( regardless of whether the aircraft is drawn or not )
	//

	ASSERT( en );

	raw = (aircraft *) get_local_entity_data( en );

	door_state = update_entity_reversable_keyframed_value( en, &raw->loading_door_state, 0.5 );

	if ( ( in_flight_articulation_test ) && ( get_comms_model() == COMMS_MODEL_SERVER ) )
	{
		switch ( door_state )
		{

			case 0:

			{
				open_client_server_entity_loading_doors( en );

				break;
			}

			case 2:

			{
				close_client_server_entity_loading_doors( en );

				break;
			}
		}
	}
}






//
// CARGO DOORS
//


//#define AIRCRAFT_CARGO_DOORS_CLOSED_FLOAT_VALUE 						0.0

#define AIRCRAFT_CARGO_DOORS_OPENING_FLOAT_VALUE 						1.0

//#define AIRCRAFT_CARGO_DOORS_OPEN_FLOAT_VALUE							2.0

#define AIRCRAFT_CARGO_DOORS_CLOSING_FLOAT_VALUE							3.0





void animate_aircraft_cargo_doors( entity *en )
{
	aircraft
		*raw;

	//
	// articulate the doors on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (aircraft *) get_local_entity_data( en );

	animate_entity_reversable_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_CARGO_DOOR, raw->cargo_door_state );
}





void update_aircraft_cargo_doors( entity *en )
{
	aircraft
		*raw;

	int
		door_state;

	//
	// updates the doors status ( regardless of whether the aircraft is drawn or not )
	//

	ASSERT( en );

	raw = (aircraft *) get_local_entity_data( en );

	door_state = update_entity_reversable_keyframed_value( en, &raw->cargo_door_state, 0.5 );

	if ( ( in_flight_articulation_test ) && ( get_comms_model() == COMMS_MODEL_SERVER ) )
	{
		switch ( door_state )
		{

			case 0:

			{
				open_client_server_entity_cargo_doors( en );

				break;
			}

			case 2:

			{
				close_client_server_entity_cargo_doors( en );

				break;
			}
		}
	}
}






//
// RUDDER
//


void animate_aircraft_rudder( entity *en )
{
	aircraft
		*raw;

	//
	// articulate the rudder on aircraft
	//

	ASSERT (en);

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (aircraft *) get_local_entity_data (en);

	animate_entity_simple_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_RUDDER, raw->rudder_state );
//	animate_entity_simple_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_KA52_RUDDER, raw->rudder_state ); // doesn't work anyway /thealx/
}





void update_aircraft_rudder( entity *en )
{
	aircraft
		*raw;

	//
	// updates the rudder on an aircraft
	//

	ASSERT( en );

	raw = (aircraft *) get_local_entity_data( en );

	update_entity_simple_keyframed_value( en, &raw->rudder_state, 0.16666 );
}






//
// UNDERCARRIAGE
//


//#define AIRCRAFT_UNDERCARRIAGE_UP_FLOAT_VALUE 						0.0

#define AIRCRAFT_UNDERCARRIAGE_LOWERING_FLOAT_VALUE 					1.0

//#define AIRCRAFT_UNDERCARRIAGE_DOWN_FLOAT_VALUE						2.0

#define AIRCRAFT_UNDERCARRIAGE_RAISING_FLOAT_VALUE						3.0





void animate_aircraft_undercarriage( entity *en )
{
	aircraft
		*raw;

	//
	// articulate the landing gear on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (aircraft *) get_local_entity_data( en );

	animate_entity_reversable_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_UNDERCARRIAGE, raw->undercarriage_state );

}





void update_aircraft_undercarriage( entity *en )
{
	aircraft
		*raw;

	int
		old_uc_state,
		uc_state;

	float
		val;

	//
	// updates the undercarriage status ( regardless of whether the aircraft is drawn or not )
	//

	ASSERT( en );

	raw = (aircraft *) get_local_entity_data( en );

	old_uc_state = raw->undercarriage_state;

	uc_state = update_entity_reversable_keyframed_value( en, &raw->undercarriage_state, 0.5 );

	if ((uc_state == 1) || (uc_state == 3))
	{

		set_local_entity_float_value (en, FLOAT_TYPE_VIEW_INTEREST_LEVEL, DEFAULT_VIEW_INTEREST_LEVEL);
	}

	if ( ( in_flight_articulation_test ) && ( get_comms_model() == COMMS_MODEL_SERVER ) )
	{
		switch ( uc_state )
		{

			case 0:

			{
				lower_client_server_entity_undercarriage( en );

				break;
			}

			case 2:

			{
				raise_client_server_entity_undercarriage( en );

				break;
			}
		}
	}

	//
	// Notify Flight Dynamics of u/c state
	//

	if (en == get_gunship_entity ())
	{
		get_keyframed_animation_state (raw->undercarriage_state, &uc_state, &val);

		switch ( uc_state )
		{
			case AIRCRAFT_UNDERCARRIAGE_UP:
			{
				set_current_flight_dynamics_undercarriage_state (0.0);

				if (old_uc_state == AIRCRAFT_UNDERCARRIAGE_RAISING)
				{
					play_client_server_cpg_message (get_gunship_entity (), 0.5, 1.0, SPEECH_CATEGORY_CPG_SYSTEMS, 1.0, SPEECH_CPG_GEAR_UP);
				}

				break;
			}
			case AIRCRAFT_UNDERCARRIAGE_LOWERING:
			{
				set_current_flight_dynamics_undercarriage_state (val);
				break;
			}
			case AIRCRAFT_UNDERCARRIAGE_DOWN:
			{
				set_current_flight_dynamics_undercarriage_state (1.0);

				if (old_uc_state == AIRCRAFT_UNDERCARRIAGE_LOWERING)
				{
					play_client_server_cpg_message (get_gunship_entity (), 0.5, 1.0, SPEECH_CATEGORY_CPG_SYSTEMS, 1.0, SPEECH_CPG_GEAR_DOWN);
				}

				break;
			}
			case AIRCRAFT_UNDERCARRIAGE_RAISING:
			{
				set_current_flight_dynamics_undercarriage_state (1.0 - val);
				break;
			}
		}
	}
}






//
// WEAPON SYSTEM READY
//


//#define AIRCRAFT_WEAPON_SYSTEM_READY_CLOSED_FLOAT_VALUE 						0.0

#define AIRCRAFT_WEAPON_SYSTEM_READY_OPENING_FLOAT_VALUE 						1.0

//#define AIRCRAFT_WEAPON_SYSTEM_READY_OPEN_FLOAT_VALUE							2.0

#define AIRCRAFT_WEAPON_SYSTEM_READY_CLOSING_FLOAT_VALUE							3.0





void animate_aircraft_weapon_system_ready( entity *en )
{
	aircraft
		*raw;

	//
	// articulate the "weapon system readies" on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (aircraft *) get_local_entity_data( en );

	animate_entity_reversable_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_READY, raw->weapon_system_ready_state );
}





void update_aircraft_weapon_system_ready( entity *en )
{
	aircraft
		*raw;

	int
		wsr_state;

	//
	// updates the weapon system ready status ( regardless of whether the aircraft is drawn or not )
	//

	ASSERT( en );

	raw = (aircraft *) get_local_entity_data( en );

	wsr_state = update_entity_reversable_keyframed_value( en, &raw->weapon_system_ready_state, 0.5 );

	if ( in_flight_articulation_test )
	{
		switch ( wsr_state )
		{

			case 0:

			{
				open_entity_weapon_system_ready( en );

				break;
			}

			case 2:

			{
				close_entity_weapon_system_ready( en );

				break;
			}
		}
	}
}





void animate_aircraft_shadow (entity *en)
{
	aircraft
		*raw;

	ASSERT (en);

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (aircraft *) get_local_entity_data (en);

#ifndef OGRE_EE
	raw->inst3d->object_has_shadow = TRUE;

	//
	// don't draw shadow of aircraft landed on carriers
	//

	if (get_local_entity_int_value (en, INT_TYPE_LANDED_ON_CARRIER))
	{
		raw->inst3d->object_has_shadow = FALSE;
	}

	//
	// don't draw shadow of dead things below terrain
	//

	switch (raw->operational_state)
	{
	 	case OPERATIONAL_STATE_STOPPED:
	 	case OPERATIONAL_STATE_DYING:
	 	case OPERATIONAL_STATE_DEAD:
		{
			if (get_local_entity_float_value (en, FLOAT_TYPE_RADAR_ALTITUDE) < get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE))
			{
				raw->inst3d->object_has_shadow = FALSE;
			}
		}
	}
#endif
}





void initiate_aircraft_crew_ejection (entity *en)
{
	ASSERT (en);

	//
	// eject only implemented for Ka-52 Hokum B
	//

	switch (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE))
	{
		case ENTITY_SUB_TYPE_AIRCRAFT_KA52_HOKUM_B:
		{
			if (!get_local_entity_int_value (en, INT_TYPE_EJECTED))
			{
				if (en == get_gunship_entity ())
				{
					if (select_view_menu_view_players_gunship ())
					{
						if (get_view_mode() != VIEW_MODE_VIRTUAL_COCKPIT_EJECT) {
							set_view_mode (VIEW_MODE_VIRTUAL_COCKPIT_EJECT);
							return;
						}
						else
							set_view_menu_display_visible_status (FALSE);							
					}
					else
					{
						debug_fatal ("Cannot view player's gunship");
					}

					set_client_server_entity_int_value (en, INT_TYPE_EJECTED, TRUE);

					exit_game (NULL);
				}
				else
				{
					set_client_server_entity_int_value (en, INT_TYPE_EJECTED, TRUE);

					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_BLADE, FALSE);
					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_PILOT, FALSE);
					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_CO_PILOT, FALSE);
					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_LH_DOOR, FALSE);
					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_RH_DOOR, FALSE);
				}
			}
			break;
		}
		case ENTITY_SUB_TYPE_AIRCRAFT_KA50_HOKUM:
		{
			if (!get_local_entity_int_value (en, INT_TYPE_EJECTED))
			{
				if (en == get_gunship_entity ())
				{
					if (select_view_menu_view_players_gunship ())
					{
						if (get_view_mode() != VIEW_MODE_VIRTUAL_COCKPIT_EJECT) {
							set_view_mode (VIEW_MODE_VIRTUAL_COCKPIT_EJECT);
							return;
						}
						else
							set_view_menu_display_visible_status (FALSE);							
					}
					else
					{
						debug_fatal ("Cannot view player's gunship");
					}

					set_client_server_entity_int_value (en, INT_TYPE_EJECTED, TRUE);

					exit_game (NULL);
				}
				else
				{
					set_client_server_entity_int_value (en, INT_TYPE_EJECTED, TRUE);

					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_BLADE, FALSE);
					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_PILOT, FALSE);
					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_LH_DOOR, FALSE);
				}
			}
			break;
		}
		default: 	//magitek
		{
			if (get_local_entity_type (en) == ENTITY_TYPE_FIXED_WING)
				if (!get_local_entity_int_value (en, INT_TYPE_EJECTED))
				{
					set_client_server_entity_int_value (en, INT_TYPE_EJECTED, TRUE);

					launch_client_server_weapon (en, ENTITY_SUB_TYPE_WEAPON_HOKUM_PILOT, FALSE);
				}
			break;
		}
	}
}





void uneject_aircraft_crew (entity *en)
{
	weapon_config_types
		config_type;

	ASSERT (en);

	//
	// eject only implemented for Ka-52 Hokum B
	//

	if (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_AIRCRAFT_KA52_HOKUM_B)
	{
		if (get_local_entity_int_value (en, INT_TYPE_EJECTED))
		{
			if (en == get_gunship_entity ())
			{
				reset_hokum_crew_eject_sequence ();
			}

			set_client_server_entity_int_value (en, INT_TYPE_EJECTED, FALSE);

			//
			// reload weapons
			//

			config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

			set_client_server_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE, config_type);
		}
	}
}





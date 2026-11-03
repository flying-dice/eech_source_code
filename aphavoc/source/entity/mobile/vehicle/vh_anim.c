

































































#include "project.h"





#define DEBUG_MODULE 0










//
// LOADING DOORS
//


//#define VEHICLE_LOADING_DOORS_CLOSED_FLOAT_VALUE 					0.0

#define VEHICLE_LOADING_DOORS_OPENING_FLOAT_VALUE 						1.0

//#define VEHICLE_LOADING_DOORS_OPEN_FLOAT_VALUE						2.0

#define VEHICLE_LOADING_DOORS_CLOSING_FLOAT_VALUE						3.0





void animate_vehicle_loading_doors( entity *en )
{
	vehicle
		*raw;

	//
	// articulate the doors on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	if (get_local_entity_type (en) != ENTITY_TYPE_SHIP_VEHICLE)
	{
		//
		// Don't animate ship doors until we've sorted out what's happening with them
		//

		return;
	}

	raw = (vehicle *) get_local_entity_data( en );

	animate_entity_reversable_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_LOADING_DOOR, raw->loading_door_state );
}





void update_vehicle_loading_doors( entity *en )
{
	vehicle
		*raw;

	int
		door_state;

	//
	// updates the doors status ( regardless of whether the vehicle is drawn or not )
	//

	ASSERT( en );

	raw = (vehicle *) get_local_entity_data( en );

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
// WEAPON SYSTEM READY
//


//#define VEHICLE_WEAPON_SYSTEM_READY_CLOSED_FLOAT_VALUE 						0.0

#define VEHICLE_WEAPON_SYSTEM_READY_OPENING_FLOAT_VALUE 							1.0

//#define VEHICLE_WEAPON_SYSTEM_READY_OPEN_FLOAT_VALUE							2.0

#define VEHICLE_WEAPON_SYSTEM_READY_CLOSING_FLOAT_VALUE							3.0





void animate_vehicle_weapon_system_ready( entity *en )
{
	vehicle
		*raw;

	//
	// articulate the "weapon system readies" on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (vehicle *) get_local_entity_data( en );

	animate_entity_reversable_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_READY, raw->weapon_system_ready_state );
}





void update_vehicle_weapon_system_ready( entity *en )
{
	vehicle
		*raw;

	int
		wsr_state;

	//
	// updates the weapon system ready status ( regardless of whether the vehicle is drawn or not )
	//

	ASSERT( en );

	raw = (vehicle *) get_local_entity_data( en );

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






//
// RADAR
//






void animate_vehicle_radar( entity *en )
{
	vehicle
		*raw;

	//
	// articulate the radar on the 3d objects
	//

	ASSERT( en );

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (vehicle *) get_local_entity_data( en );

	animate_entity_simple_keyframed_sub_objects( raw->inst3d, OBJECT_3D_SUB_OBJECT_RADAR, raw->radar_rotation_state );
}





void update_vehicle_radar( entity *en )
{
	vehicle
		*raw;

	//
	// updates the radar rotation ( regardless of whether the vehicle is drawn or not )
	//

	ASSERT( en );

	raw = (vehicle *) get_local_entity_data( en );

	update_entity_simple_keyframed_value( en, &raw->radar_rotation_state, 0.16666 );
}






//
// HEAD-LIGHTS
//


void set_vehicle_headlight_state (entity *en, int state)
{
	object_3d_instance
		*inst3d;

	ASSERT (en);

	inst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);

	ASSERT (inst3d);

	set_sub_object_type_visible_status (inst3d, OBJECT_3D_SUB_OBJECT_EFFECT_HEADLIGHTS, state);

	set_local_entity_int_value (en, INT_TYPE_LIGHTS_ON, state);
}







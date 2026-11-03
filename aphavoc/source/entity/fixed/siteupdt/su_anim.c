

































































#include "project.h"





#define DEBUG_MODULE 0










//
// LOADING DOORS
//


//#define SITE_LOADING_DOORS_CLOSED_FLOAT_VALUE 					0.0

#define SITE_LOADING_DOORS_OPENING_FLOAT_VALUE 						1.0

//#define SITE_LOADING_DOORS_OPEN_FLOAT_VALUE						2.0

#define SITE_LOADING_DOORS_CLOSING_FLOAT_VALUE						3.0





void animate_site_loading_doors( entity *en, object_3d_instance *inst3d )
{
	site_updatable
		*raw;

	//
	// articulate the doors on the 3d objects
	//

	ASSERT (en);

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (site_updatable *) get_local_entity_data (en);

	animate_entity_reversable_keyframed_sub_objects( inst3d, OBJECT_3D_SUB_OBJECT_LOADING_DOOR, raw->loading_door_state );
}





void update_site_loading_doors( entity *en )
{
	site_updatable
		*raw;

	int
		door_state;

	//
	// updates the doors status ( regardless of whether the site is drawn or not )
	//

	ASSERT( en );

	raw = (site_updatable *) get_local_entity_data( en );

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
// RADAR
//






void animate_site_radar( entity *en, object_3d_instance *inst3d )
{
	site_updatable
		*raw;

	//
	// articulate the radar on the 3d objects
	//

	ASSERT (en);

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (site_updatable *) get_local_entity_data (en);

	animate_entity_simple_keyframed_sub_objects( inst3d, OBJECT_3D_SUB_OBJECT_RADAR, raw->radar_rotation_state );
}





void update_site_radar( entity *en )
{
	site_updatable
		*raw;

	//
	// updates the radar rotation ( regardless of whether the site is drawn or not )
	//

	ASSERT( en );

	raw = (site_updatable *) get_local_entity_data( en );

	update_entity_simple_keyframed_value( en, &raw->radar_rotation_state, (0.9 + 0.2 * frand1()) * 0.16666 );
}





void animate_ground_crew( entity *en, object_3d_instance *inst3d )
{
	site_updatable
		*raw;

	ASSERT (en);

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (site_updatable *) get_local_entity_data (en);

	animate_entity_simple_keyframed_sub_objects( inst3d, OBJECT_3D_SUB_OBJECT_GROUND_CREW, raw->ground_crew_animation );
}





void update_ground_crew( entity *en )
{
	site_updatable
		*raw;

	ASSERT( en );

	raw = (site_updatable *) get_local_entity_data( en );

	update_entity_simple_keyframed_value( en, &raw->ground_crew_animation, (0.9 + 0.2 * frand1()) * 0.016666 );
}





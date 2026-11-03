

































































#include "project.h"





#define DEBUG_MODULE 0

#define SHIP_BLURRED_ROTOR_THRESHOLD	3.0

#define SHIP_TEST_SPEED_MAX 8.0
#define SHIP_TEST_SPEED_INC 1.5





static float
	ship_test_speed = 0.0,
	ship_test_speed_inc = SHIP_TEST_SPEED_INC;





void animate_ship_rotors (entity *en)
{
	ship_vehicle
		*raw;

	object_3d_instance
		*inst3d;

	float
		ang,
		speed;

	ASSERT (en);

	if (get_local_entity_int_value (en, INT_TYPE_OBJECT_DRAWN_ONCE_THIS_FRAME))
	{
		return;
	}

	raw = (ship_vehicle *) get_local_entity_data (en);

	inst3d = raw->vh.inst3d;

	if ((in_flight_articulation_test) && (get_external_view_entity() == en))
	{
		//
		// debug articulation test
		//

		ship_test_speed += (ship_test_speed_inc * get_delta_time());

		if (ship_test_speed > SHIP_TEST_SPEED_MAX)
		{
			ship_test_speed_inc = -SHIP_TEST_SPEED_INC;
		}
		else if (ship_test_speed <= 0.0)
		{
			ship_test_speed_inc = SHIP_TEST_SPEED_INC;
		}

		ship_test_speed = bound (ship_test_speed, 0.0, SHIP_TEST_SPEED_MAX);

		speed = ship_test_speed;
	}
	else
	{
		//
		// normal operation
		//

		speed = get_local_entity_float_value (en, FLOAT_TYPE_VELOCITY);
	}

	//
	// set the correct blade type visible status
	//

	if (speed > SHIP_BLURRED_ROTOR_THRESHOLD)
	{
		set_sub_object_type_visible_status (inst3d, OBJECT_3D_SUB_OBJECT_PROPELLER_MOVING, TRUE);
		set_sub_object_type_visible_status (inst3d, OBJECT_3D_SUB_OBJECT_PROPELLER_STATIC, FALSE);

		ang = PI / 20.0;
	}
	else
	{
		set_sub_object_type_visible_status (inst3d, OBJECT_3D_SUB_OBJECT_PROPELLER_MOVING, FALSE);
		set_sub_object_type_visible_status (inst3d, OBJECT_3D_SUB_OBJECT_PROPELLER_STATIC, TRUE);

		ang = 2.0 * speed * get_delta_time ();
	}

	//
	// rotate hub
	//

	modify_sub_object_type_heading_pitch_and_roll (inst3d, OBJECT_3D_SUB_OBJECT_PROPELLER_HUB, ang, 0.0, 0.0);
}





void damage_ship_3d_object (entity *en)
{
	ship_vehicle
		*raw;

	ASSERT (en);

	raw = (ship_vehicle *) get_local_entity_data (en);

	if (raw->vh.object_3d_shape == get_local_entity_int_value (en, INT_TYPE_DESTROYED_3D_SHAPE))
	{
		return;
	}

	//
	// destruct old object
	//

	destruct_3d_object (raw->vh.inst3d);

	//
	// construct new (damaged) object
	//

	raw->vh.object_3d_shape = get_local_entity_int_value (en, INT_TYPE_DESTROYED_3D_SHAPE);

	raw->vh.inst3d = construct_3d_object (raw->vh.object_3d_shape);

	//
	// set id number for new object
	//

	set_routed_vehicle_id_number (en);
}





void set_ship_vehicle_id_number (entity *en)
{
	ship_vehicle
		*raw;

	ASSERT (en);

	raw = (ship_vehicle *) get_local_entity_data (en);

	if (raw->vh.mob.side == ENTITY_SIDE_BLUE_FORCE)
	{
		set_local_entity_id_number
		(
			en,
			TEXTURE_ANIMATION_INDEX_USA_NAVY__000,
			TEXTURE_ANIMATION_INDEX_USA_NAVY__001,
			TEXTURE_ANIMATION_INDEX_USA_NAVY__002,
			TEXTURE_ANIMATION_INDEX_LAST,
			TEXTURE_ANIMATION_INDEX_LAST
		);
	}
	else if (raw->vh.mob.side == ENTITY_SIDE_RED_FORCE)
	{
		set_local_entity_id_number
		(
			en,
			TEXTURE_ANIMATION_INDEX_CIS_PENNANT__000,
			TEXTURE_ANIMATION_INDEX_CIS_PENNANT__001,
			TEXTURE_ANIMATION_INDEX_CIS_PENNANT__002,
			TEXTURE_ANIMATION_INDEX_LAST,
			TEXTURE_ANIMATION_INDEX_LAST
		);
	}
}














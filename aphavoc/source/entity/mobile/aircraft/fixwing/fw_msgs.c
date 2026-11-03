

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0



























































































































































































static int response_to_waypoint_lift_off_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{









	if (get_comms_model () == COMMS_MODEL_SERVER)
	{

		ASSERT (get_local_entity_int_value (receiver, INT_TYPE_OPERATIONAL_STATE) == OPERATIONAL_STATE_TAXIING);

		set_client_server_entity_int_value (receiver, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_TAKEOFF);


	}

	return (TRUE);
}





static int response_to_waypoint_start_up_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{









	set_client_server_fixed_wing_afterburner_state (receiver, ON);

	open_client_server_entity_flaps( receiver );


	//
	// SPECIAL_EFFECT_HOOK FOR PROCEEDING TO TAKEOFF
	//



	//
	//


	return (TRUE);
}





static int response_to_waypoint_touch_down_reached (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{









	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		set_client_server_entity_int_value (receiver, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_TAXIING);

		open_client_server_entity_airbrakes( receiver );

		close_client_server_entity_flaps( receiver );



		//
		// sound effect
		//
		{
			vec3d
				*position;

			sound_sample_indices
				sound_sample_index;







			position = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION);

			sound_sample_index = SOUND_SAMPLE_INDEX_JET_TOUCHDOWN_SCREECH;

			create_client_server_sound_effect_entity
			(
				receiver,
				ENTITY_SIDE_NEUTRAL,
				ENTITY_SUB_TYPE_EFFECT_SOUND_MISC,
				SOUND_CHANNEL_SOUND_EFFECT,
				SOUND_LOCALITY_ALL,
				NULL,												// position
				1.0,												// amplification
				1.0,												// Werewolf pitch
				TRUE,												// valid sound effect
				FALSE,											// looping
				1,													// sample count
				&sound_sample_index							// sample index list
			);
		}
	}

	return (TRUE);
}





static int response_to_check_mobile_reached_guide (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	float
		range,
		velocity;

	vec3d
		*pos1,
		pos2,
		guide_pos,
		*mv;

	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_GUIDE);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	//
	// Check if landed
	//

	if (get_local_entity_int_value (receiver, INT_TYPE_LANDED))
	{
		return FALSE;
	}

	//
	// check range to guide
	//

	if (get_guide_criteria_valid (sender, GUIDE_CRITERIA_RADIUS))
	{
		range = get_local_entity_float_value (receiver, FLOAT_TYPE_DISTANCE);

		velocity = get_local_entity_float_value (receiver, FLOAT_TYPE_VELOCITY);

		//
		// Early-out checks
		//

		if ((range - velocity) > get_guide_criteria_value (sender, GUIDE_CRITERIA_RADIUS))
		{
			return FALSE;
		}

		if (range <= get_guide_criteria_value (sender, GUIDE_CRITERIA_RADIUS))
		{
		}
		else
		{
			//
			// Pass through check
			//

			mv = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_MOTION_VECTOR);

			pos1 = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION);

			if (get_local_entity_int_value (receiver, INT_TYPE_UPDATED))
			{
				pos2.x = pos1->x - (mv->x * get_delta_time ());
				pos2.y = pos1->y - (mv->y * get_delta_time ());
				pos2.z = pos1->z - (mv->z * get_delta_time ());
			}
			else
			{
				pos2.x = pos1->x + (mv->x * get_delta_time ());
				pos2.y = pos1->y + (mv->y * get_delta_time ());
				pos2.z = pos1->z + (mv->z * get_delta_time ());
			}

			get_local_entity_vec3d (receiver, VEC3D_TYPE_GUIDE_POSITION, &guide_pos);

			range = get_3d_perp_dist_of_point_from_line (pos1, &pos2, &guide_pos, NULL);

			if (range > get_guide_criteria_value (sender, GUIDE_CRITERIA_RADIUS))
			{
				return FALSE;
			}
		}
	}

	//
	// check weapon vector
	//

	if (get_guide_criteria_valid (sender, GUIDE_CRITERIA_WEAPON_VECTOR))
	{
		entity_sub_types
			selected_weapon;

		vec3d
			*weapon_vector,
			*weapon_to_intercept_point_vector;

		float
			launch_angle_error;

		selected_weapon = get_local_entity_int_value (receiver, INT_TYPE_SELECTED_WEAPON);

		ASSERT (selected_weapon != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON);

		if (get_local_entity_int_value (receiver, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID))
		{
			weapon_vector = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_WEAPON_VECTOR);

			ASSERT (weapon_vector);

			weapon_to_intercept_point_vector = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR);

			ASSERT (weapon_to_intercept_point_vector);

			launch_angle_error = acos (get_3d_unit_vector_dot_product (weapon_vector, weapon_to_intercept_point_vector));

			if (fabs (launch_angle_error) > pow(frand1(), 4) * weapon_database[selected_weapon].max_launch_angle_error)
			{
				return FALSE;
			}
		}
	}

	return TRUE;
}




































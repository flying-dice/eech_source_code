

































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;

	list_type = va_arg (pargs, list_types);







	switch (list_type)
	{

		case LIST_TYPE_TARGET:

		{
			if (sender)
			{
				if (receiver == get_gunship_entity ())
				{
					if (get_local_entity_int_value (sender, INT_TYPE_THREAT_TYPE) != THREAT_TYPE_INVALID)
					{
						activate_common_threat_warning_system (sender);
					}
				}
			}

			break;
		}
	}

	return (TRUE);
}




























































































































































































































































































































































































































#define HELICOPTER_LAST_TO_REACH_DISTANCE		(750.0 * 750.0)

static int response_to_check_mobile_reached_guide (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	float
		range;

	vec3d
		guide_pos;

	entity
		*wp;

	entity_sub_types
		waypoint_type;

	if (receiver == get_external_view_entity ())
	{

		wp = NULL;
	}

	ASSERT (sender);

	ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_GUIDE);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	wp = get_local_entity_parent (sender, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (wp);

	//
	// Check if player should skip waypoint
	//

	if (get_local_entity_int_value (receiver, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
	{
		waypoint_type = get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE);

		if (waypoint_database [waypoint_type].player_skip_waypoint)
		{
			return TRUE;
		}
	}
	else
	{
	
		//
		// Check if landed
		//
	
		if (get_local_entity_int_value (receiver, INT_TYPE_LANDED))
		{
			return FALSE;
		}
	}
	
	//
	// Don't allow movement before rotors have spun up.
	//

	if ((get_local_entity_float_value (receiver, FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER) > 0.0) && (get_local_entity_int_value (receiver, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI))
	{
		return FALSE;
	}

	if (get_local_entity_int_value (receiver, INT_TYPE_TASK_LEADER))
	{
		//
		// check range to guide
		//
	
		if (get_guide_criteria_valid (sender, GUIDE_CRITERIA_RADIUS))
		{
			range = get_local_entity_float_value (receiver, FLOAT_TYPE_DISTANCE);
	
			if (range > get_guide_criteria_value (sender, GUIDE_CRITERIA_RADIUS))
			{






				return FALSE;
			}
		}
	
		//
		// check altitude
		//
	
		if (get_guide_criteria_valid (sender, GUIDE_CRITERIA_ALTITUDE))
		{
			if ((get_local_entity_int_value (receiver, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI) &&
				(get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_WAYPOINT_LANDED || 
				get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_WAYPOINT_TROOP_INSERT) &&
				(!get_local_entity_int_value (receiver, INT_TYPE_AUTO_PILOT)))
			{
				if (get_local_entity_int_value (receiver, INT_TYPE_AIRBORNE_AIRCRAFT))
				{

					// if player, not autopilot, land/insert wp and not airborne

					return FALSE;
				}
			}
			else
			{
	
				get_local_entity_vec3d (sender, VEC3D_TYPE_POSITION, &guide_pos);
					
				range = get_local_entity_float_value (receiver, FLOAT_TYPE_ALTITUDE);
		
				range -= get_local_entity_float_value (receiver, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE);
		
				range -= guide_pos.y;
		
				if (fabs (range) > get_guide_criteria_value (sender, GUIDE_CRITERIA_ALTITUDE))
				{






					return FALSE;
				}
			}
		}
	
		//
		// check heading
		//
	
		if (get_local_entity_int_value (receiver, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
		{
			if (get_guide_criteria_valid (sender, GUIDE_CRITERIA_HEADING))
			{
				float
					heading,
					delta_heading,
					required_heading;
		
				heading = get_local_entity_float_value (receiver, FLOAT_TYPE_HEADING);
		
				get_guide_required_heading (sender, receiver, &required_heading);
		
				delta_heading = (required_heading - heading);
		
				if (delta_heading < -PI)
				{
					delta_heading += PI2;
				}
				else if (delta_heading > PI)
				{
					delta_heading -= PI2;
				}
		
				if (fabs (delta_heading) > get_guide_criteria_value (sender, GUIDE_CRITERIA_HEADING))
				{






					return FALSE;
				}
			}
		}
		
		//
		// check weapon vector
		//
	
		if (get_local_entity_int_value (receiver, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
		{
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
		
				if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
				{
					debug_log ("GUIDE: no selectedweapon");
					
					return FALSE;
				}
				
				if (get_local_entity_int_value (receiver, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID))
				{
					entity *target;
					
					weapon_vector = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_WEAPON_VECTOR);
			
					ASSERT (weapon_vector);
			
					weapon_to_intercept_point_vector = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR);
			
					ASSERT (weapon_to_intercept_point_vector);
			
					target = get_local_entity_parent (receiver, LIST_TYPE_TARGET);
					
					launch_angle_error = acos (get_3d_unit_vector_dot_product (weapon_vector, weapon_to_intercept_point_vector));

					if (fabs (launch_angle_error) > pow(frand1(), 6) * weapon_database[selected_weapon].max_launch_angle_error)
					{






						return FALSE;
					}
				}
			}
		}
	}
	else
	{
		entity
			*leader;

		vec3d
			*pos,
			*leader_pos;

		ASSERT (get_guide_criteria_valid (sender, GUIDE_CRITERIA_LAST_TO_REACH));

		//
		// None Task Leader Entities (only for last to reach)
		// Check distance to task leader
		//

		leader = (entity *) get_local_entity_ptr_value (sender, PTR_TYPE_TASK_LEADER);

		ASSERT (leader);

		if (get_local_entity_int_value (leader, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
		{
			pos = get_local_entity_vec3d_ptr (receiver, VEC3D_TYPE_POSITION);
			
			leader_pos = get_local_entity_vec3d_ptr (leader, VEC3D_TYPE_POSITION);

			range = get_sqr_2d_range (pos, leader_pos);

			if (range > HELICOPTER_LAST_TO_REACH_DISTANCE)
			{
				return FALSE;
			}
		}
	}
		
	return TRUE;
}








































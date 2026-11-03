

































































#include "project.h"





#define DEBUG_MODULE	0




#ifdef 0
void update_anti_aircraft_weapon_fire (entity *en)
{
   anti_aircraft
      *raw;

	int
		continue_burst_fire;

	float
		launch_angle_error;

	vec3d
		*weapon_vector,
		*weapon_to_intercept_point_vector;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

   raw = get_local_entity_data (en);

	//
	// check valid selected weapon
	//

	if (raw->vh.selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		raw->vh.weapon_burst_timer = 0.0;

		return;
	}

	//
	// check selected weapon is ready
	//

	if (!raw->vh.selected_weapon_system_ready)
	{
		if (raw->vh.weapon_burst_timer > 0.0)
		{

		}

		return;
	}

	//
	// check valid target
	//

	if (!raw->vh.mob.target_link.parent)
	{
		if (raw->vh.weapon_burst_timer > 0.0)
		{

		}

		return;
	}

	//
	// check within max launch angle error
	//

	if (!get_local_entity_int_value (en, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID))
	{
		if (raw->vh.weapon_burst_timer > 0.0)
		{

		}

		return;
	}

	weapon_vector = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_WEAPON_VECTOR);

	ASSERT (weapon_vector);

	weapon_to_intercept_point_vector = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR);

	ASSERT (weapon_to_intercept_point_vector);

	launch_angle_error = acos (get_3d_unit_vector_dot_product (weapon_vector, weapon_to_intercept_point_vector));

	if (fabs (launch_angle_error) > weapon_database[raw->vh.selected_weapon].max_launch_angle_error)
	{
		if (raw->vh.weapon_burst_timer > 0.0)
		{

		}

		return;
	}

	//
	// update burst
	//

	continue_burst_fire = FALSE;

	if (raw->vh.weapon_burst_timer > 0.0)
	{
		raw->vh.weapon_burst_timer -= get_delta_time ();

		if (raw->vh.weapon_burst_timer > 0.0)
		{
			continue_burst_fire = TRUE;
		}
		else
		{

		}
	}

	//
	// fire weapon
	//

	if (raw->vh.ready_to_fire || continue_burst_fire)
	{
		if (!get_local_entity_int_value (get_session_entity (), INT_TYPE_SUPPRESS_AI_FIRE))
		{
			launch_client_server_weapon (en, raw->vh.selected_weapon, FALSE);
		}
	}
}
#endif




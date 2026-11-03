

































































#include "project.h"





#define DEBUG_MODULE 0





void basic_vehicle_movement (entity *en)
{
	vehicle
		*raw;

	entity
		*wp;

	vec3d
		*wp_pos,
		wp_vec,
		new_pos,
		face_normal;

	float
		sqr_range,
		current_heading,
		required_heading,
		delta_heading,
		new_heading,
		current_velocity,
		new_velocity;

	raw = get_local_entity_data (en);


	//
	// Only update if entity has 2 waypoints
	//


	wp = get_local_entity_first_waypoint (en);

	if (!get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT))
	{







		return;
	}


	//
	// current waypoint vector
	//


	wp = raw->mob.current_waypoint_link.parent;

	wp_pos = get_local_entity_vec3d_ptr (wp, VEC3D_TYPE_POSITION);

	wp_vec.x = wp_pos->x - raw->mob.position.x;
	wp_vec.y = wp_pos->y - raw->mob.position.y;
	wp_vec.z = wp_pos->z - raw->mob.position.z;

	sqr_range = (wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z);

	current_velocity = raw->mob.velocity;

	if ((sqr_range < ((current_velocity * current_velocity) / 2.0)) || (sqr_range < 1 * CENTIMETRE))
	{

		wp = get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT);

		if (!wp)
		{

			return;
		}

		delete_local_entity_from_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT, wp, NULL);

		wp_pos = get_local_entity_vec3d_ptr (wp, VEC3D_TYPE_POSITION);

		wp_vec.x = wp_pos->x - raw->mob.position.x;
		wp_vec.y = wp_pos->y - raw->mob.position.y;
		wp_vec.z = wp_pos->z - raw->mob.position.z;

		//
		// safe guard against many waypoints exactly on top of each other
		//

		sqr_range = (wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z);

		if (fabs (sqr_range) < 1 * CENTIMETRE)
		{
			wp_vec.z = 0;
			wp_vec.y = 0;
			wp_vec.z = 1;
		}

		//
		// End of Safe Guard
		//
	}

	normalise_3d_vector (&wp_vec);


	//
	// heading
	//


	current_heading = get_heading_from_attitude_matrix (raw->mob.attitude);

	required_heading = atan2 (wp_vec.x, wp_vec.z);

	delta_heading = required_heading - current_heading;

	//
	// turn shortest way
	//

	if (delta_heading > PI)
	{
		delta_heading -= 2.0 * PI;
  	}
	else if (delta_heading < -PI)
	{
		delta_heading += 2.0 * PI;
	}

  	new_heading = current_heading + delta_heading * get_delta_time ();


	//
	// attitude
	//


	get_3d_terrain_face_normal (&face_normal, raw->mob.position.x, raw->mob.position.z);

	get_3d_transformation_matrix_from_face_normal_and_heading (&raw->mob.attitude, &face_normal, new_heading);


	//
	// velocity
	//


	new_velocity = get_local_entity_float_value (wp, FLOAT_TYPE_VELOCITY);

	raw->mob.velocity = new_velocity;


	//
	// position
	//


	new_pos.x = raw->mob.position.x + (new_velocity * raw->mob.zv.x * get_delta_time ());
	new_pos.y = raw->mob.position.y + (new_velocity * raw->mob.zv.y * get_delta_time ());
	new_pos.z = raw->mob.position.z + (new_velocity * raw->mob.zv.z * get_delta_time ());

	ASSERT(point_inside_map_area(&new_pos));
	
	new_pos.y = get_3d_terrain_elevation (new_pos.x, new_pos.z);

	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &new_pos);
}







































































#include "project.h"





#define DEBUG_MODULE 0





void basic_aircraft_movement (entity *en)
{
	aircraft
		*raw;

	entity
		*wp;

	vec3d
		wp_pos,
		wp_vec,
		new_pos;

	float
		terrain_elevation,
		sqr_range,
		current_heading,
		required_heading,
		delta_heading,
		new_heading,
		current_pitch,
		required_pitch,
		delta_pitch,
		new_pitch,
		current_roll,
		required_roll,
		delta_roll,
		new_roll,
		current_velocity,
		new_velocity;

	raw = get_local_entity_data (en);

	debug_fatal ("NOT CODED");


	//
	// Only update if entity has 2 waypoints
	//


	wp = get_local_entity_first_waypoint (en);

	if (!get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT))
	{







		return;
	}

	if (!raw->mob.current_waypoint_link.parent)
	{

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT, wp, NULL);
	}


	//
	// current waypoint vector
	//


	wp = raw->mob.waypoint_root.first_child;

	get_local_entity_vec3d (wp, VEC3D_TYPE_POSITION, &wp_pos);

	//
	// debug end
	//

	sqr_range = (wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z);

	current_velocity = raw->mob.velocity;

	//if ((sqr_range < ((current_velocity * current_velocity) / 2.0)) || (sqr_range < 1 * CENTIMETRE))
	if (notify_local_entity (ENTITY_MESSAGE_WAYPOINT_REACHED, raw->mob.current_waypoint_link.parent, en, sqr_range))
	{

		int
			index;

		float
			velocity,
			radius;

		//wp = select_local_entity_next_waypoint (en);
		wp = get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT); 

		if (!wp)
		{

			return;
		}

		delete_local_entity_from_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT, wp, NULL);

		index = get_local_entity_int_value (member, INT_TYPE_FORMATION_POSITION);

		get_local_waypoint_formation_position (index, wp, &wp_pos);

		//
		// set internal waypoint to new wp's position, client/server.
		//

		velocity = get_local_entity_float_value (wp, FLOAT_TYPE_VELOCITY);

		radius = get_local_entity_float_value (wp, FLOAT_TYPE_RADIUS);

		set_client_server_entity_vec3d (raw->mob.waypoint_root.first_child, VEC3D_TYPE_POSITION, &wp_pos);

		set_client_server_entity_float_value (raw->mob.waypoint_root.first_child, FLOAT_TYPE_VELOCITY, velocity);

		set_client_server_entity_float_value (raw->mob.waypoint_root.first_child, FLOAT_TYPE_RADIUS, radius);

		//
		//
		//

		wp_vec.x = wp_pos.x - raw->mob.position.x;
		wp_vec.y = wp_pos.y - raw->mob.position.y;
		wp_vec.z = wp_pos.z - raw->mob.position.z;

		//
		// safe guard against many waypoints exactly on top of each other
		//

		sqr_range = (wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z);
	}
/*
	if (fabs (sqr_range) < 1 * CENTIMETRE)
	{
		wp_vec.z = 0;
		wp_vec.y = 0;
		wp_vec.z = 1;
	}

	normalise_3d_vector (&wp_vec);
*/


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
	// pitch
	//


	current_pitch = get_pitch_from_attitude_matrix (raw->mob.attitude);

	required_pitch = asin (wp_vec.y);

	delta_pitch = required_pitch - current_pitch;

	new_pitch = current_pitch + delta_pitch * get_delta_time ();


	//
	// roll
	//


	current_roll = get_roll_from_attitude_matrix (raw->mob.attitude);

	//
	// should really limit to delta_heading / 2.0 (90 degs), but this looks better for now
	//

	required_roll = delta_heading;

	delta_roll = required_roll - current_roll;

	new_roll = current_roll + delta_roll * get_delta_time ();


	//
	// attitude
	//


	get_3d_transformation_matrix (&raw->mob.attitude, new_heading, new_pitch, new_roll);


	//
	// velocity
	//


	new_velocity = get_local_entity_float_value (wp, FLOAT_TYPE_VELOCITY);

	raw->mob.velocity += (new_velocity - raw->mob.velocity) * get_delta_time ();


	//
	// position
	//


	new_pos.x = raw->mob.position.x + (new_velocity * raw->mob.zv.x * get_delta_time ());
	new_pos.y = raw->mob.position.y + (new_velocity * raw->mob.zv.y * get_delta_time ());
	new_pos.z = raw->mob.position.z + (new_velocity * raw->mob.zv.z * get_delta_time ());

	ASSERT(point_inside_map_area(&new_pos));
	
	terrain_elevation = get_3d_terrain_elevation (new_pos.x, new_pos.z);

	new_pos.y = max (new_pos.y, terrain_elevation + get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE));

	//
	// debug - to make the aircraft gradually climb to 50m
	//

	//if (new_pos.y < terrain_elevation + 50)
	//{
		//new_pos.y += 5 * get_delta_time ();
	//}

	//
	// end
	//

	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &new_pos);
}





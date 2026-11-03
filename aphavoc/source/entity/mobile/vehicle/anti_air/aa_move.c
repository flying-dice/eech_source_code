

































































#include "project.h"





//
// 3 levels of debug
//

#define DEBUG_MODULE 									0





static void anti_aircraft_falling_movement (entity *en);





void anti_aircraft_death_movement (entity *en)
{

	anti_aircraft
		*raw;

	float
		heading,
		pitch,
		roll,
		speed,
		terrain_elevation;

	vec3d
		*pos,
		*velocity,
		new_pos;

	raw = (anti_aircraft *) get_local_entity_data (en);

	//
	// first check to see if vehicle is on the ground or not
	//

	pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	terrain_elevation = get_3d_terrain_point_data_elevation (&raw->vh.terrain_info);

	if ((get_terrain_type_class (get_3d_terrain_point_data_type (&raw->vh.terrain_info)) == TERRAIN_CLASS_WATER) ||
			(pos->y > (terrain_elevation + get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE) + 0.2)))
	{

		anti_aircraft_falling_movement (en);

		return;
	}

	//
	// if vehicle has come to a stop, remove from update list
	//

	velocity = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_MOTION_VECTOR);

	speed = get_3d_vector_magnitude (velocity);

	if (speed == 0.0)
	{

//		delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
		set_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_DEAD);

		return;
	}

	//
	// work out new position 
	//

	new_pos.x = pos->x + (velocity->x * get_entity_movement_delta_time());
	new_pos.z = pos->z + (velocity->z * get_entity_movement_delta_time());
	
	ASSERT(point_inside_map_area(&new_pos));

	get_3d_terrain_point_data (new_pos.x, new_pos.z, &raw->vh.terrain_info);

	terrain_elevation = get_3d_terrain_point_data_elevation (&raw->vh.terrain_info);

	new_pos.y = terrain_elevation + get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE);

	//
	// update velocity
	//

	normalise_3d_vector_given_magnitude (velocity, speed);

	speed -= (4.0 * get_entity_movement_delta_time());

	speed = max (0.0f, speed);
	
	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, speed);

	velocity->x *= speed;
	velocity->y *= speed;
	velocity->z *= speed;

	//
	// update attitude
	//
		
	heading = get_heading_from_attitude_matrix (raw->vh.mob.attitude);

	pitch = get_pitch_from_attitude_matrix (raw->vh.mob.attitude);

	roll = get_roll_from_attitude_matrix (raw->vh.mob.attitude);
	
	heading -= (PI * 0.1 * get_entity_movement_delta_time());

	get_3d_transformation_matrix (raw->vh.mob.attitude, heading, pitch, roll);

	//
	// set new position
	//

	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &new_pos);
}





void anti_aircraft_falling_movement (entity *en)
{

	anti_aircraft
		*raw;

	float
		speed,
		altitude,
		terrain_elevation,
		old_terrain_elevation;

	vec3d
		*pos,
		*velocity,
		new_pos;

	raw = (anti_aircraft *) get_local_entity_data (en);

	//
	// get current position and velocity
	//

	pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	old_terrain_elevation = get_3d_terrain_point_data_elevation (&raw->vh.terrain_info);

	velocity = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_MOTION_VECTOR);

	new_pos.x = pos->x + (velocity->x * get_entity_movement_delta_time());
	new_pos.y = pos->y + (velocity->y * get_entity_movement_delta_time());
	new_pos.z = pos->z + (velocity->z * get_entity_movement_delta_time());
	
	ASSERT(point_inside_map_area(&new_pos));

	get_3d_terrain_point_data (new_pos.x, new_pos.z, &raw->vh.terrain_info);

	//
	// update velocity
	//

	velocity->y -= (G * get_entity_movement_delta_time());	

	//
	// check if vehicle has hit the floor
	//

	terrain_elevation = get_3d_terrain_point_data_elevation (&raw->vh.terrain_info);

	altitude = new_pos.y - (terrain_elevation + get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE));
		
	if (altitude <= 0.0)
	{
		//
		// no lateral movement below ground
		//

		new_pos.x = pos->x;
		new_pos.z = pos->z;

		if (get_terrain_type_class (get_3d_terrain_point_data_type (&raw->vh.terrain_info)) == TERRAIN_CLASS_WATER)
		{

			//
			// VEHICLE IS BELOW GROUND AND IN WATER
			//

			
			if (pos->y > old_terrain_elevation)
			{
				//
				// vehicle has hit the water this frame - create a "splash"
				//
		
				if (get_comms_model() == COMMS_MODEL_SERVER)
				{
					vec3d
						temp_pos;

					temp_pos.x = new_pos.x;
					temp_pos.y = terrain_elevation;
					temp_pos.z = new_pos.z;

					set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &temp_pos);

					create_client_server_collision_effect (&temp_pos, DYNAMICS_COLLISION_SURFACE_WATER, 1);
				}

				//
				// also kill any fires that may be burning
				//

				clear_smoke_list_generator_lifetime (en, ENTITY_SUB_TYPE_EFFECT_SMOKE_LIST_FIRE);
			}

			{
				//
				// make vehicle sink until it is no longer visible
				//
	
				struct OBJECT_3D_BOUNDS
					*bounding_box;
	
				vec3d
					d;
	
				bounding_box = get_object_3d_bounding_box (get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE));
	
				d.x = bounding_box->xmax - bounding_box->xmin;
				d.y = bounding_box->ymax - bounding_box->ymin;
				d.z = bounding_box->zmax - bounding_box->zmin;
	
				if (altitude < -get_3d_vector_magnitude (&d))
				{
					//
					// vehicle is no longer visible
					//
	
					velocity->x = 0.0;
					velocity->y = 0.0;
					velocity->z = 0.0;

//					delete_local_entity_from_parents_child_list (en, LIST_TYPE_VIEW);
	
//					delete_local_entity_from_parents_child_list (en, LIST_TYPE_UPDATE);
					set_local_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_DEAD);
	
				}
			}
		}
		else
		{

			//
			// VEHICLE HAS HIT SOLID GROUND
			//

			
			new_pos.y = terrain_elevation + get_local_entity_float_value (en, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE);
			
			if (get_comms_model() == COMMS_MODEL_SERVER)
			{
				vec3d
					temp_pos;

				temp_pos.x = new_pos.x;
				temp_pos.y = terrain_elevation;
				temp_pos.z = new_pos.z;

				set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &temp_pos);

				create_client_server_collision_effect (&temp_pos, DYNAMICS_COLLISION_SURFACE_GROUND, 2);
			}

			velocity->y = 0.0;
		}
	}

	//
	// update speed
	//

	speed = get_3d_vector_magnitude (velocity);
	
	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, speed);

	//
	// set new position
	//

	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &new_pos);
}





void anti_aircraft_impact_movement (entity *en)
{

	anti_aircraft
		*raw;

	float
		speed;

	vec3d
		*velocity;

	int
		seed;

	//
	// when a vehicle is killed it needs to be "moved out the way", so give it an instantaneous sideways velocity
	//

	raw = (anti_aircraft *) get_local_entity_data (en);

	seed = get_client_server_entity_random_number_seed(en);

	speed = 4.0 * frand1x (&seed);

	velocity = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_MOTION_VECTOR);

	velocity->x += (speed * raw->vh.mob.xv.x);

	velocity->z += (speed * raw->vh.mob.xv.z);
}





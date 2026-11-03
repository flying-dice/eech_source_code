

































































#include "project.h"





//
// 3 levels of debug
//

#define DEBUG_MODULE 			0

#define DEBUG_WAYPOINT_VECTOR 0
#define PERSON_WALK_VELOCITY	(5 * METRE)





static void person_movement_get_waypoint_position (entity *en, vec3d *wp_pos)
{

	entity
		*wp,
		*guide;

	float
		distance;

	vec3d
		*pos;

	ASSERT (en);

	ASSERT (wp_pos);

	guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

	ASSERT (guide);

	wp = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (wp);

	if (get_local_entity_int_value (wp, INT_TYPE_MOBILE_FOLLOW_WAYPOINT_OFFSET))
	{
		vec3d
			offset;

		get_local_entity_vec3d (guide, VEC3D_TYPE_GUIDE_POSITION, wp_pos);

		get_local_entity_formation_position_offset (en, wp, &offset);

		wp_pos->x += offset.x;
		wp_pos->y += offset.y;
		wp_pos->z += offset.z;
	}
	else
	{
		//
		// Task leader follows guide,.... other members follow task leader
		//
	
		if (get_local_entity_int_value (en, INT_TYPE_TASK_LEADER))
		{
			get_local_entity_vec3d (guide, VEC3D_TYPE_GUIDE_POSITION, wp_pos);
		}
		else
		{
			//
			// set wp pos to offset from task leader
			//
	
			entity
				*group,
				*task_leader;
	
			vec3d
				*xv,
				*leader_pos;
	
			formation_type
				*formation;
	
			int
				type,
				formation_count,
				formation_index,
				leader_formation_index;
	
			//
			// find task leader
			//

			group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

			task_leader = (entity *) get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER);

			ASSERT (task_leader);
	
			//
			// get formation
			//
	
			type = get_local_entity_int_value (group, INT_TYPE_GROUP_FORMATION);
	
			formation = get_formation ((formation_types) type);
	
			formation_count = formation->number_in_formation;
	
			formation_index = get_local_entity_int_value (en, INT_TYPE_GROUP_MEMBER_NUMBER);
	
			leader_formation_index = get_local_entity_int_value (task_leader, INT_TYPE_GROUP_MEMBER_NUMBER);
	
			ASSERT (formation_index < formation_count);
			ASSERT (leader_formation_index < formation_count);
	
			//
			// calculate position
			//
	
			leader_pos = get_local_entity_vec3d_ptr (task_leader, VEC3D_TYPE_POSITION);
	
			xv = get_local_entity_vec3d_ptr (task_leader, VEC3D_TYPE_XV);
	
			//
			// take leader position and SUBTRACT leaders formation position (coz leader is not necessarily formation pos 0)
			//
			
			wp_pos->x = leader_pos->x - ((xv->x * formation->sites [leader_formation_index].x) - (xv->z * formation->sites [leader_formation_index].z));
			wp_pos->y = leader_pos->y - formation->sites [leader_formation_index].y;
			wp_pos->z = leader_pos->z - ((xv->x * formation->sites [leader_formation_index].z) + (xv->z * formation->sites [leader_formation_index].x));
	
			//
			// then ADD members formation position
			//
	
			wp_pos->x += ((xv->x * formation->sites [formation_index].x) - (xv->z * formation->sites [formation_index].z));
			wp_pos->y += formation->sites [formation_index].y;
			wp_pos->z += ((xv->x * formation->sites [formation_index].z) + (xv->z * formation->sites [formation_index].x));
		}
	}

	//
	// calculate distance of entity to next waypoint position
	//

	pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	distance = get_2d_range (pos, wp_pos);

	#if DEBUG_WAYPOINT_VECTOR

	if (distance > 0.0)
	{
		create_debug_3d_line (pos, wp_pos, sys_col_black, 0.0);
	}

	#endif

	set_local_entity_float_value (en, FLOAT_TYPE_DISTANCE, distance);
}





void person_movement (entity *en)
{

	vec3d
		new_pos,
		normalised_wp_vec,
		wp_vec,
		wp_pos;

	person
		*raw;

	entity
		*task,
		*guide,
		*group;

	float
		ps_vel,
		distance,
		movement_distance,
		heading,
		x_inc,
		y_inc,
		z_inc,
		remainder;

	int
		state_selector,
		state;

	guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

	raw = (person *) get_local_entity_data (en);

	get_keyframed_animation_state (raw->person_animation_state, &state, &remainder);

	if ((!guide) || (raw->vh.sleep > 0.0))
	{
		//no guide - therefore not moving - reset anim state
		if ((state == PERSON_ANIM_WALK) || (state == PERSON_ANIM_RUN))
		{
			state = PERSON_ANIM_NONE;
		}

		// if no anim - select stand anim
		if (state == PERSON_ANIM_NONE)
		{
			state_selector = (rand16 () % 4);

			state = PERSON_ANIM_STAND1 + state_selector;

			remainder = 0.0;
			
			raw->person_animation_state = state + remainder;

			ASSERT ( raw->person_animation_state < NUM_PERSON_ANIMS );
		}

		return;
	}

	if (get_local_entity_parent (en, LIST_TYPE_TARGET))
	{
		//targetting animation/position

		return;
	}

	person_movement_get_waypoint_position (en, &wp_pos);

	wp_vec.x = wp_pos.x - raw->vh.mob.position.x;
	wp_vec.y = wp_pos.y - raw->vh.mob.position.y;
	wp_vec.z = wp_pos.z - raw->vh.mob.position.z;

	normalised_wp_vec = wp_vec;

	distance = normalise_any_3d_vector (&normalised_wp_vec);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	
	ASSERT(group);
	
	guide = get_local_entity_first_child (group, LIST_TYPE_GUIDE_STACK);
	
	if (distance > 0.5)
		while (guide)
		{
			task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

			ASSERT (task);

			if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_TROOP_MOVEMENT_INSERT_CAPTURE ||
					get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_TROOP_MOVEMENT_INSERT_DEFEND ||
					get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_ENGAGE)
			{
				state = PERSON_ANIM_RUN;
				break;
			}
			else
				state = PERSON_ANIM_WALK;

			guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE_STACK);
		}
	else // person actually stopped
		state = PERSON_ANIM_NONE;
	
	ps_vel = get_person_database_velocity ((person_animations) state);

	raw->person_animation_state = state + remainder;

	movement_distance = min (ps_vel * get_entity_movement_delta_time (), distance);

	x_inc = normalised_wp_vec.x * movement_distance;
	y_inc = normalised_wp_vec.y * movement_distance;
	z_inc = normalised_wp_vec.z * movement_distance;

	new_pos.x = raw->vh.mob.position.x + x_inc;
	new_pos.y = raw->vh.mob.position.y + y_inc;
	new_pos.z = raw->vh.mob.position.z + z_inc;

	if (state == PERSON_ANIM_WALK || state == PERSON_ANIM_RUN)
	{
		ASSERT(point_inside_map_area(&new_pos));
		new_pos.y = get_3d_terrain_elevation (new_pos.x, new_pos.z) + 0.01;
	}

	heading = atan2 (normalised_wp_vec.x, normalised_wp_vec.z);

	get_3d_transformation_heading_matrix (raw->vh.mob.attitude, heading);

	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &new_pos);
	
	raw->vh.mob.velocity = ps_vel;

	raw->vh.mob.motion_vector.x = x_inc / get_entity_movement_delta_time ();
	raw->vh.mob.motion_vector.y = y_inc / get_entity_movement_delta_time ();
	raw->vh.mob.motion_vector.z = z_inc / get_entity_movement_delta_time ();
}





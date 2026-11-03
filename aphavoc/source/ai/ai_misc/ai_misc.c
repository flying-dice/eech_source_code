

































































#include "project.h"

#include "ai_misc.h"





#define LANDING_DEBUG 0

#define DEBUG_MODULE 0





entity *get_closest_waypoint (entity *task, vec3d *pos, float min_range, float *actual_range)
{

	float
		best_range,
		range;

	vec3d
		*waypoint_pos;

	entity
		*closest_waypoint = NULL,
		*current_waypoint;

	best_range = 99999999;

	current_waypoint = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT);

	while (current_waypoint)
	{

		waypoint_pos = get_local_entity_vec3d_ptr (current_waypoint, VEC3D_TYPE_POSITION);

		range = get_approx_2d_range (waypoint_pos, pos);

		if (actual_range)
		{
	
			*actual_range = range;
		}

		if (range <= min_range)
		{







			return current_waypoint;
		}

		if (range < best_range)
		{

			best_range = range;

			closest_waypoint = current_waypoint;






		}

		current_waypoint = get_local_entity_child_succ (current_waypoint, LIST_TYPE_WAYPOINT);
	} 

	if ( closest_waypoint )
	{

		waypoint_pos = get_local_entity_vec3d_ptr (closest_waypoint, VEC3D_TYPE_POSITION);

		best_range = get_2d_range (waypoint_pos, pos);
	}

	if (actual_range)
	{

		*actual_range = best_range;
	}

	return closest_waypoint;
}





entity *get_closest_halfway_waypoint (entity *task, vec3d *pos, float min_range, float *actual_range)
{

	float
		best_range,
		range;

	vec3d
		*waypoint_pos1,
		*waypoint_pos2,
		mid_pos;

	entity
		*closest_waypoint = NULL,
		*waypoint1,
		*waypoint2;

	best_range = 99999999;

	waypoint1 = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT);
	waypoint2 = get_local_entity_child_succ (waypoint1, LIST_TYPE_WAYPOINT);

	while (waypoint2)
	{

		waypoint_pos1 = get_local_entity_vec3d_ptr (waypoint1, VEC3D_TYPE_POSITION);
		waypoint_pos2 = get_local_entity_vec3d_ptr (waypoint2, VEC3D_TYPE_POSITION);

		mid_pos.x = waypoint_pos1->x + ( ( waypoint_pos2->x - waypoint_pos1->x ) / 2 );
		mid_pos.y = waypoint_pos1->y + ( ( waypoint_pos2->y - waypoint_pos1->y ) / 2 );
		mid_pos.z = waypoint_pos1->z + ( ( waypoint_pos2->z - waypoint_pos1->z ) / 2 );

		range = get_approx_2d_range (&mid_pos, pos);

		if (range < best_range)
		{

			best_range = range;
			closest_waypoint = waypoint1;
		}

		waypoint1 = waypoint2;
		waypoint2 = get_local_entity_child_succ (waypoint2, LIST_TYPE_WAYPOINT);
	}

	if ( closest_waypoint )
	{

		waypoint1 = closest_waypoint;
		waypoint2 = get_local_entity_child_succ (waypoint1, LIST_TYPE_WAYPOINT);

		waypoint_pos1 = get_local_entity_vec3d_ptr (waypoint1, VEC3D_TYPE_POSITION);
		waypoint_pos2 = get_local_entity_vec3d_ptr (waypoint2, VEC3D_TYPE_POSITION);

		mid_pos.x = waypoint_pos1->x + ( ( waypoint_pos2->x - waypoint_pos1->x ) / 2 );
		mid_pos.y = waypoint_pos1->y + ( ( waypoint_pos2->y - waypoint_pos1->y ) / 2 );
		mid_pos.z = waypoint_pos1->z + ( ( waypoint_pos2->z - waypoint_pos1->z ) / 2 );

		best_range = get_2d_range (&mid_pos, pos);
	}

	if (actual_range)
	{

		*actual_range = best_range;
	}

	return ( closest_waypoint );
}





entity *get_closest_free_landing_site (entity_sub_types sub_type, entity *force, vec3d *pos, float min_range, float *actual_range, int sites_required)
{

	int
		sites_available;

	float
		best_range,
		range;

	vec3d
		*landing_pos;

	entity
		*closest_landing = NULL,
		*current_keysite,
		*current_landing;

	best_range = 99999999;

	current_keysite = get_local_entity_first_child (force, LIST_TYPE_KEYSITE_FORCE);

	while (current_keysite)
	{
		if (get_local_entity_int_value (current_keysite, INT_TYPE_IN_USE))
		{
			if (get_local_entity_int_value (current_keysite, INT_TYPE_KEYSITE_USABLE_STATE) == KEYSITE_STATE_USABLE)
			{
				current_landing = get_local_entity_first_child (current_keysite, LIST_TYPE_LANDING_SITE);
		
				while (current_landing)
				{
		
					if (get_local_entity_int_value (current_landing, INT_TYPE_ENTITY_SUB_TYPE) == sub_type)
					{
		
						sites_available = get_local_entity_int_value (current_landing, INT_TYPE_FREE_LANDING_SITES) - get_local_entity_int_value (current_landing, INT_TYPE_RESERVED_LANDING_SITES);
		
						if (sites_available >= sites_required)
						{
		
							landing_pos = get_local_entity_vec3d_ptr (current_landing, VEC3D_TYPE_POSITION);
		
							range = get_approx_2d_range (landing_pos, pos);
	





		
							if (range <= min_range)
							{
		











								if (actual_range)
								{
									*actual_range = range;	
								}
	
								return current_landing;
							}
		
							if (range < best_range)
							{
		
								best_range = range;
		
								closest_landing = current_landing;
		












						}
					}
		
					current_landing = get_local_entity_child_succ (current_landing, LIST_TYPE_LANDING_SITE);
				}
			}
		}
		
		current_keysite = get_local_entity_child_succ (current_keysite, LIST_TYPE_KEYSITE_FORCE);
	}

	if (actual_range)
	{
		*actual_range = best_range;
	}

	//






































		{
		
			int
				sites_available;
		
			float
				best_range,
				range;
		
			vec3d
				*landing_pos;
		
			entity
				*closest_landing = NULL,
				*current_keysite,
				*current_landing;
		
			best_range = 99999999;
		
			current_keysite = get_local_entity_first_child (force, LIST_TYPE_KEYSITE_FORCE);
		
			while (current_keysite)
			{
				if (get_local_entity_int_value (current_keysite, INT_TYPE_IN_USE))
				{
					if (get_local_entity_int_value (current_keysite, INT_TYPE_KEYSITE_USABLE_STATE) == KEYSITE_STATE_USABLE)
					{
						current_landing = get_local_entity_first_child (current_keysite, LIST_TYPE_LANDING_SITE);
				
						while (current_landing)
						{
				
							if (get_local_entity_int_value (current_landing, INT_TYPE_ENTITY_SUB_TYPE) == sub_type)
							{
				
								sites_available = get_local_entity_int_value (current_landing, INT_TYPE_FREE_LANDING_SITES) - get_local_entity_int_value (current_landing, INT_TYPE_RESERVED_LANDING_SITES);
				
								if (sites_available >= sites_required)
								{
				
									landing_pos = get_local_entity_vec3d_ptr (current_landing, VEC3D_TYPE_POSITION);
				
									range = get_approx_2d_range (landing_pos, pos);
			
									if (range <= min_range)
									{
				
										if (actual_range)
										{
											*actual_range = range;
										}
	
										debug_log ("AI_MISC: found keysite %s", get_local_entity_string (current_keysite, STRING_TYPE_KEYSITE_NAME));
									}
				
									if (range < best_range)
									{
				
										best_range = range;
				
										closest_landing = current_landing;
									}
								}
							}
				
							current_landing = get_local_entity_child_succ (current_landing, LIST_TYPE_LANDING_SITE);
						}
					}
				}
		
				current_keysite = get_local_entity_child_succ (current_keysite, LIST_TYPE_KEYSITE_FORCE);
			}
		
			if (actual_range)
			{
				*actual_range = best_range;
			}

			//debug_log ("AI_MISC: found keysite %s", get_local_entity_string (current_keysite, STRING_TYPE_KEYSITE_NAME));
		}
	}

	//
	// END
	//

	//ASSERT (closest_landing);

	return closest_landing;
}





int get_closest_road_node (vec3d *pos, float error)
{

	int
		loop,
		best_node = 0;

	float
		range,
		best_range;

	ASSERT (pos);
		
	ASSERT (road_node_positions);

	best_range = 99999999;

	for (loop = 0; loop < total_number_of_road_nodes; loop ++)
	{

		if (road_nodes [loop].number_of_links > 0)
		{
	
			range = get_approx_2d_range (&road_node_positions [loop], pos);
	
			if (range < best_range)
			{
	
				best_range = range;
	
				best_node = loop;
	
				if (range <= error)
				{
	
					return best_node;
				}
			}
		}
	}







	return best_node;
}





int get_closest_side_road_node (entity_sides side, vec3d *pos, float error)
{

	int
		loop,
		best_node = 0;

	float
		range,
		best_range;

	ASSERT (pos);
		
	ASSERT (road_node_positions);

	best_range = 99999999;

	for (loop = 0; loop < total_number_of_road_nodes; loop ++)
	{

//		if (road_nodes [loop].side_aware & side)
		{

			if (road_nodes [loop].number_of_links > 0)
			{
		
				range = get_approx_2d_range (&road_node_positions [loop], pos);
		
				if (range < best_range)
				{
		
					best_range = range;
		
					best_node = loop;
		
					if (range <= error)
					{
		
						return best_node;
					}
				}
			}
		}
	}







	return best_node;
}




















































































































































































































































































































































































































































entity *get_closest_mobile_entity (vec3d *pos)
{

	entity
		*closest_en = NULL,
		*en;

	int
		range,
		best_range = INT_MAX;

	vec3d
		pos2,
		test_pos;

	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_MOBILE))
		{
	
			get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &pos2);
	
			test_pos.x = pos->x - pos2.x;
	
			test_pos.y = 0.0;
	
			test_pos.z = pos->z - pos2.z;
	
			range = get_3d_vector_magnitude (&test_pos);
	
			if (range < best_range)
			{
	
				best_range = range;
	
				closest_en = en;
	
				if (range < 1.0)
				{
				
					return closest_en;
				}
			}
		}

		en = get_local_entity_succ (en);
	}
	
	return closest_en;
}





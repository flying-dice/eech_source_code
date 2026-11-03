

































































#include "project.h"





#define DEBUG_MODULE 									0
#define MIN_CRATE_DROPPING_RANGE 					(100 * METRE)





void update_fixed_wing_crate_dropping (entity *en)
{

	entity
		*current_waypoint,
		*crate;

	vec3d
		*crate_pos,
		*en_pos;

	float
		range;

	current_waypoint = get_local_entity_current_waypoint (en);

	// only perform code if next waypoint is FINISH_DROP_OFF (ie, drop crates between DROP_OFF and FINISH_DROP_OFF)
	if ((current_waypoint) && (get_local_entity_int_value (current_waypoint, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF))
	{
	
		crate = get_local_entity_first_child (en, LIST_TYPE_LAUNCHED_WEAPON);
	
		if (crate)
		{
		
			crate_pos = get_local_entity_vec3d_ptr (crate, VEC3D_TYPE_POSITION);
			
			en_pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
		
			range = get_sqr_3d_range (crate_pos, en_pos);
		}
		
		if ((!crate) || (range > (MIN_CRATE_DROPPING_RANGE * MIN_CRATE_DROPPING_RANGE)))
		{
		
			if (get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON) == ENTITY_SUB_TYPE_WEAPON_CRATE)
			{
		
				if (aircraft_fire_weapon (en, AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY, FALSE) != AIRCRAFT_FIRE_OK)
				{
		
					//
					// go round again
					//
	





				}
				else
				{
	





				}
			}
		}
	}
}





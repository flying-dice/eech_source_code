

































































#include "project.h"





#define DEBUG_MODULE 0





void draw_debug_guide_entity (entity *en)
{

	waypoint
		*raw;

	entity
		*member,
		*this_waypoint;

	vec3d
		direction,
		guide_pos,
		*member_pos,
		*this_waypoint_position;

	float
		length;

	ASSERT (en);

	raw = (waypoint *) get_local_entity_data (en);

	//
	// get guide position
	//

	get_local_entity_vec3d (en, VEC3D_TYPE_GUIDE_POSITION, &guide_pos);

	//
	// get waypoint position
	//

	this_waypoint_position = NULL;

	this_waypoint = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (this_waypoint);

	//
	// draw guide position
	//

	direction.x = 0.0;
	direction.y = 1.0;
	direction.z = 0.0;
		
	create_vectored_debug_3d_object (&guide_pos, &direction, OBJECT_3D_SPHERICAL_TEST, 0.0, 1.0);

	if (get_local_entity_int_value (this_waypoint, INT_TYPE_POSITION_TYPE) != POSITION_TYPE_VIRTUAL)
	{
		this_waypoint_position = get_local_entity_vec3d_ptr (this_waypoint, VEC3D_TYPE_POSITION);

		length = get_3d_range (&guide_pos, this_waypoint_position);

		if (length > 0.0)
		{
			create_debug_3d_line (&guide_pos, this_waypoint_position, sys_col_yellow, 0.0);
		}
	}

	//
	// draw vector to group position
	//

	member = get_local_entity_first_child (en, LIST_TYPE_FOLLOWER);

	while (member)
	{
		member_pos = get_local_entity_vec3d_ptr (member, VEC3D_TYPE_POSITION);
		
		length = get_3d_range (&guide_pos, member_pos);

		if (length > 0.0)
		{
			create_debug_3d_line (&guide_pos, member_pos, sys_col_light_red, 0.0);
		}

		member = get_local_entity_child_succ (member, LIST_TYPE_FOLLOWER);
	}
}





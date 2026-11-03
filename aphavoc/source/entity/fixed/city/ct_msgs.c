

































































#include "project.h"





#define DEBUG_MODULE 0





static void update_city_bounding_box (entity *this_city, entity *this_building)
{
	city
		*raw;

	object_3d_index_numbers
		object_3d_index;

	object_3d_bounds
		*bounding_box;

	vec3d
		*building_position,
		position_offset,
		p1,
		p2;

	matrix3x3
		attitude;

	float
		heading;

	ASSERT (this_city);

	ASSERT (this_building);

	raw = (city *) get_local_entity_data (this_city);

	//
	// get building bounding box minmax points (assume on flat ground)
	//

	object_3d_index = get_local_entity_int_value (this_building, INT_TYPE_OBJECT_3D_SHAPE);

	ASSERT (object_3d_index != OBJECT_3D_INVALID_OBJECT_INDEX);

	bounding_box = get_object_3d_bounding_box_without_lines (object_3d_index);

	ASSERT (bounding_box);

	p1.x = bounding_box->xmin;
	p1.y = bounding_box->ymin;
	p1.z = bounding_box->zmin;

	p2.x = bounding_box->xmax;
	p2.y = bounding_box->ymax;
	p2.z = bounding_box->zmax;

	//
	// rotate minmax points (no longer minmax after rotate)
	//

	heading = get_local_entity_float_value (this_building, FLOAT_TYPE_HEADING);

	get_3d_transformation_matrix (attitude, heading, 0.0, 0.0);

	multiply_matrix3x3_vec3d (&p1, attitude, &p1);
	multiply_matrix3x3_vec3d (&p2, attitude, &p2);

	//
	// get building position offset from city centre
	//

	building_position = get_local_entity_vec3d_ptr (this_building, VEC3D_TYPE_POSITION);

	position_offset.x = building_position->x - raw->fix.position.x;
	position_offset.y = building_position->y - raw->fix.position.y;
	position_offset.z = building_position->z - raw->fix.position.z;

	//
	// offset p1 and p2
	//

	p1.x += position_offset.x;
	p1.y += position_offset.y;
	p1.z += position_offset.z;

	p2.x += position_offset.x;
	p2.y += position_offset.y;
	p2.z += position_offset.z;

	//
	// adjust city bounding box
	//

	if (p1.x < raw->bounding_box_min.x) raw->bounding_box_min.x = p1.x;
	if (p1.y < raw->bounding_box_min.y) raw->bounding_box_min.y = p1.y;
	if (p1.z < raw->bounding_box_min.z) raw->bounding_box_min.z = p1.z;
	if (p1.x > raw->bounding_box_max.x) raw->bounding_box_max.x = p1.x;
	if (p1.y > raw->bounding_box_max.y) raw->bounding_box_max.y = p1.y;
	if (p1.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p1.z;

	if (p2.x < raw->bounding_box_min.x) raw->bounding_box_min.x = p2.x;
	if (p2.y < raw->bounding_box_min.y) raw->bounding_box_min.y = p2.y;
	if (p2.z < raw->bounding_box_min.z) raw->bounding_box_min.z = p2.z;
	if (p2.x > raw->bounding_box_max.x) raw->bounding_box_max.x = p2.x;
	if (p2.y > raw->bounding_box_max.y) raw->bounding_box_max.y = p2.y;
	if (p2.z > raw->bounding_box_max.z) raw->bounding_box_max.z = p2.z;

	ASSERT (raw->bounding_box_min.x <= raw->bounding_box_max.x);
	ASSERT (raw->bounding_box_min.y <= raw->bounding_box_max.y);
	ASSERT (raw->bounding_box_min.z <= raw->bounding_box_max.z);
}





static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	list_types
		list_type;

	#if DEBUG_MODULE

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	list_type = va_arg (pargs, list_types);

	switch (list_type)
	{

		case LIST_TYPE_CITY_BUILDING:

		{
			update_city_bounding_box (receiver, sender);

			break;
		}
	}

	return (TRUE);
}





#if DEBUG_MODULE

static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





#if DEBUG_MODULE

static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif





void overload_city_message_responses (void)
{
	overload_fixed_message_responses (ENTITY_TYPE_CITY);

	message_responses[ENTITY_TYPE_CITY][ENTITY_MESSAGE_LINK_CHILD]		= response_to_link_child;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_CITY][ENTITY_MESSAGE_UNLINK_CHILD]	= response_to_unlink_child;

	message_responses[ENTITY_TYPE_CITY][ENTITY_MESSAGE_LINK_PARENT]	= response_to_link_parent;

	message_responses[ENTITY_TYPE_CITY][ENTITY_MESSAGE_UNLINK_PARENT]	= response_to_unlink_parent;

	#endif
}







































































#include "project.h"





#define DEBUG_MODULE 0





static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	entity
		*enemy_force,
		*allied_force;

	sector
		*raw;

	entity_sides
		side;

	list_types
		list_type;

	session_list_types
		session_type;







	raw = (sector *) get_local_entity_data (receiver);

	list_type = va_arg (pargs, list_types);



	if (list_type == LIST_TYPE_SECTOR)
	{
		float
			height;

		vec3d
			*position;

		object_3d_index_numbers
			object_3d_index;

		object_3d_bounds
			*bounding_box;

		//
		// check for tallest fixed structure
		//

		if (get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED))
		{
			object_3d_index = get_local_entity_int_value (sender, INT_TYPE_OBJECT_3D_SHAPE);

			if (object_3d_index != OBJECT_3D_INVALID_OBJECT_INDEX)
			{
				bounding_box = get_object_3d_bounding_box (object_3d_index);

	 			position = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);

				height = position->y + bounding_box->ymax;

				if (height > raw->tallest_structure_height)
				{
					raw->tallest_structure_height = height;
				}
			}
		}

		//
		// set defence levels / activity array
		//

		add_mobile_values_to_sector (receiver, sender);

		//
		// set side awareness + notify force
		//

		if
		(
			(get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_AIRCRAFT)) ||
			(get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_VEHICLE))
		)
		{
			side = (entity_sides) get_local_entity_int_value (sender, INT_TYPE_SIDE);


			//
			// SIDE NOT NEUTRAL.......
			//


			if (side == ENTITY_SIDE_NEUTRAL)
			{
				return TRUE;
			}

			ASSERT (get_valid_current_game_session ());

			set_sector_fog_of_war_value (sender, receiver);


			//
			// ONLY IF GAME INITIALISED......
			//


			if (get_game_status () != GAME_STATUS_INITIALISED)
			{
				return TRUE;
			}


			//
			// SERVER ONLY...... 
			//


			if (get_comms_model () == COMMS_MODEL_CLIENT)
			{
				return TRUE;
			}

			session_type = get_current_game_session_type ();

			// sector occupation timer

			//
			// notify force
			//

			allied_force = get_local_force_entity (side);

			if (allied_force)
			{

				notify_local_entity (ENTITY_MESSAGE_FORCE_ENTERED_SECTOR, allied_force, sender, receiver);
			}

			enemy_force = get_local_force_entity (get_enemy_side (side));

			if (enemy_force)
			{

				notify_local_entity (ENTITY_MESSAGE_FORCE_ENTERED_SECTOR, enemy_force, sender, receiver);
			}
		}
	}

	return (TRUE);
}





static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{

	list_types
		list_type;







	list_type = va_arg( pargs, list_types );

	if ( list_type == LIST_TYPE_SECTOR )
	{
		remove_mobile_values_from_sector (receiver, sender);
	}

	return (TRUE);
}






















































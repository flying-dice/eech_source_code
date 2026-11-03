

































































#include "project.h"





#define DEBUG_MODULE 0





int set_initial_rotation_angle_of_anti_aircraft_wheels (object_3d_instance *inst3d)
{
	object_3d_sub_object_search_data
		search;

	int
		depth;

	float
		ang;

	//
	// vary wheel start positions (ok to use a random number as this is for visual effect only)
	//

	ASSERT (inst3d);

	//
	// fixed wheels
	//

	depth = 0;

	while (TRUE)
	{
		search.search_depth = depth;
		search.search_object = inst3d;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_FIXED_WHEEL;

		if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
		{
			ang = sfrand1() * PI;

			search.result_sub_object->relative_pitch = ang;
		}
		else
		{
			break;
		}

		depth++;
	}

	//
	// steerable wheels
	//

	depth = 0;

	while (TRUE)
	{
		search.search_depth = depth;
		search.search_object = inst3d;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_STEERABLE_WHEEL;

		if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
		{
			ang = sfrand1() * PI;

			search.result_sub_object->relative_pitch = ang;
		}
		else
		{
			break;
		}

		depth++;
	}

	return TRUE;
}





void damage_anti_aircraft_3d_object (entity *en)
{
	anti_aircraft
		*raw;

	ASSERT (en);

	raw = (anti_aircraft *) get_local_entity_data (en);

	set_sub_object_type_visible_status (raw->vh.inst3d, OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_READY, FALSE);

	if (get_local_entity_int_value (en, INT_TYPE_VIEW_TYPE) == VIEW_TYPE_INFANTRY_WITH_SAM) // replace MANPAD only
	{
		//
		// destruct old object
		//

		destruct_3d_object (raw->vh.inst3d);

		//
		// construct new (damaged) object
		//

		raw->vh.object_3d_shape = get_local_entity_int_value (en, INT_TYPE_DESTROYED_3D_SHAPE);

		raw->vh.inst3d = construct_3d_object (raw->vh.object_3d_shape);

		//
		// set id number for new object
		//

		set_anti_aircraft_id_number (en);
	}
}





void set_anti_aircraft_id_number (entity *en)
{
	anti_aircraft
		*raw;

	ASSERT (en);

	raw = (anti_aircraft *) get_local_entity_data (en);

	if (raw->vh.mob.side == ENTITY_SIDE_BLUE_FORCE)
	{
		set_local_entity_id_number
		(
			en,
			TEXTURE_ANIMATION_INDEX_USA_TANK_DIGIT__000,
			TEXTURE_ANIMATION_INDEX_USA_TANK_DIGIT__001,
			TEXTURE_ANIMATION_INDEX_USA_TANK_DIGIT__002,
			TEXTURE_ANIMATION_INDEX_LAST,
			TEXTURE_ANIMATION_INDEX_LAST
		);
	}
	else if (raw->vh.mob.side == ENTITY_SIDE_RED_FORCE)
	{
		set_local_entity_id_number
		(
			en,
			TEXTURE_ANIMATION_INDEX_CIS_TANK_DIGIT__000,
			TEXTURE_ANIMATION_INDEX_CIS_TANK_DIGIT__001,
			TEXTURE_ANIMATION_INDEX_CIS_TANK_DIGIT__002,
			TEXTURE_ANIMATION_INDEX_LAST,
			TEXTURE_ANIMATION_INDEX_LAST
		);
	}
}





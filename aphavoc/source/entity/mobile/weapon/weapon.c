

































































#include "project.h"





#define DEBUG_MODULE									0

#define DEBUG_WEAPON_VECTOR	0

#define DEBUG_MODULE_EFFECTIVE_SHIP_WEAPONS	0




























void load_local_entity_weapon_config (entity *en)
{
	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	int
		package;

	ASSERT (en);

	package_status = (weapon_package_status *) get_local_entity_ptr_value (en, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

	if (package_status)
	{
		memset (package_status, 0, SIZE_WEAPON_PACKAGE_STATUS_ARRAY);

		config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

		ASSERT (weapon_config_type_valid (config_type));







		for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
		{
			if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				package_status[package].number = weapon_config_database[config_type][package].number;
				package_status[package].rearming_timer = 0;
				package_status[package].damaged = FALSE;
			}
		}
	}


	//

	if (!get_local_entity_weapon_available (en, get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON)))
	{
		#if 0
		{
			entity_sub_types
				weapon_sub_type;

			weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_DEFAULT_WEAPON_TYPE);

			if (!get_local_entity_weapon_available (en, weapon_sub_type))
			{
				weapon_sub_type = get_first_available_weapon_sub_type (en);
			}

			set_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON, weapon_sub_type);
		}
		#else
		{
			set_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON, ENTITY_SUB_TYPE_WEAPON_NO_WEAPON);
		}
		#endif
	}

	//


	reset_entity_weapon_config_animation (en);

	if (en == get_gunship_entity ())
	{
		repair_gunship_weapon_damage ();
	}


	//
	// set Comanche stub wing visibility (not a very elegant solution)
	//


	set_comanche_stub_wing_visibility (en);
}














































int get_local_entity_weapon_available (entity *launcher, entity_sub_types weapon_sub_type)
{
	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	int
		package,
		number = 0;

	ASSERT (launcher);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	if (weapon_sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		package_status = (weapon_package_status *) get_local_entity_ptr_value (launcher, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

		if (package_status)
		{
			config_type = (weapon_config_types) get_local_entity_int_value (launcher, INT_TYPE_WEAPON_CONFIG_TYPE);

			ASSERT (weapon_config_type_valid (config_type));

			for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
			{
				if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
				{
					ASSERT (package_status[package].number <= weapon_config_database[config_type][package].number);

					if (weapon_config_database[config_type][package].sub_type == weapon_sub_type)
					{
						if (!package_status[package].damaged)
						{
							number += package_status[package].number;
						}
					}
				}
			}
		}
	}

	return (number);
}





























































































































































































































































































































































































































//
// NOTE: KEEP THIS FUNCTION IN SYNC WITH THE SIMILAR FUNCTION IN WN_MOVE.C
//
























































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































































int get_local_entity_weapon_ammunition_available (entity *launcher)
{
	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	int
		package;

	entity_sub_types
		weapon_type;

	ASSERT (launcher);

	//
	// get list of weapons available on the launcher
	//

	package_status = (weapon_package_status *) get_local_entity_ptr_value (launcher, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

	if (!package_status)
	{
		return FALSE;
	}

	config_type = (weapon_config_types) get_local_entity_int_value (launcher, INT_TYPE_WEAPON_CONFIG_TYPE);

	ASSERT (weapon_config_type_valid (config_type));

	for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
	{
		weapon_type = weapon_config_database[config_type][package].sub_type;

		//
		// check package is valid
		//

		if (weapon_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			ASSERT (package_status[package].number <= weapon_config_database[config_type][package].number);

			//
			// check package is not damaged
			//

			if (!package_status[package].damaged)
			{
				//
				// check package has rounds left
				//

				if (package_status[package].number > 0)
				{
					return TRUE;
				}
			}
		}
	}

	return FALSE;
}





int get_local_entity_weapon_salvo_size (entity *launcher, entity_sub_types weapon_sub_type)
{
	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	int
		package;

	ASSERT (launcher);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	if (weapon_sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		package_status = (weapon_package_status *) get_local_entity_ptr_value (launcher, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

		if (package_status)
		{
			config_type = (weapon_config_types) get_local_entity_int_value (launcher, INT_TYPE_WEAPON_CONFIG_TYPE);

			ASSERT (weapon_config_type_valid (config_type));

			for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
			{
				if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
				{
					if (weapon_config_database[config_type][package].sub_type == weapon_sub_type)
					{
						if (!package_status[package].damaged)
						{
							if (package_status[package].number > 0)
							{
								return (weapon_config_database[config_type][package].salvo_size);
							}
						}
					}
				}
			}
		}
	}

	return (1);
}






//
// WARNING: This function totally abuses the damage flag system and it is likely that the server and
//          clients will get out of sync but this should be ok as the detach function is fail safe.
//
// This function tags any of the ships selected weapon packages as damaged if they cannot see the
// target. This is an attempt to prevent the weapon systems from 'stalling' if the next available
// weapon cannot see the target.
//
// If none of the weapons can see the target then none of them are tagged as damaged. Also, if only
// once instance of the weapon package exists then this system is not invoked. This is to prevent
// the weapon type from being made unavailable and effecting other functions.
//
// This function assumes that the damage flags are not otherwise used.
//


void suppress_ineffective_ship_weapons (entity *source, entity *target)
{
	int
		package,
		num_effective_weapons;

	float
		dx,
		dz,
		heading,
		min_heading,
		max_heading,
		bearing,
		normalised_max_heading,
		normalised_bearing;

	vec3d
		target_position;

	object_3d_instance
		*inst3d;

	object_3d_sub_object_search_data
		search_weapon_system_heading,
		search_weapon_system_pitch;

	entity_sub_types
		selected_weapon;

	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	viewpoint
		vp;










	ASSERT (source);

	ASSERT(target);

	ASSERT (get_local_entity_int_value (source, INT_TYPE_IDENTIFY_SHIP_VEHICLE));

	// check valid package status

	package_status = (weapon_package_status *) get_local_entity_ptr_value (source, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

	if (!package_status)
	{
		return;
	}

	// get config type

	config_type = (weapon_config_types) get_local_entity_int_value (source, INT_TYPE_WEAPON_CONFIG_TYPE);

	ASSERT (weapon_config_type_valid (config_type));

	// set all packages to undamaged

	for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
		if (package_status[package].number && weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			package_status[package].damaged = FALSE;

	// check valid config type

	if (config_type == WEAPON_CONFIG_TYPE_UNARMED)
	{
		return;
	}

	// check valid selected weapon

	selected_weapon = get_local_entity_int_value (source, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (selected_weapon));

	get_local_entity_vec3d (target, VEC3D_TYPE_POSITION, &target_position);

	// update the source's 3D instance position and attitude (in case it has not been drawn yet)

	inst3d = (object_3d_instance *) get_local_entity_ptr_value (source, PTR_TYPE_INSTANCE_3D_OBJECT);

	ASSERT (inst3d);

	get_local_entity_vec3d (source, VEC3D_TYPE_POSITION, (vec3d *) &inst3d->vp.position);

	get_local_entity_attitude_matrix (source, inst3d->vp.attitude);

	// check each instance of the selected weapon to see if the target is within its view cone

	num_effective_weapons = 0;

	for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
	{
		if (package_status[package].number && weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			search_weapon_system_heading.search_depth = weapon_config_database[config_type][package].heading_depth;
			search_weapon_system_heading.search_object = inst3d;
			search_weapon_system_heading.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_HEADING;

			if (find_object_3d_sub_object (&search_weapon_system_heading) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
			{
				search_weapon_system_pitch.search_depth = weapon_config_database[config_type][package].pitch_depth;
				search_weapon_system_pitch.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_PITCH;

				if (find_object_3d_sub_object_from_sub_object (&search_weapon_system_heading, &search_weapon_system_pitch) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					//
					// get viewpoint of unrotated heading and pitch devices
					//

					search_weapon_system_heading.result_sub_object->relative_heading = 0.0;

					search_weapon_system_pitch.result_sub_object->relative_pitch = 0.0;

					get_3d_sub_object_world_viewpoint (&search_weapon_system_pitch, &vp);

					search_weapon_system_heading.result_sub_object->relative_heading = package_status[package].weapon_system_heading;

					search_weapon_system_pitch.result_sub_object->relative_pitch = -package_status[package].weapon_system_pitch;

					//
					// (keep angles in range 0 to 360 degrees)
					//

					heading = get_heading_from_attitude_matrix (vp.attitude);

					if (heading < 0.0)
					{
						heading += rad (360.0);
					}

					min_heading = heading + weapon_config_database[config_type][package].min_heading_limit;

					if (min_heading < 0.0)
					{
						min_heading += rad (360.0);
					}
					else if (min_heading > rad (360.0))
					{
						min_heading -= rad (360.0);
					}

					max_heading = heading + weapon_config_database[config_type][package].max_heading_limit;

					if (max_heading < 0.0)
					{
						max_heading += rad (360.0);
					}
					else if (max_heading > rad (360.0))
					{
						max_heading -= rad (360.0);
					}

					dx = target_position.x - vp.x;
					dz = target_position.z - vp.z;

					bearing = atan2 (dx, dz);

					if (bearing < 0.0)
					{
						bearing += rad (360.0);
					}

					normalised_max_heading = max_heading - min_heading;

					if (normalised_max_heading < 0.0)
					{
						normalised_max_heading += rad (360.0);
					}

					normalised_bearing = bearing - min_heading;

					if (normalised_bearing < 0.0)
					{
						normalised_bearing += rad (360.0);
					}

					if (normalised_bearing <= normalised_max_heading)
					{
						num_effective_weapons++;








					}
					else
					{
						package_status[package].damaged = TRUE;








					}














































				}
			}
		}
	}

	// if no effective weapons are available then tag them all as undamaged (so the weapon type is still available)

	if (num_effective_weapons == 0)
		for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
			if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
				package_status[package].damaged = FALSE;
}

































































































































int check_guided_missile_type_alive (entity *en)
{
	entity *launched_weapon;
	entity_sub_types selected_weapon;

	ASSERT(en);

	launched_weapon = get_local_entity_first_child (en, LIST_TYPE_LAUNCHED_WEAPON);
	selected_weapon = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	if (!launched_weapon || !selected_weapon || !weapon_database[selected_weapon].guidance_type)
		return FALSE;

	while (launched_weapon)
	{
		if (weapon_database[get_local_entity_int_value (launched_weapon, INT_TYPE_ENTITY_SUB_TYPE)].guidance_type &&
				get_local_entity_int_value (launched_weapon, INT_TYPE_ENTITY_SUB_TYPE) == selected_weapon)
		{
			return TRUE;
		}

		launched_weapon = get_local_entity_child_succ (launched_weapon, LIST_TYPE_LAUNCHED_WEAPON);
	}

	return FALSE;
}

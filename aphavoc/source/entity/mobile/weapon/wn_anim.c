

































































#include "project.h"





#define DEBUG_MODULE 0





void reset_entity_weapon_config_animation (entity *en)
{
	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	object_3d_instance
		*inst3d;

	object_3d_sub_object_search_data
		search_weapon_system_heading,
		search_weapon_system_pitch,
		search_weapon_system_weapon;

	int
		search_weapon_system_heading_depth,
		search_weapon_system_pitch_depth,
		search_weapon_system_weapon_depth,
		package;

	package_status = (weapon_package_status *) get_local_entity_ptr_value (en, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

	if (package_status)
	{
		config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

		ASSERT (weapon_config_type_valid (config_type));

		inst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);

		ASSERT (inst3d);


		//
		// disable all weapons
		//


		search_weapon_system_heading_depth = 0;

		while (TRUE)
		{
			search_weapon_system_heading.search_depth = search_weapon_system_heading_depth;
			search_weapon_system_heading.search_object = inst3d;
			search_weapon_system_heading.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_HEADING;

			if (find_object_3d_sub_object (&search_weapon_system_heading) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
			{
				search_weapon_system_pitch_depth = 0;

				while (TRUE)
				{
					search_weapon_system_pitch.search_depth = search_weapon_system_pitch_depth;
					search_weapon_system_pitch.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_PITCH;

					if (find_object_3d_sub_object_from_sub_object (&search_weapon_system_heading, &search_weapon_system_pitch) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
					{
						search_weapon_system_pitch.result_sub_object->visible_object = FALSE;

						search_weapon_system_weapon_depth = 0;

						while (TRUE)
						{
							search_weapon_system_weapon.search_depth = search_weapon_system_weapon_depth;
							search_weapon_system_weapon.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_WEAPON;

							if (find_object_3d_sub_object_from_sub_object (&search_weapon_system_pitch, &search_weapon_system_weapon) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
							{
								search_weapon_system_weapon.result_sub_object->visible_object = FALSE;
							}
							else
							{
								break;
							}

							search_weapon_system_weapon_depth++;
						}
					}
					else
					{
						break;
					}

					search_weapon_system_pitch_depth++;
				}
			}
			else
			{
				break;
			}

			search_weapon_system_heading_depth++;
		}


		//
		// enable weapons
		//


		for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
		{
			if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				ASSERT (package_status[package].number <= weapon_config_database[config_type][package].number);

				search_weapon_system_heading.search_depth = weapon_config_database[config_type][package].heading_depth;
				search_weapon_system_heading.search_object = inst3d;
				search_weapon_system_heading.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_HEADING;

				if (find_object_3d_sub_object (&search_weapon_system_heading) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					if (weapon_config_database[config_type][package].rotate)
					{
						search_weapon_system_heading.result_sub_object->relative_heading = package_status[package].weapon_system_heading;
					}

					search_weapon_system_pitch.search_depth = weapon_config_database[config_type][package].pitch_depth;
					search_weapon_system_pitch.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_PITCH;

					if (find_object_3d_sub_object_from_sub_object (&search_weapon_system_heading, &search_weapon_system_pitch) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
					{
						if (weapon_config_database[config_type][package].rotate)
						{
							search_weapon_system_pitch.result_sub_object->relative_pitch = -package_status[package].weapon_system_pitch;
						}

						if (!(get_local_entity_type (en) == ENTITY_TYPE_PERSON && weapon_config_database[config_type][package].heading_depth != 0)) // hide troops secondary weapons
							search_weapon_system_pitch.result_sub_object->visible_object = TRUE;

						search_weapon_system_weapon_depth = weapon_config_database[config_type][package].number - package_status[package].number;

						while (TRUE)
						{
							search_weapon_system_weapon.search_depth = search_weapon_system_weapon_depth;
							search_weapon_system_weapon.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_WEAPON;

							if (find_object_3d_sub_object_from_sub_object (&search_weapon_system_pitch, &search_weapon_system_weapon) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
							{
								search_weapon_system_weapon.result_sub_object->visible_object = TRUE;
							}
							else
							{
								break;
							}

							search_weapon_system_weapon_depth++;
						}
					}
				}
			}
		}
	}
}





void update_weapon_animation (entity *en)
{
	weapon
		*raw;

	ASSERT (en);

	raw = (weapon *) get_local_entity_data (en);

	ASSERT (entity_sub_type_weapon_valid (raw->mob.sub_type));

	switch (raw->mob.sub_type)
	{
		case ENTITY_SUB_TYPE_WEAPON_HOKUM_PILOT:
		case ENTITY_SUB_TYPE_WEAPON_HOKUM_CO_PILOT:
		{
			update_weapon_ejector_seat_animation (raw);

			break;
		}
		case ENTITY_SUB_TYPE_WEAPON_CRATE:
		{
			update_weapon_crate_animation (raw);

			break;
		}
	}
}





void update_infantry_weapons (entity *en, entity_sub_types weapon_sub_type)
{
	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	object_3d_instance
		*inst3d;

	object_3d_sub_object_search_data
		search_weapon_system_heading,
		search_weapon_system_pitch;

	int
		package;

	package_status = (weapon_package_status *) get_local_entity_ptr_value (en, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

	if (package_status)
	{
		config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

		ASSERT (weapon_config_type_valid (config_type));

		inst3d = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);

		for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
			if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				search_weapon_system_heading.search_depth = weapon_config_database[config_type][package].heading_depth;
				search_weapon_system_heading.search_object = inst3d;
				search_weapon_system_heading.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_HEADING;

				if (find_object_3d_sub_object (&search_weapon_system_heading) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
				{
					search_weapon_system_heading.result_sub_object->visible_object = weapon_config_database[config_type][package].sub_type == weapon_sub_type;
					
					search_weapon_system_pitch.search_depth = weapon_config_database[config_type][package].pitch_depth;
					search_weapon_system_pitch.sub_object_index = OBJECT_3D_SUB_OBJECT_WEAPON_SYSTEM_PITCH;

					if (find_object_3d_sub_object_from_sub_object (&search_weapon_system_heading, &search_weapon_system_pitch) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
						search_weapon_system_pitch.result_sub_object->visible_object = weapon_config_database[config_type][package].sub_type == weapon_sub_type;
				}
			}
	}
}

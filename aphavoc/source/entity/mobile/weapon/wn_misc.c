

































































#include "project.h"





#define DEBUG_MODULE 0





void get_local_entity_weapon_load(entity* en, weapon_count weapon_load[], unsigned max_num_weapons)
{
	weapon_package_status
		*package_status;

	unsigned next_free = 0;

	ASSERT(en);
	ASSERT(weapon_load);
	ASSERT(max_num_weapons > 0);

	package_status = (weapon_package_status *) get_local_entity_ptr_value(en, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

	if (package_status)
	{
		int package;
		weapon_config_types config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

		for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
		{
			entity_sub_types weapon_type;
			int number;

			if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
			{
				if (!package_status[package].damaged)
				{
					unsigned i;

					weapon_type = weapon_config_database[config_type][package].sub_type;
					number = package_status[package].number;

					for (i=0; i < next_free; i++)
					{
						if (weapon_load[i].weapon_type == weapon_type)
						{
							weapon_load[i].count += number;
							break;
						}
					}

					if (i == next_free && next_free <= max_num_weapons-1 && weapon_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)  // weapon not in array already
					{
						next_free++;
						weapon_load[i].weapon_type = weapon_type;
						weapon_load[i].count = number;
					}
				}
			}
		}
	}

	weapon_load[next_free].weapon_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;
}


















































































int get_local_entity_weapon_count (entity *en, entity_sub_types weapon_sub_type)
{
	int
		weapon_count,
		package;

	weapon_package_status
		*package_status;

	weapon_config_types
		config_type;

	ASSERT (en);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	weapon_count = 0;

	if (weapon_sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
	{
		package_status = (weapon_package_status *) get_local_entity_ptr_value (en, PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY);

		if (package_status)
		{
			config_type = (weapon_config_types) get_local_entity_int_value (en, INT_TYPE_WEAPON_CONFIG_TYPE);

			for (package = 0; package < NUM_WEAPON_PACKAGES; package++)
			{
				if (weapon_config_database[config_type][package].sub_type != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
				{
					if (weapon_config_database[config_type][package].sub_type == weapon_sub_type)
					{
						if (!package_status[package].damaged)
						{
							weapon_count += package_status[package].number;
						}
					}
				}
			}
		}
	}

	return (weapon_count);
}
































































































































































































































































































































































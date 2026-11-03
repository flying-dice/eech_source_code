

































































#include "project.h"





#define DEBUG_MODULE 0




//
// APACHE WEAPON SELECTION
//




#define NUM_APACHE_WEAPON_TYPES	(6)





static entity_sub_types get_apache_next_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		next_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	next_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_APACHE_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			next_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (next_weapon_sub_type);
}





static entity_sub_types get_apache_previous_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		previous_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	previous_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_APACHE_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			previous_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (previous_weapon_sub_type);
}




//
// HAVOC WEAPON SELECTION
//




#define NUM_HAVOC_WEAPON_TYPES	(7)





static entity_sub_types get_havoc_next_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		next_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	next_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_HAVOC_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9M120_ATAKA_V;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9M120_ATAKA_V:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S8;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S8:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S13;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S13:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			next_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (next_weapon_sub_type);
}





static entity_sub_types get_havoc_previous_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		previous_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	previous_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_HAVOC_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9M120_ATAKA_V:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S8:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9M120_ATAKA_V;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S13:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S8;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S13;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			previous_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (previous_weapon_sub_type);
}




//
// COMANCHE WEAPON SELECTION
//




#define NUM_COMANCHE_WEAPON_TYPES	(6)





static entity_sub_types get_comanche_next_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		next_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	next_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_COMANCHE_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			next_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (next_weapon_sub_type);
}





static entity_sub_types get_comanche_previous_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		previous_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	previous_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_COMANCHE_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			previous_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (previous_weapon_sub_type);
}




//
// HOKUM WEAPON SELECTION
//




#define NUM_HOKUM_WEAPON_TYPES	(7)





static entity_sub_types get_hokum_next_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		next_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	next_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_HOKUM_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9K121_VIKHR;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9K121_VIKHR:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S8;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S8:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S13;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S13:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			next_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (next_weapon_sub_type);
}





static entity_sub_types get_hokum_previous_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type,
		previous_weapon_sub_type;

	int
		attempts;

	ASSERT (en);

	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	previous_weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

	attempts = NUM_HOKUM_WEAPON_TYPES;

	while (attempts)
	{
		switch (weapon_sub_type)
		{

			case ENTITY_SUB_TYPE_WEAPON_NO_WEAPON:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_9K121_VIKHR:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S8:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_9K121_VIKHR;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_S13:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S8;

				break;
			}

			case ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_S13;

				break;
			}

			default:

			{
				weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND;

				break;
			}
		}

		if (get_local_entity_weapon_available (en, weapon_sub_type))
		{
			previous_weapon_sub_type = weapon_sub_type;

			break;
		}

		attempts--;
	}

	return (previous_weapon_sub_type);
}




//
// GENERAL WEAPON SELECTION
//




entity_sub_types get_first_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type;

	ASSERT (en);

	for (weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON + 1; weapon_sub_type < NUM_ENTITY_SUB_TYPE_WEAPONS; weapon_sub_type++)
	{
		if (!(weapon_database[weapon_sub_type].weapon_class & (WEAPON_CLASS_DECOY | WEAPON_CLASS_CARGO | WEAPON_CLASS_DEBRIS)))
		{
			if (get_local_entity_weapon_available (en, weapon_sub_type))
			{
				return (weapon_sub_type);
			}
		}
	}

	return (ENTITY_SUB_TYPE_WEAPON_NO_WEAPON);
}





entity_sub_types get_next_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type;

	int
		i;

	ASSERT (en);


	//
	// customised weapon selection for featured helicopters
	//


	if (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_HELICOPTER))
	{
		switch (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE))
		{

			case ENTITY_SUB_TYPE_AIRCRAFT_AH64D_APACHE_LONGBOW:

			{
				weapon_sub_type = get_apache_next_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}

			case ENTITY_SUB_TYPE_AIRCRAFT_MI28N_HAVOC_B:

			{
				weapon_sub_type = get_havoc_next_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}

			case ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE:

			{
				weapon_sub_type = get_comanche_next_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}

			case ENTITY_SUB_TYPE_AIRCRAFT_KA52_HOKUM_B:

			{
				weapon_sub_type = get_hokum_next_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}
		}
	}


	//
	// general weapon selection
	//


	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	for (i = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON + 1; i < NUM_ENTITY_SUB_TYPE_WEAPONS; i++)
	{
		weapon_sub_type++;

		if (weapon_sub_type == NUM_ENTITY_SUB_TYPE_WEAPONS)
		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON + 1;
		}

		if (!(weapon_database[weapon_sub_type].weapon_class & (WEAPON_CLASS_DECOY | WEAPON_CLASS_CARGO | WEAPON_CLASS_DEBRIS)))
		{
			if (get_local_entity_weapon_available (en, weapon_sub_type))
			{
				return (weapon_sub_type);
			}
		}
	}

	return (ENTITY_SUB_TYPE_WEAPON_NO_WEAPON);
}





entity_sub_types get_previous_available_weapon_sub_type (entity *en)
{
	entity_sub_types
		weapon_sub_type;

	int
		i;

	ASSERT (en);


	//
	// customised weapon selection for featured helicopters
	//


	if (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_HELICOPTER))
	{
		switch (get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE))
		{

			case ENTITY_SUB_TYPE_AIRCRAFT_AH64D_APACHE_LONGBOW:

			{
				weapon_sub_type = get_apache_previous_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}

			case ENTITY_SUB_TYPE_AIRCRAFT_MI28N_HAVOC_B:

			{
				weapon_sub_type = get_havoc_previous_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}

			case ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE:

			{
				weapon_sub_type = get_comanche_previous_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}

			case ENTITY_SUB_TYPE_AIRCRAFT_KA52_HOKUM_B:

			{
				weapon_sub_type = get_hokum_previous_available_weapon_sub_type (en);

				return (weapon_sub_type);

				break;
			}
		}
	}


	//
	// general weapon selection
	//


	weapon_sub_type = get_local_entity_int_value (en, INT_TYPE_SELECTED_WEAPON);

	ASSERT (entity_sub_type_weapon_valid (weapon_sub_type));

	for (i = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON + 1; i < NUM_ENTITY_SUB_TYPE_WEAPONS; i++)
	{
		weapon_sub_type--;

		if (weapon_sub_type <= ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			weapon_sub_type = NUM_ENTITY_SUB_TYPE_WEAPONS - 1;
		}

		if (!(weapon_database[weapon_sub_type].weapon_class & (WEAPON_CLASS_DECOY | WEAPON_CLASS_CARGO | WEAPON_CLASS_DEBRIS)))
		{
			if (get_local_entity_weapon_available (en, weapon_sub_type))
			{
				return (weapon_sub_type);
			}
		}
	}

	return (ENTITY_SUB_TYPE_WEAPON_NO_WEAPON);
}





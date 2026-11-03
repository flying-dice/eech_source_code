

































































#include "project.h"





#define DEBUG_MODULE 0





const division_data
	division_database[NUM_ENTITY_SUB_TYPE_DIVISIONS] =
	{

		//
		// ENTITY_SUB_TYPE_DIVISION_AIRBORNE_HELICOPTER_DIVISION
		//

		{
			"%s Airborne Division",
			NUM_ENTITY_SUB_TYPE_DIVISIONS,								// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_AIRBORNE_FIXED_WING_DIVISION
		//

		{
			"%s Airborne Division",
			NUM_ENTITY_SUB_TYPE_DIVISIONS,								// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_AIRBORNE_TRANSPORT_DIVISION
		//

		{
			"%s Airborne Division",
			NUM_ENTITY_SUB_TYPE_DIVISIONS,								// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_ARMOURED_DIVISION
		//

		{
			"%s Armoured Division",
			NUM_ENTITY_SUB_TYPE_DIVISIONS,								// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_CARRIER_DIVISION
		//

		{
			"%s Naval Division",
			NUM_ENTITY_SUB_TYPE_DIVISIONS,								// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_INFANTRY_DIVISION
		//

		{
			"%s Infantry Division",
			NUM_ENTITY_SUB_TYPE_DIVISIONS,								// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_HC_ATTACK_COMPANY
		//

		{
			"%s Aviation Group",
			ENTITY_SUB_TYPE_DIVISION_AIRBORNE_HELICOPTER_DIVISION,	// default_group_division
			0,																			// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_HC_TRANSPORT_COMPANY
		//

		{
			"%s Transport Group",
			ENTITY_SUB_TYPE_DIVISION_AIRBORNE_TRANSPORT_DIVISION,		// default_group_division
			0,																			// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_FW_ATTACK_COMPANY
		//

		{
			"%s Attack Squadron",
			ENTITY_SUB_TYPE_DIVISION_AIRBORNE_FIXED_WING_DIVISION,	// default_group_division
			0,																			// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_FW_FIGHTER_COMPANY
		//

		{
			"%s Fighter Squadron",
			ENTITY_SUB_TYPE_DIVISION_AIRBORNE_FIXED_WING_DIVISION,	// default_group_division
			0,																			// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_FW_TRANSPORT_COMPANY
		//

		{
			"%s Transport Squadron",
			ENTITY_SUB_TYPE_DIVISION_AIRBORNE_TRANSPORT_DIVISION,		// default_group_division
			0,																			// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_ADA_COMPANY
		//

		{
			"%s SAM Brigade ADA",
			ENTITY_SUB_TYPE_DIVISION_ARMOURED_DIVISION,				// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_ARMOURED_COMPANY
		//

		{
			"%s Armoured Brigade",
			ENTITY_SUB_TYPE_DIVISION_ARMOURED_DIVISION,				// default_group_division
			4,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_ARTILLERY_COMPANY
		//

		{
			"%s Artillery Brigade",
			ENTITY_SUB_TYPE_DIVISION_ARMOURED_DIVISION,				// default_group_division
			2,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_CARRIER_COMPANY
		//

		{
			"%s Carrier Task Force",
			ENTITY_SUB_TYPE_DIVISION_CARRIER_DIVISION,				// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_INFANTRY_COMPANY
		//

		{
			"%s Infantry Brigade",
			ENTITY_SUB_TYPE_DIVISION_INFANTRY_DIVISION,				// default_group_division
			0,																		// maximum_groups_per_division
		},

		//
		// ENTITY_SUB_TYPE_DIVISION_SPECIAL_FORCES_COMPANY
		//

		{
			"%s Special Forces Brigade",
			ENTITY_SUB_TYPE_DIVISION_INFANTRY_DIVISION,				// default_group_division
			0,																		// maximum_groups_per_division
		},
	};





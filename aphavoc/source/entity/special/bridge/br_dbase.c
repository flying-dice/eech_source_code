

































































#include "project.h"





#define DEBUG_MODULE 0





bridge_data
	bridge_database[NUM_ENTITY_SUB_TYPE_BRIDGES];





void initialise_bridge_database()
{
	bridge_data
		*bridge_info;

	int
		loop;


	//
	// ENTITY_SUB_TYPE_BRIDGE_BOAT
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_BOAT];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= BOAT_BRIDGE_START;
	bridge_info->end_section_type								= BOAT_BRIDGE_END;

	bridge_info->default_leg_height							= 5.0;

	bridge_info->min_suitable_span							= 0.0;
	bridge_info->max_suitable_span							= 300.0;

	bridge_info->min_suitable_depth							= 0.0;
	bridge_info->max_suitable_depth							= 12.0;

	bridge_info->max_suitable_slope							= rad (5);

	bridge_info->num_elements_in_mid_section_pattern	= 3;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]						= BOAT_BRIDGE_DECK;
	bridge_info->mid_section_type [1]						= BOAT_BRIDGE_MID;
	bridge_info->mid_section_type [2]						= BOAT_BRIDGE_DECK;


	//
	// ENTITY_SUB_TYPE_BRIDGE_CAUSEWAY
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_CAUSEWAY];

	//

	bridge_info->valid_bridge_type							= FALSE;

	bridge_info->start_section_type							= CAUSEWAY_BRIDGE_END;
	bridge_info->end_section_type								= CAUSEWAY_BRIDGE_START;

	bridge_info->default_leg_height							= 500.0;

	bridge_info->min_suitable_span							= 250.0;
	bridge_info->max_suitable_span							= FLT_MAX;

	bridge_info->min_suitable_depth							= 0.0;
	bridge_info->max_suitable_depth							= 20.0;

	bridge_info->max_suitable_slope							= rad (2.0);

	bridge_info->num_elements_in_mid_section_pattern	= 9;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]						= CAUSEWAY_BRIDGE_MID;
	bridge_info->mid_section_type [1]						= CAUSEWAY_BRIDGE_MID;
	bridge_info->mid_section_type [2]						= CAUSEWAY_BRIDGE_END;

	bridge_info->mid_section_type [3]						= STONE_BRIDGE_MID;
	bridge_info->mid_section_type [4]						= STONE_BRIDGE_MID;
	bridge_info->mid_section_type [5]						= STONE_BRIDGE_MID;

	bridge_info->mid_section_type [6]						= CAUSEWAY_BRIDGE_START;
	bridge_info->mid_section_type [7]						= CAUSEWAY_BRIDGE_MID;
	bridge_info->mid_section_type [8]						= CAUSEWAY_BRIDGE_MID;


	//
	// ENTITY_SUB_TYPE_BRIDGE_CONCRETE_ARCH
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_CONCRETE_ARCH];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= CONCRETE_WIDE_BRIDGE_START;
	bridge_info->end_section_type								= CONCRETE_WIDE_BRIDGE_END;

	bridge_info->default_leg_height							= 500.0;

	bridge_info->min_suitable_span							= 250.0;
	bridge_info->max_suitable_span							= FLT_MAX;

	bridge_info->min_suitable_depth							= 0.0;
	bridge_info->max_suitable_depth							= FLT_MAX;

	bridge_info->max_suitable_slope							= rad (15);

	bridge_info->num_elements_in_mid_section_pattern	= 16;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]						= CONCRETE_WIDE_BRIDGE_DECK;

	bridge_info->mid_section_type [1]						= CONCRETE_ARCH_BRIDGE_START;

	for ( loop = 2 ; loop <= 13 ; loop ++ )
	{
		bridge_info->mid_section_type [loop]				= CONCRETE_ARCH_BRIDGE_DECK;
	}

	bridge_info->mid_section_type [14]						= CONCRETE_ARCH_BRIDGE_END;

	bridge_info->mid_section_type [15]						= CONCRETE_WIDE_BRIDGE_DECK;


	//
	// ENTITY_SUB_TYPE_BRIDGE_CONCRETE_WIDE
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_CONCRETE_WIDE];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= CONCRETE_WIDE_BRIDGE_START;
	bridge_info->end_section_type								= CONCRETE_WIDE_BRIDGE_END;

	bridge_info->min_suitable_span							= 0.0;
	bridge_info->max_suitable_span							= 300.0;

	bridge_info->min_suitable_depth							= 0.0;
	bridge_info->max_suitable_depth							= 20.0;

	bridge_info->max_suitable_slope							= rad (90);

	bridge_info->default_leg_height							= 500.0;

	bridge_info->num_elements_in_mid_section_pattern	= 1;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]						= CONCRETE_WIDE_BRIDGE_DECK;


	//
	// ENTITY_SUB_TYPE_BRIDGE_DESERT
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_DESERT];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= DESERT_BRIDGE_START;
	bridge_info->end_section_type								= DESERT_BRIDGE_END;

	bridge_info->default_leg_height							= 500.0;

	bridge_info->min_suitable_span							= 0.0;
	bridge_info->max_suitable_span							= 500.0;

	bridge_info->min_suitable_depth							= 10.0;
	bridge_info->max_suitable_depth							= 30.0;

	bridge_info->max_suitable_slope							= rad (20);

	bridge_info->num_elements_in_mid_section_pattern	= 1;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]						= DESERT_BRIDGE_MID;


	//
	// ENTITY_SUB_TYPE_BRIDGE_GIRDER
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_GIRDER];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= GIRDER_BRIDGE_START;
	bridge_info->end_section_type								= GIRDER_BRIDGE_END;

	bridge_info->default_leg_height							= 500.0;

	bridge_info->min_suitable_span							= 250.0;
	bridge_info->max_suitable_span							= FLT_MAX;

	bridge_info->min_suitable_depth							= 0.0;
	bridge_info->max_suitable_depth							= FLT_MAX;

	bridge_info->max_suitable_slope							= rad (90);

	bridge_info->num_elements_in_mid_section_pattern	= 3;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]								= GIRDER_BRIDGE_DECK;
	bridge_info->mid_section_type [1]								= GIRDER_BRIDGE_MID;
	bridge_info->mid_section_type [2]								= GIRDER_BRIDGE_DECK;


	//
	// ENTITY_SUB_TYPE_BRIDGE_STONE
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_STONE];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= STONE_BRIDGE_START;
	bridge_info->end_section_type								= STONE_BRIDGE_END;

	bridge_info->default_leg_height							= 500.0;

	bridge_info->min_suitable_span							= 0.0;
	bridge_info->max_suitable_span							= 500.0;

	bridge_info->min_suitable_depth							= 10.0;
	bridge_info->max_suitable_depth							= 30.0;

	bridge_info->max_suitable_slope							= rad (20);

	bridge_info->num_elements_in_mid_section_pattern	= 1;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	bridge_info->mid_section_type [0]						= STONE_BRIDGE_MID;


	//
	// ENTITY_SUB_TYPE_BRIDGE_SUSPENSION
	//


	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_SUSPENSION];

	//

	bridge_info->valid_bridge_type							= TRUE;

	bridge_info->start_section_type							= SUSPENSION_BRIDGE_START;
	bridge_info->end_section_type								= SUSPENSION_BRIDGE_END;

	bridge_info->default_leg_height							= 500.0;

	bridge_info->min_suitable_span							= 200.0;
	bridge_info->max_suitable_span							= FLT_MAX;

	bridge_info->min_suitable_depth							= 10.0;
	bridge_info->max_suitable_depth							= FLT_MAX;

	bridge_info->max_suitable_slope							= rad (10);

	bridge_info->num_elements_in_mid_section_pattern	= 20;

	bridge_info->mid_section_type = (bridge_segment_types *) malloc_fast_mem (sizeof (int) * bridge_info->num_elements_in_mid_section_pattern);

	//

	for ( loop = 0 ; loop <= 9 ; loop ++ )
	{
		bridge_info->mid_section_type [loop]						= SUSPENSION_BRIDGE_DECK;
	}

	bridge_info->mid_section_type [10]								= SUSPENSION_BRIDGE_MID;

	for ( loop = 11 ; loop <= 19 ; loop ++ )
	{
		bridge_info->mid_section_type [loop]						= SUSPENSION_BRIDGE_DECK;
	}
}





void deinitialise_bridge_database()
{
	bridge_data
		*bridge_info;

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_BOAT];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_CAUSEWAY];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_CONCRETE_ARCH];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_CONCRETE_WIDE];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_DESERT];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_GIRDER];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_STONE];

	free_mem (bridge_info->mid_section_type);

	//

	bridge_info = &bridge_database [ENTITY_SUB_TYPE_BRIDGE_SUSPENSION];

	free_mem (bridge_info->mid_section_type);

	//
}





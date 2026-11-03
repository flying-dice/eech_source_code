

































































#include "project.h"





#define DEBUG_MODULE 0





bridge_segment_data
	segment_database[NUM_BRIDGE_SEGMENT_TYPES];





void initialise_segment_database()
{
	bridge_segment_data
		*segment_info;


	//
	// BRIDGE_SEGMENT_TYPE_NONE
	//


	segment_info = &segment_database [BRIDGE_SEGMENT_TYPE_NONE];

	//

	segment_info->normal_object					= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->destroyed_object				= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->pred_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->succ_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->both_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 0.0;

	segment_info->initial_damage_level			= 0;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 1;


	//
	// BOAT_BRIDGE_START 
	//


	segment_info = &segment_database [BOAT_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_BOAT_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_BOAT_BRIDGE_START_DAMAGE2;
	segment_info->pred_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_START;
	segment_info->succ_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_START_DAMAGE_1;
	segment_info->both_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_START_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 20.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 1;


	//
	// BOAT_BRIDGE_END
	//


	segment_info = &segment_database [BOAT_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_BOAT_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_BOAT_BRIDGE_END_DAMAGE_2;
	segment_info->pred_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_END_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_END;
	segment_info->both_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_END_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 20.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 1;


	//
	// BOAT_BRIDGE_MID
	//


	segment_info = &segment_database [BOAT_BRIDGE_MID];

	//

	segment_info->normal_object					= OBJECT_3D_BOAT_BRIDGE_MID;
	segment_info->destroyed_object				= OBJECT_3D_BOAT_BRIDGE_MID_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_MID_DAMAGE_2;
	segment_info->succ_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_MID_DAMAGE_1;
	segment_info->both_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_MID_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 20.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 1;


	//
	// BOAT_BRIDGE_DECK
	//


	segment_info = &segment_database [BOAT_BRIDGE_DECK];

	//

	segment_info->normal_object					= OBJECT_3D_BOAT_BRIDGE_DECK;
	segment_info->destroyed_object				= OBJECT_3D_BOAT_BRIDGE_DECK_DAMAGE;
	segment_info->pred_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_DECK_DAMAGE;
	segment_info->succ_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_DECK_DAMAGE;
	segment_info->both_destroyed_object			= OBJECT_3D_BOAT_BRIDGE_DECK_DAMAGE;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION;

	segment_info->length								= 20.0;

	segment_info->initial_damage_level			= 400;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 1;


	//
	// CAUSEWAY_BRIDGE_START
	//


	segment_info = &segment_database [CAUSEWAY_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->destroyed_object				= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->pred_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->succ_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->both_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 0.0;

	segment_info->initial_damage_level			= 0;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// CAUSEWAY_BRIDGE_END
	//


	segment_info = &segment_database [CAUSEWAY_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->destroyed_object				= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->pred_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->succ_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->both_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 0.0;

	segment_info->initial_damage_level			= 0;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// CAUSEWAY_BRIDGE_MID
	//


	segment_info = &segment_database [CAUSEWAY_BRIDGE_MID];

	//

	segment_info->normal_object					= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->destroyed_object				= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->pred_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->succ_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;
	segment_info->both_destroyed_object			= OBJECT_3D_INVALID_OBJECT_INDEX;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 0.0;

	segment_info->initial_damage_level			= 0;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// CONCRETE_ARCH_BRIDGE_START
	//


	segment_info = &segment_database [CONCRETE_ARCH_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_CONCRETE_ARCH_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_CONCRETE_ARCH_BRIDGE_START_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_START;
	segment_info->succ_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_START_DAMAGE_1;
	segment_info->both_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_START_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// CONCRETE_ARCH_BRIDGE_END
	//


	segment_info = &segment_database [CONCRETE_ARCH_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_CONCRETE_ARCH_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_CONCRETE_ARCH_BRIDGE_END_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_END_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_END;
	segment_info->both_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_END_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// CONCRETE_ARCH_BRIDGE_DECK
	//


	segment_info = &segment_database [CONCRETE_ARCH_BRIDGE_DECK];

	//

	segment_info->normal_object					= OBJECT_3D_CONCRETE_ARCH_BRIDGE_DECK;
	segment_info->destroyed_object				= OBJECT_3D_CONCRETE_ARCH_BRIDGE_DECK_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_DECK_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_DECK_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_CONCRETE_ARCH_BRIDGE_DECK_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// CONCRETE_WIDE_BRIDGE_START
	//


	segment_info = &segment_database [CONCRETE_WIDE_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_CONCRETE_WIDE_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_CONCRETE_WIDE_BRIDGE_START_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_START;
	segment_info->succ_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_START_DAMAGE_1;
	segment_info->both_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_START_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 30.0;

	segment_info->initial_damage_level			= 800;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// CONCRETE_WIDE_BRIDGE_END
	//


	segment_info = &segment_database [CONCRETE_WIDE_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_CONCRETE_WIDE_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_CONCRETE_WIDE_BRIDGE_END_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_END_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_END;
	segment_info->both_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_END_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 30.0;

	segment_info->initial_damage_level			= 800;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// CONCRETE_WIDE_BRIDGE_DECK
	//


	segment_info = &segment_database [CONCRETE_WIDE_BRIDGE_DECK];

	//

	segment_info->normal_object					= OBJECT_3D_CONCRETE_WIDE_BRIDGE_DECK;
	segment_info->destroyed_object				= OBJECT_3D_CONCRETE_WIDE_BRIDGE_DECK_DAMAGE_4;
	segment_info->pred_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_DECK_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_DECK_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_CONCRETE_WIDE_BRIDGE_DECK_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 30.0;

	segment_info->initial_damage_level			= 800;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// DESERT_BRIDGE_START
	//


	segment_info = &segment_database [DESERT_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_YEMEN_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_YEMEN_BRIDGE_START;
	segment_info->pred_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_START;
	segment_info->succ_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_START;
	segment_info->both_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_START;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 200;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// DESERT_BRIDGE_END
	//


	segment_info = &segment_database [DESERT_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_YEMEN_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_YEMEN_BRIDGE_END;
	segment_info->pred_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_END;
	segment_info->succ_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_END;
	segment_info->both_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_END;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 200;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// DESERT_BRIDGE_MID
	//


	segment_info = &segment_database [DESERT_BRIDGE_MID];

	//

	segment_info->normal_object					= OBJECT_3D_YEMEN_BRIDGE_MID;
	segment_info->destroyed_object				= OBJECT_3D_YEMEN_BRIDGE_MID_DAMAGE_4;
	segment_info->pred_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_MID_DAMAGE_2;
	segment_info->succ_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_MID_DAMAGE_3;
	segment_info->both_destroyed_object			= OBJECT_3D_YEMEN_BRIDGE_MID_DAMAGE_4;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 200;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// GIRDER_BRIDGE_START
	//


	segment_info = &segment_database [GIRDER_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_GIRDER_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_GIRDER_BRIDGE_START_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_START;
	segment_info->succ_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_START_DAMAGE_1;
	segment_info->both_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_START_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 10;


	//
	// GIRDER_BRIDGE_END
	//


	segment_info = &segment_database [GIRDER_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_GIRDER_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_GIRDER_BRIDGE_END_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_END_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_END;
	segment_info->both_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_END_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 10;


	//
	// GIRDER_BRIDGE_MID
	//


	segment_info = &segment_database [GIRDER_BRIDGE_MID];

	//

	segment_info->normal_object					= OBJECT_3D_GIRDER_BRIDGE_MID;
	segment_info->destroyed_object				= OBJECT_3D_GIRDER_BRIDGE_MID_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_MID_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_MID_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_MID_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 20.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 10;


	//
	// GIRDER_BRIDGE_DECK
	//


	segment_info = &segment_database [GIRDER_BRIDGE_DECK];

	//

	segment_info->normal_object					= OBJECT_3D_GIRDER_BRIDGE_DECK;
	segment_info->destroyed_object				= OBJECT_3D_GIRDER_BRIDGE_DECK_DAMAGE_1;
	segment_info->pred_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_DECK_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_DECK_DAMAGE_1;
	segment_info->both_destroyed_object			= OBJECT_3D_GIRDER_BRIDGE_DECK_DAMAGE_1;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION;

	segment_info->length								= 50.0;

	segment_info->initial_damage_level			= 500;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 10;


	//
	// STONE_BRIDGE_START
	//


	segment_info = &segment_database [STONE_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_STONE_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_STONE_BRIDGE_START;
	segment_info->pred_destroyed_object			= OBJECT_3D_STONE_BRIDGE_START;
	segment_info->succ_destroyed_object			= OBJECT_3D_STONE_BRIDGE_START;
	segment_info->both_destroyed_object			= OBJECT_3D_STONE_BRIDGE_START;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 200;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// STONE_BRIDGE_END
	//


	segment_info = &segment_database [STONE_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_STONE_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_STONE_BRIDGE_END;
	segment_info->pred_destroyed_object			= OBJECT_3D_STONE_BRIDGE_END;
	segment_info->succ_destroyed_object			= OBJECT_3D_STONE_BRIDGE_END;
	segment_info->both_destroyed_object			= OBJECT_3D_STONE_BRIDGE_END;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 200;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// STONE_BRIDGE_MID
	//


	segment_info = &segment_database [STONE_BRIDGE_MID];

	//

	segment_info->normal_object					= OBJECT_3D_STONE_BRIDGE_MID;
	segment_info->destroyed_object				= OBJECT_3D_STONE_BRIDGE_MID_DAMAGE_4;
	segment_info->pred_destroyed_object			= OBJECT_3D_STONE_BRIDGE_MID_DAMAGE_2;
	segment_info->succ_destroyed_object			= OBJECT_3D_STONE_BRIDGE_MID_DAMAGE_3;
	segment_info->both_destroyed_object			= OBJECT_3D_STONE_BRIDGE_MID_DAMAGE_4;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 200;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// SUSPENSION_BRIDGE_START
	//


	segment_info = &segment_database [SUSPENSION_BRIDGE_START];

	//

	segment_info->normal_object					= OBJECT_3D_SUSPENSION_BRIDGE_START;
	segment_info->destroyed_object				= OBJECT_3D_SUSPENSION_BRIDGE_START_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_START_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_START_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_START_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 750;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// SUSPENSION_BRIDGE_END
	//


	segment_info = &segment_database [SUSPENSION_BRIDGE_END];

	//

	segment_info->normal_object					= OBJECT_3D_SUSPENSION_BRIDGE_END;
	segment_info->destroyed_object				= OBJECT_3D_SUSPENSION_BRIDGE_END_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_END_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_END_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_END_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 750;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;


	//
	// SUSPENSION_BRIDGE_MID
	//


	segment_info = &segment_database [SUSPENSION_BRIDGE_MID];

	//

	segment_info->normal_object					= OBJECT_3D_SUSPENSION_BRIDGE_MID;
	segment_info->destroyed_object				= OBJECT_3D_SUSPENSION_BRIDGE_MID_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_MID_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_MID_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_MID_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 800;
	segment_info->target_type						= TARGET_TYPE_GROUND;
	segment_info->armor_level		= 5;


	//
	// SUSPENSION_BRIDGE_DECK
	//


	segment_info = &segment_database [SUSPENSION_BRIDGE_DECK];

	//

	segment_info->normal_object					= OBJECT_3D_SUSPENSION_BRIDGE_DECK;
	segment_info->destroyed_object				= OBJECT_3D_SUSPENSION_BRIDGE_DECK_DAMAGE_3;
	segment_info->pred_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_DECK_DAMAGE_1;
	segment_info->succ_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_DECK_DAMAGE_2;
	segment_info->both_destroyed_object			= OBJECT_3D_SUSPENSION_BRIDGE_DECK_DAMAGE_3;

	segment_info->sub_type							= ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION;

	segment_info->length								= 10.0;

	segment_info->initial_damage_level			= 100;
	segment_info->target_type						= TARGET_TYPE_INVALID;
	segment_info->armor_level		= 5;
}



































































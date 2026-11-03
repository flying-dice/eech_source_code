

































































#include "project.h"





#define DEBUG_MODULE 0





const fixed_data
   fixed_database[NUM_ENTITY_SUB_TYPE_FIXED] =
   {

      //
      // ENTITY_SUB_TYPE_FIXED_UNKNOWN
      //

      {
         OBJECT_3D_CRATE, 										// default_3d_shape
			FALSE,	  													// align_with_terrain
			FALSE,	  													// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION
      //

      {
			OBJECT_3D_STONE_BRIDGE_START,							// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_BRIDGE_SUPPORTED_MID_SECTION
      //

      {
			OBJECT_3D_STONE_BRIDGE_MID,							// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION
      //

      {
			OBJECT_3D_STONE_BRIDGE_MID,							// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_CITY
      //

      {
			OBJECT_3D_INVALID_OBJECT_INDEX,						// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,		 	 	  											// collision_test_fixed
			TRUE,		 													// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_CITY_BUILDING
      //

      {
			OBJECT_3D_CRATE,										// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,  	  													// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_CRATER
      //

      {
			OBJECT_3D_EFFECT_CRATER,								// default_3d_shape
			TRUE,  	  													// align_with_terrain
			FALSE,	  	  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_ELECTRICITY_PYLON_END_TERMINATOR
      //

      {
         OBJECT_3D_PYLON, 		 									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_ELECTRICITY_PYLON_STANDARD
      //

      {
         OBJECT_3D_PYLON, 		 									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_ELECTRICITY_PYLON_START_TERMINATOR
      //

      {
         OBJECT_3D_PYLON, 		 									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_ELECTRICITY_STATION
      //

      {
         OBJECT_3D_PYLON, 		 									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_ELECTRICITY_SUBSTATION
      //

      {
         OBJECT_3D_PYLON, 		 									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_MILITARY_BUILDING
      //

      {
         OBJECT_3D_CRATE, 	  									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			TRUE,	  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
      // ENTITY_SUB_TYPE_FIXED_SCENIC_OBJECT
      //

      {
         OBJECT_3D_CRATE, 	  									// default_3d_shape
			FALSE,  	  													// align_with_terrain
			FALSE,  		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },

      //
		// ENTITY_SUB_TYPE_FIXED_TERRAIN_OBJECT
      //

      {
         OBJECT_3D_CRATE,										// default_3d_shape
			FALSE,														// align_with_terrain
			FALSE,		  												// collision_test_fixed
			FALSE,														// ground_radar_clutter
      },
	};





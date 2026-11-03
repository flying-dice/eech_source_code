

































































#include "project.h"





#define DEBUG_MODULE 0





//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
keysite_data
   keysite_database[NUM_ENTITY_SUB_TYPE_KEYSITES] =
   {


      //
      // ENTITY_SUB_TYPE_KEYSITE_AIRBASE
      //


      {
         "Airbase",
         "Airbase",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				-0.2, 										// 	ammo
				-0.4, 										// 	fuel
			},

			1.0,												// importance
			0.3,												// minimum efficiency
			80.0 * KILOMETRE,								// importance radius
			400.0 * KILOMETRE,							// air coverage radius
			1000.0,											// recon distance
			80.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_AIRBASE,								// map icon
			FALSE,											// align with terrain (buildings)
			TRUE,												// unique name
			FALSE,											// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_LARGE,		// air force capacity
			TRUE,												// report ammo level
			TRUE,												// report fuel level

			3,													// task assign count
			2,													// task reserve count
			TRUE,												// requires CAP
			FALSE,											// requires BARCAP
			TRUE,												// repairable

			TRUE,												// OCA Target
			TRUE,												// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			TRUE,												// Troop Insertion Target
			TRUE,												// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_ANCHORAGE
      //


      {
         "Carrier",
         "Carrier",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				0.0, 											// 	ammo
				0.0, 											// 	fuel
			},

			1.0,												// importance
			0.3,												// minimum efficiency
			60.0 * KILOMETRE,								// importance radius
			100.0 * KILOMETRE,							// air coverage radius
			280.0,											// recon distance
			50.0,												// sector side max value

			MAP_LAYER_CONTROL_SHIPS,					// map layer type
			MAP_ICON_CARRIER,								// map icon
			FALSE,											// align with terrain (buildings)
			TRUE,												// unique name
			TRUE,												// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_SMALL,		// air force capacity
			TRUE,												// report ammo level
			TRUE,												// report fuel level

			1,													// task assign count
			1,													// task reserve count
			FALSE,											// requires CAP
			TRUE,												// requires BARCAP
			FALSE,											// repairable

			TRUE,												// OCA Target
			TRUE,												// Recon Target
			FALSE,											// Ground Strike Target
			TRUE,												// Ship Strike Target
			FALSE,											// Troop Insertion Target
			TRUE,												// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_FACTORY
      //


      {
         "Factory",
         "Factory",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				+1.0,											// 	ammo
				+0.0,											// 	fuel
			},

			0.5,												// importance
			0.3,												// minimum efficiency
			40.0 * KILOMETRE,								// importance radius
			0.0 * KILOMETRE,								// air coverage radius
			300.0,											// recon distance
			20.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_BUILDING,							// map icon
			FALSE,											// align with terrain (buildings)
			FALSE,											// unique name
			FALSE,											// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_NONE,			// air force capacity
			TRUE,												// report ammo level
			FALSE,											// report fuel level

			0,													// task assign count
			0,													// task reserve count
			FALSE,											// requires CAP 
			FALSE,											// requires BARCAP
			TRUE,												// repairable

			FALSE,											// OCA Target
			FALSE,											// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			TRUE,												// Troop Insertion Target
			FALSE,											// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_FARP
      //


      {
         "FARP",
         "FARP",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				-0.05, 										// 	ammo
				-0.03, 										// 	fuel
			},

			0.4,												// importance
			0.3,												// minimum efficiency
			40.0 * KILOMETRE,								// importance radius
			100.0 * KILOMETRE,							// air coverage radius
			320.0,											// recon distance
			40.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_FARP,									// map icon
			TRUE,												// align with terrain (buildings)
			FALSE,											// unique name
			TRUE,												// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_SMALL,		// air force capacity
			TRUE,												// report ammo level
			TRUE,												// report fuel level

			1,													// task assign count
			1,													// task reserve count
			TRUE,												// requires CAP 
			FALSE,											// requires BARCAP
			TRUE,												// repairable

			FALSE,											// OCA Target
			TRUE,												// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			TRUE,												// Troop Insertion Target
			FALSE,											// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_MILITARY_BASE
      //


      {
         "Military Base",
         "Military Base",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				-0.2,											// 	ammo
				-0.2,											// 	fuel
			},

			0.8,												// importance
			0.3,												// minimum efficiency
			30.0 * KILOMETRE,								// importance radius
			0.0 * KILOMETRE,								// air coverage radius
			450.0,											// recon distance
			30.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_BUILDING,							// map icon
			FALSE,											// align with terrain (buildings)
			FALSE,											// unique name
			TRUE,												// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_NONE,			// air force capacity
			TRUE,												// report ammo level
			TRUE,												// report fuel level

			0,													// task assign count
			0,													// task reserve count
			FALSE,											// requires CAP 
			FALSE,											// requires BARCAP
			TRUE,												// repairable

			FALSE,											// OCA Target
			TRUE,												// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			TRUE,												// Troop Insertion Target
			TRUE,												// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_PORT
      //


      {
         "Port",
         "Port",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				+0.2, 										// 	ammo
				+0.2, 										// 	fuel
			},

			0.2,												// importance
			0.3,												// minimum efficiency
			20.0 * KILOMETRE,								// importance radius
			0.0 * KILOMETRE,								// air coverage radius
			700.0,											// recon distance
			20.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_BUILDING,							// map icon
			FALSE,											// align with terrain (buildings)
			FALSE,											// unique name
			FALSE,											// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_NONE,			// air force capacity
			TRUE,												// report ammo level
			TRUE,												// report fuel level

			0,													// task assign count
			0,													// task reserve count
			FALSE,											// requires CAP 
			FALSE,											// requires BARCAP
			FALSE,											// repairable

			FALSE,											// OCA Target
			FALSE,											// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			FALSE,											// Troop Insertion Target
			FALSE,											// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_POWER_STATION
      //


      {
         "Power Station",
         "Power Station",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				0.0, 											// 	ammo
				-0.5, 										// 	fuel
			},

			0.3,												// importance
			0.3,												// minimum efficiency
			30.0 * KILOMETRE,								// importance radius
			0.0 * KILOMETRE,								// air coverage radius
			400.0,											// recon distance
			0.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_POWER_STATION,						// map icon
			FALSE,											// align with terrain (buildings)
			FALSE,											// unique name
			TRUE,												// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_NONE,			// air force capacity
			FALSE,											// report ammo level
			TRUE,												// report fuel level

			0,													// task assign count
			0,													// task reserve count
			FALSE,											// requires CAP
			FALSE,											// requires BARCAP
			FALSE,											// repairable

			FALSE,											// OCA Target
			TRUE,												// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			FALSE,											// Troop Insertion Target
			TRUE,												// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_OIL_REFINERY
      //


      {
         "Oil Refinery",
         "Refinery",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				0.0, 											// 	ammo
				+1.0, 										// 	fuel
			},

			0.4,												// importance
			0.3,												// minimum efficiency
			30.0 * KILOMETRE,								// importance radius
			0.0 * KILOMETRE,								// air coverage radius
			800.0,											// recon distance
			20.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_OIL_REFINERY,						// map icon
			FALSE,											// align with terrain (buildings)
			FALSE,											// unique name
			TRUE,												// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_NONE,			// air force capacity
			FALSE,											// report ammo level
			TRUE,												// report fuel level

			0,													// task assign count
			0,													// task reserve count
			FALSE,											// requires CAP
			FALSE,											// requires BARCAP
			FALSE,											// repairable

			FALSE,											// OCA Target
			TRUE,												// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			FALSE,											// Troop Insertion Target
			TRUE,												// Campaign Objective
      },


      //
      // ENTITY_SUB_TYPE_KEYSITE_RADIO_TRANSMITTER
      //


      {
         "Radio Transmitter",
         "Transmitter",

			{													// Supply usage
				0.0,											//    air/ground supply, ignore
				0.0,											//    air/air supply, ignore
				0.0, 											// 	ammo
				0.0,	 										// 	fuel
			},

			0.5,												// importance
			0.3,												// minimum efficiency
			30.0 * KILOMETRE,								// importance radius
			0.0 * KILOMETRE,								// air coverage radius
			250.0,											// recon distance
			30.0,												// sector side max value

			MAP_LAYER_CONTROL_KEYSITES,				// map layer type
			MAP_ICON_RADIO_TRANSMITTER,				// map icon
			FALSE,											// align with terrain (buildings)
			FALSE,											// unique name
			TRUE,												// hidden by fog
			KEYSITE_AIR_FORCE_CAPACITY_NONE,			// air force capacity
			FALSE,											// report ammo level
			FALSE,											// report fuel level

			0,													// task assign count
			0,													// task reserve count
			FALSE,											// requires CAP 
			FALSE,											// requires BARCAP
			FALSE,											// repairable

			FALSE,											// OCA Target
			TRUE,												// Recon Target
			TRUE,												// Ground Strike Target
			FALSE,											// Ship Strike Target
			FALSE,											// Troop Insertion Target
			FALSE,											// Campaign Objective
      },
	};





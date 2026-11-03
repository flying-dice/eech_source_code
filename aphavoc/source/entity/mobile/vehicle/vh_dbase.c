

































































#include "project.h"





#define DEBUG_MODULE 0




//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
vehicle_data
	vehicle_database[NUM_ENTITY_SUB_TYPE_VEHICLES] =
	{

		//
		// ENTITY_SUB_TYPE_VEHICLE_M1A2_ABRAMS
		//

		{
			"M1A2 Abrams",													// full_name





			ENTITY_FORCE_GROUND,											// force
			WEAPON_CONFIG_TYPE_M1A2_ABRAMS_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M1A2_ABRAMS_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M1A2_ABRAMS_1,						// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M2HB_12P7MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,	  															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			4000.0,		  													// surface_scan_range
			0.333,															// potential_surface_to_air_threat
			1.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			1.0,																// armour_front
			0.75,																// armour_side
			0.5,																// armour_rear

			1500,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_T80U
		//

		{
			"T-80U",															// full_name





			ENTITY_FORCE_GROUND,											// force
			WEAPON_CONFIG_TYPE_T80U_1,									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_T80U_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_T80U_1,									// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NSVT_12P7MM_ROUND, 				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,		 														// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			4000.0,															// surface_scan_range
			0.333,  															// potential_surface_to_air_threat
			1.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			1.0,																// armour_front
			0.75,																// armour_side
			0.5,																// armour_rear

			1350,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M2A2_BRADLEY
		//

		{
			"M2A2 Bradley",                                 	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M2A2_BRADLEY_1,		 				// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M2A2_BRADLEY_1,		 				// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M2A2_BRADLEY_1,		 				// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M242_25MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,	  															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			3000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			2000.0,															// air_scan_ceiling
			4000.0,															// air_scan_range
			4000.0,															// surface_scan_range
			0.666,															// potential_surface_to_air_threat
			1.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.75,																// armour_front
			0.5,																// armour_side
			0.5,																// armour_rear

			1000,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			125,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_BMP2
		//

		{
			"BMP-2",                                        	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_BMP2_1,									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_BMP2_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_BMP2_1,	  								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND,			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,																// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			3000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			2000.0,															// air_scan_ceiling
			4000.0,															// air_scan_range
			4000.0,															// surface_scan_range
			0.666, 															// potential_surface_to_air_threat
			1.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.5,																// armour_front
			0.25,																// armour_side
			0.25,																// armour_rear

			1000,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			125,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_BMP3
		//

		{
			"BMP-3",                                        	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_BMP3_1,									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_BMP3_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_BMP3_1,									// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A72_30MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,		 														// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			5000.0,	  														// surface_scan_range
			0.666,  															// potential_surface_to_air_threat
			1.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.5,																// armour_front
			0.25,																// armour_side
			0.25,																// armour_rear

			1000,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			125,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M113A2
		//

		{
			"M113A2",                                       	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M113A2_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M113A2_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M113A2_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M2HB_12P7MM_ROUND,  				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.333,															// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.5,																// armour_front
			0.5,																// armour_side
			0.5,																// armour_rear

			800,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			75,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_BTR80
		//

		{
			"BTR-80",                                       	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_BTR80_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_BTR80_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_BTR80_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_KPVT_14P5MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,																// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.333, 															// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.25,																// armour_front
			0.25,																// armour_side
			0.0,																// armour_rear

			800,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			75,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M1025_HUMVEE
		//

		{
			"M1025 HumVee",                                 	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON, 	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			650,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_BRDM2
		//

		{
			"BRDM-2",                                       	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_BRDM2_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_BRDM2_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_BRDM2_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_KPVT_14P5MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,	 									// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.333,															// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.25,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			650,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M109A2
		//

		{
			"M109A2",                                       	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M109A2_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M109A2_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M109A2_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M2HB_12P7MM_ROUND,  				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.333,		 													// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.75,																// armour_front
			0.5,																// armour_side
			0.25,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_2S19
		//

		{
			"2S19 Msta",                                         	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_2S19_1,		 							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_2S19_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_2S19_1,									// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NSVT_12P7MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			TRUE,		 														// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.333,															// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.75,																// armour_front
			0.5,																// armour_side
			0.25,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M270_MLRS
		//

		{
			"M270 MLRS",                                    	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M270_MLRS_1,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M270_MLRS_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M270_MLRS_1,							// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M26A1_227MM_ROCKET,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,	 		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,	  															// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.25,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			700,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_BM21_GRAD
		//

		{
			"BM-21 Grad",													// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_BM21_GRAD_1,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_BM21_GRAD_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_BM21_GRAD_1,							// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_BM21_122MM_ROCKET, 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			700,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M163_VULCAN
		//

		{
			"M163 Vulcan",                                  	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M163_VULCAN_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M163_VULCAN_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M163_VULCAN_1,						// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M61A1_20MM_ROUND, 				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_AAA,												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			2000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2500.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.666,															// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.25,																// armour_front
			0.25,																// armour_side
			0.25,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_SA19_GRISON
		//

		{
			"SA-19 Grison",                                 	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_SA19_GRISON_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_SA19_GRISON_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_SA19_GRISON_1,						// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A38M_30MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM,	  											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			6000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			4000.0,															// air_scan_ceiling
			7500.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.5,																// armour_front
			0.25,																// armour_side
			0.0,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_ARMOURED_VEHICLE,						// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M1037_AVENGER
		//

		{
			"M1097 Avenger",                                	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M1037_AVENGER_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M1037_AVENGER_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M1037_AVENGER_1,						// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_FIM92A_STINGER,					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM,												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			4000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			4000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			125,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M48A1_CHAPARRAL
		//

		{
			"M48A1 Chaparral",                              	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_M48A1_CHAPARRAL_1,	 				// default_weapon_config_type
			WEAPON_CONFIG_TYPE_M48A1_CHAPARRAL_1,	 				// min_weapon_config_type
			WEAPON_CONFIG_TYPE_M48A1_CHAPARRAL_1,	 				// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_MIM_72G_CHAPARRAL,						// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM,												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			4000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			5000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			125,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_SA13_GOPHER
		//

		{
			"SA-13 Gopher",                                 	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_SA13_GOPHER_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_SA13_GOPHER_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_SA13_GOPHER_1,						// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_9M333,						// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM,												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			3000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			5000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.5,																// armour_front
			0.25,																// armour_side
			0.0,																// armour_rear

			600,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			100,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M998_HUMVEE
		//

		{
			"M998 HumVee",                                  	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON, 	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			350,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_UAZ469B
		//

		{
			"UAZ-469B",                                     	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON, 	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			350,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M923A1_BIG_FOOT
		//

		{
			"M923A1 Big Foot",											// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON, 	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			350,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M923A1_BIG_FOOT_COVERED
		//

		{
			"M923A1 Big Foot",											// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON, 	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			350,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_URAL_4320
		//

		{
			"Ural-4320",													// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON, 	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			300,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_M978_HEMTT
		//

		{
			"M978 HEMTT",													// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			300,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE, 								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_URAL_FUEL_TANKER
		//

		{
			"Ural Fuel Tanker",		  									// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,			// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (20.0),              	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			300,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_TARAWA_CLASS
		//

		{
			"Tarawa Class",                                 	// full_name





			ENTITY_FORCE_SEA,                               	// force
			WEAPON_CONFIG_TYPE_TARAWA_CLASS_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_TARAWA_CLASS_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_TARAWA_CLASS_1,						// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M61A1_CIWS_20MM_ROUND,				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM, 												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (24.0),	            	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			8000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			5000.0,															// air_scan_ceiling
			10000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.8,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			12000,															// initial_damage_level
			WARHEAD_EFFECTIVE_CARRIER,									// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_HIGH,										// explosive_power
			TRUE,																// offensive_capability
			3000,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_KIEV_CLASS
		//

		{
			"Kiev Class",                                   	// full_name





			ENTITY_FORCE_SEA,                               	// force
			WEAPON_CONFIG_TYPE_KIEV_CLASS_1,		  					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_KIEV_CLASS_1,		  					// min_weapon_config_type
			WEAPON_CONFIG_TYPE_KIEV_CLASS_1,		  					// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_AK_630_30MM_ROUND,			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM,												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (32.0),               	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			8000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			5000.0,															// air_scan_ceiling
			10000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.8,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			15000,															// initial_damage_level
			WARHEAD_EFFECTIVE_CARRIER,									// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_HIGH,										// explosive_power
			TRUE,																// offensive_capability
			3500,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_OLIVER_HAZARD_PERRY_CLASS
		//

		{
			"Oliver Hazard Perry Class",								// full_name





			ENTITY_FORCE_SEA,												// force
			WEAPON_CONFIG_TYPE_OLIVER_HAZARD_PERRY_CLASS_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_OLIVER_HAZARD_PERRY_CLASS_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_OLIVER_HAZARD_PERRY_CLASS_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_RIM_66B_STANDARD,					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM, 												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (29.0),						// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			6000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			5000.0,															// air_scan_ceiling
			8000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			8000,																// initial_damage_level
			WARHEAD_EFFECTIVE_FRIGATE,									// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_HIGH,										// explosive_power
			TRUE,																// offensive_capability
			1500,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_KRIVAK_II_CLASS
		//

		{
			"Krivak II Class",											// full_name





			ENTITY_FORCE_SEA,												// force
			WEAPON_CONFIG_TYPE_KRIVAK_II_CLASS_1,					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_KRIVAK_II_CLASS_1,					// min_weapon_config_type
			WEAPON_CONFIG_TYPE_KRIVAK_II_CLASS_1,					// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_9M33_OSA,						// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_SAM,												// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (32.0),						// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			6000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			5000.0,															// air_scan_ceiling
			8000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			8000,																// initial_damage_level
			WARHEAD_EFFECTIVE_FRIGATE,									// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_HIGH,										// explosive_power
			TRUE,																// offensive_capability
			1000,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_LCU
		//

		{
			"Landing Craft Utility",	  								// full_name





			ENTITY_FORCE_SEA,                               	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,	 					// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (7.5),	            	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			1000,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_SHIP,								// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_LCAC
		//

		{
			"Landing Craft Air Cushioned",							// full_name





			ENTITY_FORCE_SEA,                               	// force
			WEAPON_CONFIG_TYPE_UNARMED,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UNARMED,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,  						// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (7.5),	            	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			1000,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_SHIP,								// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			FALSE,															// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_AIST_CLASS
		//

		{
			"Aist Class",			 										// full_name





			ENTITY_FORCE_SEA,                               	// force
			WEAPON_CONFIG_TYPE_AIST_CLASS_1,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AIST_CLASS_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AIST_CLASS_1,							// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_AK_630_30MM_ROUND,			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH, 		 								// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_SHIP_VEHICLE,				// force_info_catagory
			THREAT_TYPE_AAA,		 										// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (7.5),	            	// cruise_velocity


			rad (5.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			10.0,																// air_scan_floor
			1000.0,															// air_scan_ceiling
			2000.0,															// air_scan_range
			0.0,																// surface_scan_range
			0.666,	 														// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			1000,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_SHIP,								// armor_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,								// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,										// explosive_power
			TRUE,																// offensive_capability
			50,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_INFANTRY
		//

		{
			"Infantry",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_INFANTRY_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_INFANTRY_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_INFANTRY_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M16A2_5P56MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_INFANTRY_SAM_STANDING
		//

		{
			"FIM-92A Stinger MANPAD",											// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_INFANTRY_SAM_STANDING_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_INFANTRY_SAM_STANDING_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_INFANTRY_SAM_STANDING_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_FIM92A_STINGER,		 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (5.0),     	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			5.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			5000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_FLAMMABLE,								// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_INFANTRY_SAM_KNEELING
		//

		{
			"FIM-92A Stinger MANPAD",											// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_INFANTRY_SAM_KNEELING_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_INFANTRY_SAM_KNEELING_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_INFANTRY_SAM_KNEELING_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_FIM92A_STINGER,	  				// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (0.0),       	       	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			5.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			5000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_FLAMMABLE,								// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_CIS_INFANTRY
		//

		{
			"Infantry",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_AK_47_7P62MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_CIS_INFANTRY_SAM_STANDING
		//

		{
			"SA-7B MANPAD",										// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_SAM_STANDING_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_SAM_STANDING_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_SAM_STANDING_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_9K34_STRELA_3,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (5.0),   	           	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			5.0,																// air_scan_floor
			1500.0,															// air_scan_ceiling
			3000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_FLAMMABLE,								// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_CIS_INFANTRY_SAM_KNEELING
		//

		{
			"SA-7B MANPAD",										// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_SAM_KNEELING_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_SAM_KNEELING_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CIS_INFANTRY_SAM_KNEELING_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_9K34_STRELA_3,						// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (0.0),     	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			5.0,																// air_scan_floor
			1500.0,															// air_scan_ceiling
			3000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_FLAMMABLE,								// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_MANPAD
		//

		{
			"MANPAD",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_MANPAD_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_MANPAD_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_MANPAD_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_FIM92A_STINGER,		 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (5.0),     	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			5.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			5000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_FLAMMABLE,								// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_MACHINEGUNNER
		//

		{
			"Machinegunner",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_MACHINEGUNNER_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_MACHINEGUNNER_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_MACHINEGUNNER_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M60E4_7P62MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_RS_MANPAD
		//

		{
			"MANPAD",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_RS_MANPAD_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_MANPAD_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_MANPAD_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_9K34_STRELA_3,		 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,		 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			knots_to_metres_per_second (5.0),     	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			1000.0,															// recon_radius
			15.0,																// target_scan_delay
			5.0,																// air_scan_floor
			3000.0,															// air_scan_ceiling
			5000.0,															// air_scan_range
			0.0,																// surface_scan_range
			1.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_FLAMMABLE,								// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			TRUE,																// offensive_capability
			150,																// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_RS_MACHINEGUNNER
		//

		{
			"Machinegunner",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_RS_MACHINEGUNNER_1,		// default_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_MACHINEGUNNER_1,		// min_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_MACHINEGUNNER_1,		// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_PKM_7P62MM_ROUND,		 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_LIGHT_FIRING_POINT
		//

		{
			"Firing point",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_LIGHT_FIRING_POINT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_LIGHT_FIRING_POINT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_LIGHT_FIRING_POINT_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M60E4_7P62MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_MEDIUM_FIRING_POINT
		//

		{
			"Firing point",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_MEDIUM_FIRING_POINT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_MEDIUM_FIRING_POINT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_MEDIUM_FIRING_POINT_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M2HB_12P7MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_US_HEAVY_FIRING_POINT
		//

		{
			"Firing point",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_US_HEAVY_FIRING_POINT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_US_HEAVY_FIRING_POINT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_US_HEAVY_FIRING_POINT_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M61A1_20MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_RS_LIGHT_FIRING_POINT
		//

		{
			"Firing point",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_RS_LIGHT_FIRING_POINT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_LIGHT_FIRING_POINT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_LIGHT_FIRING_POINT_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_PKM_7P62MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_LOW,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_RS_MEDIUM_FIRING_POINT
		//

		{
			"Firing point",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_RS_MEDIUM_FIRING_POINT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_MEDIUM_FIRING_POINT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_MEDIUM_FIRING_POINT_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NSVT_12P7MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_MEDIUM,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},

		//
		// ENTITY_SUB_TYPE_VEHICLE_RS_HEAVY_FIRING_POINT
		//

		{
			"Firing point",     		                            	// full_name





			ENTITY_FORCE_GROUND,                            	// force
			WEAPON_CONFIG_TYPE_RS_HEAVY_FIRING_POINT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_HEAVY_FIRING_POINT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_RS_HEAVY_FIRING_POINT_1,								// max_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A72_30MM_ROUND,			 			// default_weapon_type
			TARGET_TYPE_GROUND,											// target_type

			TARGET_PRIORITY_HIGH,			 								// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,				// force_info_catagory
			THREAT_TYPE_INVALID,											// threat_type



			FALSE,															// carries_smoke_grenades





			0.8,   										  	         	// cruise_velocity


			rad (0.0),														// max_turn_rate (rad / second)


			500.0,															// recon_radius
			0.0,																// target_scan_delay
			0.0,																// air_scan_floor
			0.0,																// air_scan_ceiling
			0.0,																// air_scan_range
			0.0,																// surface_scan_range
			0.0,																// potential_surface_to_air_threat
			0.0,																// potential_surface_to_surface_threat
			10.0,																// Ammo_economy
			10.0,																// Fuel_economy
			0.0,																// armour_front
			0.0,																// armour_side
			0.0,																// armour_rear

			20,																// initial_damage_level
			WARHEAD_EFFECTIVE_LIGHT_VEHICLE,							// armor_level

			EXPLOSIVE_QUALITY_NONE,										// explosive_quality
			EXPLOSIVE_POWER_LOW,											// explosive_power
			FALSE,															// offensive_capability
			0,																	// points value



		},
	};







































































#include "project.h"





#define DEBUG_MODULE 0






// Xhit: Introduced main_rotor_radius and number_of_main_rotors. (030328)
//		 IMPORTANT: number_of_main_rotors is 2 only if the helicopter got 2 main rotors
//		 on different axis (like the CHINOOK).


//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
aircraft_data
   aircraft_database[NUM_ENTITY_SUB_TYPE_AIRCRAFT] =
   {

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_AH64D_APACHE_LONGBOW
      //

      {
         "AH-64D Apache Longbow",											// full_name





			GUNSHIP_TYPE_APACHE,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AH64D_APACHE_LONGBOW_39,					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64D_APACHE_LONGBOW_1,					// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64D_APACHE_LONGBOW_50,					// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64D_APACHE_LONGBOW_39,					// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64D_APACHE_LONGBOW_47,					// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64D_APACHE_LONGBOW_36,					// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (110.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (90),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																		// player_controllable
			200,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_MI28N_HAVOC_B
      //

      {
         "Mi-28N Havoc B",														// full_name





			GUNSHIP_TYPE_HAVOC,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_MI28N_HAVOC_B_7,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_MI28N_HAVOC_B_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_MI28N_HAVOC_B_30,							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_MI28N_HAVOC_B_7,							// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_MI28N_HAVOC_B_8,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_MI28N_HAVOC_B_6,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND,					// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (110.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (90),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0, 																	// air_scan_range
			4000.0, 																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																		// player_controllable
			200,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE
      //

      {
         "RAH-66 Comanche",													// full_name





			GUNSHIP_TYPE_COMANCHE,												// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_RAH66_COMANCHE_62,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_RAH66_COMANCHE_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_RAH66_COMANCHE_384,							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_RAH66_COMANCHE_128,							// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_RAH66_COMANCHE_379,							// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_RAH66_COMANCHE_62,							// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (100.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (100),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			500,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																		// player_controllable
			200,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_KA52_HOKUM_B
      //

      {
         "Ka-52 Alligator",														// full_name





			GUNSHIP_TYPE_HOKUM,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_KA52_HOKUM_B_7,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_KA52_HOKUM_B_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_KA52_HOKUM_B_30,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_KA52_HOKUM_B_7,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_KA52_HOKUM_B_8,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_KA52_HOKUM_B_6,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND,					// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (90.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (160),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			650,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																		// player_controllable
			200,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_UH60_BLACK_HAWK
      //

      ////Moje 030524 1st line under "OBJECT_3D_UH60_BLACKHAWK" changed from "NUM_GUNSHIP_TYPES"
      ////Moje 040209 CHanged loadout schemes
      {
         "UH-60L Blackhawk",									// full_name





			GUNSHIP_TYPE_BLACKHAWK,							// gunship_type
			ENTITY_FORCE_AIR,							// force
			WEAPON_CONFIG_TYPE_UH60_BLACK_HAWK_10,					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_UH60_BLACK_HAWK_1,					// min_weapon_config_type
			WEAPON_CONFIG_TYPE_UH60_BLACK_HAWK_12,					// max_weapon_config_type
			WEAPON_CONFIG_TYPE_UH60_BLACK_HAWK_10,					// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_UH60_BLACK_HAWK_6,					// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_UH60_BLACK_HAWK_7,					// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II,			// default_weapon_type

			TARGET_PRIORITY_MEDIUM,							// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,					// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,						// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (85.0),							// cruise_velocity
         24.0,											// cruise_altitude
         12.0,											// attack_altitude








			rad (80),								// max_turn_rate (deg / sec)






			5000.0,									// recon_radius
			10.0,									// target_scan_delay
			0.0,									// air_scan_range
			4000.0,		 							// surface_scan_range
			0.8,									// potential_air_to_air_threat
			1.0,									// potential_air_to_surface_threat
			10.0,									// Ammo_economy
			1136.0,									// Fuel_default_weight
			10.0,									// Fuel_economy


			650,								// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,					// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,						// explosive_power
			TRUE,								// offensive_capability
			TRUE,								// player_controllable JB 030408 Fly any aircraft
			150,								// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_MI24D_HIND
      //

      ////Moje 030524 1st line under "OBJECT_3D_MI24_HIND" changed from "NUM_GUNSHIP_TYPES"
      ////Moje 040209 CHanged loadout schemes
      {
         "Mi-24V Hind",															// full_name





			GUNSHIP_TYPE_HIND,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_MI24D_HIND_60,									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_MI24D_HIND_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_MI24D_HIND_60,									// max_weapon_config_type
			WEAPON_CONFIG_TYPE_MI24D_HIND_60,									// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_MI24D_HIND_49,									// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_MI24D_HIND_31,									// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_YAK_B_12P7MM_ROUND,					// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (80.0),								// cruise_velocity
         24.0,																		// cruise_altitude
         12.0,																		// attack_altitude








			rad (70),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			2000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			0.8,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			650,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			150,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_CH46E_SEA_KNIGHT
      //

      {
         "CH-46E Sea Knight",													// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_CH46E_SEA_KNIGHT_1,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CH46E_SEA_KNIGHT_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CH46E_SEA_KNIGHT_1,							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_CH46E_SEA_KNIGHT_1,							// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_CH46E_SEA_KNIGHT_1,							// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_CH46E_SEA_KNIGHT_1,							// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,	 							// default_weapon_type

			TARGET_PRIORITY_MEDIUM,	  											// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (70.0),								// cruise_velocity
         40.0,																		// cruise_altitude
         40.0,																		// attack_altitude








			rad (60),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			750,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,																	// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			50,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_KA29_HELIX_B
      //

      {
         "Ka-29 Helix B",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_KA29_HELIX_B_1, 								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_KA29_HELIX_B_1, 								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_KA29_HELIX_B_3, 								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_KA29_HELIX_B_2,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_KA29_HELIX_B_1,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_KA29_HELIX_B_3,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_9M120_ATAKA_V,											// default_weapon_type

			TARGET_PRIORITY_MEDIUM,	 											// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (75.0),								// cruise_velocity
         25.0,																		// cruise_altitude
         20.0,																		// attack_altitude








			rad (160),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			2000.0,																	// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			750,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			75,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_CH3_JOLLY_GREEN_GIANT
      //

      {
         "CH-3 Jolly Green Giant",											// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_CH3_JOLLY_GREEN_GIANT_1,					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CH3_JOLLY_GREEN_GIANT_1,					// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CH3_JOLLY_GREEN_GIANT_1,					// max_weapon_config_type
			WEAPON_CONFIG_TYPE_CH3_JOLLY_GREEN_GIANT_1,					// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_CH3_JOLLY_GREEN_GIANT_1,					// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_CH3_JOLLY_GREEN_GIANT_1,					// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,	 							// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (70.0),								// cruise_velocity
         40.0,																		// cruise_altitude
         40.0,																		// attack_altitude








			rad (40),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			800,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,																	// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			50,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_MI17_HIP
      //

      {
         "Mi-17 Hip",															// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_MI17_HIP_1,	  								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_MI17_HIP_1,	  								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_MI17_HIP_3,	  								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_MI17_HIP_2,									// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_MI17_HIP_1,									// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_MI17_HIP_3,									// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,											// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (75.0),								// cruise_velocity
         40.0,																		// cruise_altitude
         40.0,																		// attack_altitude








			rad (50),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			800,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			50,																		// points value




      },

      //
      // ENTITY_SUB_TYPE_AIRCRAFT_CH47D_CHINOOK
      //

      {
         "CH-47D Chinook",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_CH47D_CHINOOK_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CH47D_CHINOOK_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CH47D_CHINOOK_1,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_CH47D_CHINOOK_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_CH47D_CHINOOK_1,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_CH47D_CHINOOK_1,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (70.0),								// cruise_velocity
         40.0,																		// cruise_altitude
         40.0,																		// attack_altitude








			rad (45),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			800,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,																	// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			50,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_MI6_HOOK
		//

		{
			"Mi-6 Hook",															// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_MI6_HOOK_1, 									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_MI6_HOOK_1, 									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_MI6_HOOK_1, 									// max_weapon_config_type
			WEAPON_CONFIG_TYPE_MI6_HOOK_1,									// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_MI6_HOOK_1,									// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_MI6_HOOK_1,									// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NSVT_12P7MM_ROUND,		  						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,  											// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (70.0),								// cruise_velocity
			40.0,																		// cruise_altitude
			40.0,																		// attack_altitude








			rad (45),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			1000,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,																	// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			50,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_A10A_THUNDERBOLT
		//

		{
			"A-10A Thunderbolt",													// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_A10A_THUNDERBOLT_1,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_A10A_THUNDERBOLT_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_A10A_THUNDERBOLT_1,							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_A10A_THUNDERBOLT_1,							// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_A10A_THUNDERBOLT_1,							// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_A10A_THUNDERBOLT_1,							// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GAU8A_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (300.0),								// cruise_velocity
			700.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180.0),															// max_turn_rate (deg / sec)






			6000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			5000.0,																	// surface_scan_range
			0.8,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			850,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			300,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_SU25_FROGFOOT
		//

		{
			"Su-25 Frogfoot",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_SU25_FROGFOOT_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_SU25_FROGFOOT_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_SU25_FROGFOOT_1,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_SU25_FROGFOOT_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_SU25_FROGFOOT_1,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_SU25_FROGFOOT_1,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GSH_2_30_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (300.0),								// cruise_velocity
			700.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad(180),																// max_turn_rate (deg / sec)






			6000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			5000.0,																	// surface_scan_range
			0.8,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			850,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			300,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_F16_FIGHTING_FALCON
		//

		{
			"F-16C Fighting Falcon",												// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_F16_FIGHTING_FALCON_2,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_F16_FIGHTING_FALCON_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_F16_FIGHTING_FALCON_2,						// max_weapon_config_type
			WEAPON_CONFIG_TYPE_F16_FIGHTING_FALCON_1,						// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_F16_FIGHTING_FALCON_2,						// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_F16_FIGHTING_FALCON_2,						// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M61A1_20MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (450.0),								// cruise_velocity
			900.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			10000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			5000.0,																	// air_scan_range
			6000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			400,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_MIG29_FULCRUM
		//

		{
			"MiG-29 Fulcrum",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_MIG29_FULCRUM_2,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_MIG29_FULCRUM_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_MIG29_FULCRUM_2,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_MIG29_FULCRUM_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_MIG29_FULCRUM_2,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_MIG29_FULCRUM_2,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GSH_30_1_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (450.0),								// cruise_velocity
			900.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			10000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			5000.0,																	// air_scan_range
			6000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			400,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_AV8B_HARRIER
		//

		{
			"AV-8B Harrier",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AV8B_HARRIER_1,	 							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AV8B_HARRIER_1,	 							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AV8B_HARRIER_1,	 							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AV8B_HARRIER_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AV8B_HARRIER_1,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AV8B_HARRIER_1,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GAU12U_25MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (350.0),								// cruise_velocity
			800.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			6000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			5000.0,																	// surface_scan_range
			0.8,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			200,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_YAK41_FREESTYLE
		//

		{
			"Yak-41 Freestyle",													// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_YAK141_FREESTYLE_1,							// default_weapon_config_type
			WEAPON_CONFIG_TYPE_YAK141_FREESTYLE_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_YAK141_FREESTYLE_1,							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_YAK141_FREESTYLE_1,							// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_YAK141_FREESTYLE_1,							// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_YAK141_FREESTYLE_1,							// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GSH_30_1_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (350.0),								// cruise_velocity
			800.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			6000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			5000.0,																	// surface_scan_range
			0.8,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			200,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_FA18_HORNET
		//

		{
			"F/A-18C Hornet",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_FA18_HORNET_2,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_FA18_HORNET_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_FA18_HORNET_2,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_FA18_HORNET_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_FA18_HORNET_2,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_FA18_HORNET_2,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M61A1_20MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (450.0),								// cruise_velocity
			1000.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			10000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			6000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			350,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_SU33_FLANKER
		//

		{
			"Su-33 Flanker",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_SU33_FLANKER_2,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_SU33_FLANKER_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_SU33_FLANKER_2,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_SU33_FLANKER_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_SU33_FLANKER_2,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_SU33_FLANKER_2,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GSH_30_1_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (450.0),								// cruise_velocity
			1000.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			10000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			6000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			350,																		// points value




		},


		//
		// ENTITY_SUB_TYPE_AIRCRAFT_SU34_FULLBACK
		//

		{
			"Su-34 Fullback",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_SU34_FULLBACK_2,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_SU34_FULLBACK_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_SU34_FULLBACK_2,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_SU34_FULLBACK_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_SU34_FULLBACK_2,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_SU34_FULLBACK_2,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_GSH_30_1_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_FIXED_WING,							// force_info_catagory
			THREAT_TYPE_AIRBORNE_RADAR,										// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (450.0),								// cruise_velocity
			1000.0,																	// cruise_altitude
			100.0,																	// attack_altitude








			rad (180),																// max_turn_rate (deg / sec)






			10000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			6000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			700,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			FALSE,																	// player_controllable
			350,																		// points value




		},


		//
		// COMANCHE HOKUM
		//



		//
		// ENTITY_SUB_TYPE_AIRCRAFT_AH64A_APACHE
		//

      ////Moje 030816 1st line under "OBJECT_3D_AH_64A" changed from "NUM_GUNSHIP_TYPES"
      ////Moje 040209 CHanged loadout schemes
      {
         "AH-64A Apache",														// full_name





			GUNSHIP_TYPE_AH64A,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AH64A_APACHE_24,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64A_APACHE_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64A_APACHE_32,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64A_APACHE_22,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64A_APACHE_24,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AH64A_APACHE_18,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND,						// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (110.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         10.0,																		// attack_altitude








			rad (200),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		//DL initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			120,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_KA50_HOKUM
		//

      ////Moje 030816 1st line under "OBJECT_3D_KA_50" changed from "NUM_GUNSHIP_TYPES"
      {
         "Ka-50 Black Shark",															// full_name





			GUNSHIP_TYPE_KA50,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_KA50_HOKUM_7,									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_KA50_HOKUM_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_KA50_HOKUM_30,									// max_weapon_config_type
			WEAPON_CONFIG_TYPE_KA50_HOKUM_7,									// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_KA50_HOKUM_8,									// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_KA50_HOKUM_6,									// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND,					// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (115.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         10.0,																		// attack_altitude








			rad (240),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			120,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_OH58D_KIOWA_WARRIOR
		//

      {
         "OH-58D Kiowa Warrior",												//full_name





			GUNSHIP_TYPE_KIOWA,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_OH58D_KIOWA_WARRIOR_8,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_OH58D_KIOWA_WARRIOR_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_OH58D_KIOWA_WARRIOR_20,						// max_weapon_config_type
			WEAPON_CONFIG_TYPE_OH58D_KIOWA_WARRIOR_8,						// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_OH58D_KIOWA_WARRIOR_9,						// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_OH58D_KIOWA_WARRIOR_2,						// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (120.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (280),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			100,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_MV22_OSPREY
		//

      {
         "MV-22 Osprey",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_MV22_OSPREY_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_MV22_OSPREY_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_MV22_OSPREY_1,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_MV22_OSPREY_1,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_MV22_OSPREY_1,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_MV22_OSPREY_1,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (280.0),								// cruise_velocity
         100.0,																	// cruise_altitude
         100.0,																	// attack_altitude








			rad (100),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			FALSE,			 														// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			75,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_AH1T_SEACOBRA
		//

      {
         "AH-1T Sea Cobra",														// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AH1T_SEACOBRA_1,								// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1T_SEACOBRA_1,								// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1T_SEACOBRA_3,								// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1T_SEACOBRA_2,								// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1T_SEACOBRA_1,								// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1T_SEACOBRA_3,								// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (110.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (200),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			100,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_AH1W_SUPERCOBRA
		//

      {
         "AH-1W Super Cobra",													// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AH1W_SUPERCOBRA_1,				 			// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1W_SUPERCOBRA_1,				 			// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1W_SUPERCOBRA_3,				 			// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1W_SUPERCOBRA_2,				 			// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1W_SUPERCOBRA_1,				 			// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1W_SUPERCOBRA_3,				 			// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (110.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (200),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			100,																		// points value



      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_CH53E_SUPER_STALLION
		//

      {
         "CH-53E Super Stallion",											// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_CH53E_SUPER_STALLION_1,					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_CH53E_SUPER_STALLION_1,					// min_weapon_config_type
			WEAPON_CONFIG_TYPE_CH53E_SUPER_STALLION_1,					// max_weapon_config_type
			WEAPON_CONFIG_TYPE_CH53E_SUPER_STALLION_1,					// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_CH53E_SUPER_STALLION_1,					// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_CH53E_SUPER_STALLION_1,					// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,						// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (85.0),								// cruise_velocity
         50.0,																		// cruise_altitude
         50.0,																		// attack_altitude








			rad (160),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,																		// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,																	// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			75,																		// points value




      },

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_C17_GLOBEMASTER_III
		//

		{
			"C-17 Globemaster III",												// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_C17_GLOBEMASTER_III_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_C17_GLOBEMASTER_III_1,						// min_weapon_config_type
			WEAPON_CONFIG_TYPE_C17_GLOBEMASTER_III_1,						// max_weapon_config_type
			WEAPON_CONFIG_TYPE_C17_GLOBEMASTER_III_1,						// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_C17_GLOBEMASTER_III_1,						// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_C17_GLOBEMASTER_III_1,						// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_FIXED_WING,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (240.0),								// cruise_velocity
			700.0,																	// cruise_altitude
			700.0,																	// attack_altitude








			rad (40),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,				  														// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			850,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,  																	// offensive_capability
			FALSE,																	// player_controllable
			150,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_IL76MD_CANDID_B
		//

		{
			"Il-76MD Candid-B",													// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_IL76MD_CANDID_B_1,		  					// default_weapon_config_type
			WEAPON_CONFIG_TYPE_IL76MD_CANDID_B_1,							// min_weapon_config_type
			WEAPON_CONFIG_TYPE_IL76MD_CANDID_B_1,							// max_weapon_config_type
			WEAPON_CONFIG_TYPE_IL76MD_CANDID_B_1,							// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_IL76MD_CANDID_B_1,							// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_IL76MD_CANDID_B_1,							// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_FIXED_WING,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (240.0),								// cruise_velocity
			700.0,																	// cruise_altitude
			700.0,																	// attack_altitude








			rad (40),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,				  														// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			850,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,  																	// offensive_capability
			FALSE,																	// player_controllable
			150,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_C130J_HERCULES_II
		//

		{
			"C-130J Hercules II",			  									// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_C130J_HERCULES_II_1,						// default_weapon_config_type
			WEAPON_CONFIG_TYPE_C130J_HERCULES_II_1,		  				// min_weapon_config_type
			WEAPON_CONFIG_TYPE_C130J_HERCULES_II_1,		  				// max_weapon_config_type
			WEAPON_CONFIG_TYPE_C130J_HERCULES_II_1,		  				// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_C130J_HERCULES_II_1,		  				// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_C130J_HERCULES_II_1,		  				// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_FIXED_WING,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (260.0),								// cruise_velocity
			700.0,																	// cruise_altitude
			700.0,																	// attack_altitude








			rad (60),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,				  														// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			850,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,  																	// offensive_capability
			FALSE,																	// player_controllable
			150,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_AN12B_CUB
		//

		{
			"An-12B Cub",															// full_name





			NUM_GUNSHIP_TYPES,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AN12B_CUB_1,									// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AN12B_CUB_1,									// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AN12B_CUB_1,									// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AN12B_CUB_1,									// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AN12B_CUB_1,									// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AN12B_CUB_1,									// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_NO_WEAPON,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_UNARMED_FIXED_WING,						// force_info_catagory
			THREAT_TYPE_INVALID,													// threat_type



			5,																		// armor_level







			knots_to_metres_per_second (260.0),								// cruise_velocity
			700.0,																	// cruise_altitude
			700.0,																	// attack_altitude








			rad (50),																// max_turn_rate (deg / sec)






			3000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			0.0,																		// air_scan_range
			0.0,				  														// surface_scan_range
			0.0,																		// potential_air_to_air_threat
			0.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			850,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_HIGH,												// explosive_power
			FALSE,  																	// offensive_capability
			FALSE,																	// player_controllable
			150,																		// points value




		},

		//
		// ENTITY_SUB_TYPE_AIRCRAFT_AH1Z_VIPER
		//

      {
         "AH-1Z Viper",													// full_name





			GUNSHIP_TYPE_VIPER,													// gunship_type
			ENTITY_FORCE_AIR,														// force
			WEAPON_CONFIG_TYPE_AH1Z_VIPER_17,				 			// default_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1Z_VIPER_1,				 			// min_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1Z_VIPER_18,				 			// max_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1Z_VIPER_14,				 			// air_to_air_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1Z_VIPER_17,				 			// air_to_surface_weapon_config_type
			WEAPON_CONFIG_TYPE_AH1Z_VIPER_11,				 			// scout_weapon_config_type
			ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND,								// default_weapon_type

			TARGET_PRIORITY_MEDIUM,												// target_priority_type
			FORCE_INFO_CATAGORY_ARMED_HELICOPTER,							// force_info_catagory
			THREAT_TYPE_INVALID,			 										// threat_type



			5,																		// armor_level







         knots_to_metres_per_second (110.0),								// cruise_velocity
         15.0,																		// cruise_altitude
         7.5,																		// attack_altitude








			rad (200),																// max_turn_rate (deg / sec)






			5000.0,																	// recon_radius
			10.0,																		// target_scan_delay
			3000.0,																	// air_scan_range
			4000.0,																	// surface_scan_range
			1.0,																		// potential_air_to_air_threat
			1.0,																		// potential_air_to_surface_threat
			10.0,																		// Ammo_economy
			1136.0,																	// Fuel_default_weight
			10.0,																		// Fuel_economy


			600,																		// initial_damage_level

			EXPLOSIVE_QUALITY_EXPLOSIVE,										// explosive_quality
			EXPLOSIVE_POWER_MEDIUM,												// explosive_power
			TRUE,																		// offensive_capability
			TRUE,																	// player_controllable JB 030408 Fly any aircraft
			100,																		// points value



      },
	};





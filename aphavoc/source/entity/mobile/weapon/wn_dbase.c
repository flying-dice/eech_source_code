

































































#include "project.h"





#define DEBUG_MODULE 										0

#define DEBUG_MODULE_DISPLAY_TARGET_POINT_OFFSETS	0





#define MAX_CRUISE_TIME	((float) (511.0))	// change this then update EN_FLOAT.C FLOAT_TYPE_WEAPON_LIFETIME pack size




//
// Weapon Dynamics
//
//		Powered Weapons
//			Given:	max_range
//						muzzle_velocity
//						cruise_velocity
//						g_max
//			Compute:	burn_time = (cruise_velocity - muzzle_velocity) / g_max
//						burn_dist = (muzzle_velocity * burn_time) + (g_max * burn_time * burn_time) / 2.0
//						cruise_dist = max_range - burn_dist
//						cruise_time = cruise_dist / cruise_velocity
//
//		Short Life Ballistic Weapons (cannon shells)
//			Given:	max_range
//						muzzle_velocity
//	  		Compute:	cruise_velocity = muzzle_velocity
//						g_max = 0.0
//	  					burn_time = 0.0
//						cruise_time = max_range / cruise_velocity
//
//		Long Life Ballistic Weapons (artillery shells)
//			Given:	max_range (horizontal)
//	  		Compute:	muzzle_velocity = sqrt (max_range * G)
//						cruise_velocity = muzzle_velocity
//						g_max = 0.0
//						burn_time = 0.0
//						cruise_time = 2.0 * muzzle_velocity * sin (45) / G
//
//			Notes: 	Maximum range when elevation is 45 degrees.
//
//						Assume range is horizontal.
//
//						range = ((u * u) / G) * sin (2.0 * theta)		// theta = 45 degrees, therefore, sin (2.0 * theta) = 1.0
//
//						u = sqrt (range * G)
//
//						t = (2.0 * u * sin (theta)) / G					// theta = 45 degrees
//




//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition

weapon_data
	weapon_database[NUM_ENTITY_SUB_TYPE_WEAPONS] =
	{

		//
		// ENTITY_SUB_TYPE_WEAPON_NO_WEAPON
		//

		{
			"No Weapon",														// full_name


			"?",																// name abbreviation




			(weapon_guidance_types) 0,		 																// guidance_type

			(weapon_decoy_types) 0,																		// decoy_type
			(weapon_warhead_types) 0,																		// warhead_type
			(threat_types) 0,																		// threat_type










			0,																		// soft_damage_capability
			0,																		// hard_damage_capability









			0,																		// weapon_class



			0.0,																	// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			0.0,																	// cruise_velocity





			0.0,																	// burst_duration
			0.0,																	// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// BALLISTIC WEAPONS
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_M16A2_5P56MM_ROUND
		//

		{
			"M16A2 5.56mm Round",		 											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			4,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,


			0.0,		  															// min_range
			1800.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M60E4_7P62MM_ROUND
		//

		{
			"M60E4 7.62mm Round",		 											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			4,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,


			0.0,		  															// min_range
			1800.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M2HB_12P7MM_ROUND
		//

		{
			"M2HB 12.7mm Round",		 											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			4,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,


			0.0,		  															// min_range
			1800.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M61A1_20MM_ROUND
		//

		{
			"M61A1 20mm Round",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			12,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE +
			WEAPON_CLASS_SURFACE_TO_AIR +
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,																	// min_range
			1600.0,																// max_range
			600.0,																// effective_range



			884.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M61A1_CIWS_20MM_ROUND
		//

		{
			"M61A1 CIWS 20mm Round",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			12,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE +
			WEAPON_CLASS_SURFACE_TO_AIR +
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,																	// min_range
			1600.0,																// max_range
			600.0,																// effective_range



			884.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M197_20MM_ROUND
		//

		{
			"M197 20mm Round",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			12,																	// soft_damage_capability
			5,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,			 														// min_range
			1600.0,																// max_range
			500.0,																// effective_range



			884.0,																// cruise_velocity





			3.0,																	// burst_duration
			750.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_GAU12U_25MM_ROUND
		//

		{
			"GAU-12/U 25mm Round", 			  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			15,																	// soft_damage_capability
			3,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,																	// min_range
			1600.0,																// max_range
			600.0,																// effective_range



			1051.0,																// cruise_velocity





			3.0,																	// burst_duration
			1650.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M242_25MM_ROUND
		//

		{
			"M242 25mm Round", 				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			16,																	// soft_damage_capability
			6,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,		  															// min_range
			3500.0,																// max_range
			2000.0,																// effective_range



			1000.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			9.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_GAU8A_30MM_ROUND
		//

		{
			"GAU-8/A 30mm Round",				  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_DUAL_PURPOSE,						// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			100,																	// soft_damage_capability
			100,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,																	// min_range
			1200.0,																// max_range
			600.0,																// effective_range



			1030.0,																// cruise_velocity





			3.0,																	// burst_duration
			2100.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M230_30MM_ROUND
		//

		{
			"M230 30mm Round",				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			30,																	// soft_damage_capability
			15,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,	 																// min_range
			1200.0,																// max_range
			800.0,																// effective_range



			792.0,																// cruise_velocity





			3.0,																	// burst_duration
			625.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_OTO_MELARA_76MM_ROUND
		//

		{
			//
			// customised for Oliver Hazard Perry Class (pitch limits 20.0 to 85.0 degrees)
			//

			"M75 76mm Round",													// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			450,																	// soft_damage_capability
			450,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			2571.4,																// min_range
			4000.0,																// max_range
			2000.0,																// effective_range



			198.1,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			20.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M256_120MM_ROUND
		//

		{
			//
			// customised for M1A2 Abrams (pitch limits -9.0 to 20.0 degrees)
			//

			"M256 120mm Round",				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			634,																	// soft_damage_capability
			634,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,																	// min_range
			4000.0,																// max_range
			2000.0,																// effective_range



			247.1,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			15.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M284_155MM_ROUND
		//

		{
			"M284 155mm Round",				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			1610,																	// soft_damage_capability
			1610,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,		  															// min_range
			30000.0,																// max_range
			15000.0,																// effective_range



			542.5,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			20.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M26A1_227MM_ROCKET
		//

		{
			//
			// customised for M270 MLRS (pitch limits 15.0 to 50.0 degrees)
			//

			"M26A1 227mm Rocket",				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			2500,																	// soft_damage_capability
			2500,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			15801.5,																// min_range
			31600.0,																// max_range
			15000.0,																// effective_range



			556.8,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AK_47_7P62MM_ROUND
		//

		{
			"AK-47 7.62mm Round",		 											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			4,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,		  															// min_range
			1800.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_PKM_7P62MM_ROUND
		//

		{
			"PKM 7.62mm Round",		 											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			4,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,		  															// min_range
			1800.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_NSVT_12P7MM_ROUND // also used as A-12,7
		//

		{
			"12.7 Machine Gun",		 											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			4,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,		  															// min_range
			1800.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			10.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_YAK_B_12P7MM_ROUND
		//

		{
			"Yak-B",				  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			5,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,		  															// min_range
			1500.0,																// max_range
			800.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			4000.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_KPVT_14P5MM_ROUND
		//

		{
			"14.5mm Round",	  					  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			6,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,		  															// min_range
			1600.0,																// max_range
			600.0,																// effective_range



			600.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			12.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_GSH23L_23MM_ROUND
		//

		{
			"UPK-23",						  								// full_name


			"CP",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			25,																	// soft_damage_capability
			8,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,		  															// min_range
			2000.0,																// max_range
			500.0,																// effective_range



			930.0,																// cruise_velocity





			3.0,																	// burst_duration
			3200.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_GSH_30_1_30MM_ROUND
		//

		{
			"GSh-301 30mm Round",				  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			40,																	// soft_damage_capability
			25,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,	  																// min_range
			1600.0,																// max_range
			800.0,																// effective_range



			800.0,																// cruise_velocity





			3.0,																	// burst_duration
			1500.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A38M_30MM_ROUND
		//

		{
			"2A38M 30mm Round",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			25,																	// soft_damage_capability
			10,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			0.0,																	// min_range
			2500.0,																// max_range
			1500.0,																// effective_range



			1200.0,																// cruise_velocity





			3.0,																	// burst_duration
			2000.0,																// rate_of_fire
			15.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A42_30MM_ROUND
		//

		{
			"2A42 30mm Round",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			30,																	// soft_damage_capability
			15,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,			 														// min_range
			4000.0,																// max_range
			800.0,																// effective_range



			1000.0,																// cruise_velocity





			3.0,																	// burst_duration
			600.0,																// rate_of_fire
			15.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A42_30MM_HE_ROUND
		//

		{
			"2A42 30mm HE Round",	  		  								// full_name


			"HE",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			30,																	// soft_damage_capability
			15,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,			 														// min_range
			2000.0,																// max_range
			600.0,																// effective_range



			1000.0,																// cruise_velocity





			3.0,																	// burst_duration
			600.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A42_30MM_AP_ROUND
		//

		{
			"2A42 30mm AP Round",	  		  								// full_name


			"AP",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_DUAL_PURPOSE,						// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			25,																	// soft_damage_capability
			25,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,			 														// min_range
			2000.0,																// max_range
			600.0,																// effective_range



			1000.0,																// cruise_velocity





			3.0,																	// burst_duration
			350.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A72_30MM_ROUND
		//

		{
			"2A72 30mm Round",  		  		  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			30,																	// soft_damage_capability
			15,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR +									// weapon_class
			WEAPON_CLASS_SURFACE_TO_SURFACE,



			0.0,	 																// min_range
			2000.0,																// max_range
			1000.0,																// effective_range



			800.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			16.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_GSH_2_30_30MM_ROUND
		//

		{
			"AO-17A 30mm Round",	  	  		  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_DUAL_PURPOSE,						// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			30,																	// soft_damage_capability
			15,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR +										// weapon_class
			WEAPON_CLASS_AIR_TO_SURFACE,



			0.0,																	// min_range
			1500.0,																// max_range
			800.0,																// effective_range



			1000.0,																// cruise_velocity





			3.0,																	// burst_duration
			100.0,																// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AK_630_30MM_ROUND
		//

		{
			"ADMG-630 30mm Round",											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			30,																	// soft_damage_capability
			15,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			0.0,																	// min_range
			1500.0,																// max_range
			1000.0,																// effective_range



			800.0,																// cruise_velocity





			3.0,																	// burst_duration
			1000.0,																// rate_of_fire
			9.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AK_726_76MM_ROUND
		//

		{
			"76mm Round",														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			450,																	// soft_damage_capability
			450,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,		  															// min_range
			4000.0,																// max_range
			2000.0,																// effective_range



			198.1,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			20.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_A_190_100MM_ROUND
		//

		{
			"100mm Round",														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			650,																	// soft_damage_capability
			650,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,																	// min_range
			4000.0,																// max_range
			2000.0,																// effective_range



			198.1,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A70_100MM_ROUND
		//

		{
			"2K23 100mm Round",	  	  		  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			650,																	// soft_damage_capability
			650,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,																	// min_range
			4000.0,																// max_range
			2000.0,																// effective_range



			198.1,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			22.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_BM21_122MM_ROCKET
		//

		{
			//
			// customised for BM-21 (pitch limits 5.0 to 55.0 degrees)
			//

			"BM-21 122mm Rocket",	  	  		  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			1400,																	// soft_damage_capability
			1400,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			5209.4,	  															// min_range
			30000.0,																// max_range
			15000.0,																// effective_range



			542.5,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			20.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A46M1_125MM_ROUND
		//

		{
			//
			// customised for T80U (pitch limits -5.0 to 14.0 degrees)
			//

			"2A46M1 125mm Round",	  	  		  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			688,																	// soft_damage_capability
			688,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,																	// min_range
			4000.0,																// max_range
			3000.0,																// effective_range



			289.1,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			15.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_2A65_152MM_ROUND
		//

		{
			"2A65 152mm Round",	 	 	  		  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			1548,																	// soft_damage_capability
			1548,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			0.0,	 																// min_range
			24000.0,																// max_range
			12000.0,																// effective_range



			485.2,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// UNGUIDED ROCKETS
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_HYDRA70_M255
		//

		{
			"Hydra 70 M255",					  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			500,																	// soft_damage_capability
			120,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			100.0,																// min_range
			5000.0,																// max_range
			800.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_HYDRA70_M261
		//

		{
			"Hydra 70 M261",			 		  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			600,																	// soft_damage_capability
			400,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			100.0,																// min_range
			5000.0,																// max_range
			800.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_M73_HE
		//

		{
			"M73 HE submunition",			 		  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			600,																	// soft_damage_capability
			400,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			100.0,																// min_range
			5000.0,																// max_range
			800.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_MK_40_FFAR
		//

		{
			"LAU-69/A",	  						  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			500,																	// soft_damage_capability
			150,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			100.0,																// min_range
			5000.0,																// max_range
			750.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_S5
		//

		{
			"S-5",	  							  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			500,																	// soft_damage_capability
			150,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			100.0,																// min_range
			3000.0,																// max_range
			600.0,																// effective_range



			700.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_S8
		//

		{
			"S-8",								  								// full_name


			"RK",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			600,																	// soft_damage_capability
			400,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,				// weapon_class



			100.0,																// min_range
			4000.0,																// max_range
			600.0,																// effective_range



			750.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_S13
		//

		{
			"S-13",	  							  								// full_name


			"RK",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			800,																	// soft_damage_capability
			550,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			100.0,																// min_range
			6000.0,																// max_range
			600.0,																// effective_range



			800.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// AIR-TO-AIR GUIDED MISSILES
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_AIM92A_STINGER
		//

		{
			"AIM-92A Stinger",					  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			5000.0,																// max_range
			2500.0,																// effective_range



			680.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AIM9M_SIDEWINDER
		//

		{
			"AIM-9M Sidewinder",					  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,	 										// threat_type











			1000,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			18000.0,																// max_range
			5000.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AIM120C_AMRAAM
		//

		{
			"AIM-120C AMRAAM",		 				  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_ACTIVE_RADAR, 							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			1400,																	// soft_damage_capability
			50,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			65000.0,																// max_range
			5000.0,																// effective_range



			1300.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M39_IGLA_V
		//

		{
			"9K38 Igla-V",			  				  								// full_name


			"IG",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED, 					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,	 										// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			5200.0,																// max_range
			3000.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_R_60
		//

		{
			"R-60",						  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,	  				// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			1132,																	// soft_damage_capability
			40,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			8000.0,																// max_range
			5000.0,																// effective_range



			1000.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_R_60M
		//

		{
			"R-60M",						  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			1132,																	// soft_damage_capability
			40,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			15000.0,																// max_range
			5000.0,																// effective_range



			1000.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_R_27R
		//

		{
			"R-27R",						  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			1220,																	// soft_damage_capability
			40,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			40000.0,																// max_range
			5000.0,																// effective_range



			1500.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_R_27T
		//

		{
			"R-27T",						  							// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			1250,																	// soft_damage_capability
			40,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			20000.0,																// max_range
			5000.0,																// effective_range



			1500.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_R_73
		//

		{
			"R-73",																// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			1300,																	// soft_damage_capability
			40,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_AIR,										// weapon_class



			500.0,																// min_range
			30000.0,																// max_range
			5000.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// AIR-TO-SURFACE GUIDED MISSILES
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_AGM114L_LONGBOW_HELLFIRE
		//

		{
			"AGM-114L Longbow Hellfire", 									// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_ACTIVE_RADAR,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			2000,																	// soft_damage_capability
			2000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			8000.0,																// max_range
			3000.0,																// effective_range



			442.0,																// cruise_velocity


			23.5,				//X-PATCH//15.876,							// cruise_time


			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AGM114K_HELLFIRE_II
		//

		{
			"AGM-114K Hellfire II",											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_LASER_MISSILE,										// threat_type











			2000,																	// soft_damage_capability
			2000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			8000.0,																// max_range
			3000.0,																// effective_range



			442.0,																// cruise_velocity


			23.5,				//X-PATCH//15.876,							// cruise_time


			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AGM114R_HELLFIRE_II
		//

		{
			"AGM-114R Hellfire II",											// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_LASER_MISSILE,										// threat_type











			2000,																	// soft_damage_capability
			2000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			8000.0,																// max_range
			3000.0,																// effective_range



			442.0,																// cruise_velocity


			23.5,				//X-PATCH//15.876,							// cruise_time


			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AGM65D_MAVERICK
		//

		{
			"AGM-65D Maverick", 												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			3000,																	// soft_damage_capability
			3000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			24000.0,																// max_range
			6000.0,																// effective_range



			340.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_AGM65F_MAVERICK
		//

		{
			"AGM-65F Maverick", 												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			3000,																	// soft_damage_capability
			3000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			24000.0,																// max_range
			6000.0,																// effective_range



			340.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M120_ATAKA_V
		//

		{
			"9M120 Ataka-V",				 				 								// full_name


			"AK",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_RADIO_COMMAND,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			2000,																	// soft_damage_capability
			2000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			6000.0,																// max_range
			3000.0,																// effective_range



			400.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9K121_VIKHR
		//

		{
			"9K121 Vikhr",				 				 								// full_name


			"VK",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_LASER_MISSILE,										// threat_type











			2000,																	// soft_damage_capability
			2000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			10000.0,																// max_range
			3000.0,																// effective_range



			400.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M114_SHTURM
		//

		{
			"9K114 Shturm",				 				 						// full_name


			"SH",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_RADIO_COMMAND,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			744,								// soft_damage_capability
			744,								// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,					// weapon_class



			400.0,								// min_range
			5000.0,								// max_range
			3000.0,								// effective_range



			340.0,							// cruise_velocity





			0.0,							// burst_duration
			FIRE_SINGLE_WEAPON,					// rate_of_fire
			0.0,							// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_KH_25MT
		//

		{
			"Kh-25MT",						 				 						// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			1000,																	// soft_damage_capability
			1000,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			10000.0,																// max_range
			4000.0,																// effective_range



			340.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_KH_29L
		//

		{
			"Kh-29L",						 				 						// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_LASER_MISSILE,										// threat_type











			6354,																	// soft_damage_capability
			6354,																	// hard_damage_capability









			WEAPON_CLASS_AIR_TO_SURFACE,									// weapon_class



			500.0,																// min_range
			40000.0,																// max_range
			6000.0,																// effective_range



			340.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// SURFACE-TO-AIR GUIDED MISSILES
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_MIM_72G_CHAPARRAL
		//

		{
			"MIM-72G Chaparral",						  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			300,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			5000.0,																// max_range
			3500.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_FIM92A_STINGER
		//

		{
			"FIM-92A Stinger",				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			300,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			5000.0,																// max_range
			3500.0,																// effective_range



			680.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_RIM_66B_STANDARD
		//

		{
			"SM-1MR Standard",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			300,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			30000.0,																// max_range
			15000.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_RIM_7M_SEA_SPARROW
		//

		{
			"RIM-7P Sea Sparrow",													// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			300,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			55000.0,																// max_range
			15000.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9K34_STRELA_3
		//

		{
			"9K32 Strela 2",		  				  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			300,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			3200.0,																// max_range
			2000.0,																// effective_range



			510.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M333
		//

		{
			"9M333",						 			  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_IR_MISSILE,											// threat_type











			400,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			5000.0,																// max_range
			3500.0,																// effective_range



			510.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M311
		//

		{
			"9M311",					 			  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_RADIO_COMMAND,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			450,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			8000.0,																// max_range
			5000.0,																// effective_range



			890.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_4K60_STORM
		//

		{
			"M-11 Shtorm",															// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_RADIO_COMMAND,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			350,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			24000.0,																// max_range
			12000.0,																// effective_range



			952.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			25.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M33_OSA
		//

		{
			"9K33M",																// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_RADIO_COMMAND,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE,							// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			400,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_AIR,									// weapon_class



			500.0,																// min_range
			12000.0,																// max_range
			8000.0,																// effective_range



			850.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			24.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// SURFACE-TO-SURFACE GUIDED MISSILES
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_BGM_71D_TOW_2
		//

		{
			"BGM-71D TOW-2", 														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_WIRE_GUIDED,							// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			1017,																	// soft_damage_capability
			1017,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			100.0,																// min_range
			3750.0,																// max_range
			3000.0,																// effective_range



			300.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			30.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M113_KONKURS
		//

		{
			"9M113 Konkurs",	 	 												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR,					// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_RF_MISSILE,											// threat_type











			802,																	// soft_damage_capability
			802,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			100.0,																// min_range
			4000.0,																// max_range
			3000.0,																// effective_range



			300.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			30.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M117_BASTION
		//

		{
			"9M117 Bastion",														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER_BEAM_RIDING,	// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_LASER_MISSILE,										// threat_type











			440,																	// soft_damage_capability
			440,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			100.0,																// min_range
			5000.0,																// max_range
			3000.0,																// effective_range



			300.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			30.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_9M119M_INVAR
		//

		{
			"9M119 Svir",		 	 		 										// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER_BEAM_RIDING,	// guidance_type

			WEAPON_DECOY_TYPE_NONE,											// decoy_type
			WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK,			// warhead_type
			THREAT_TYPE_LASER_MISSILE,										// threat_type











			688,																	// soft_damage_capability
			688,																	// hard_damage_capability









			WEAPON_CLASS_SURFACE_TO_SURFACE,								// weapon_class



			100.0,																// min_range
			4000.0,																// max_range
			3000.0,																// effective_range



			300.0,																// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			30.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// DECOYS
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_CHAFF
		//

		{
			"CHAFF",	  							  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_CHAFF,  										// decoy_type
			WEAPON_WARHEAD_TYPE_NONE,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			0,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DECOY,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			25.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_FLARE
		//

		{
			"FLARE",	  							  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_FLARE,	 									// decoy_type
			WEAPON_WARHEAD_TYPE_NONE,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			0,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DECOY,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			50.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_SMOKE_GRENADE
		//

		{
			"SMOKE GRENADE",					  								// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_SMOKE_GRENADE,								// decoy_type
			WEAPON_WARHEAD_TYPE_SMOKE_GRENADE,							// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			0,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DECOY,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			6.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// CARGO
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_CRATE
		//

		{
			"CRATE",																// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,			 								// decoy_type
			WEAPON_WARHEAD_TYPE_NONE,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			0,																		// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_CARGO,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			0.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},


		//
		// DEBRIS
		//



		//
		// ENTITY_SUB_TYPE_WEAPON_HOKUM_BLADE
		//

		{
			"Rotor Blade",														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,			 								// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DEBRIS,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			20.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_HOKUM_PILOT
		//

		{
			"Ejected Pilot",													// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,			 								// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DEBRIS,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			60.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_HOKUM_CO_PILOT
		//

		{
			"Ejected Co-pilot",												// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,			 								// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DEBRIS,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			60.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_HOKUM_LH_DOOR
		//

		{
			"Crew Door",														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,			 								// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DEBRIS,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			8.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},

		//
		// ENTITY_SUB_TYPE_WEAPON_HOKUM_RH_DOOR
		//

		{
			"Crew Door",														// full_name


			"?",																// name abbreviation




			WEAPON_GUIDANCE_TYPE_NONE,										// guidance_type

			WEAPON_DECOY_TYPE_NONE,			 								// decoy_type
			WEAPON_WARHEAD_TYPE_SOLID_SHOT,										// warhead_type
			THREAT_TYPE_INVALID,												// threat_type











			550,																	// soft_damage_capability
			0,																		// hard_damage_capability









			WEAPON_CLASS_DEBRIS,												// weapon_class



			0.0,		 															// min_range
			0.0,																	// max_range
			0.0,																	// effective_range



			8.0,		 															// cruise_velocity





			0.0,																	// burst_duration
			FIRE_SINGLE_WEAPON,												// rate_of_fire
			0.0,																	// reload_time









			0.0,																	// detonation_radius
			0.0,																	// rearming_time

			0.0,																	// max_altitude	
		},
	};
















































































































































































weapon_package
	weapon_config_database[NUM_WEAPON_CONFIG_TYPES][NUM_WEAPON_PACKAGES] =
	{

		//
		// INCLUDE ORDER IS CRITICAL
		//

















































		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_M1025_HUMVEE










		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_M998_HUMVEE
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_UAZ469B
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_M923A1_BIG_FOOT
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_M923A1_BIG_FOOT_COVERED
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_URAL_4320
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_M978_HEMTT
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_URAL_FUEL_TANKER




		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_LCU
		// unarmed									// ENTITY_SUB_TYPE_VEHICLE_LCAC

















	};





muzzle_flash_animation
	muzzle_flash_animation_database[NUM_MUZZLE_FLASH_TYPES] =
	{
		// MUZZLE_FLASH_INVALID
		{




		},
		// MUZZLE_FLASH_SMALL_CALIBRE_STRAIGHT
		{




		},
		// MUZZLE_FLASH_SMALL_CALIBRE_FLARED
		{




		},
		// MUZZLE_FLASH_MEDIUM_CALIBRE_STRAIGHT
		{




		},
		// MUZZLE_FLASH_MEDIUM_CALIBRE_FLARED
		{




		},
		// MUZZLE_FLASH_LARGE_CALIBRE_STRAIGHT
		{




		},
		// MUZZLE_FLASH_LARGE_CALIBRE_FLARED
		{




		},
		// MUZZLE_FLASH_SMALL_ROCKET
		{




		},
		// MUZZLE_FLASH_MEDIUM_ROCKET
		{




		},
		// MUZZLE_FLASH_LARGE_ROCKET
		{




		},
	};





vec3d
	target_point_offsets[OBJECT_3D_LAST];























































































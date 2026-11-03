

































































struct WEAPON_DATA
{
	char
		*full_name;
	const char
		*hud_name,
		*mfd_name,
		*abbrev,
		*weapon_loading_list_name,
		*weapon_loading_name_text;

	object_3d_index_numbers


	weapon_guidance_types
		guidance_type;

	weapon_aiming_types


	weapon_decoy_types
		decoy_type;

	weapon_warhead_types
		warhead_type;

	threat_types
		threat_type;

	entity_sub_types


	sound_sample_indices




	int





		report_ammo_low_count;

	int

		damage_capability,
		armor_penetration_capability,







		flight_profile_or_self_destr,		// for Hellfire, Vikhr and self destruction weapons


	unsigned int
		weapon_class;

	float


		min_range,							// metres
		max_range,							// metres
		effective_range,					// metres
		circular_error_probable,			// max range error/max range


		cruise_velocity,					// meters/second

		boost_time,                    // seconds



		burst_duration,					// seconds
		rate_of_fire,						// rounds/minute
		reload_time,						// seconds


		drag_coefficient,                    // projectile drag, used for calculating deceleration






		detonation_radius,				// meters
		rearming_time,					// seconds

		max_altitude;					// meters
};

typedef struct WEAPON_DATA weapon_data;




//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern weapon_data
	weapon_database[NUM_ENTITY_SUB_TYPE_WEAPONS];





extern void validate_weapon_database (void);





extern weapon_package
	weapon_config_database[NUM_WEAPON_CONFIG_TYPES][NUM_WEAPON_PACKAGES];





extern muzzle_flash_animation
	muzzle_flash_animation_database[NUM_MUZZLE_FLASH_TYPES];





extern vec3d
	target_point_offsets[OBJECT_3D_LAST];





extern void initialise_target_point_offsets (void);





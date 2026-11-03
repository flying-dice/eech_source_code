

































































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
		default_3d_shape;

	weapon_guidance_types
		guidance_type;

	weapon_aiming_types
		aiming_type;

	weapon_decoy_types
		decoy_type;

	weapon_warhead_types
		warhead_type;

	threat_types
		threat_type;

	entity_sub_types
		launch_sound_effect_sub_type;

	sound_sample_indices
		interior_launch_sound_effect,
		exterior_launch_sound_effect,
		continuous_sound_effect;

	int
		weapon_selected_cpg_speech,
		weapon_low_cpg_speech,
		weapon_out_cpg_speech,
		weapon_launched_cpg_speech,
		weapon_launched_wingman_speech,
		report_ammo_low_count;

	int
		gun_shake,
		damage_capability,
		armor_penetration_capability,
		smoke_trail_type,
		acquire_parent_forward_velocity,
		ignore_gravity,
		survive_ground_impact,
		in_flight_collision_check,
		viewable_weapon,
		boresight_weapon,					// meaningful for featured helicopters only
		flight_profile_or_self_destr,		// for Hellfire, Vikhr and self destruction weapons
		tracer_color;

	unsigned int
		weapon_class;

	float
		start_weight,								// kilogrammes
		final_weight,								// kilogrammes
		min_range,							// metres
		max_range,							// metres
		effective_range,					// metres
		circular_error_probable,			// max range error/max range
		muzzle_velocity,              // meters/second
		muzzle_velocity_max_error,		// meters/second
		cruise_velocity,					// meters/second
		g_max,                        // g
		boost_time,                    // seconds
		cruise_time,                  // seconds
		cruise_time_max_error,        // seconds
		inhibit_time,						// seconds
		burst_duration,					// seconds
		rate_of_fire,						// rounds/minute
		reload_time,						// seconds
		max_launch_angle_error,			// radians
		max_seeker_limit,				// cos of angle
		drag_coefficient,                    // projectile drag, used for calculating deceleration
		inverse_rate_of_fire,           // 1.0 / rate_of_fire
		flightpath_deviation,			// meters
		boost_power,					// newton * s (kilogrammes * m / s^2)
		sustain_power,					// newton * s (kilogrammes * m / s^2)
		sustain_time,					// seconds
		diameter,						// meters
		detonation_radius,				// meters
		rearming_time,					// seconds
		tracer_fire_time,					// seconds
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





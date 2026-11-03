

































































struct WEAPON_DATA
{
	char
		*full_name;
	const char






	object_3d_index_numbers


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


	sound_sample_indices




	int







	int

		damage_capability,
		armor_penetration_capability,










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
		max_launch_angle_error,			// radians








		detonation_radius,				// meters
		rearming_time,					// seconds

		max_altitude;					// meters
};

typedef struct WEAPON_DATA weapon_data;




//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern weapon_data
	weapon_database[NUM_ENTITY_SUB_TYPE_WEAPONS];











extern weapon_package
	weapon_config_database[NUM_WEAPON_CONFIG_TYPES][NUM_WEAPON_PACKAGES];












extern vec3d
	target_point_offsets[OBJECT_3D_LAST];











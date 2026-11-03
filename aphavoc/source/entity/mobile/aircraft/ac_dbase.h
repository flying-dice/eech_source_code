

































































struct AIRCRAFT_DATA
{
	const char
		*full_name,
		*long_display_name,
		*short_display_name,
		*easy_display_name;

	object_3d_index_numbers
		default_3d_shape;

	gunship_types
		gunship_type;

	entity_forces
		force;

	weapon_config_types
		default_weapon_config_type,
		min_weapon_config_type,
		max_weapon_config_type,
		air_to_air_weapon_config_type,
		air_to_surface_weapon_config_type,
		scout_weapon_config_type;

	entity_sub_types
		default_weapon_type;

	target_symbol_types
		target_symbol_type;

	target_priority_types
		target_priority_type;

	force_info_catagories
		force_info_catagory;

	threat_types
		threat_type;

	view_categories
		view_category;

	view_types
		view_type;

	int
		map_icon,
		armor_level;

	float
		centre_of_gravity_to_ground_distance,
		centre_of_gravity_to_ground_distance_destroyed,
		chase_view_min_distance,
		chase_view_max_distance,
		chase_view_min_distance_test,
		chase_view_max_distance_test,
		cruise_velocity,
		cruise_altitude,
		attack_altitude,
		destroyed_bank_offset,
		destroyed_pitch_offset,
		fuselage_angle,
		liftoff_velocity,
		main_rotor_shaft_angle,
		main_rotor_direction,
		main_rotor_blade_droop_angle,
		propellor_direction,
		max_turn_rate,
		max_roll,
		max_vertical_turn_allowance,
		avoidance_radius,
		g_max,
		power_output,
		tail_rotor_direction,
		recon_radius,
		target_scan_delay,
		air_scan_range,
		surface_scan_range,
		potential_air_to_air_threat,
		potential_air_to_surface_threat,
		ammo_economy,
		fuel_default_weight,
		fuel_economy,
		// Xhit: added "number_of_main_rotors" for downwash effect. (030328) 
		// The reason why it is a float is because that for some reason all integer values are packed
		// where float can be choosen not to be packed. 
		number_of_main_rotors,
		// Xhit: added "main_rotor_radius" for downwash effect. (030328) 
		main_rotor_radius;
	int
		initial_damage_level;

	unsigned int
		explosive_quality 	: NUM_EXPLOSIVE_QUALITY_BITS,
		explosive_power 		: NUM_EXPLOSIVE_POWER_BITS,
		offensive_capability	: 1,
		player_controllable	: 1,
		points_value			: 12;

	sound_sample_indices
		continuous_sound_effect_index,
		wind_up_sound_effect_index,
		wind_down_sound_effect_index;
};

typedef struct AIRCRAFT_DATA aircraft_data;





//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern aircraft_data
	aircraft_database[NUM_ENTITY_SUB_TYPE_AIRCRAFT];







































































struct AIRCRAFT_DATA
{
	const char
		*full_name,




	object_3d_index_numbers


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


	target_priority_types
		target_priority_type;

	force_info_catagories
		force_info_catagory;

	threat_types
		threat_type;

	view_categories


	view_types


	int

		armor_level;

	float






		cruise_velocity,
		cruise_altitude,
		attack_altitude,








		max_turn_rate,






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

		// Xhit: added "main_rotor_radius" for downwash effect. (030328) 

	int
		initial_damage_level;

	unsigned int
		explosive_quality 	: NUM_EXPLOSIVE_QUALITY_BITS,
		explosive_power 		: NUM_EXPLOSIVE_POWER_BITS,
		offensive_capability	: 1,
		player_controllable	: 1,
		points_value			: 12;

	sound_sample_indices



};

typedef struct AIRCRAFT_DATA aircraft_data;





//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern aircraft_data
	aircraft_database[NUM_ENTITY_SUB_TYPE_AIRCRAFT];





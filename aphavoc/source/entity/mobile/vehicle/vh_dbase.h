

































































struct VEHICLE_DATA
{
	const char
		*full_name,




	object_3d_index_numbers


	entity_forces
		force;

	weapon_config_types
		default_weapon_config_type,
		min_weapon_config_type,
		max_weapon_config_type;

	entity_sub_types
		default_weapon_type;

	target_types
		target_type;

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


	int
		carries_smoke_grenades;

	float




		cruise_velocity,


		max_turn_rate,


		recon_radius,
		target_scan_delay,
		air_scan_floor,
		air_scan_ceiling,
		air_scan_range,
		surface_scan_range,
		potential_surface_to_air_threat,
		potential_surface_to_surface_threat,
		ammo_economy,
		fuel_economy,
		armour_front,
		armour_side,
		armour_rear;

	int
		initial_damage_level,
		armor_level;

	unsigned int
		explosive_quality 	: NUM_EXPLOSIVE_QUALITY_BITS,
		explosive_power 		: NUM_EXPLOSIVE_POWER_BITS,
		offensive_capability	: 1,
		points_value			: 12;

	sound_sample_indices


};

typedef struct VEHICLE_DATA vehicle_data;




//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern vehicle_data
	vehicle_database[NUM_ENTITY_SUB_TYPE_VEHICLES];





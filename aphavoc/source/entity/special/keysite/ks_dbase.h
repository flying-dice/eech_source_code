

































































struct KEYSITE_DATA
{
   const char
		*full_name,
		*short_name;

	supply_type
		default_supply_usage;

	float
		importance,
		minimum_efficiency,
		importance_radius,
		air_coverage_radius,
		recon_distance,
		sector_side_max_value;

	unsigned int
		map_layer_type				: 6,
		map_icon						: NUM_MAP_ICON_BITS,
		align_with_terrain		: NUM_ALIGN_WITH_TERRAIN_BITS,
		unique_name					: 1,
		hidden_by_fog_of_war		: 1,
		air_force_capacity		: 2,
		report_ammo_level		: 1,
		report_fuel_level		: 1,

		assign_task_count			: 2,
		reserve_task_count		: 2,
		requires_cap				: 1,
		requires_barcap			: 1,
		repairable					: 1,

		oca_target					: 1,
		recon_target				: 1,
		ground_strike_target		: 1,
		ship_strike_target		: 1,
		troop_insertion_target	: 1,
		campaign_objective		: 1;
};

typedef struct KEYSITE_DATA keysite_data;





//VJ WUT mod bug, date: 26-mar-03
//remove "const" from definition
extern keysite_data
   keysite_database[NUM_ENTITY_SUB_TYPE_KEYSITES];





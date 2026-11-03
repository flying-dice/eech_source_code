

































































struct CARGO
{
	mobile
		mob;

	list_root
		task_dependent_root;

	list_link
		cargo_link,
		movement_dependent_link;

	terrain_3d_point_data
		terrain_info;
};

typedef struct CARGO cargo;





#define CARGO_AMMO_SIZE 10

#define CARGO_FUEL_SIZE 10































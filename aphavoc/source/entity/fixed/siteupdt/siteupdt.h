

































































struct SITE_UPDATABLE
{
	fixed
		fix;

	float
		loading_door_timer,
		loading_door_state,
		radar_rotation_state,
		ground_crew_animation;

	int
		damage_level;

	list_link
		member_link,
		update_link;
};

typedef struct SITE_UPDATABLE site_updatable;































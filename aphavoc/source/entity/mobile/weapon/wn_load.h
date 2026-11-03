

































































enum WEAPON_LOADING_HARDPOINT_TYPES
{
	WEAPON_LOADING_HARDPOINT_GUN1,
	WEAPON_LOADING_HARDPOINT_GUN2,
	WEAPON_LOADING_HARDPOINT_INNER,
	WEAPON_LOADING_HARDPOINT_MID,
	WEAPON_LOADING_HARDPOINT_OUTER,
	WEAPON_LOADING_HARDPOINT_WINGTIP,

	NUM_WEAPON_LOADING_HARDPOINT_TYPES
};

typedef enum WEAPON_LOADING_HARDPOINT_TYPES weapon_loading_hardpoint_types;





extern void initialise_weapon_loading_gunship_database (void);

extern void deinitialise_weapon_loading_gunship_database (void);

extern void deactivate_weapon_payload_markers (entity *en);

extern void weapon_loading_update_currently_selected_weapons (entity *en);

extern entity_sub_types weapon_loading_get_current_hardpoint_weapon (gunship_types gunship_type, weapon_loading_hardpoint_types hardpoint);

extern void weapon_loading_select_next_weapon (entity *en, weapon_loading_hardpoint_types hardpoint);

extern int weapon_loading_get_valid_weapon_count (entity *en, weapon_loading_hardpoint_types hardpoint);

extern void notify_weapon_loading_weapon_config_change (entity *en);

extern int get_weapon_loading_hardpoint_weapon_count (entity *en, weapon_loading_hardpoint_types hardpoint, entity_sub_types weapon_type);






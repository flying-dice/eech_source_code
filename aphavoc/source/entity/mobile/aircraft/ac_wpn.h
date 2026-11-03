

































































enum AIRCRAFT_FIRE_RESULT
{
	AIRCRAFT_FIRE_OK								= (1 << 0),
	AIRCRAFT_FIRE_WEAPON_SYSTEM_NOT_READY	= (1 << 1),
	AIRCRAFT_FIRE_NO_WEAPON						= (1 << 2),
	AIRCRAFT_FIRE_NO_TARGET						= (1 << 3),
	AIRCRAFT_FIRE_NO_LOS							= (1 << 4),
	AIRCRAFT_FIRE_SUPPRESSED					= (1 << 5),

	AIRCRAFT_FIRE_CHECK_ALL						= ((1 << 6) - 1),

	NUM_AIRCRAFT_FIRE_RESULTS					= (1 << 7)
};

typedef enum AIRCRAFT_FIRE_RESULT aircraft_fire_result;





extern aircraft_fire_result aircraft_fire_weapon (entity *en, unsigned int check_flags, int increase_accuracy);

extern void update_aircraft_weapon_fire (entity *en);

extern void update_aircraft_decoy_release (entity *en);

extern int aircraft_select_best_weapon (entity *en, entity *target);





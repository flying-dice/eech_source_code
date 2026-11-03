

































































enum BEST_WEAPON_SELECTION_CRITERIA
{
	BEST_WEAPON_CRITERIA_MINIMAL			=	0,

	BEST_WEAPON_ALTITUDE_CHECK				=	1 << 0,
	BEST_WEAPON_LOS_CHECK					= 	1 << 1,
	BEST_WEAPON_RANGE_CHECK					=	1 << 2,
	BEST_WEAPON_CRITERIA_ALL				=	(1 << 3) - 1
//	BEST_WEAPON_SNAPSHOT					=   1 << 3,
//	BEST_WEAPON_CRITERIA_ALL				=	(1 << 4) - 1
};

typedef enum BEST_WEAPON_SELECTION_CRITERIA best_weapon_selection_criteria;





extern entity_sub_types get_best_weapon_for_target (entity *launcher, entity *target, unsigned int criteria);

extern float get_local_entity_max_weapon_range (entity *launcher);

extern float get_local_group_max_weapon_range (entity *group);





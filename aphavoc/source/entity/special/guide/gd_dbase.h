

































































struct GUIDE_DATA
{
	unsigned int
		evade_fire				: 1,
		follow_formation		: 1,
		position_type			: NUM_GUIDE_POSITION_TYPE_BITS,
		terrain_follow_mode	: NUM_GUIDE_TERRAIN_FOLLOW_BITS;

	guide_criteria_element
		criteria [NUM_GUIDE_CRITERIA_TYPES];
};

typedef struct GUIDE_DATA guide_data;





//VJ 040321 GWUT change
//remove "const" from definition
extern guide_data
	guide_database[NUM_ENTITY_SUB_TYPE_GUIDES];






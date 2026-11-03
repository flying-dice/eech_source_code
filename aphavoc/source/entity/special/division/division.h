

































































struct DIVISION
{
	entity_sub_types
		sub_type;

	int
		division_id;

	list_root
		division_root;

	list_link
		division_link,
		division_headquarters_link;

	char
		division_name [STRING_TYPE_DIVISION_NAME_MAX_LENGTH + 1];
};

typedef struct DIVISION division;





enum PLATOON_ID_TYPES
{
	PLATOON_ID_NONE,

	PLATOON_ID_NUMBER,
	PLATOON_ID_LETTER,
	PLATOON_ID_CALLSIGN,
	PLATOON_ID_KEYSITE,

	NUM_PLATOON_ID_TYPES
};





enum PLATOON_SHORT_NAME_TYPES
{
	PLATOON_SHORT_NAME_INVALID,
	
	PLATOON_SHORT_NAME_CALLSIGN,
	PLATOON_SHORT_NAME_ARMOUR,
	PLATOON_SHORT_NAME_SHIPS,
	PLATOON_SHORT_NAME_KEYSITE,

	NUM_PLATOON_SHORT_NAME_TYPES
};





extern entity *create_new_division (entity_sub_types type, entity_sides side, entity *parent, entity *hq, int local_only);

extern entity *add_group_to_division (entity *group, entity *known_division);



extern int set_local_division_name (entity *en, char *s);

extern void initialise_division_database (void);

extern void deinitialise_division_database (void);





extern void add_division_id_data (int side, int type, int count, int *number_list);

extern int get_next_free_division_id (int side, int type);





#include "dv_dbase.h"





















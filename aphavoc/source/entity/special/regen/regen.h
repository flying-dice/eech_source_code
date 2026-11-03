

































































struct REGEN
{

	entity_sub_types
		sub_type;

	vec3d
		position;

	list_root
		member_root;

	list_link
		current_waypoint_link,
		regen_link,
		update_link;

	float
		sleep;

	unsigned int
		alive							:NUM_ALIVE_BITS,
		regen_creation_sub_type	:NUM_ENTITY_SUB_TYPE_BITS,
		side							:NUM_SIDE_BITS;
};

typedef struct REGEN regen;





enum REGENERATION_BUILDING_TYPES
{

	REGENERATION_INVALID,
	REGENERATION_NONE,
	REGENERATION_FIXEDWING,
	REGENERATION_HELICOPTER,
	REGENERATION_ROUTED_VEHICLE,
	REGENERATION_PEOPLE,
	REGENERATION_SHIP_VEHICLE,
};

typedef enum REGENERATION_BUILDING_TYPES regeneration_building_types;





struct REGEN_LIST_ELEMENT
{
	int
		type,
		sub_type,
		group;

};

typedef struct REGEN_LIST_ELEMENT regen_list_element;

struct REGEN_MANAGEMENT_ELEMENT
{
	int
		size,
		count,
		front;
};

typedef struct REGEN_MANAGEMENT_ELEMENT regen_management_element;





extern int
	regen_to_landing_conversion [];





#define REGEN_QUEUE_DEFAULT_SIZE 5

#define REGEN_QUEUE_MINIMUM_SIZE REGEN_QUEUE_DEFAULT_SIZE





extern regen_list_element
	*regen_queue [NUM_ENTITY_SIDES][NUM_ENTITY_SUB_TYPE_REGENS];

extern regen_management_element
	regen_manager [NUM_ENTITY_SIDES][NUM_ENTITY_SUB_TYPE_REGENS];





#define REGEN_UPDATE_SLOW		1.5

#define REGEN_UPDATE_MEDIUM	1.0

#define REGEN_UPDATE_FAST		0.5

extern float
	regen_frequency [NUM_ENTITY_SIDES];













#include "rg_int.h"







#include "rg_updt.h"







extern void initialise_object_3d_regeneration_database ( void );



extern enum ENTITY_SUB_TYPE_REGEN get_object_3d_regeneration_type ( object_3d_index_numbers object, int *landing_sub_type );

extern float get_regen_frequency_difficulty_modifier (void);





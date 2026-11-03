



































































#include "popread.h"



#include "ai/ai_misc/ai_misc.h"





enum FRONTLINE_FORCE_PLACEMENT_TYPES
{

	FRONTLINE_FORCE_NONE,
	FRONTLINE_FORCE_PRIMARY,
	FRONTLINE_FORCE_SECONDARY,
	FRONTLINE_FORCE_ARTILLERY,
	NUM_FRONTLINE_FORCE_TYPES
};

typedef enum FRONTLINE_FORCE_PLACEMENT_TYPES frontline_force_placement_types;





struct FRONTLINE_FORCES_PLACEMENT_DATA
{

	unsigned int
		side						: NUM_SIDE_BITS,
		route_node				: NUM_ROUTE_NODE_BITS,
		force_placement_type	: NUM_FRONTLINE_FORCE_TYPES;
};

typedef struct FRONTLINE_FORCES_PLACEMENT_DATA frontline_forces_placement_data;





/* forward declarations for types used below */
struct SESSION_LIST_DATA_TYPE;
/*********************************************/

extern void create_faction (entity_sides side, int colour);

extern void create_campaign (struct SESSION_LIST_DATA_TYPE *session);

extern void clear_factions (void);

extern void destroy_campaign (void);

extern entity *create_faction_members (entity *keysite, entity_sub_types group_type, formation_component_types formation, int number, entity_sides side, vec3d *position, int client_server_create, int place_at_ground_height);

extern entity *create_landed_faction_members (entity *this_keysite, entity_sub_types member_type, entity_sub_types group_type, int number);

extern entity *create_landed_faction_group (entity *keysite, entity_sub_types group_type, formation_component_types formation_component);

extern entity *create_landing_faction_members (entity *keysite, entity_sub_types member_type, entity_sub_types group_type, int number, entity *cwp, vec3d *start_position);

extern void place_frontline_forces (entity *force, int force_size);





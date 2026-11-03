

































































#define LANDING_ROUTE_CHECK (1 && DEBUG)

#define LANDING_LOCK_USER_ONLY 0

#define LANDING_KEYSITE_DEBUG_OUTPUT 0





struct LANDING
{

	entity_sub_types
		sub_type;

	vec3d
		position;

	list_root
		task_dependent_root,
		assigned_task_root,
		takeoff_queue_root;

	list_link
		#if LANDING_ROUTE_CHECK

		update_link,

		#endif
		landing_site_link;

	int
		route_node;

	unsigned int
		reserved_landing_sites,
		free_landing_sites,
		total_landing_sites;

	unsigned int
		inside_hangar	: NUM_INSIDE_HANGAR_BITS,
		landing_lock	: NUM_FORMATION_BITS,
		landed_lock		: NUM_FORMATION_BITS,
		takeoff_lock	: NUM_FORMATION_BITS;
};

typedef struct LANDING landing;

















#include "ld_msgs.h"



#include "ld_str.h"









extern entity *get_local_entity_landing_entity (entity *en, entity_sub_types landing_type);

extern entity *get_local_landing_entity_route (entity *landing_en, entity_sub_types type);

extern entity *get_local_landing_entity_task (entity *landing_en, entity_sub_types type);

extern entity *get_local_group_member_landing_entity_from_keysite (entity *en);

extern entity *get_local_group_member_landing_entity_from_task (entity *en);



extern int check_available_landing_route_lock (entity *landing_en);

extern int check_landing_route_lock_clear (entity *landing_en, entity *en);

extern int check_takeoff_route_lock_clear (entity *landing_en, entity *en);

extern int check_landing_site_lock_clear (entity *landing_en, entity *en);

extern void release_mobile_entity_landing_locks (entity *en);







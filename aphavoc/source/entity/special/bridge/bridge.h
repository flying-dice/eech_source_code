

































































#define BRIDGE_DATA_FILENAME			"route\\bridge.dat"
#define VALID_BRIDGE_WARZONE_FILE	"route\\bridge.pop"





struct BRIDGE
{
	entity_sub_types
		sub_type;

	int
		alive,
		start_node,
		end_node;

	float
		pitch,
		scale;				// for leg sections

	list_root
		segment_root,
		task_dependent_root;
};

typedef struct BRIDGE bridge;





#include	"br_creat.h"

#include	"br_dbase.h"

#include	"br_dstry.h"

#include	"br_float.h"

#include	"br_funcs.h"

#include	"br_int.h"

#include	"br_list.h"

#include	"br_msgs.h"

#include	"br_pack.h"

#include	"br_str.h"

#include	"br_vec3d.h"





extern void initialise_valid_warzone_bridge_database (void);

extern entity *create_bridge( vec3d *end1, vec3d *end2, int start_node, int end_node, entity_sub_types type, float max_leg_height );

extern void create_local_bridge_entities (pack_modes pack_mode);

extern entity_sub_types get_most_suitable_bridge_type (vec3d *end1, vec3d *end2, float largest_depth);

extern float get_bridge_max_leg_height (vec3d *end1, vec3d *end2, entity_sub_types type);

extern void repair_client_server_bridge_entity (entity *en);

extern void repair_local_bridge_entity (entity *en);






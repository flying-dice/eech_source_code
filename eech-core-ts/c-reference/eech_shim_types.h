/*
 * Harness shim: the entity runtime environment of the extracted EECH
 * functions.
 *
 * This is NOT a port of the entity system. It supplies, for the scenario's
 * entities only, the accessor behaviour the ported TypeScript entity overloads
 * also implement (gp_int.c, ks_int.c, gp_vec3d.c, ...). The C reference
 * comparison therefore verifies the extracted functions (group.c,
 * keysite.c, force.c, range.c, en_msgs.c), not these accessors; the accessors
 * are verified by source reading (see docs/slices/assess-group-supplies.md).
 */

#ifndef EECH_SHIM_TYPES_H
#define EECH_SHIM_TYPES_H

typedef enum ENTITY_TYPES entity_types;
typedef enum ENTITY_SIDES entity_sides;
typedef enum LIST_TYPES list_types;
typedef enum INT_TYPES int_types;
typedef enum FLOAT_TYPES float_types;
typedef enum VEC3D_TYPES vec3d_types;
typedef enum PTR_TYPES ptr_types;
typedef enum ENTITY_MESSAGES entity_messages;
typedef int entity_sub_types;
typedef struct SUPPLY_TYPE supply_type;
typedef struct VEC3D vec3d;

typedef struct ENTITY
{
	entity_types
		type;

	int
		index;

	void
		*data;

	struct ENTITY
		*first_child[NUM_LIST_TYPES],
		*parent[NUM_LIST_TYPES],
		*child_succ[NUM_LIST_TYPES],
		*child_pred[NUM_LIST_TYPES];
} entity;

/* raw structs: only the members the extracted code and the shim use */

typedef struct
{
	entity_sides
		side;

	supply_type
		supplies;

	int
		resupply_source;		/* group_database [sub_type].resupply_source, supplied by the scenario */
} group;

typedef struct
{
	entity_sub_types
		sub_type;

	entity_sides
		side;

	int
		in_use;

	vec3d
		position;

	supply_type
		supplies;
} keysite;

typedef struct
{
	entity_sides
		side;
} force;

typedef struct
{
	vec3d
		position;
} mobile;

extern entity
	*session_entity;

#define get_session_entity() (session_entity)

extern int (*message_responses[NUM_ENTITY_TYPES][NUM_ENTITY_MESSAGES]) (entity_messages message, entity *receiver, entity *sender, va_list pargs);

void *get_local_entity_data (entity *en);
entity_types get_local_entity_type (entity *en);
entity *get_local_entity_first_child (entity *en, list_types type);
entity *get_local_entity_parent (entity *en, list_types type);
entity *get_local_entity_child_succ (entity *en, list_types type);
int get_local_entity_int_value (entity *en, int_types type);
float get_local_entity_float_value (entity *en, float_types type);
void set_client_server_entity_float_value (entity *en, float_types type, float value);
vec3d *get_local_entity_vec3d_ptr (entity *en, vec3d_types type);

int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...);
entity *get_local_force_entity (entity_sides side);
entity *get_closest_keysite (entity_sub_types type, entity_sides side, vec3d *pos, float min_range, float *actual_range, int outside_of_range, entity *exclude_keysite);
float get_2d_range (const vec3d *v1, const vec3d *v2);
float get_approx_2d_range (const vec3d *v1, const vec3d *v2);
void assess_group_supplies (entity *en);

#endif

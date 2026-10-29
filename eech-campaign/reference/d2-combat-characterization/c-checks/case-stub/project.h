/* A stand-in for the engine's project.h, just enough for csrc/eech_cmb_case.c (the controlled combat cases, EEDCS-46) to compile on its own for case_checks.c.  The campaign
   functions the driver calls are implemented by the check as a small model of the entity list, so that what the driver DECIDES (what it refuses before it launches, what it retires,
   what it mutates) can be asserted against the real driver code.  This is a check of the driver's guards; it is NOT a native run and says nothing about the campaign's physics. */
#ifndef CASE_STUB_PROJECT_H
#define CASE_STUB_PROJECT_H

#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef enum
{
	ENTITY_TYPE_UNKNOWN = 0,
	ENTITY_TYPE_ANTI_AIRCRAFT,
	ENTITY_TYPE_CITY_BUILDING,
	ENTITY_TYPE_FIXED_WING,
	ENTITY_TYPE_HELICOPTER,
	ENTITY_TYPE_OBJECT,
	ENTITY_TYPE_PERSON,
	ENTITY_TYPE_PYLON,
	ENTITY_TYPE_ROUTED_VEHICLE,
	ENTITY_TYPE_SCENIC,
	ENTITY_TYPE_SEGMENT,
	ENTITY_TYPE_SHIP_VEHICLE,
	ENTITY_TYPE_SITE,
	ENTITY_TYPE_SITE_UPDATABLE,
	ENTITY_TYPE_WEAPON,
	NUM_ENTITY_TYPES
} entity_types;

typedef enum { ENTITY_MESSAGE_COLLISION = 0, NUM_ENTITY_MESSAGES } entity_messages;

typedef enum
{
	INT_TYPE_ALIVE = 0,
	INT_TYPE_DAMAGE_LEVEL,
	INT_TYPE_INITIAL_DAMAGE_LEVEL,
	INT_TYPE_EXPLOSIVE_QUALITY,
	INT_TYPE_WEAPON_KILL_CODE,
	INT_TYPE_DAMAGE_CAPABILITY,
	NUM_INT_TYPES
} int_types;

typedef enum { LIST_TYPE_LAUNCHED_WEAPON = 0, LIST_TYPE_TARGET, NUM_LIST_TYPES } list_types;

typedef enum { VEC3D_TYPE_POSITION = 0 } vec3d_types;

enum
{
	WEAPON_KILL_CODE_OK = 0,
	WEAPON_KILL_CODE_EXHAUSTED,
	WEAPON_KILL_CODE_OUT_OF_BOUNDS,
	WEAPON_KILL_CODE_HIT_LAND,
	WEAPON_KILL_CODE_HIT_WATER,
	WEAPON_KILL_CODE_HIT_TARGET,
	WEAPON_KILL_CODE_OVERSHOT_TARGET,
	WEAPON_KILL_CODE_SELF_DESTRUCT
};

#define EXPLOSIVE_QUALITY_EXPLOSIVE 2
#define EXPLOSIVE_QUALITY_FLAMMABLE 4

typedef struct { float x, y, z; } vec3d;

struct ENTITY
{
	entity_types type;
	void *data;
};

typedef struct ENTITY entity;

/* the map the stand-in world has: the game's own test is `point_inside_map_area`, a plain rectangle on x and z */
#define MIN_MAP_X 0.0f
#define MAX_MAP_X 10000.0f
#define MIN_MAP_Z 0.0f
#define MAX_MAP_Z 10000.0f
#define point_inside_map_area(POS) (((POS)->x >= MIN_MAP_X) && ((POS)->x <= MAX_MAP_X) && ((POS)->z >= MIN_MAP_Z) && ((POS)->z <= MAX_MAP_Z))

extern entity *entities;
extern int number_of_entities;

entity *get_local_entity_list (void);
entity *get_local_entity_succ (entity *en);
int get_local_entity_int_value (entity *en, int_types type);
void set_client_server_entity_int_value (entity *en, int_types type, int value);
void set_local_entity_int_value (entity *en, int_types type, int value);
const char *campaign_roster_host_id (entity *en);
const char *get_entity_type_name (entity_types type);
entity *stub_first_child (entity *parent, list_types list);
entity *stub_child_succ (entity *child, list_types list);
entity *get_local_entity_parent (entity *en, list_types list);
void stub_vec3d_get (entity *en, vec3d_types type, vec3d *v);
void stub_vec3d_set (entity *en, vec3d_types type, vec3d *v);
void stub_destroy (entity *en);
void set_client_server_entity_parent (entity *en, list_types list, entity *parent);
void notify_local_entity (int message, entity *receiver, entity *sender, ...);
void create_client_server_weapon_killed_explosion_effect (entity *en);
int vehicle_critically_damaged (entity *en);

#define get_local_entity_first_child(EN, LIST) stub_first_child ((EN), (LIST))
#define get_local_entity_child_succ(EN, LIST) stub_child_succ ((EN), (LIST))
#define get_local_entity_vec3d(EN, TYPE, V) stub_vec3d_get ((EN), (TYPE), (V))
#define set_client_server_entity_vec3d(EN, TYPE, V) stub_vec3d_set ((EN), (TYPE), (V))
#define destroy_client_server_entity_family(EN) stub_destroy ((EN))

#endif

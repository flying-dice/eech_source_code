/* A stand-in for the engine's project.h, just enough for aphavoc/source/entity/mobile/mb_hostaim.c AND mb_hostobj.c to compile on their own (object_checks.c, EEDCS-46 R3).  The entity heap is a small array
   of plain structures whose attributes the checks set, and the engine operations the registry applies (set the damage level, kill) are functions of the checks that record what they were asked. */
#ifndef HOSTOBJ_STUB_PROJECT_H
#define HOSTOBJ_STUB_PROJECT_H

#include <limits.h>
#include <math.h>
#include <setjmp.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TRUE 1
#define FALSE 0

typedef struct vec3d
{
	float x, y, z;
} vec3d;

typedef enum
{
	ENTITY_TYPE_UNKNOWN = 0,
	ENTITY_TYPE_CITY_BUILDING,
	ENTITY_TYPE_PYLON,
	ENTITY_TYPE_SCENIC,
	ENTITY_TYPE_SEGMENT,
	ENTITY_TYPE_SITE,
	ENTITY_TYPE_SITE_UPDATABLE,
	ENTITY_TYPE_ROUTED_VEHICLE,
	ENTITY_TYPE_WEAPON,
	ENTITY_TYPE_BRIDGE
} entity_types;

typedef enum
{
	INT_TYPE_ALIVE = 1,
	INT_TYPE_DAMAGE_LEVEL,
	INT_TYPE_INITIAL_DAMAGE_LEVEL,
	INT_TYPE_ENTITY_SUB_TYPE,
	INT_TYPE_SIDE
} int_types;

typedef enum
{
	VEC3D_TYPE_POSITION = 1
} vec3d_types;

typedef enum
{
	COMMS_MODEL_SERVER = 0,
	COMMS_MODEL_CLIENT
} comms_model_types;

struct ENTITY
{
	int type;
	int alive;
	int damage_level;
	int initial_damage_level;
	int sub_type;
	int side;
	vec3d position;
	int has_position;
};

typedef struct ENTITY entity;

#define MAX_NUM_ENTITIES 64

extern struct ENTITY fake_entities[MAX_NUM_ENTITIES];
extern comms_model_types fake_comms_model;

#define get_local_entity_ptr(INDEX) (&fake_entities[(INDEX)])
#define get_local_entity_type(EN) ((entity_types) (EN)->type)
#define get_comms_model() (fake_comms_model)

int get_local_entity_index (entity *en);
int get_local_entity_int_value (entity *en, int_types type);
vec3d *get_local_entity_vec3d_ptr (entity *en, vec3d_types type);
void set_client_server_entity_int_value (entity *en, int_types type, int value);
void kill_client_server_entity (entity *en);

/* debug_fatal in the engine unwinds to the entry point; here it unwinds to the check that expects it */
extern jmp_buf harness_fatal;
extern char harness_message[512];

void debug_fatal (const char *format, ...);
int rand16 (void);
float get_delta_time (void);
unsigned int get_system_time (void);
const char *campaign_roster_host_id (entity *en);
int campaign_roster_find_host_id (const char *host_id, entity **member);

#endif

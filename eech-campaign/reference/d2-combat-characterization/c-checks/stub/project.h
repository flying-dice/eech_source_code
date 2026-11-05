/* A stand-in for the engine's project.h, just enough for csrc/eech_cmb_fixture.c (the combat observer, EEDCS-46) to compile on its own for observer_checks.c.  The
   functions that read entity fields are implemented by the check, which counts every read so that "the observer never read this entity" can be asserted. */
#ifndef CMB_STUB_PROJECT_H
#define CMB_STUB_PROJECT_H

#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef enum
{
	ENTITY_TYPE_UNKNOWN = 0,
	ENTITY_TYPE_ANTI_AIRCRAFT,
	ENTITY_TYPE_BRIDGE,
	ENTITY_TYPE_CARGO,
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

typedef enum
{
	ENTITY_MESSAGE_COLLISION = 0,
	ENTITY_MESSAGE_OTHER,
	NUM_ENTITY_MESSAGES
} entity_messages;

typedef enum
{
	INT_TYPE_DAMAGE_LEVEL = 0,
	INT_TYPE_ALIVE,
	INT_TYPE_PLAYER,
	INT_TYPE_ENTITY_SUB_TYPE,
	INT_TYPE_WEAPON_KILL_CODE,
	INT_TYPE_WEAPON_WARHEAD_TYPE,
	NUM_INT_TYPES
} int_types;

#define ENTITY_PLAYER_AI 0
#define NUM_WEAPON_KILL_CODES 8

struct ENTITY
{
	entity_types type;
	void *data;
};

typedef struct ENTITY entity;

extern entity *entities;
extern int number_of_entities;
extern int random_number_seed;

int get_local_entity_int_value (entity *en, int_types type);
const char *get_entity_type_name (entity_types type);

#endif

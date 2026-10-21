/*
 * EECH headless engine: the entry points a host calls. One engine per
 * process (EECH is global state); every entry sets the FPU to EECH's
 * round-toward-zero and restores the caller's environment on return.
 *
 * A debug_fatal inside EECH unwinds to the entry point, which returns
 * EECH_ENGINE_FATAL; the message is in eech_engine_fatal_message () and the
 * engine refuses every later call.
 */
#ifndef EECH_ENGINE_H
#define EECH_ENGINE_H

#include <stdint.h>
#include <stddef.h>

#include "entity/special/group/gp_roster.h"
#include "entity/special/guide/gd_supply_arrival.h"

#define EECH_ENGINE_OK 0
#define EECH_ENGINE_FATAL 1
#define EECH_ENGINE_POISONED 2
#define EECH_ENGINE_STATE 3
#define EECH_ENGINE_BAD_ARGUMENT 4

struct eech_engine_config
{
	/* the installation root: contains cohokum/ (the working directory) and common/ */
	const char *install_root;
	/* EECH paths, relative to cohokum/: e.g. "..\\common\\maps\\map16", "camp01", "georgia.chc" */
	const char *map_path;
	const char *campaign_directory;
	const char *campaign_filename;
	/* GUNSHIP_TYPE_* of the (server) player; its side is the player's side */
	int gunship_type;
	uint32_t random_seed;
	/* extra EECH.INI-style command line arguments, NULL-terminated (may be NULL) */
	const char *const *arguments;
};

#define EECH_HOST_ID_MAX CAMPAIGN_HOST_ID_MAX

typedef campaign_member_descriptor eech_member_descriptor;
typedef campaign_bind_member_fn eech_bind_mobile_callback;
typedef campaign_supply_reach_descriptor eech_supply_reach_descriptor;
typedef campaign_observe_supply_reach_fn eech_supply_reached_callback;

struct eech_engine_host
{
	/* Called synchronously while the physical entity exists. */
	eech_bind_mobile_callback bind_mobile;
	/* Called synchronously for the physical predicate at four SUPPLY waypoints. */
	eech_supply_reached_callback supply_reached;
	void *user;
};

int eech_engine_boot (const struct eech_engine_config *config, const struct eech_engine_host *host);

/* advance simulated time by milliseconds and run one iteration of EECH's flight loop */
int eech_engine_frame (uint32_t milliseconds, const struct eech_engine_host *host);

const char *eech_engine_fatal_message (void);

/*
 * Writes the generated part of an installation under root: the 3D object
 * database and texture names (cohokum/3ddata, eech_synth3d.c) and the
 * briefing texts (common/data/brief_en.dat). The retail versions of these
 * files are not in the repository. An installation that already holds the
 * retail 3D database (cohokum/3ddata/textures.pal) keeps it. Needs no booted
 * engine.
 */
int eech_engine_prepare_installation (const char *root);

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* observation (eech_observe.c) */

#define EECH_OBJECT_HELICOPTER 1
#define EECH_OBJECT_FIXED_WING 2
#define EECH_OBJECT_GROUND_VEHICLE 3
#define EECH_OBJECT_AIR_DEFENCE 4
#define EECH_OBJECT_SHIP 5
#define EECH_OBJECT_INFANTRY 6
#define EECH_OBJECT_WEAPON 7
#define EECH_OBJECT_KEYSITE 8

struct eech_object
{
	int id;			/* EECH entity index (reused after destruction) */
	int kind;		/* EECH_OBJECT_* */
	int sub_type;		/* EECH sub-type within the kind */
	int side;		/* 0 neutral, 1 blue, 2 red */
	int alive;
	int group_id;		/* the group's entity index; for weapons, the launcher; -1 */
	const char *type_name;	/* EECH database name ("AH-64D Apache Longbow", "KEYSITE_FARP") */
	const char *name;	/* group callsign or keysite name, or NULL */
	const char *task;	/* the group's primary task (TASK_*), or NULL */
	const char *state;	/* the group's verbose operational state ("En route", "Engaging", ...), or NULL */
	float x, y, z;		/* EECH world metres: x east, y up, z north */
	float heading, pitch, roll;	/* radians */
	float efficiency;	/* keysites */
	float ammo, fuel;	/* keysites: supply levels, percent */
	int usable;		/* keysites: KEYSITE_STATE_* (0 usable, 1 out of action, 2 repairing) */
};

typedef void (*eech_object_callback) (const struct eech_object *object, void *user);

/* calls back once per observed object and stores the count. Strings live until the next frame. */
int eech_engine_objects (eech_object_callback callback, void *user, int *count);

struct eech_member
{
	uint64_t member_key;
	uint64_t group_key;
	const char *host_id;
	int origin_entity_id;
	int current_entity_id;
	int entity_type;
	int sub_type;
	int side;
	int initial_alive;
	int physical_present;
	int in_group;
};

typedef void (*eech_member_callback) (const struct eech_member *member, void *user);

/* Durable campaign roster. Strings live for the engine session. */
int eech_engine_members (eech_member_callback callback, void *user, int *count);

struct eech_roster_stats
{
	uint64_t duplicate_comparisons;
	double duplicate_scan_seconds;
};

int eech_engine_roster_stats (struct eech_roster_stats *stats);

typedef campaign_supply_task_snapshot eech_supply_task;
typedef void (*eech_supply_task_callback) (const eech_supply_task *task, void *user);

/* Actual current SUPPLY task, cargo and endpoint state, copied during the call. */
int eech_engine_supply_tasks (eech_supply_task_callback callback, void *user, int *count);

struct eech_clock
{
	float elapsed_seconds, time_of_day_seconds;
	int day;
};

int eech_engine_clock (struct eech_clock *clock);

/*
 * Writes the campaign's force state to the log sink (stderr): per force its
 * hardware reserves and regen queues, per keysite its state, supplies and
 * regen sites, and per air group its mode, task and members.
 */
int eech_engine_diagnostics (void);

#endif

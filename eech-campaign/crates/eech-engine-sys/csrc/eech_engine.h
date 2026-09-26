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

int eech_engine_boot (const struct eech_engine_config *config);

/* advance simulated time by milliseconds and run one iteration of EECH's flight loop */
int eech_engine_frame (uint32_t milliseconds);

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

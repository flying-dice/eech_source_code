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
#include "entity/mobile/vehicle/routed/rv_hostmv.h"
#include "entity/mobile/mb_hostlos.h"
#include "entity/mobile/mb_hostsense.h"

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
typedef campaign_loss_descriptor eech_loss_descriptor;
typedef campaign_loss_task_dependent eech_loss_task_dependent;
typedef campaign_loss_landing_action eech_loss_landing_action;
typedef campaign_observe_loss_fn eech_observe_loss_callback;

#define EECH_LOSS_TASK_DEPENDENT_MAX CAMPAIGN_LOSS_TASK_DEPENDENT_MAX
#define EECH_LOSS_LANDING_ACTION_MAX CAMPAIGN_LOSS_LANDING_ACTION_MAX
#define EECH_LOSS_EVENT_CREDIT CAMPAIGN_LOSS_EVENT_CREDIT
#define EECH_LOSS_EVENT_PHYSICAL CAMPAIGN_LOSS_EVENT_PHYSICAL
#define EECH_LOSS_FORWARD CAMPAIGN_LOSS_FORWARD
#define EECH_LOSS_WITHHOLD CAMPAIGN_LOSS_WITHHOLD
#define EECH_LOSS_LANDING_UNRESERVE_SITE CAMPAIGN_LOSS_LANDING_UNRESERVE_SITE
#define EECH_LOSS_LANDING_UNLOCK_ROUTE CAMPAIGN_LOSS_LANDING_UNLOCK_ROUTE
#define EECH_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE CAMPAIGN_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE
#define EECH_LOSS_LANDING_UNLOCK_SITE CAMPAIGN_LOSS_LANDING_UNLOCK_SITE
typedef campaign_supply_reach_descriptor eech_supply_reach_descriptor;
typedef campaign_observe_supply_reach_fn eech_supply_reached_callback;
typedef campaign_movement_step eech_movement_step;
typedef campaign_movement_result eech_movement_result;
typedef campaign_move_vehicle_fn eech_move_vehicle_callback;
typedef campaign_los_fn eech_line_of_sight_callback;
typedef campaign_sense_fn eech_sense_callback;

struct eech_engine_host
{
	/* Called synchronously while the physical entity exists. */
	eech_bind_mobile_callback bind_mobile;
	/* Called synchronously at the original credit and physical-loss sites. */
	eech_observe_loss_callback observe_loss;
	/* Called synchronously for the physical arrival and proximity predicates: SUPPLY and mixed-criteria arrivals, waypoint action and recon radius. */
	eech_supply_reached_callback supply_reached;
	/* Called synchronously for each host-owned movement step (routed MOVE/STOP, SHIP_MOVE, PERSON_BEGIN/END, FIXED_WING_MOVE, HELICOPTER_BEGIN/END) and pose placement (PLACE); the host owns the mobile's position, speed and attitude. */
	eech_move_vehicle_callback move_vehicle;
	/* Called synchronously for each host-owned line-of-sight query (mb_hostlos.h): the host owns the calculation and asks for one terrain elevation or one object-ray answer at a time, or answers directly. */
	eech_line_of_sight_callback line_of_sight;
	/* Called synchronously for each host-owned physical sensing exchange of an AI target scan (mb_hostsense.h): the host owns whether and at what range a candidate is perceived (the reference host: range, envelope, tolerance, signature and emitter arithmetic and the state a scan keeps between candidates; another host: its own model); the positions and unit figures sent with it are transitional mirrors. */
	eech_sense_callback sense;
	void *user;
};

int eech_engine_boot (const struct eech_engine_config *config, const struct eech_engine_host *host);

/* advance simulated time by milliseconds and run one iteration of EECH's flight loop */
int eech_engine_frame (uint32_t milliseconds, const struct eech_engine_host *host);

/* Copy of what one report applied; an observation of performed actions, not engine state. */
struct eech_loss_report_receipt
{
	int landing_reserved_before, landing_reserved_after;
	int group_losses_before, group_losses_after;
	int primary_task_losses_before, primary_task_losses_after;
	int group_logical_members_before, group_logical_members_after, group_physical_members_before;
	int tasks_retired_expire, tasks_completed_on_report;
};

/*
 * Applies one previously withheld campaign loss, exactly once, by opaque host identity.
 * The receipt is written only when the report succeeds; on any other result it is untouched.
 */
int eech_engine_report_loss (const char *host_id, struct eech_loss_report_receipt *receipt);

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

#ifdef EECH_DEPENDENT_FIXTURE
/* TEST-ONLY (feature dependent-fixture): see eech_engine.c. Not part of the shipping module. */
#define EECH_FIXTURE_LINK 1
#define EECH_FIXTURE_MOVE 2
#define EECH_FIXTURE_SINK 3
int eech_engine_dependent_fixture (int op, int parent_id, int child_id, const double position[3], const struct eech_engine_host *host);
#endif

#ifdef EECH_LOSS_FIXTURE
/* TEST-ONLY (feature loss-fixture): see eech_engine.c. Not part of the shipping module. */
int eech_engine_loss_fixture (int entity_id, const struct eech_engine_host *host);
/* TEST-ONLY: releases the tasks that target this hosted member's group, as the campaign does when it kills a group; *released is how many. */
int eech_engine_loss_release (int entity_id, int *released, const struct eech_engine_host *host);
/* TEST-ONLY: one side's total losses, force-info current hardware and regen queue lengths, read-only. */
int eech_engine_loss_state (int side, int *values, int capacity, int *count);
/* TEST-ONLY: the reason a bounded family would refuse to withhold this hosted member's loss ("" if it would accept), read-only. */
int eech_engine_loss_shape (int entity_id, char *reason, int capacity);
/* TEST-ONLY: the task dependents and guide stack of this hosted member's group, as text, read-only. */
int eech_engine_loss_group (int entity_id, char *text, int capacity);
#endif

#ifdef EECH_LOS_FIXTURE
/*
 * TEST-ONLY (feature los-fixture, EEDCS-42): see eech_los_fixture.c. Not part of the shipping module.
 * A trace folds line-of-sight queries: how many, how many were visible, the same per criteria value, how many sweep
 * points were skipped for lying near the map edge, and an order-dependent FNV-1a hash over each query's criteria,
 * six position floats (bit patterns) and answer. lookup_hash is the same kind of hash over the solver's ordered environment
 * lookups (EEDCS-42 transcript): each terrain sample (phase, point, returned elevation), each object ray (phase, start, end,
 * hit, collision point when hit) and a query-end marker with the answer; lookups counts the terrain and object events.
 */
#define EECH_LOS_CRITERIA 32
struct eech_los_trace
{
	uint64_t hash;
	uint64_t calls;
	uint64_t visible;
	uint64_t skipped;
	uint64_t lookup_hash;
	uint64_t lookups;
	uint64_t by_criteria_calls[EECH_LOS_CRITERIA];
	uint64_t by_criteria_visible[EECH_LOS_CRITERIA];
};
/* The run-wide trace of every query the campaign has made since boot or the last reset. */
int eech_engine_los_trace (struct eech_los_trace *out);
int eech_engine_los_trace_reset (void);
/*
 * Runs the deterministic sweep between two live entities and returns its own trace in `out`; the run-wide trace is not
 * changed. `host` is the engine host, passed so the same entry serves a build whose queries cross the host boundary.
 */
int eech_engine_los_probe (int source_id, int target_id, uint32_t seed, int count, struct eech_los_trace *out, const struct eech_engine_host *host);
#endif

#ifdef EECH_SENSOR_FIXTURE
/*
 * TEST-ONLY (feature sensor-fixture, EEDCS-43): see eech_sensor_fixture.c. Not part of the shipping module.
 * The report is `key=value` text lines: the physical and decision hashes, per scan function the calls, the candidates and each
 * candidate's verdict, the eligibility skips, which edges the run exercised, the entity kinds that scan, and sample candidates.
 * Fails (bad argument) when the report does not fit `capacity`.
 */
/* The run-wide trace of every AI target scan the campaign has made since boot or the last reset. */
int eech_engine_sensor_trace (char *text, int capacity);
int eech_engine_sensor_trace_reset (void);
/* Runs the scan group functions for every eligible live source, read-only, and reports a trace of just that sweep; the run-wide trace is not changed.
 * The surface-to-air scan asks the host for line of sight, so `host` must carry every callback, as for a frame. */
int eech_engine_sensor_probe (char *text, int capacity, const struct eech_engine_host *host);
#endif

#endif

/*
 * Host-observed physical arrival and proximity.  One seam serves the SUPPLY
 * waypoints and every other mobile arrival (fixed-wing, helicopter, routed
 * vehicle, ship, anti-aircraft, person), the waypoint-action radius and the
 * helicopter recon radius.  The "supply" names are retained historical naming.
 *
 * Campaign code still selects the waypoint, criteria and required members,
 * and keeps the weapon-vector random tolerance.  The host receives copied
 * primitive facts and returns the physical gates plus, where the weapon
 * vector applies, the launch angle error.  No EECH pointer crosses this
 * boundary.
 */
#ifndef GD_SUPPLY_ARRIVAL_H
#define GD_SUPPLY_ARRIVAL_H

#include <stddef.h>
#include <stdint.h>

#define CAMPAIGN_SUPPLY_HOST_ID_MAX 128
#define CAMPAIGN_SUPPLY_ERROR_MAX 256

enum campaign_supply_waypoint_kind
{
	CAMPAIGN_SUPPLY_WAYPOINT_NONE,
	CAMPAIGN_SUPPLY_WAYPOINT_PICK_UP,
	CAMPAIGN_SUPPLY_WAYPOINT_PREPARE_FOR_DROP_OFF,
	CAMPAIGN_SUPPLY_WAYPOINT_DROP_OFF,
	CAMPAIGN_SUPPLY_WAYPOINT_FINISH_DROP_OFF,
	CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT	/* not a SUPPLY waypoint: any other waypoint (kept name; covers every mixed-criteria arrival) */
};

enum campaign_arrival_query
{
	CAMPAIGN_ARRIVAL_QUERY_REACHED = 1,	/* per-member reached-waypoint criteria */
	CAMPAIGN_ARRIVAL_QUERY_ACTION,		/* leader inside the waypoint action radius */
	CAMPAIGN_ARRIVAL_QUERY_RECON		/* helicopter inside the recon radius (strict, squared 2D) */
};

enum campaign_supply_mobile_kind
{
	CAMPAIGN_SUPPLY_MOBILE_FIXED_WING = 1,
	CAMPAIGN_SUPPLY_MOBILE_HELICOPTER = 2,
	CAMPAIGN_SUPPLY_MOBILE_ROUTED_VEHICLE = 3,
	CAMPAIGN_SUPPLY_MOBILE_SHIP_VEHICLE = 4,
	CAMPAIGN_SUPPLY_MOBILE_ANTI_AIRCRAFT = 5,
	CAMPAIGN_SUPPLY_MOBILE_PERSON = 6
};

typedef struct campaign_supply_reach_descriptor
{
	uint64_t member_key;
	uint64_t task_key;
	uint64_t waypoint_key;
	char host_id[CAMPAIGN_SUPPLY_HOST_ID_MAX + 1];
	int query;				/* campaign_arrival_query */
	int waypoint_kind;
	int waypoint_sub_type;		/* the campaign waypoint's entity sub-type */
	int player_skip_waypoint;	/* waypoint_database fact: a human member's arrival is waived */
	int landing_waypoint;		/* LANDED or TROOP_INSERT waypoint: an unassisted human must be on the ground */
	int mobile_kind;
	int is_task_leader;
	int is_ai;
	int leader_is_ai;
	int landed;
	int updated;
	float rotor_spin_up_seconds;
	float distance;
	float velocity;
	float delta_seconds;
	float radius;
	float position[3];
	float motion[3];
	float guide_position[3];
	float leader_position[3];
	/* mixed-criteria facts (QUERY_REACHED); all copied, never pointers */
	int radius_valid;
	int altitude_valid;
	int heading_valid;
	int last_to_reach_valid;
	int weapon_gate;		/* the weapon-vector rule applies to this member: the host reports an angle */
	int vectors_valid;		/* INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID and both vectors are present */
	int airborne;
	int auto_pilot;
	float altitude_criterion;
	float heading_criterion;
	float altitude;			/* member FLOAT_TYPE_ALTITUDE */
	float heading;			/* member FLOAT_TYPE_HEADING */
	float centre_of_gravity_to_ground;
	float guide_altitude;		/* guide entity position y */
	int required_heading_present;
	float required_heading;
	float weapon_vector[3];
	float intercept_vector[3];
	/* QUERY_ACTION / QUERY_RECON */
	float action_radius;
	float waypoint_position[3];
} campaign_supply_reach_descriptor;

/*
 * gates_passed: every physical gate that precedes the weapon-vector tolerance
 * held.  angle_present/launch_angle_error: the angle between the weapon and
 * intercept vectors, required when the descriptor has weapon_gate and
 * vectors_valid and gates_passed.
 */
typedef struct campaign_arrival_result
{
	int gates_passed;
	int angle_present;
	float launch_angle_error;
} campaign_arrival_result;

typedef int (*campaign_observe_supply_reach_fn)
(
	const campaign_supply_reach_descriptor *descriptor,
	campaign_arrival_result *result,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_supply_arrival_host
{
	campaign_observe_supply_reach_fn observe_reach;
	void *user;
} campaign_supply_arrival_host;

typedef struct campaign_supply_task_snapshot
{
	uint64_t task_key;
	uint64_t waypoint_key;
	char leader_host_id[CAMPAIGN_SUPPLY_HOST_ID_MAX + 1];
	int task_entity_index;
	int waypoint_entity_index;
	int waypoint_kind;
	int task_completed;
	int supplier_entity_index;
	int receiver_entity_index;
	int cargo_type;
	int cargo_on_leader;
	float supplier_ammo;
	float supplier_fuel;
	float receiver_ammo;
	float receiver_fuel;
} campaign_supply_task_snapshot;

typedef void (*campaign_supply_task_snapshot_fn) (const campaign_supply_task_snapshot *task, void *user);

struct ENTITY;

void campaign_supply_arrival_reset (void);
void campaign_supply_arrival_set_host (const campaign_supply_arrival_host *host);
void campaign_supply_arrival_release_entity (struct ENTITY *entity);
int campaign_supply_arrival_is_target (struct ENTITY *guide);
int campaign_supply_arrival_check (struct ENTITY *member, struct ENTITY *guide);
int campaign_radius_arrival_is_target (struct ENTITY *member, struct ENTITY *guide);
int campaign_radius_arrival_check (struct ENTITY *member, struct ENTITY *guide);
int campaign_waypoint_action_reached (struct ENTITY *guide, struct ENTITY *leader);
int campaign_recon_reached (struct ENTITY *member, struct ENTITY *waypoint);
int campaign_supply_task_snapshots (campaign_supply_task_snapshot_fn callback, void *user);

#endif

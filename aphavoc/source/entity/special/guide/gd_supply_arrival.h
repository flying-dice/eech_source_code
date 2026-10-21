/*
 * Host-observed physical arrival for the four SUPPLY waypoints and for any
 * other fixed-wing or helicopter waypoint whose only physical criterion is
 * RADIUS (alone or with LAST_TO_REACH).
 *
 * Campaign code still selects the waypoint, criteria and required members.
 * The host receives copied primitive facts and returns only the physical
 * per-member predicate.  No EECH pointer crosses this boundary.
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
	CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT	/* not a SUPPLY waypoint: any other radius-only waypoint */
};

enum campaign_supply_mobile_kind
{
	CAMPAIGN_SUPPLY_MOBILE_FIXED_WING = 1,
	CAMPAIGN_SUPPLY_MOBILE_HELICOPTER = 2
};

typedef struct campaign_supply_reach_descriptor
{
	uint64_t member_key;
	uint64_t task_key;
	uint64_t waypoint_key;
	char host_id[CAMPAIGN_SUPPLY_HOST_ID_MAX + 1];
	int waypoint_kind;
	int waypoint_sub_type;		/* the campaign waypoint's entity sub-type */
	int player_skip_waypoint;	/* waypoint_database fact: a human member's arrival is waived */
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
} campaign_supply_reach_descriptor;

typedef int (*campaign_observe_supply_reach_fn)
(
	const campaign_supply_reach_descriptor *descriptor,
	int *reached,
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
int campaign_supply_task_snapshots (campaign_supply_task_snapshot_fn callback, void *user);

#endif

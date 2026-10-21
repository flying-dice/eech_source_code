/*
 * Campaign member identities supplied by the selected host.
 *
 * The roster is append-only for one engine session.  A MEMBER unlink changes
 * membership, not physical identity: group reordering and amalgamation both
 * unlink live entities.  The physical incarnation ends only when the entity
 * heap releases its slot.
 */
#ifndef GP_ROSTER_H
#define GP_ROSTER_H

#include <stddef.h>
#include <stdint.h>

#define CAMPAIGN_HOST_ID_MAX 128
#define CAMPAIGN_HOST_ERROR_MAX 256
#define CAMPAIGN_LOSS_TASK_DEPENDENT_MAX 32
#define CAMPAIGN_LOSS_LANDING_ACTION_MAX 3

typedef enum campaign_loss_event_kind
{
	CAMPAIGN_LOSS_EVENT_CREDIT = 1,
	CAMPAIGN_LOSS_EVENT_PHYSICAL = 2
} campaign_loss_event_kind;

typedef enum campaign_loss_decision
{
	CAMPAIGN_LOSS_FORWARD = 1,
	CAMPAIGN_LOSS_WITHHOLD = 2
} campaign_loss_decision;

typedef enum campaign_loss_landing_action_kind
{
	CAMPAIGN_LOSS_LANDING_UNRESERVE_SITE = 1,
	CAMPAIGN_LOSS_LANDING_UNLOCK_ROUTE = 2,
	CAMPAIGN_LOSS_LANDING_UNLOCK_TAKEOFF_ROUTE = 3,
	CAMPAIGN_LOSS_LANDING_UNLOCK_SITE = 4
} campaign_loss_landing_action_kind;

typedef struct campaign_loss_task_dependent
{
	uint64_t task_key;
	int entity_type;
	int sub_type;
	int task_state;
	int task_completed;
	int primary_task;
	int persistent_task;
	int objective_is_member;
	int task_dependent_children;
	int guide_children;
	float expire_timer;
	float stop_timer;
} campaign_loss_task_dependent;

typedef struct campaign_loss_landing_action
{
	uint64_t landing_key;
	int action_kind;
	int count;
	int landing_sub_type;
	int reserved_landing_sites;
	int free_landing_sites;
	int landing_lock;
	int landed_lock;
	int takeoff_lock;
} campaign_loss_landing_action;

typedef struct campaign_member_descriptor
{
	uint64_t member_key;
	uint64_t group_key;
	int entity_index;
	int entity_type;
	int sub_type;
	int side;
} campaign_member_descriptor;

typedef int (*campaign_bind_member_fn)
(
	const campaign_member_descriptor *descriptor,
	char *host_id,
	size_t host_id_size,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_loss_descriptor
{
	const char *host_id;
	uint64_t member_key;
	uint64_t group_key;
	int event_kind;
	int entity_type;
	int sub_type;
	int side;
	int force_info_category;
	int group_sub_type;
	int primary_task_sub_type;
	uint64_t primary_task_key;
	int group_member_number;
	int campaign_member_count;
	int player;
	int landed;
	int member_task_dependent_count;
	int member_task_dependent_copied;
	campaign_loss_task_dependent member_task_dependents[CAMPAIGN_LOSS_TASK_DEPENDENT_MAX];
	int group_task_dependents;
	int restricted_guides;
	int landing_action_count;
	int landing_action_copied;
	campaign_loss_landing_action landing_actions[CAMPAIGN_LOSS_LANDING_ACTION_MAX];
	int movement_parent;
	int movement_dependents;
	int takeoff_queue;
} campaign_loss_descriptor;

typedef int (*campaign_observe_loss_fn)
(
	const campaign_loss_descriptor *descriptor,
	int *decision,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_roster_host
{
	campaign_bind_member_fn bind_member;
	campaign_observe_loss_fn observe_loss;
	void *user;
} campaign_roster_host;

typedef struct campaign_member_snapshot
{
	uint64_t member_key;
	uint64_t group_key;
	const char *host_id;
	int origin_entity_index;
	int current_entity_index;
	int entity_type;
	int sub_type;
	int side;
	int initial_alive;
	int physical_present;
	int in_group;
} campaign_member_snapshot;

typedef void (*campaign_member_snapshot_fn) (const campaign_member_snapshot *member, void *user);

/* Observation of what one report applied, copied from the actions performed; never an input to the campaign. */
typedef struct campaign_loss_report_receipt
{
	int landing_reserved_before;
	int landing_reserved_after;
	int group_losses_before;
	int group_losses_after;
	int primary_task_losses_before;
	int primary_task_losses_after;
	int tasks_retired_expire;
	int tasks_completed_on_report;
} campaign_loss_report_receipt;

typedef struct campaign_roster_stats
{
	uint64_t duplicate_comparisons;
	double duplicate_scan_seconds;
} campaign_roster_stats;

struct ENTITY;

void campaign_roster_reset (void);
void campaign_roster_set_host (const campaign_roster_host *host);
void campaign_roster_link_member (struct ENTITY *group, struct ENTITY *member);
void campaign_roster_unlink_member (struct ENTITY *group, struct ENTITY *member);
void campaign_roster_release_entity (struct ENTITY *entity);
int campaign_roster_observe_credit (struct ENTITY *member);
int campaign_roster_observe_physical_loss (struct ENTITY *member, struct ENTITY *group);
void campaign_roster_record_task_termination (struct ENTITY *task, int termination_reason);
int campaign_roster_has_pending_loss (void);
int campaign_roster_report_loss (const char *host_id, campaign_loss_report_receipt *receipt);
int campaign_roster_members (campaign_member_snapshot_fn callback, void *user);
void campaign_roster_get_stats (campaign_roster_stats *stats);

#endif

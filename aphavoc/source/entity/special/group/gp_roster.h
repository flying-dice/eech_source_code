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

typedef struct campaign_roster_host
{
	campaign_bind_member_fn bind_member;
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
uint64_t campaign_roster_member_key (struct ENTITY *member);
const char *campaign_roster_host_id (struct ENTITY *member);
int campaign_roster_members (campaign_member_snapshot_fn callback, void *user);
void campaign_roster_get_stats (campaign_roster_stats *stats);

#endif

/*
 * Host-owned line of sight (EEDCS-42, D2).  The host owns the physical calculation of whether one point sees another: the
 * direction and range, the step counts and increments, the comparison of each sample against the terrain, the
 * ordering of the coarse, source-end, target-end and object phases, the early exits and the target-end exclusion distance,
 * and the entity form's eye height.  Campaign code keeps what to ask and what the answer means: target selection, weapon choice and guide
 * placement still call check_entity_line_of_sight and check_position_line_of_sight and act on the boolean.
 *
 * The exchange is a short synchronous dialogue on one host callback, with at most one query open:
 *
 *   BEGIN     the two ray end points, the criteria, the two eye-height facts, and the host identities of the two ends when the
 *             campaign roster has them (an empty identity is a legitimate end with none: a weapon, a structure, a position).
 *             The host answers DONE with the visibility, or asks the campaign for one environmental value.
 *   TERRAIN   the campaign's answer to NEED_TERRAIN: the terrain elevation at the sample point.
 *   OBJECTS   the campaign's answer to NEED_OBJECTS: whether a collision entity blocks the ray and, only if it does, the point.
 *
 * Terrain elevation and object collision are declared transitional coupling: the campaign still owns the terrain heights (D3)
 * and the sector entity lists the collision scan walks (D5), so the host asks for them one value at a time, in the order the
 * original solver read them.  A host that owns them (a DCS host, later) answers DONE at BEGIN and is never asked.  No EECH
 * pointer, entity index or list layout crosses the boundary: the two entities stay private to the campaign for the one open
 * query so that the object scan can exclude them.
 *
 * There is no campaign fallback: a hosted engine with no line-of-sight provider stops with an error at the first query.
 */
#ifndef MB_HOSTLOS_H
#define MB_HOSTLOS_H

#include <stddef.h>

#define CAMPAIGN_LOS_HOST_ID_MAX 128
#define CAMPAIGN_LOS_ERROR_MAX 256

enum campaign_los_mode
{
	CAMPAIGN_LOS_BEGIN = 1,
	CAMPAIGN_LOS_TERRAIN,
	CAMPAIGN_LOS_OBJECTS
};

enum campaign_los_state
{
	CAMPAIGN_LOS_DONE = 1,
	CAMPAIGN_LOS_NEED_TERRAIN,
	CAMPAIGN_LOS_NEED_OBJECTS
};

/* the phase a request belongs to: informational for the campaign, and what the original solver's transcript names */
enum campaign_los_phase
{
	CAMPAIGN_LOS_PHASE_NONE = 0,
	CAMPAIGN_LOS_PHASE_COARSE,
	CAMPAIGN_LOS_PHASE_SOURCE_FINE,
	CAMPAIGN_LOS_PHASE_TARGET_FINE,
	CAMPAIGN_LOS_PHASE_SOURCE_OBJECTS,
	CAMPAIGN_LOS_PHASE_TARGET_OBJECTS
};

typedef struct campaign_los_step
{
	int mode;
	/* BEGIN.  entity_form: the query was asked of two entities (the positions are their poses), not of two positions.
	   source_eye and target_eye: that end is a vehicle whose pose the ray lifts by 1.5 m (a fact the campaign read from the entity;
	   the lift itself is the host's).  criteria: the original MOBILE_LOS_CHECK_* bits.  source_host_id and target_host_id: empty,
	   or the identity the host issued for that end */
	int entity_form;
	int criteria;
	int source_eye;
	int target_eye;
	char source_host_id[CAMPAIGN_LOS_HOST_ID_MAX + 1];
	char target_host_id[CAMPAIGN_LOS_HOST_ID_MAX + 1];
	float source_position[3];
	float target_position[3];
	/* TERRAIN */
	float elevation;
	/* OBJECTS: collision_point is meaningful only when hit */
	int hit;
	float collision_point[3];
} campaign_los_step;

typedef struct campaign_los_result
{
	int state;
	int phase;
	/* NEED_TERRAIN: the sample point (x, the ray's height there, z) */
	float point[3];
	/* NEED_OBJECTS: the ray */
	float start[3];
	float end[3];
	/* DONE */
	int visible;
} campaign_los_result;

typedef int (*campaign_los_fn)
(
	const campaign_los_step *step,
	campaign_los_result *result,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_los_host
{
	campaign_los_fn line_of_sight;
	void *user;
} campaign_los_host;

struct ENTITY;
struct VEC3D;

void campaign_los_set_host (const campaign_los_host *host);

/* The one query path of both public checks.  entity_form and the two eye facts are as for BEGIN; source and target stay private */
int campaign_los_query
(
	struct ENTITY *source,
	struct ENTITY *target,
	const struct VEC3D *source_position,
	const struct VEC3D *target_position,
	int criteria,
	int entity_form,
	int source_eye,
	int target_eye
);

#endif

/*
 * Host-owned movement of alive routed vehicles, ships and infantry.  The host
 * keeps the authoritative position, speed and orientation of an alive mobile,
 * keyed by its host identity, and performs the physical step.  Campaign code keeps the
 * route, formation, hold and engage policy: it decides whether to move, stop
 * or sample terrain, and supplies the guide's desired speed, the vehicle's
 * acceleration limit and the waypoint position.  No EECH pointer crosses this
 * boundary.
 *
 * The campaign entity keeps a copy of the pose for the systems that read it
 * (combat, perception, arrival, Tacview).  On MOVE and STOP that copy is sent
 * only so the host can refuse a divergence; the host never adopts it.  PLACE
 * is an explicit campaign placement (a position, velocity or attitude setter
 * called by campaign or network code): the host adopts the pose it carries.
 *
 * Terrain stays campaign-side (a later stage): the sampled elevation, forest
 * flag and face normal at the pre-move position are inputs, as are the map
 * bounds.
 */
#ifndef RV_HOSTMV_H
#define RV_HOSTMV_H

#include <stddef.h>
#include <stdint.h>

#define CAMPAIGN_MOVEMENT_HOST_ID_MAX 128
#define CAMPAIGN_MOVEMENT_ERROR_MAX 256

enum campaign_movement_mode
{
	CAMPAIGN_MOVEMENT_MOVE = 1,	/* step towards the waypoint */
	CAMPAIGN_MOVEMENT_STOP,		/* the campaign zeroes the speed (hold, no waypoint, no guide) */
	CAMPAIGN_MOVEMENT_PLACE,	/* the campaign placed the vehicle: adopt the pose */
	CAMPAIGN_MOVEMENT_SHIP_MOVE,	/* a ship's step towards the waypoint */
	CAMPAIGN_MOVEMENT_PERSON_BEGIN,	/* infantry, phase one: the horizontal advance, held by the host */
	CAMPAIGN_MOVEMENT_PERSON_END	/* infantry, phase two: the host places at the campaign's terrain elevation */
};

enum campaign_movement_kind
{
	CAMPAIGN_MOVEMENT_KIND_ROUTED = 1,
	CAMPAIGN_MOVEMENT_KIND_SHIP,
	CAMPAIGN_MOVEMENT_KIND_PERSON
};

typedef struct campaign_movement_step
{
	char host_id[CAMPAIGN_MOVEMENT_HOST_ID_MAX + 1];
	int mode;
	/* the campaign's copy of the pose: compared on MOVE/STOP, adopted on PLACE */
	float position[3];
	float velocity;
	float attitude[9];		/* row-major 3x3 */
	/* MOVE */
	float delta_seconds;
	float desired_velocity;		/* the guide's speed */
	float max_acceleration;
	float waypoint[3];
	int tacview_logging;		/* the host reports `rotated` only when this is set */
	int terrain_follow;
	int terrain_present;		/* the campaign sampled terrain at the pre-move position */
	float elevation;
	int forest;
	float face_normal[3];
	float map_min[3];
	float map_max[3];
	/* SHIP_MOVE: the frame delta (the engine's reciprocal, for the motion vector) and the class's g limit;
	   the guide's speed is desired_velocity */
	float frame_delta;
	float g_max;
	/* PERSON_BEGIN: the animation's speed; terrain_follow announces that PERSON_END will follow;
	   PERSON_END: elevation is the campaign's terrain sample at the position PERSON_BEGIN returned */
	float ps_vel;
} campaign_movement_step;

typedef struct campaign_movement_result
{
	float position[3];
	float velocity;
	float attitude[9];
	int rotated;
	/* the motion vector the step leaves on a ship or infantry member */
	float motion_vector[3];
	/* PERSON_BEGIN only: the step awaits PERSON_END, and only `position` (the horizontal advance) is meaningful */
	int pending;
} campaign_movement_result;

typedef int (*campaign_move_vehicle_fn)
(
	const campaign_movement_step *step,
	campaign_movement_result *result,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_movement_host
{
	campaign_move_vehicle_fn move_vehicle;
	void *user;
} campaign_movement_host;

struct ENTITY;
struct VEC3D;

void campaign_movement_set_host (const campaign_movement_host *host);

/* the vehicle's physical state is host-owned: it has a host identity and a movement host is set */
int campaign_movement_is_hosted (struct ENTITY *vehicle);

/* one MOVE step towards the waypoint; the host's pose is applied to the entity */
void campaign_movement_move (struct ENTITY *vehicle, struct ENTITY *guide, const struct VEC3D *waypoint);

/* the campaign is zeroing the speed (policy stays campaign-side): the host's speed, to be applied through the legacy setter */
float campaign_movement_stop (struct ENTITY *vehicle);

/* one ship step; the host's pose and motion vector are applied to the entity */
void campaign_movement_ship_move (struct ENTITY *vehicle, struct ENTITY *guide, const struct VEC3D *waypoint);

/*
 * One infantry step in two phases around the campaign's terrain sample.  `begin` returns the
 * horizontal advance and whether an `end` is pending (terrain_follow); the campaign samples the terrain
 * at `advanced` and calls `end` with the elevation, which applies the final pose.  Nothing is applied
 * to the entity before `end` (or before `begin` when no terrain follow is needed).
 */
int campaign_movement_person_begin (struct ENTITY *vehicle, float ps_vel, const struct VEC3D *waypoint, int terrain_follow, struct VEC3D *advanced);
void campaign_movement_person_end (struct ENTITY *vehicle, float elevation);

/* bracket the campaign's own writes of host results, which are not placements */
void campaign_movement_apply_begin (void);
void campaign_movement_apply_end (void);

/* a campaign pose write: tell the host (no-op during the host-result application or for unhosted entities) */
void campaign_movement_place (struct ENTITY *vehicle);

#endif

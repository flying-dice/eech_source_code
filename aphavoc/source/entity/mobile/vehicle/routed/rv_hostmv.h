/*
 * Host-owned movement of alive routed vehicles, ships, infantry, fixed-wing aircraft and AI helicopters.  The host
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
	CAMPAIGN_MOVEMENT_PERSON_END,	/* infantry, phase two: the host places at the campaign's terrain elevation */
	CAMPAIGN_MOVEMENT_FIXED_WING_MOVE,	/* a fixed-wing aircraft's step (taxi, flight, landing or absolute) */
	CAMPAIGN_MOVEMENT_HELICOPTER_BEGIN,	/* an alive AI helicopter, phase one: the motion vector, the speed and whether it moved */
	CAMPAIGN_MOVEMENT_HELICOPTER_END,	/* an alive AI helicopter, phase two: the attitude, the position and the floor clamp */
	CAMPAIGN_MOVEMENT_ROTOR_SET,		/* explicit rotor seed/reset or campaign write: the host adopts hc_rotor */
	CAMPAIGN_MOVEMENT_ROTOR_ALIVE,		/* alive helicopter rotor rpm and spin-up timer integration */
	CAMPAIGN_MOVEMENT_ROTOR_DOWN,		/* dying helicopter rotor wind-down */
	CAMPAIGN_MOVEMENT_FALL,			/* dying helicopter, before the terrain sample: the candidate position only */
	CAMPAIGN_MOVEMENT_CONTACT,		/* dying helicopter: altitude over the terrain and whether it touched */
	CAMPAIGN_MOVEMENT_SETTLE,		/* dying helicopter: settle on or in the terrain class; bury zeroes rotor and vector */
	CAMPAIGN_MOVEMENT_TUMBLE,		/* dying helicopter: tumble attitude, then the host position is finalised */
	CAMPAIGN_MOVEMENT_CRASH_BEGIN,		/* crash slide, phase one: attitude from the OLD terrain normal, candidate position */
	CAMPAIGN_MOVEMENT_CRASH_END,		/* crash slide, phase two: friction and slope from the NEW terrain normal */
	CAMPAIGN_MOVEMENT_IMPACT,		/* impact shove along the helicopter's attitude x axis */
	CAMPAIGN_MOVEMENT_FALL_GRAVITY,		/* dying helicopter, after the terrain sample: gravity, the motion vector and the speed */
	CAMPAIGN_MOVEMENT_ROTOR_IDLE,		/* idle helicopter (no follower parent): linear rotor wind-down by the frame delta */
	/* EEDCS-37: routed, anti-air, ship and fixed-wing wreck/fall/sink/crash motion */
	CAMPAIGN_MOVEMENT_GV_DEATH_CANDIDATE,	/* routed/AA wreck slide, before the terrain refresh: candidate x/z (no map bound) */
	CAMPAIGN_MOVEMENT_GV_DEATH_DECAY,	/* routed/AA wreck slide, after the refresh: speed decay, heading, y = elevation + cg, attitude */
	CAMPAIGN_MOVEMENT_GV_FALL_CANDIDATE,	/* routed/AA falling, before the refresh: candidate position, all axes (no map bound) */
	CAMPAIGN_MOVEMENT_GV_FALL_GRAVITY,	/* routed/AA falling, after the refresh: gravity, the vector and the speed */
	CAMPAIGN_MOVEMENT_GV_FALL_SETTLE,	/* routed/AA falling: contact / splash / sunk / ground impact flags; the settled position, vector and speed are STAGED host-side, nothing is applied */
	CAMPAIGN_MOVEMENT_GV_IMPACT,		/* routed/AA impact shove (the DLL drew the float speed) */
	CAMPAIGN_MOVEMENT_SHIP_SINK,		/* ship sinking: candidate position, drag, sink rate, pitch/roll drift, attitude */
	CAMPAIGN_MOVEMENT_VEHICLE_VECTOR_SEED,	/* routed/AA/ship: ownership claim; the host adopts hc_motion_vector only when it retains no vehicle vector, else it must match bit-identically */
	CAMPAIGN_MOVEMENT_FW_DEATH_ATTITUDE,	/* dying fixed-wing, after the gravity step: heading/pitch/roll easing toward the velocity direction */
	CAMPAIGN_MOVEMENT_FW_CRASH_CANDIDATE,	/* fixed-wing crash slide, phase one: candidate x/z, map-bounded; y held */
	CAMPAIGN_MOVEMENT_FW_CRASH_VELOCITY,	/* fixed-wing crash slide, phase two: candidate y, friction and slope push from the NEW normal; stopped flag */
	CAMPAIGN_MOVEMENT_GV_FALL_VECTOR,	/* routed/AA falling, after the settle effects: the staged vector (sunk zero / ground y zero) is written */
	CAMPAIGN_MOVEMENT_GV_FALL_COMMIT,	/* routed/AA falling, last: the staged speed then the staged settled position are finalised */
	CAMPAIGN_MOVEMENT_FW_SETTLE,		/* dying fixed-wing: settle by terrain class, double 3.0 river depth, no rotor; bury zeroes the vector only */
	CAMPAIGN_MOVEMENT_FW_CRASH_ATTITUDE,	/* fixed-wing crash slide, phase three: pitch/roll easing toward the NEW normal, rest flag */
	CAMPAIGN_MOVEMENT_RETIRE,			/* the physical incarnation ends (heap release): the host retires the id, drops any open step, and answers no later step for it */
	/* EEDCS-38 */
	CAMPAIGN_MOVEMENT_CREATE_PLACE,		/* initial placement of a newly created mobile: the host builds the attitude from the terrain face, the heading and the fuselage angle, and adopts the pose; once per identity */
	CAMPAIGN_MOVEMENT_EXPIRE,			/* a dead mobile's lifetime: the host accumulates the death timer, gates it and decides whether the wreck has expired (a ship: whether it has sunk out of sight) */
	/* EEDCS-39 hosted helicopter control: the guidance flow Plan -> (DLL lookups) -> Guide -> rotor gate -> GuidedBegin -> GuidedEnd */
	CAMPAIGN_MOVEMENT_HELICOPTER_PLAN,		/* the campaign's task and waypoint facts; the host answers hc_query: the lookups it needs, planned from its own pose */
	CAMPAIGN_MOVEMENT_HELICOPTER_GUIDE,		/* the DLL's lookup results (own-sector structures, terrain samples); the host answers hc_guidance */
	CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_BEGIN,	/* after the rotor and campaign gates: the host applies the guidance it retained to the motion vector and speed */
	CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_END,	/* the steering facts: the host steers and moves from its retained guidance and answers the attitude, position and angles */
	CAMPAIGN_MOVEMENT_HELICOPTER_ABANDON,		/* the campaign decided not to fly the step (human wait, landed): the host drops the open flow */
	/* EEDCS-39 R1 */
	CAMPAIGN_MOVEMENT_FIXED_WING_LOOKAHEAD,		/* a terrain-following fixed-wing waypoint is being selected: the host answers hc_query with the one point it needs sampled, from its own pose; nothing is advanced */
	/* EEDCS-40 */
	CAMPAIGN_MOVEMENT_DEPENDENT_CARRY,		/* a movement-dependent mobile follows its parent: the host validates the parent's move, positions the dependent and answers its pose */
	/* EEDCS-44 */
	CAMPAIGN_MOVEMENT_PERSON_FACE			/* an infantry member with a target turns to face it: the host forms the heading from its own position and the target's, turns its record and answers the attitude and the heading */
};

enum campaign_movement_kind
{
	CAMPAIGN_MOVEMENT_KIND_ROUTED = 1,
	CAMPAIGN_MOVEMENT_KIND_SHIP,
	CAMPAIGN_MOVEMENT_KIND_PERSON,
	CAMPAIGN_MOVEMENT_KIND_FIXED_WING,
	CAMPAIGN_MOVEMENT_KIND_HELICOPTER
};

/* EEDCS-39: a structure of the aircraft's own sector, as the DLL's structure scan saw it */
typedef struct campaign_hc_structure
{
	float position[3];
	float avoidance_radius;
	float height;
} campaign_hc_structure;

/* EEDCS-39: one terrain sample's getter returns in call order; second is meaningful only when second_present */
typedef struct campaign_hc_terrain_sample
{
	float first;
	float second;
	int second_present;
} campaign_hc_terrain_sample;

#define CAMPAIGN_HC_TERRAIN_SAMPLES 2

/* EEDCS-39: what the host's plan asks the DLL to look up: the structure scan at structure_position, and terrain_sample_count
   (0 to CAMPAIGN_HC_TERRAIN_SAMPLES) x/z positions in the order to sample them */
typedef struct campaign_hc_query
{
	int structure_scan;
	float structure_position[3];
	int terrain_sample_count;
	float terrain_sample[CAMPAIGN_HC_TERRAIN_SAMPLES][2];
} campaign_hc_query;

/* EEDCS-39: the host's guidance for the step */
typedef struct campaign_hc_guidance_answer
{
	float lift;
	float waypoint[3];
	float required_height;
	float world_acceleration[3];
	float wp_vec[3];
} campaign_hc_guidance_answer;

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
	/* FIXED_WING_MOVE: the campaign's guidance for the step.  desired_velocity is the waypoint speed, g_max the
	   class's g limit, elevation the terrain sample under the aircraft; the other shared fields are as for a ship.
	   fw_phase selects taxi, flight, landing or the absolute update; fw_motion_vector is the motion vector the previous
	   step left (the old direction of travel).  On PLACE it is the one placement vector for every aircraft (fixed-wing and
	   helicopter): hc_motion_vector is never read for PLACE.
	   FLIGHT and LAND (EEDCS-39): the host decides the physical facing and turn itself.  From its own retained pose,
	   velocity, attitude and motion vector it finds whether the aircraft faces away from the waypoint, applies the
	   1500 m task-leader gate, holds the turn (0.25 s when aiming a cannon, 3.0 s when facing away), keeps the turn
	   multiplier and chooses between the flight branch and the absolute turn-around; FLIGHT also computes the steering
	   acceleration (eech_fw_navigate).  The campaign sends only the intent and observations: waypoint (the resolved
	   waypoint) and desired_velocity (the guide's speed, cruise when it is zero), fw_aiming_cannon (the campaign's cannon
	   policy found a target to engage) and fw_elapsed_time (the session clock the holds run on), fw_group_leader and
	   fw_task_leader (different flags), the class's turn allowance and avoidance radius, the task leader's host id (read
	   when !fw_task_leader; on LAND only when the host finds the aircraft facing away) and, on FLIGHT only, the group's
	   other members in list order, self excluded, each with its class's avoidance radius (read only when
	   !fw_group_leader).  The member lists are fw_member_count entries long, owned by the caller and read only during
	   the call; a group of any size is sent whole.  An id is an observation the campaign passes on if its roster has
	   it: a leader id left empty and a NULL or empty member entry mean unresolved.  The host asks for one only on the
	   path that reads it (facing away, or flying the navigator) and then refuses the step, before it mutates anything,
	   if it is unresolved or not a fixed-wing aircraft the host retains.  No turn multiplier crosses the boundary */
	int fw_phase;
	float fw_motion_vector[3];
	float fw_cruise_velocity;
	float fw_fuselage_angle;
	float fw_cg_to_ground;
	float fw_max_roll;
	int fw_aiming_cannon;
	float fw_elapsed_time;
	int fw_navigating;
	int fw_group_leader;
	int fw_task_leader;
	float fw_max_vertical_turn_allowance;
	float fw_avoidance_radius;
	char fw_leader_host_id[CAMPAIGN_MOVEMENT_HOST_ID_MAX + 1];
	int fw_member_count;
	const char *const *fw_member_host_id;	/* fw_member_count NUL-terminated ids, 1 to CAMPAIGN_MOVEMENT_HOST_ID_MAX bytes each */
	const float *fw_member_radius;		/* fw_member_count radii, in the same order */
	/* HELICOPTER_BEGIN/END: the DLL's guidance for the step.  hc_motion_vector is the DLL's copy of the vector the
	   host retains (what the previous step left, or what HELICOPTER_BEGIN answered for END): the host refuses a difference.
	   BEGIN: hc_acceleration_x/z and hc_wp_vec_y are the guidance's world acceleration and climb demand.  END:
	   hc_desired_heading and hc_desired_pitch/roll are the guidance's attitude demands and hc_max_turn_rate the
	   aircraft's physical turn limit; the host wraps the heading, bounds the turn, blends the attitude and integrates.
	   elevation is the terrain sample under the aircraft, hc_cg_to_ground the class's height, and terrain_follow the
	   guide's terrain-follow mode.  These are physical inputs and retained coupling, not campaign decisions */
	float hc_acceleration_x;
	float hc_acceleration_z;
	float hc_wp_vec_y;
	float hc_desired_heading;
	float hc_max_turn_rate;
	float hc_desired_pitch;
	float hc_desired_roll;
	float hc_cg_to_ground;
	float hc_motion_vector[3];
	/* ROTOR_* and the death/crash modes: hc_rotor is the DLL's copy of the retained rotor state {main rpm, tail rpm,
	   spin-up timer}; the host refuses a difference except on ROTOR_SET, where it is the explicit seed/reset (bind descriptor,
	   creation, unpack, setter) the host adopts.
	   ROTOR_ALIVE: hc_lift, hc_power_output and delta_seconds are the integration inputs; hc_flag is the DLL's decision
	   that the spin-up gate applies (the host reports `moved` TRUE when it decremented the timer, i.e. the gate held).
	   ROTOR_DOWN: delta_seconds only.
	   ROTOR_IDLE: delta_seconds is the FRAME delta (get_delta_time), not the movement delta.
	   FALL: delta_seconds and the map bounds; the host holds the candidate position for CONTACT/SETTLE/TUMBLE.
	   FALL_GRAVITY: hc_gravity and delta_seconds, after the DLL's terrain sample at the candidate.
	   CONTACT: hc_flag selects the point (0 the held position, 1 the FALL candidate); elevation and hc_cg_to_ground (the
	   PRE-kill CG) are the terrain sample; CONTACT(candidate) altitude is retained host-side for SETTLE.  SETTLE:
	   hc_terrain_class (0 solid, 1 sea, 2 river/lake, 3 forest), elevation, hc_cg_to_ground (the FRESH post-kill CG, final
	   heights only) and hc_bury_limit; the bury tests use the retained contact altitude.  TUMBLE: hc_draws are the two
	   random draws (doubles, as frand1 () returns them) the DLL made after the effects.
	   CRASH_BEGIN: face_normal is the OLD terrain normal, hc_pitch_offset/hc_bank_offset the destroyed attitude offsets;
	   CRASH_END: face_normal is the NEW (refreshed) terrain normal, forest/elevation/hc_cg_to_ground the terrain sample.
	   IMPACT: hc_signed_speed is the shove the DLL decided (after its random draw) */
	float hc_rotor[3];
	float hc_lift;
	float hc_power_output;
	int hc_flag;
	float hc_gravity;
	int hc_terrain_class;
	float hc_bury_limit;
	float hc_pitch_offset;
	float hc_bank_offset;
	float hc_signed_speed;
	/* EEDCS-37.  hc_motion_vector is the DLL's copy of the vector for every kind in these modes (the host refuses a
	   difference except on VEHICLE_VECTOR_SEED while none is retained).  GV_*: the host derives heading, pitch and roll from its own retained attitude (none is sent); gv_wide selects the anti-air
	   double literal for the speed decay; gv_old_elevation is the terrain elevation before the refresh, gv_bbox_diagonal
	   the bounding box diagonal, gv_water the water terrain class; elevation and hc_cg_to_ground are the refreshed sample;
	   GV_IMPACT: hc_signed_speed is the float the DLL drew (4.0*frand1x narrowed to float, as the original stored it); hc_gravity is the gravity.  SHIP_SINK: delta_seconds.
	   FW_DEATH_ATTITUDE: hc_flag is whether the DLL drew the roll (hc_draws[1]), altitude is carried in elevation (the
	   altitude above terrain) and the speed is hc_signed_speed.
	   Fixed-wing members use the helicopter FALL, FALL_GRAVITY, CONTACT and IMPACT modes (host-dispatched by member kind;
	   the arithmetic is type-agnostic) with hc_motion_vector the DLL copy of the retained fw vector and hc_rotor zero (a
	   fixed wing has no rotor: the host ignores it).  FW_SETTLE: hc_terrain_class, elevation, hc_cg_to_ground (the FRESH
	   post-kill CG), hc_bury_limit; the bury uses the contact altitude retained from CONTACT(candidate).
	   FW_CRASH_CANDIDATE: delta_seconds, map bounds.  FW_CRASH_VELOCITY: face_normal is the NEW normal, forest, elevation,
	   hc_cg_to_ground, delta_seconds; the host retains the candidate and the post-slope speed.  FW_CRASH_ATTITUDE:
	   face_normal (NEW), hc_pitch_offset, hc_bank_offset, delta_seconds.
	   CREATE_PLACE (EEDCS-38): position is the campaign's chosen placement point, velocity the creation speed, hc_desired_heading the
	   campaign's heading, fw_fuselage_angle the airframe angle (zero for a ground mobile) and face_normal the terrain face under
	   the point; fw_motion_vector is the aircraft's creation vector.  The host answers the attitude matrix and the heading, pitch and
	   roll that build it (result.angles), which the campaign applies through its own angle setter */
	float gv_old_elevation;
	float gv_bbox_diagonal;
	int gv_wide;
	int gv_water;
	/* EEDCS-38 EXPIRE: death_timer is the campaign's mirror of the wreck's timer (the host adopts it while it retains none, then
	   refuses a difference); death_range the campaign's squared 2D range to the nearest keysite, its own world knowledge;
	   death_policy the campaign's decision: bit0 the wreck never expires (anti-air), bit1 the wreck is still occupied by a player
	   (the host resets the timer rather than expiring it).  The gate, the range-to-lifetime curve and the comparison are the host's.
	   For a ship, gv_bbox_diagonal is the shape's bounding box diagonal and the host decides from its own height whether it has sunk */
	float death_timer;
	float death_range;
	int death_policy;
	/* EEDCS-39 hosted helicopter control.  PLAN: the campaign's task and waypoint facts (hc_group_leader .. hc_leader_radar_altitude),
	   delta_seconds, terrain_follow, elevation (the cached terrain elevation), waypoint, the map bounds, hc_motion_vector and
	   hc_power_output, and the group's other members in list order (read only when !hc_group_leader; a NULL or empty entry means
	   the roster lacks the id).  GUIDE: the DLL's own-sector structures (uncapped, sector order) and the getter returns of the
	   terrain samples the plan asked for, with hc_motion_vector.  GUIDED_BEGIN: hc_motion_vector.  GUIDED_END: delta_seconds,
	   hc_max_turn_rate, elevation (the cached sample, no new one), hc_cg_to_ground, terrain_follow, the map bounds, g_max,
	   hc_motion_vector, and the steering facts: the waypoint kind, whether the aircraft leads its task, the guide radius, the
	   required heading (campaign intent), the task leader's host id (read only on the path that needs it), the targeting
	   decision, the weapon vectors and the rotor shaft angle.  No desired steering crosses the boundary.  The lists are owned by
	   the caller and read only during the call */
	int hc_group_leader;
	int hc_task_leader;
	int hc_task_leader_is_self;
	int hc_position_hold;
	int hc_terrain_follow_attack_altitude;
	int hc_waypoint_kind;			/* CAMPAIGN_HC_WP_* */
	int hc_guide_radius_valid;
	float hc_guide_radius;
	int hc_guide_altitude_valid;
	float hc_waypoint_velocity[3];
	float hc_distance;
	float hc_fuel_supply_level;
	float hc_cruise_velocity;
	float hc_cruise_altitude;
	float hc_attack_altitude;
	float hc_leader_radar_altitude;
	int hc_member_count;
	const char *const *hc_member_host_id;
	int hc_structure_count;
	const campaign_hc_structure *hc_structures;
	int hc_terrain_sample_count;
	const campaign_hc_terrain_sample *hc_terrain_samples;
	int hc_required_heading_valid;
	float hc_required_heading;
	char hc_leader_host_id[CAMPAIGN_MOVEMENT_HOST_ID_MAX + 1];
	int hc_targeting_active;
	float hc_weapon_vector[3];
	float hc_weapon_to_intercept_vector[3];
	float hc_main_rotor_shaft_angle;
	/* FIXED_WING_MOVE: the terrain observation for a terrain-following task leader's waypoint (fw_terrain false otherwise).
	   The campaign sends its minimum height, the guide position it selected, the host's own look-ahead point echoed back,
	   the elevation its terrain service answered there and the formation site and centre-of-gravity heights to add after
	   the host has shaped the waypoint height.  The waypoint height the step carries is then not read */
	int fw_terrain;
	float fw_terrain_min_height;
	float fw_terrain_guide_position[3];
	float fw_terrain_sample_point[2];
	float fw_terrain_sample_elevation;
	float fw_terrain_site_y;
	float fw_terrain_cg;
	/* EEDCS-40 DEPENDENT_CARRY: the dependent's own pose is position/velocity/attitude as always (the host refuses a difference from
	   its record).  dc_parent_host_id names the parent that notified it, dc_parent_old and dc_parent_new the parent's position before
	   and after the move (the notification names the new one while the parent still stores the old), dc_heading and dc_velocity the
	   heading and speed the parent's setter sent.  dc_helicopter selects the helicopter shape (the dependent keeps its offset, turned
	   by the heading change, and takes the heading and speed); otherwise it takes the parent's new position.  The host answers
	   position, attitude and velocity.  A parent move the host does not hold is refused */
	char dc_parent_host_id[CAMPAIGN_MOVEMENT_HOST_ID_MAX + 1];
	int dc_helicopter;
	float dc_parent_old[3];
	float dc_parent_new[3];
	float dc_heading;
	float dc_velocity;
	/* EEDCS-44 PERSON_FACE: the target's position (the person's own pose is position/velocity/attitude as always); the host answers the attitude and
	   the heading in angles[0] */
	float pf_target[3];
	/* the TUMBLE and FW_DEATH_ATTITUDE draws are frand1 () doubles and stay doubles: the original expression multiplies
	   them in double before narrowing the result to the float heading/roll.  Last, so the double's alignment cannot move
	   another field */
	double hc_draws[2];
} campaign_movement_step;

typedef struct campaign_movement_result
{
	float position[3];
	float velocity;
	float attitude[9];
	int rotated;
	/* the motion vector the step leaves on a ship or infantry member */
	float motion_vector[3];
	/* PERSON_BEGIN, HELICOPTER_BEGIN and CRASH_BEGIN: the step awaits its second phase (PERSON_END / HELICOPTER_END /
	   CRASH_END).  For PERSON_BEGIN only `position` (the horizontal advance) is meaningful.  HELICOPTER_BEGIN and
	   CRASH_BEGIN must answer TRUE and every other helicopter answer FALSE */
	int pending;
	/* a per-mode flag (always 0 or 1): HELICOPTER_BEGIN the speed exceeds the moved threshold; ROTOR_ALIVE the spin-up
	   gate held; CONTACT the helicopter touched; SETTLE it is buried; CRASH_END the slide stopped; other modes 0 */
	int moved;
	/* the rotor state {main rpm, tail rpm, spin-up timer} after ROTOR_* and SETTLE (every mode echoes the retained one) */
	float rotor[3];
	/* CONTACT: the altitude of the point over the terrain, cg height included */
	float altitude;
	/* EEDCS-37: GV_FALL_SETTLE bit0 contact, bit1 splash, bit2 sunk, bit3 ground impact; FW_DEATH_ATTITUDE -1 gate/draw
	   mismatch, 0 nothing, 1 eased, 3 eased and the roll draw was consumed; FW_CRASH_VELOCITY bit0 stopped; FW_CRASH_ATTITUDE bit0 at rest */
	int flags;
	/* EEDCS-38 CREATE_PLACE: the heading, pitch and roll the host built `attitude` from; zero in every other mode */
	float angles[3];
	/* EEDCS-38 EXPIRE: flags bit0 the wreck has expired; the host's timer after the step, which the campaign mirrors */
	float death_timer;
	/* EEDCS-39 HELICOPTER_PLAN: the lookups the DLL must make; HELICOPTER_GUIDE: the host's guidance.  Zero in every other mode */
	campaign_hc_query hc_query;
	campaign_hc_guidance_answer hc_guidance;
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

/* the campaign's guidance for one fixed-wing step: the phase is one of the CAMPAIGN_FIXED_WING_* values */
enum
{
	CAMPAIGN_FIXED_WING_TAXI = 0,
	CAMPAIGN_FIXED_WING_FLIGHT,
	CAMPAIGN_FIXED_WING_LAND,
	CAMPAIGN_FIXED_WING_ABSOLUTE
};

/* a terrain-following task leader's waypoint, the campaign's part of it: see campaign_movement_step fw_terrain.  The
   campaign selects the guide and the formation offsets and answers the terrain lookup (a transitional service); the
   host chooses the look-ahead point and performs the height arithmetic */
typedef struct campaign_fixed_wing_terrain
{
	int follow;
	float min_height;
	float guide_position[3];
	float sample_point[2];
	float sample_elevation;
	float site_y;
	float cg;
} campaign_fixed_wing_terrain;

typedef struct campaign_fixed_wing_guidance
{
	int phase;
	float wp_vel;
	float cruise_velocity;
	float g_max;
	float fuselage_angle;
	float cg_to_ground;
	float max_roll;
	float waypoint[3];
	float motion_vector[3];
	int navigating;
	float elevation;
	float delta_seconds;
	float frame_delta;
	/* FLIGHT and LAND: the turn intent and the navigation intent.  The host id strings and the member lists are only
	   read during the call; the lists hold member_count entries (FLIGHT only) and the caller owns them.  A NULL
	   leader_host_id or member entry is an identity the roster does not have: it is sent as unresolved */
	int aiming_cannon;
	float elapsed_time;
	int group_leader;
	int task_leader;
	float max_vertical_turn_allowance;
	float avoidance_radius;
	const char *leader_host_id;
	int member_count;
	const char *const *member_host_id;
	const float *member_radius;
	/* FLIGHT, LAND and ABSOLUTE: set (follow) when the waypoint's height is the host's to shape; waypoint[1] is then unshaped */
	campaign_fixed_wing_terrain terrain;
} campaign_fixed_wing_guidance;

/* the waypoint-kind group the campaign names for a hosted helicopter step (hc_waypoint_kind) */
enum
{
	CAMPAIGN_HC_WP_OTHER = 0,
	CAMPAIGN_HC_WP_ROUTE,			/* NAVIGATION, CAP_LOOP, CAP_START */
	CAMPAIGN_HC_WP_HOLDING,			/* HOLDING, HOLDING_LOOP, TAXI */
	CAMPAIGN_HC_WP_LANDED,
	CAMPAIGN_HC_WP_INSERTION		/* PREPARE_FOR_INSERTION, TROOP_INSERT */
};

/* the campaign's facts for the PLAN step of a hosted helicopter (EEDCS-39).  Facts only: no desired velocity, heading,
   pitch or roll.  The member list holds member_count ids in group list order (read only when !group_leader; a NULL entry is
   an identity the roster lacks), owned by the caller and read only during the call; it has no cap */
typedef struct campaign_hc_plan
{
	int group_leader;
	int task_leader;
	int task_leader_is_self;
	int position_hold;
	int terrain_follow;				/* the guide's terrain-follow mode is not NONE */
	int terrain_follow_attack_altitude;
	int waypoint_kind;				/* CAMPAIGN_HC_WP_* of the guide's current waypoint */
	int guide_radius_valid;
	float guide_radius;
	int guide_altitude_valid;
	float waypoint[3];
	float waypoint_velocity[3];
	float distance;
	float terrain_elevation;		/* the cached terrain_info elevation: no new sample */
	float fuel_supply_level;
	float cruise_velocity;
	float power_output;
	float cruise_altitude;
	float attack_altitude;
	float leader_radar_altitude;
	float delta_seconds;
	int member_count;
	const char *const *member_host_id;
} campaign_hc_plan;

/* the campaign's facts for the GUIDED_END step: the steering is the host's.  The leader's host id is read only on the path
   that needs it and is ignored when the aircraft leads its own task; the required heading is campaign intent */
typedef struct campaign_hc_steer
{
	float delta_seconds;
	float max_turn_rate;
	float elevation;				/* the cached sample from the altitude follow: no new one */
	float cg_to_ground;
	float g_max;
	int terrain_follow;
	int waypoint_kind;				/* CAMPAIGN_HC_WP_* of the helicopter's current waypoint */
	int task_leader_is_self;
	int guide_radius_valid;
	float guide_radius;
	int required_heading_valid;
	float required_heading;
	const char *leader_host_id;
	int targeting_active;
	float weapon_vector[3];
	float weapon_to_intercept_vector[3];
	float main_rotor_shaft_angle;
} campaign_hc_steer;

void campaign_movement_set_host (const campaign_movement_host *host);

/* the vehicle's physical state is host-owned: it has a host identity and a movement host is set */
int campaign_movement_is_hosted (struct ENTITY *vehicle);

/* a host-movement build never moves a live vehicle itself: stop with a clear error when it has no host to move it */
void campaign_movement_require_hosted (struct ENTITY *vehicle);

/* one MOVE step towards the waypoint; the host's pose is applied to the entity */
void campaign_movement_move (struct ENTITY *vehicle, struct ENTITY *guide, const struct VEC3D *waypoint);

/* the campaign is zeroing the speed (policy stays campaign-side): the host's speed, to be applied through the legacy setter */
float campaign_movement_stop (struct ENTITY *vehicle);

/* one ship step; the host's pose and motion vector are applied to the entity */
void campaign_movement_ship_move (struct ENTITY *vehicle, struct ENTITY *guide, const struct VEC3D *waypoint);

/* one fixed-wing step; the host's pose and motion vector are applied to the aircraft */
void campaign_movement_fixed_wing_move (struct ENTITY *aircraft, const campaign_fixed_wing_guidance *guidance);

/* the x and z of the one point the host needs the terrain sampled at to shape a terrain-following waypoint, from its own pose; nothing is advanced */
void campaign_movement_fixed_wing_lookahead (struct ENTITY *aircraft, float *point_x, float *point_z);

/*
 * One alive AI helicopter step in two phases around the campaign's attitude guidance.  `begin` takes the campaign's
 * acceleration and climb demand, and applies the host's motion vector and speed to the helicopter at once (the guidance
 * reads them); `end` takes the bounded turn, the desired pitch and roll and the terrain sample, and applies the attitude,
 * the position and the floor-clamped motion vector.  The rotor, the guidance, the look-ahead, the avoidance and the pilot
 * control stay campaign-side physical coupling, as do the terrain sampling, effects, sounds, random draws and the
 * lifecycle (dead state, smoke, kill); every other direct write of the helicopter's pose is announced to the host only
 * through the setters' placements.
 */
void campaign_movement_helicopter_begin (struct ENTITY *helicopter, float acceleration_x, float acceleration_z, float wp_vec_y);
void campaign_movement_helicopter_end (struct ENTITY *helicopter, float desired_heading, float max_turn_rate, float desired_pitch, float desired_roll, float elevation, float cg_to_ground, int terrain_follow);

/*
 * EEDCS-39 host-planned helicopter control, in this order: plan -> (the DLL's lookups) -> guide -> rotor gate ->
 * guided_begin -> guided_end, or abandon from any point once a plan is open.  None of them moves the helicopter's position
 * until guided_end; none of them adopts a pose, rotor or motion vector (the DLL's copies are mirrors the host validates).
 *   plan:    sends the facts and the group members, and returns what the DLL must look up (query)
 *   guide:   sends the lookups' results (own-sector structures in sector order, and each terrain sample's getter returns in
 *            call order) and returns the host's guidance (lift, waypoint, required height, acceleration, velocity demand)
 *   guided_begin: the host's motion vector and speed are applied (setter); the position is untouched
 *   guided_end:   the host's attitude, rotated flag, motion vector and position are applied; angles receives the host's
 *            heading, pitch and roll
 *   abandon: the step is dropped host-side and nothing is applied
 * Every call is fatal on a host failure or an invalid answer, like the other hosted steps.
 */
void campaign_movement_helicopter_plan (struct ENTITY *helicopter, const campaign_hc_plan *plan, campaign_hc_query *query);
void campaign_movement_helicopter_guide (struct ENTITY *helicopter, int structure_count, const campaign_hc_structure *structures, int terrain_sample_count, const campaign_hc_terrain_sample *terrain_samples, campaign_hc_guidance_answer *guidance);
void campaign_movement_helicopter_guided_begin (struct ENTITY *helicopter);
void campaign_movement_helicopter_guided_end (struct ENTITY *helicopter, const campaign_hc_steer *steer, float angles[3]);
void campaign_movement_helicopter_abandon (struct ENTITY *helicopter);

/*
 * Helicopter rotor and dying/crash physics.  The rotor state {main rpm, tail rpm, spin-up timer} is read from, and the
 * host's result is written back to, the helicopter's own fields by the wrappers (raw writes, never placements).
 *
 * rotor_set announces the entity's current rotor state as an explicit seed/reset (host adopts); every direct writer
 * of the rotor fields (setters, creation, unpack, wind-down) calls it afterwards.  It is a no-op for unhosted entities and
 * during a host-result application.  rotor_alive returns whether the spin-up gate held (the caller returns).  The DLL
 * decides `gate` (the original early-return condition's other operand); the rotor integration and timer stay host-side.
 *
 * rotor_idle is the idle wind-down (no follower parent) by the frame delta; the DLL no longer does that arithmetic.
 *
 * Death order (caller keeps the original order and the effects): rotor_down; contact(held) -> crash_begin/crash_end if
 * landed; else fall (position only) -> terrain sample -> fall_gravity -> contact(candidate) -> [server kill, splash
 * placement, effects, crater] -> water fire/explosion-trail smoke clears -> fresh post-kill CG query -> settle ->
 * [solid-ground crash sound] -> [buried: clear_helicopter_velocity, DEAD, smoke clears] -> random draws -> tumble.
 * contact uses the pre-kill CG for both calls; its candidate altitude is retained host-side and settle uses it for the bury
 * tests, with only the fresh CG setting the final solid/river heights.  fall fills `candidate`; fall_gravity applies the
 * vector (raw) and speed (setter).  settle returns whether buried (rotor and vector zeroed host-side; the caller then runs
 * clear_helicopter_velocity at the original point, which announces the zero vector) and updates `candidate`.  tumble applies
 * the attitude and the final position (through the setter with a local position; the raw position still holds the old pose).
 * crash_begin applies the attitude (old normal) and returns the candidate; crash_end (new normal) applies the pre-clear
 * vector and the position, then returns whether the slide stopped; the caller clears the velocity AFTER the position
 * setter (clear_helicopter_velocity announces the zero), otherwise crash_end has applied the speed.
 */
void campaign_movement_rotor_set (struct ENTITY *helicopter);
int campaign_movement_rotor_alive (struct ENTITY *helicopter, float lift, float power_output, float delta_seconds, int gate);
void campaign_movement_rotor_down (struct ENTITY *helicopter, float delta_seconds);
void campaign_movement_rotor_idle (struct ENTITY *helicopter, float delta_seconds);
void campaign_movement_fall (struct ENTITY *helicopter, float delta_seconds, struct VEC3D *candidate);
void campaign_movement_fall_gravity (struct ENTITY *helicopter, float gravity, float delta_seconds);
int campaign_movement_contact (struct ENTITY *helicopter, int use_candidate, float elevation, float cg_to_ground, float *altitude);
int campaign_movement_settle (struct ENTITY *helicopter, int terrain_class, float elevation, float cg_to_ground, float bury_limit, struct VEC3D *candidate);
void campaign_movement_tumble (struct ENTITY *helicopter, double heading_draw, double roll_draw);
void campaign_movement_crash_begin (struct ENTITY *helicopter, const struct VEC3D *old_normal, float pitch_offset, float bank_offset, float delta_seconds, struct VEC3D *candidate);
int campaign_movement_crash_end (struct ENTITY *helicopter, const struct VEC3D *new_normal, int forest, float elevation, float cg_to_ground, float delta_seconds);
void campaign_movement_impact (struct ENTITY *helicopter, float signed_speed);

/*
 * EEDCS-37 routed, anti-air and ship wreck/fall/sink.  The vehicle's vector is not retained host-side until the campaign
 * claims it with vehicle_vector_seed at a wreck/fall/sink entry: the host adopts the DLL copy only while it retains none, and
 * every later request (this one included) must match the retained vector bit-identically or the host refuses.  An explicit
 * PLACE never changes the retained vector or the staged fall candidate.  Order for a wreck: gv_death_candidate -> terrain
 * refresh at the candidate -> gv_death_decay (the DLL keeps the DEAD gate: routed only when damage_level <= 0, anti-air
 * always).  Order for falling: gv_fall_candidate -> terrain refresh -> gv_fall_gravity -> gv_fall_settle (flags and a STAGED
 * result only) -> the DLL effects (any POSITION setter there is an explicit PLACE) -> gv_fall_vector where the original wrote
 * the vector (sunk: before DEAD; ground: after the effect) -> DLL DEAD -> gv_commit (speed, then position, as the original
 * did last).  gv_impact when the original shoves.  Ship: ship_sink; the DLL keeps clear_ship_fires and the destroy test.
 * `anti_air` selects the wide literal.  Nothing is applied that the original did not write at the same point.
 */
/* EEDCS-37 fixed-wing dying and crash: the helicopter protocol on the fixed_wing part, with no rotor.  Death order: fw_fall ->
   DLL terrain sample -> fw_fall_gravity (vector raw, speed through the setter at once; returns the post-gravity speed the DLL
   keeps for the attitude block) -> fw_contact(candidate) with the PRE-kill CG -> [server kill, splash placement, effects] ->
   water smoke clears -> fresh post-kill CG -> fw_settle (returns buried, vector zeroed host-side) -> [buried:
   clear_fixed_wing_velocity, DEAD, smoke clears] -> [solid ground: crash sound] -> the roll draw (only if altitude > 12 and the
   vector has x or z) -> fw_death_attitude (attitude raw if eased, then the final position through the setter).
   Crash order: fw_crash_candidate -> DLL terrain sample -> fw_crash_velocity (vector raw; speed set unless stopped, then the DLL
   runs clear_fixed_wing_velocity) -> fw_crash_attitude (attitude raw, position through the setter; returns at rest) -> [at rest
   and not alive: DEAD, smoke clears] */
void campaign_movement_fw_fall (struct ENTITY *aircraft, float delta_seconds, struct VEC3D *candidate);
float campaign_movement_fw_fall_gravity (struct ENTITY *aircraft, float gravity, float delta_seconds);
int campaign_movement_fw_contact (struct ENTITY *aircraft, int use_candidate, float elevation, float cg_to_ground, float *altitude);
int campaign_movement_fw_settle (struct ENTITY *aircraft, int terrain_class, float elevation, float cg_to_ground, float bury_limit, struct VEC3D *candidate);
void campaign_movement_fw_death_attitude (struct ENTITY *aircraft, float altitude, float speed, int draw_made, double roll_draw, float delta_seconds);
void campaign_movement_fw_crash_candidate (struct ENTITY *aircraft, float delta_seconds, struct VEC3D *candidate);
int campaign_movement_fw_crash_velocity (struct ENTITY *aircraft, const struct VEC3D *new_normal, int forest, float elevation, float cg_to_ground, float delta_seconds);
int campaign_movement_fw_crash_attitude (struct ENTITY *aircraft, const struct VEC3D *new_normal, float pitch_offset, float bank_offset, float delta_seconds);
void campaign_movement_fw_impact (struct ENTITY *aircraft, float signed_speed);

void campaign_movement_vehicle_vector_seed (struct ENTITY *vehicle);
void campaign_movement_gv_death_candidate (struct ENTITY *vehicle, float delta_seconds, float *candidate_xz);
float campaign_movement_gv_death_decay (struct ENTITY *vehicle, const float *candidate_xz, float elevation, float cg_to_ground, float delta_seconds, int anti_air);
void campaign_movement_gv_fall_candidate (struct ENTITY *vehicle, float delta_seconds, struct VEC3D *candidate);
void campaign_movement_gv_fall_gravity (struct ENTITY *vehicle, float gravity, float delta_seconds);
int campaign_movement_gv_fall_settle (struct ENTITY *vehicle, const struct VEC3D *candidate, int water, float old_elevation, float elevation, float cg_to_ground, float bbox_diagonal, struct VEC3D *settled, float *speed);
void campaign_movement_gv_fall_vector (struct ENTITY *vehicle);
void campaign_movement_gv_commit (struct ENTITY *vehicle);
void campaign_movement_gv_impact (struct ENTITY *vehicle, float speed);
float campaign_movement_ship_sink (struct ENTITY *vehicle, float delta_seconds);

/*
 * One infantry step in two phases around the campaign's terrain sample.  `begin` returns the
 * horizontal advance and whether an `end` is pending (terrain_follow); the campaign samples the terrain
 * at `advanced` and calls `end` with the elevation, which applies the final pose.  Nothing is applied
 * to the entity before `end` (or before `begin` when no terrain follow is needed).
 */
int campaign_movement_person_begin (struct ENTITY *vehicle, float ps_vel, const struct VEC3D *waypoint, int terrain_follow, struct VEC3D *advanced);
void campaign_movement_person_end (struct ENTITY *vehicle, float elevation);

/*
 * EEDCS-40: a movement-dependent mobile follows its parent.  `parent_new` is the position the parent's setter notified (the
 * parent still stores the old one), `heading` and `velocity` what it sent.  Returns TRUE after applying the host's pose to the
 * dependent; FALSE (nothing applied) when it or its parent has no host identity, and the caller runs the original arithmetic.
 */
int campaign_movement_dependent_carry (struct ENTITY *child, struct ENTITY *parent, const struct VEC3D *parent_new, float heading, float velocity);

/*
 * EEDCS-44: an infantry member with a target link faces the target.  The campaign says which target; the host forms the heading from the person's
 * own position and the target's, turns its record and answers the attitude, which is stored on the entity.  Returns the heading.
 */
float campaign_movement_person_face (struct ENTITY *person, const struct VEC3D *target_position);

/* bracket the campaign's own writes of host results, which are not placements (nestable) */
void campaign_movement_apply_begin (void);
void campaign_movement_apply_end (void);

/* a campaign pose write: tell the host (no-op during the host-result application or for unhosted entities) */
void campaign_movement_place (struct ENTITY *vehicle);

/* the physical incarnation ends: tell the host to retire the identity (no-op without a host id); called before the heap free */
void campaign_movement_retire (const char *host_id);

/*
 * EEDCS-38 initial placement.  The campaign chose the formation point (x, z), the sampled terrain height (`position->y`, the
 * shared terrain service), the heading and the entity's centre-of-gravity offset above the ground; the host builds the physical
 * height and attitude and adopts the pose (the creation speed is the entity's own).  Returns TRUE and fills `placed` (the
 * host's position) and `angles` (heading, pitch, roll) when the entity is hosted; FALSE (nothing filled) otherwise, and the
 * caller then runs the original in-process arithmetic.  The caller applies `angles` through the original angle setter and
 * `placed` through the original position setter, both inside apply_begin/apply_end so neither re-issues a PLACE nor overwrites
 * the host's answer.  `aircraft_vector` (may be NULL) is an aircraft's creation motion vector.
 */
int campaign_movement_create_place (struct ENTITY *mobile, const struct VEC3D *position, float cg_to_ground, float heading, float fuselage_angle, const struct VEC3D *face_normal, const struct VEC3D *aircraft_vector, struct VEC3D *placed, float *angles);

/* EEDCS-38 physical expiry of a dead mobile.  The host owns the timer, its gate and the lifetime curve; the campaign supplies
   whether the wreck may expire at all.  The squared range to the nearest keysite is sampled by the campaign only when the host
   says its gate is open (answer 2) and is then supplied with CAMPAIGN_MOVEMENT_DEATH_RANGE_SUPPLIED.  The mirrored timer is
   written back.  Returns TRUE when the host says the wreck has expired (the caller then makes the explicit campaign removal
   request).  An entity that is not hosted is a fatal error, as for the other lifecycle steps */
#define CAMPAIGN_MOVEMENT_DEATH_NEVER_EXPIRES	1
#define CAMPAIGN_MOVEMENT_DEATH_OCCUPIED		2
#define CAMPAIGN_MOVEMENT_DEATH_RANGE_SUPPLIED	4
#define CAMPAIGN_MOVEMENT_EXPIRE_NEEDS_RANGE	2
int campaign_movement_expire (struct ENTITY *mobile, float *death_timer, float delta_seconds, int policy);

/* a ship's sink test: TRUE when the host's height for the sunk ship is below the sight of its shape (bounding box diagonal) */
int campaign_movement_ship_sunk (struct ENTITY *ship, float bbox_diagonal);

#endif

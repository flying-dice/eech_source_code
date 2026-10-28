/*
 * Host-owned physical weapon aiming and weapon-system readiness (EEDCS-44, D2).  The host owns where a launcher's weapon-system devices point and
 * whether they are ready: the pitch device's weapon, target and intercept vectors; the guided-weapon intercept prediction; the lead and ballistic
 * prediction an unguided weapon is aimed with and the table interpolation inside it; the offsets the devices are asked to reach and the
 * rate-limited step they take toward them; and the keyframed weapon-system-ready state, advanced every update and commanded open or closed.
 * Campaign code keeps what to aim at and whether to fire: the target link, the selected weapon, which package's devices are used, the ammunition and
 * damage state, whether the weapon system should be made ready, and the fire decision (the launch-angle threshold, burst and salvo timers, range,
 * altitude and line of sight).  The aiming functions in weapon.c, the readiness functions in ac_anim.c, vh_anim.c and en_anim.c and the fire code in
 * ac_wpn.c and vh_wpn.c ask the host at the points where the original computed and act on the answer.
 *
 * Every request is a pure question: the host keeps nothing between requests except the ballistic tables it generates at boot.
 *
 *   TABLE          one unguided weapon's physical figures (EEDCS-45).  The host SIMULATES the weapon's flight at every pitch and velocity test and keeps the
 *                  ballistic table that results; it owns the projectile simulation and the table.  Sent at boot, once per weapon, in database order, the
 *                  first and the last flagged (the host writes its diagnostic file across the pass).  The host answers DONE.  No table cell crosses the
 *                  boundary; the test-fixture build asks for an echo of what the host generated (the cells and one record per simulated flight).
 *   RANGE          one guided weapon's figures: the host runs its two range simulations (level and aimed) and writes its diagnostic file.  DONE.
 *   READY_UPDATE   the state of a launcher's weapon-system-ready keyframe, one update later: the state, the rate and the elapsed time.  The host answers
 *                  DONE with the advanced state and the phase the state was in before (0 closed, 1 opening, 2 open, 3 closing).
 *   READY_COMMAND  open or close the weapon-system-ready devices: the state and the command.  DONE with the state.
 *   REQUIRED       the offsets a package's devices are asked to reach to point at the launcher's intercept vector: the pitch device's unrotated attitude,
 *                  the intercept vector, the package's heading and pitch limits and whether the weapon is a ground-launched guided missile.  DONE with
 *                  the required heading and pitch.  Unbounded (no limits), the same question serves the HUD's offsets; unbounded with a tracking point and
 *                  the ballistic pitch, the player's cannon tracking.
 *   STEP           one rate-limited step of the devices toward the required offsets: the current and required offsets, the package's limits and turn
 *                  rates and the elapsed time.  DONE with the step and the new offsets.
 *   VECTORS        the pitch device's weapon vector, weapon-to-target vector and weapon-to-intercept-point vector and the range, for a launcher with a
 *                  weapon and a target (or a tracking point): the device's viewpoint, the weapon's figures, the target's position, point and motion.
 *                  The host answers DONE, or NEED_DRAW when the lead prediction reaches its random range jitter: the campaign takes ONE sample of its own
 *                  random stream (a raw integer in [0, 32767]; the generator and its seed never leave the campaign) and asks the same question again with it.
 *   LEAD           the lead and ballistic prediction alone (the decoy and fuze timer at weapon launch).  DONE or NEED_DRAW as above.  The time of flight is
 *                  a PARTIAL OUTPUT, as in the original: it comes back only when the host produced one (has_time_of_flight), and the campaign stores it into
 *                  the caller's variable only then.  A prediction that produces none (the target is off the map, the range is under a millimetre, or no ballistic
 *                  lookup succeeded) leaves the caller's variable as it was; the host invents no value, and a host that genuinely has a flight time may send it.
 *   BALLISTIC      one table interpolation (the AI salvo aim, the player's pipper).  DONE with the aiming pitch and the time of flight (a success always has both).
 *   INTERCEPT      the guided-weapon intercept prediction alone (the player's HUD).  DONE with the predicted point.
 *
 * The device viewpoint, the target point, the entity positions, speeds and motion vectors and the weapon and package figures that the campaign supplies are
 * transitional mirrors of what a host that owns the world already knows: the reference (EECH) host aims with them as the inputs of its own model, and
 * does not make the campaign the owner of a physical solver.  A host that owns its world may answer from its own state.  The host identities of the
 * source and the target are sent when the roster holds them.  No EECH pointer, entity index, list layout or seed crosses the boundary.  There is no
 * campaign fallback: a hosted engine with no aim callback stops at boot, and a request the host cannot answer stops the engine with the host's reason.
 * Unsupported: the triangulated range of the player's gunship (the campaign refuses to ask).
 *
 * The optional echo is an ordered list of the reference host's intermediate values (the points of its prediction), requested only by the
 * test-fixture build; the shipping module never asks for it.
 */
#ifndef MB_HOSTAIM_H
#define MB_HOSTAIM_H

#include <stddef.h>

#define CAMPAIGN_AIM_HOST_ID_MAX 128
#define CAMPAIGN_AIM_ERROR_MAX 256
#define CAMPAIGN_AIM_ECHO_MAX 512
#define CAMPAIGN_WEAPON_NAME_MAX 63

/* the shape of a ballistic table: five velocity tests (the muzzle velocity and four steps above it), 85 pitch indices, `max_range / 20 + 2` ranges, a (drop angle,
   flight time) pair of floats per cell */
#define CAMPAIGN_TABLE_VELOCITY_TESTS 5
#define CAMPAIGN_TABLE_PITCH_INDICES 85
#define CAMPAIGN_TABLE_RANGE_STEP 20.0
#define CAMPAIGN_TABLE_MAX_RANGE 200000.0

enum campaign_aim_mode
{
	CAMPAIGN_AIM_TABLE = 1,
	CAMPAIGN_AIM_READY_UPDATE,
	CAMPAIGN_AIM_READY_COMMAND,
	CAMPAIGN_AIM_REQUIRED,
	CAMPAIGN_AIM_STEP,
	CAMPAIGN_AIM_VECTORS,
	CAMPAIGN_AIM_LEAD,
	CAMPAIGN_AIM_BALLISTIC,
	CAMPAIGN_AIM_INTERCEPT,
	CAMPAIGN_AIM_RANGE,
	/* boot: one weapon's physical figures, for the host to keep (every weapon, not only those with a table or a range run) */
	CAMPAIGN_AIM_FIGURES,
	/* one frame of one weapon's flight (EEDCS-45): the weapon's state and the answers to the lookups so far in, the state it left as ordered actions out */
	CAMPAIGN_AIM_WEAPON_STEP,
	/* one weapon's birth (EEDCS-45): the launch kinematics of a weapon the launcher has just let go, in the same protocol as the step */
	CAMPAIGN_AIM_WEAPON_BIRTH,
	/* the rate-of-fire gate of a launcher releasing a weapon: how many rounds it lets go, or none */
	CAMPAIGN_AIM_RELEASE,
	/* one frame of a falling weapon's parachute (a crate's or an ejector seat's) */
	CAMPAIGN_AIM_ANIMATE,
	/* a decoy's update: its timer, and the restart drawn from the shared stream when it runs out (the step protocol) */
	CAMPAIGN_AIM_WEAPON_DECOY,
	/* a seeker's answer to a decoy becoming active: whether it goes for the decoy (the step protocol) */
	CAMPAIGN_AIM_WEAPON_SEDUCE,
	/* a seeker's answer to the decoy it was tracking expiring: whether it goes back to the decoy's launcher (the step protocol) */
	CAMPAIGN_AIM_WEAPON_REACQUIRE
};

enum campaign_aim_state
{
	CAMPAIGN_AIM_DONE = 1,
	CAMPAIGN_AIM_NEED_DRAW,
	/* WEAPON_STEP: the host needs the answers to the listed lookups and asks again with them */
	CAMPAIGN_AIM_NEED_LOOKUP
};

enum campaign_aim_event_kind
{
	CAMPAIGN_AIM_EVENT_INTERCEPT = 1,
	CAMPAIGN_AIM_EVENT_LEAD_PATH,
	CAMPAIGN_AIM_EVENT_LEAD_RANGE,
	CAMPAIGN_AIM_EVENT_LEAD_ITERATION,
	CAMPAIGN_AIM_EVENT_BALLISTIC,
	/* a vectors request that ran the lead prediction: the prediction's own outputs (the point, the angle and the time of flight) in f[0..4], the validity in d */
	CAMPAIGN_AIM_EVENT_LEAD_RESULT,
	/* TABLE echo: one simulated flight, in order: velocity test in a, pitch index in b, range marks filled in c, steps in d; f[0] the end time, f[1] the range values */
	CAMPAIGN_AIM_EVENT_TABLE_FLIGHT,
	/* RANGE echo: one range run, in order: zero_pitch in a, whether it ran in b, whether the diagnostic file was open in c; f[0..3] the end z, y, velocity and lifetime */
	CAMPAIGN_AIM_EVENT_RANGE_RESULT,
	/* WEAPON_STEP echo (the test fixture build only): the hooks of the original weapon_movement the host's step passed, in order.  The branch numbers they carry are the
	   fixture's own (mb_wpnfx.h) */
	CAMPAIGN_AIM_EVENT_STEP_PATH = 20,
	CAMPAIGN_AIM_EVENT_STEP_SUBMUNITIONS,
	CAMPAIGN_AIM_EVENT_STEP_TRACKING,
	CAMPAIGN_AIM_EVENT_STEP_GUIDANCE,
	CAMPAIGN_AIM_EVENT_STEP_GUIDANCE_SOURCE,
	CAMPAIGN_AIM_EVENT_STEP_TGT_BRANCH,
	CAMPAIGN_AIM_EVENT_STEP_TGT_END,
	CAMPAIGN_AIM_EVENT_STEP_INTERCEPT,
	CAMPAIGN_AIM_EVENT_STEP_GUIDED,
	CAMPAIGN_AIM_EVENT_STEP_PRECISION,
	CAMPAIGN_AIM_EVENT_STEP_GROUND,
	CAMPAIGN_AIM_EVENT_STEP_IMPACT,
	CAMPAIGN_AIM_EVENT_STEP_CONTACT_ANY,
	CAMPAIGN_AIM_EVENT_STEP_CONTACT_TREES,
	CAMPAIGN_AIM_EVENT_STEP_BIRTH_FUZE_BEGIN,
	CAMPAIGN_AIM_EVENT_STEP_BIRTH_FUZE_SOLVED,
	CAMPAIGN_AIM_EVENT_STEP_BIRTH_DISPERSION
};

/* WEAPON_STEP lookups: pure questions of the campaign's world the host's step asks, answered by value (the kind, a, b and f[0..5] are repeated in the answer, which fills
   f[6..11] as r0...).  The host's step asks them in rounds; the campaign answers each round and asks again. */
enum campaign_step_lookup
{
	CAMPAIGN_STEP_LOOKUP_VIEW_WEAPON = 1,		/* r0 the camera is viewing this weapon */
	CAMPAIGN_STEP_LOOKUP_LAUNCHER_EO,			/* r0..r2 the launcher's EO tracking point, r3 it is valid */
	CAMPAIGN_STEP_LOOKUP_LAUNCHER_INT,			/* a: 0 the launcher is not an AI, 1 its laser is on, 2 it has line of sight to its target.  r0 */
	CAMPAIGN_STEP_LOOKUP_LAUNCHER_TARGET,		/* the launcher's target.  r0 it has one, r1 it is the weapon's own target, r2..r4 its position */
	CAMPAIGN_STEP_LOOKUP_TARGET_INT,			/* a: which target (0 the weapon's own, 1 the launcher's); b: 0 mobile, 1 decoy type, 2 radar jammer on, 3 infra-red jammer on, 4 airborne aircraft, 5 not an AI.  r0 */
	CAMPAIGN_STEP_LOOKUP_TARGET_VEC,			/* a: which target; b: 0 position, 1 target point, 2 motion vector.  r0..r2 */
	CAMPAIGN_STEP_LOOKUP_WIND,					/* f0..f2 a point.  r0..r2 the wind velocity there */
	CAMPAIGN_STEP_LOOKUP_GROUND_PROBE,			/* f0..f2 a point inside the map area.  r0 it is below the ground */
	CAMPAIGN_STEP_LOOKUP_GROUND_IMPACT,			/* f0..f2 the old position, f3..f5 the new.  r0 the terrain class, r1..r3 the impact point */
	CAMPAIGN_STEP_LOOKUP_FACE_NORMAL,			/* f0, f2 x and z.  r0..r2 the terrain face normal there */
	CAMPAIGN_STEP_LOOKUP_ELEVATION,				/* f0, f2 x and z.  r0 the terrain elevation there */
	CAMPAIGN_STEP_LOOKUP_COLLISION,				/* a: which target is the weapon's now (-1 none); f0..f2 the old position, f3..f5 the new.  r0 something was hit, r1 it is not the weapon's target, r2 its type, r3..r5 where the segment meets it */
	CAMPAIGN_STEP_LOOKUP_TREES,					/* f0..f2 the old position, f3..f5 the new.  r0 a tree was hit, r1..r3 where the segment meets it */
	CAMPAIGN_STEP_LOOKUP_PARENT_VELOCITY,		/* a birth: the launcher's forward velocity.  r0 */
	CAMPAIGN_STEP_LOOKUP_FUZE_SOLVE,			/* a birth: f0..f2 the weapon's position, f3 the velocity it leaves with less its muzzle velocity.  r0 the launcher has a target and the lead prediction to it solved, r1 the time of flight */
	CAMPAIGN_STEP_LOOKUPS
};

/* WEAPON_STEP actions: what the step left, in the order the campaign applies it to its mirror of the weapon.  The echo events the original's hooks saw before an action
   are the first `d` of the echo list (the fixture build replays them before it applies the action). */
enum campaign_step_action
{
	CAMPAIGN_STEP_ACTION_STATE = 1,			/* a missile phase, b weapon on ground, c kill code; f0 velocity, f1..f3 motion vector, f4 weapon lifetime, f5 decoy timer */
	CAMPAIGN_STEP_ACTION_ATTITUDE,			/* f0..f8 the attitude matrix, row by row */
	CAMPAIGN_STEP_ACTION_SPAWN,				/* a the number of submunitions, f0 their parent velocity: they leave from the weapon as it is now */
	CAMPAIGN_STEP_ACTION_TACVIEW,			/* the weapon is written to the Tacview recording, if one is open */
	CAMPAIGN_STEP_ACTION_POSITION,			/* f0..f2 the weapon's position */
	CAMPAIGN_STEP_ACTION_TARGET_LINK,		/* a: 0 the weapon has no target now, 1 its target is the launcher's, 2 its target is what a collision lookup hit: b the target that lookup was about and f0..f5 its segment name it */
	CAMPAIGN_STEP_ACTION_SMOKE_EFFECT,		/* the killed-explosion effect, with the kill code at hit-land for its duration */
	CAMPAIGN_STEP_ACTION_BIRTH,				/* a birth: f0 the velocity the weapon leaves with, f1 its lifetime, f2 its decoy or fuze timer */
	CAMPAIGN_STEP_ACTION_DECOY,				/* a decoy's update: a the timer ran out and restarted (the campaign tells the seekers locked on its launcher), f0 the timer now */
	CAMPAIGN_STEP_ACTION_SEDUCE,			/* a seeker's answer to a decoy becoming active: a it was drawn to it, b it goes for it (the campaign makes the link) */
	CAMPAIGN_STEP_ACTION_REACQUIRE,			/* a seeker's answer to the decoy it was tracking expiring: a it goes back to the decoy's launcher (0 it has no target); the campaign unlinks it from the decoy either way */
	CAMPAIGN_STEP_ACTIONS
};

#define CAMPAIGN_STEP_ANSWERS_MAX 128
#define CAMPAIGN_STEP_DRAWS_MAX 8
#define CAMPAIGN_STEP_ACTIONS_MAX 32
#define CAMPAIGN_STEP_LOOKUPS_MAX 16
#define CAMPAIGN_STEP_ROUNDS_MAX 64

enum campaign_aim_lead_path
{
	CAMPAIGN_AIM_LEAD_PATH_TINY_RANGE = 1,
	CAMPAIGN_AIM_LEAD_PATH_OFF_MAP,
	CAMPAIGN_AIM_LEAD_PATH_STATIONARY,
	CAMPAIGN_AIM_LEAD_PATH_MOVING
};

typedef struct campaign_aim_event
{
	int kind;
	int a;
	int b;
	int c;
	int d;
	float f[12];
} campaign_aim_event;

/* One weapon's physical figures, mirrored from the campaign's weapon database (transitional: a host with a model of its own needs none of them).  The layout is the
   reference host's (eech-world csrc/wn_flight.c wf_figures). */
typedef struct campaign_weapon_figures
{
	char name[CAMPAIGN_WEAPON_NAME_MAX + 1];
	float start_weight;
	float final_weight;
	float max_range;
	float muzzle_velocity;
	float cruise_velocity;
	float g_max;
	float boost_time;
	float cruise_time;
	float inhibit_time;
	float drag_coefficient;
	float flightpath_deviation;
	float boost_power;
	float sustain_power;
	float sustain_time;
	float diameter;
	float max_altitude;
	int ignore_gravity;
	int flight_profile_or_self_destr;
	int parachute_type;
	unsigned int weapon_class;
	/* what the host's weapon step reads from the database besides the above (EEDCS-45): four floats, then the ints.  hellfire_ii and smoke_grenade are facts about the
	   weapon the original tested by sub type; submunition_count and submunition_velocity_factor are the database's burst_duration and acquire_parent_forward_velocity of
	   the next sub type, which a conventional-munitions weapon spawns (0 when there is none) */
	float cruise_time_max_error;
	float detonation_radius;
	float max_seeker_limit;
	float max_launch_angle_error;
	int survive_ground_impact;
	int in_flight_collision_check;
	int guidance_type;
	int aiming_type;
	int decoy_type;
	int warhead_type;
	int hellfire_ii;
	int smoke_grenade;
	int submunition_count;
	int submunition_velocity_factor;
	/* what the host's birth and release read besides the above (EEDCS-45): four floats, then whether the weapon takes its launcher's forward velocity */
	float muzzle_velocity_max_error;
	float circular_error_probable;
	float rate_of_fire;
	float inverse_rate_of_fire;
	int acquire_forward;
} campaign_weapon_figures;

/* WEAPON_STEP: the weapon as the campaign holds it.  The entity-local random stream never leaves the campaign: its first four outputs are all any one call of the
   original uses (raw integers in [0, 32767]) */
typedef struct campaign_weapon_state
{
	float position[3];
	float attitude[9];
	float motion_vector[3];
	float velocity;
	float weapon_lifetime;
	float decoy_timer;
	int missile_phase;
	int loal_mode;
	int weapon_on_ground;
	int parachute_status;
	int kill_code;
	int entity_samples[4];
	int has_launcher;
	int has_target;
	int server;
	/* a birth: the weapon is a submunition, and `velocity` is its parent weapon's */
	int submunition;
	/* a seduction: the decoy's weapon number, the difficulty coefficient, and the chaff, flare and smoke effectiveness settings (campaign settings, given as figures) */
	int other_weapon;
	float coefficient;
	float effectiveness[3];
} campaign_weapon_state;

typedef struct campaign_aim_step
{
	int mode;
	/* the test-fixture build asks for the reference host's intermediate values; the shipping module never does.  When it does, the campaign gives the host
	   room for them: the host writes at most echo_capacity events at echo_out and reports how many in the result (shipping requests carry no room) */
	int echo;
	campaign_aim_event *echo_out;
	int echo_capacity;
	/* the host identities of the launcher and its target (empty when the roster holds none) */
	char source_host_id[CAMPAIGN_AIM_HOST_ID_MAX + 1];
	char target_host_id[CAMPAIGN_AIM_HOST_ID_MAX + 1];

	/* TABLE, RANGE: the weapon and its figures; whether this is the first and the last weapon of the pass.  The fixture build's echo room for the table the host generated:
	   (drop angle, flight time) float pairs, velocity test by pitch index by range; the host writes at most echo_cell_capacity floats and reports how many */
	int weapon;
	campaign_weapon_figures figures;
	int pass_first;
	int pass_last;
	float *echo_cells;
	size_t echo_cell_capacity;

	/* READY_UPDATE, READY_COMMAND */
	float ready_state;
	float ready_rate;
	float delta_time;
	int ready_open;

	/* REQUIRED, STEP: the package's figures */
	float device_attitude[9];
	float intercept_vector[3];
	float heading_min;
	float heading_max;
	float pitch_min;
	float pitch_max;
	float heading_rate;
	float pitch_rate;
	int climb;
	float current_heading;
	float current_pitch;
	float required_heading;
	float required_pitch;
	/* REQUIRED without the package's limits (the player's cannon tracking and the HUD's offsets): the offsets come back as they are.  With has_tracking the request
	   carries no intercept vector: the offsets are those of a gun aimed at tracking_point from device_position, lowered by track_pitch */
	int unbounded;
	float track_pitch;

	/* VECTORS, LEAD, BALLISTIC, INTERCEPT: the weapon's figures and the launcher's and target's state */
	int aiming_type;
	float cruise_velocity;
	int acquire_forward;
	float source_velocity;
	float parent_velocity;
	float device_position[3];
	float target_position[3];
	float target_point[3];
	float target_motion_vector[3];
	int target_mobile;
	int has_tracking;
	float tracking_point[3];
	float map_min[3];
	float map_max[3];
	float comms_weapon_lag;
	int source_gunship;
	int has_weapon;
	/* the random sample the host asked for with NEED_DRAW */
	int has_draw;
	int draw_sample;
	/* BALLISTIC */
	float range;
	float height_diff_or_pitch;
	int simplified;
	int fixed_pitch;

	/* WEAPON_STEP (delta_time, map_min and map_max above are the frame's and the map's): the clock, the weapon, the lookups answered so far and the samples of the
	   campaign's shared stream given so far */
	unsigned int system_time;
	campaign_weapon_state weapon_state;
	int answer_count;
	campaign_aim_event answers[CAMPAIGN_STEP_ANSWERS_MAX];
	int draw_count;
	int draws[CAMPAIGN_STEP_DRAWS_MAX];

	/* RELEASE (delta_time and system_time above are the frame's and the clock): the launcher's last-shot clock, if it has one, and whether the launcher is not an AI */
	int release_has_timer;
	unsigned int release_last_shot;
	int release_not_ai;

	/* ANIMATE: the weapon's parachute, whether it rests on the ground, its lifetime and its vertical motion */
	int animate_status;
	int animate_on_ground;
	float animate_lifetime;
	float animate_motion_y;
} campaign_aim_step;

typedef struct campaign_aim_result
{
	int state;
	/* VECTORS: the vectors are valid; LEAD, BALLISTIC: the prediction succeeded */
	int result;
	/* READY_UPDATE, READY_COMMAND */
	float ready_state;
	int phase;
	/* REQUIRED */
	float required_heading;
	float required_pitch;
	/* STEP */
	float delta_heading;
	float delta_pitch;
	float new_heading;
	float new_pitch;
	/* VECTORS */
	float weapon_vector[3];
	float target_vector[3];
	float intercept_vector[3];
	float intercept_range;
	/* LEAD, INTERCEPT: the predicted point; LEAD, BALLISTIC: the aiming angle and the time of flight */
	float intercept_point[3];
	float angle;
	float time_of_flight;
	/* whether the host produced a time of flight: LEAD, any outcome; BALLISTIC, exactly on success; never otherwise.  When it is 0 the value above is 0 and means nothing */
	int has_time_of_flight;
	/* optional trace echo: the reference host's intermediate values, read only by the test fixture build */
	int has_echo;
	int echo_count;
	const campaign_aim_event *echo;
	/* TABLE echo: the number of floats the host wrote at echo_cells */
	size_t echo_cell_count;
	/* WEAPON_STEP: the lookups the host still needs (NEED_LOOKUP), or the actions it left (DONE) */
	int lookup_count;
	campaign_aim_event lookups[CAMPAIGN_STEP_LOOKUPS_MAX];
	int action_count;
	campaign_aim_event actions[CAMPAIGN_STEP_ACTIONS_MAX];
	/* RELEASE: the launcher lets nothing go; the rounds it lets go; whether the gate wrote the last-shot clock (it does even when it then refuses) and what it wrote */
	int release_refuse;
	int release_burst_size;
	int release_write_last_shot;
	unsigned int release_last_shot;
	/* ANIMATE: the parachute's state, and the weapon's lifetime when the animation reset it */
	int animate_status;
	int animate_write_lifetime;
	float animate_lifetime;
} campaign_aim_result;

typedef int (*campaign_aim_fn)
(
	const campaign_aim_step *step,
	campaign_aim_result *result,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_aim_host
{
	campaign_aim_fn aim;
	void *user;
} campaign_aim_host;

struct ENTITY;
struct VEC3D;

/* WEAPON_STEP: what each collision lookup of one step found, by the question it was asked (the target it was about and the segment).  The questions are public; what
   they found is the campaign's own entity, which never crosses the boundary: a target-link action names the question, and the campaign finds its own answer.  A host
   may ask several collision questions in a step and choose among the answers. */
#define CAMPAIGN_STEP_HITS_MAX 16

typedef struct campaign_step_hit
{
	int which;
	float segment[6];
	struct ENTITY *entity;
} campaign_step_hit;

typedef struct campaign_step_hits
{
	int count;
	campaign_step_hit hit[CAMPAIGN_STEP_HITS_MAX];
} campaign_step_hits;

void campaign_step_hits_clear (campaign_step_hits *hits);

/* records what the collision question (which, segment) found (a miss is a NULL entity); a question answered twice, or more than the room, stops the engine */
void campaign_step_hits_add (campaign_step_hits *hits, int which, const float *segment, struct ENTITY *entity);

/* what the collision question (which, segment) found: whether it was asked and answered, and the entity it hit (NULL for a miss) */
int campaign_step_hits_find (const campaign_step_hits *hits, int which, const float *segment, struct ENTITY **entity);

void campaign_aim_set_host (const campaign_aim_host *host);

int campaign_aim_available (void);

/* Fills the host identity fields of a request from the roster. */
void campaign_aim_identify (campaign_aim_step *step, struct ENTITY *source, struct ENTITY *target);

/* Asks the host.  Validates the answer by mode and stops the engine on a missing callback, a failing host or a malformed or illegal answer.  A NEED_DRAW
   is answered here when the request allows a draw: the campaign takes one sample of its own stream and asks again. */
campaign_aim_result campaign_aim_ask (campaign_aim_step *step, int allow_draw);

/* Hands the host one weapon's figures at boot: mode is CAMPAIGN_AIM_TABLE (an unguided weapon: the host generates its ballistic table) or CAMPAIGN_AIM_RANGE (a guided
   weapon: the host runs its range simulations).  Asks for an echo (the test fixture build) when echo_cells is not NULL, and returns the result (its echo is valid until the
   next request). */
campaign_aim_result campaign_aim_generate (int mode, int weapon, const campaign_weapon_figures *figures, int pass_first, int pass_last, float *echo_cells, size_t echo_cell_capacity);

/* Hands the host one weapon's figures to keep (CAMPAIGN_AIM_FIGURES). */
void campaign_aim_supply_figures (int weapon, const campaign_weapon_figures *figures);

/* Answers one lookup of a weapon step: copies the question into `answer` and fills f[6..11].  Stops the engine on a lookup the campaign cannot answer. */
typedef void (*campaign_step_answer_fn) (const campaign_aim_event *lookup, campaign_aim_event *answer, void *context);

/* The rate-of-fire gate of a launcher releasing a weapon (CAMPAIGN_AIM_RELEASE). */
campaign_aim_result campaign_aim_release (int weapon, int has_timer, unsigned int last_shot, int not_ai);

/* One frame of a falling weapon's parachute (CAMPAIGN_AIM_ANIMATE). */
campaign_aim_result campaign_aim_animate (int weapon, int status, int on_ground, float lifetime, float motion_y);

/* Runs one weapon's step, birth, decoy update or seduction (the request's mode says which) to its end: asks the host, answers the lookups it needs and takes the draws it asks for, and asks again, until it is done (at most
   CAMPAIGN_STEP_ROUNDS_MAX rounds).  `step` carries the weapon and is the campaign's to fill; the result holds the actions (and the echo the fixture build asked for). */
campaign_aim_result campaign_aim_weapon_step (campaign_aim_step *step, campaign_step_answer_fn answer, void *context);

#endif

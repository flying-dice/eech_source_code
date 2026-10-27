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
	CAMPAIGN_AIM_RANGE
};

enum campaign_aim_state
{
	CAMPAIGN_AIM_DONE = 1,
	CAMPAIGN_AIM_NEED_DRAW
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
	CAMPAIGN_AIM_EVENT_RANGE_RESULT
};

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
} campaign_weapon_figures;

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

#endif

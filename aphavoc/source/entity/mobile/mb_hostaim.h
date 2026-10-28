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
 * FORECAST (EEDCS-46, D2) is the first physical request of combat resolution:
 *
 *   FORECAST       the damage a weapon is CAPABLE of against a target, asked with no weapon in flight: the forecast the target selection makes (wn_tgt.c) to decide whether a
 *                  package can hurt a target at all and to weigh packages against each other.  The campaign sends, by value, the weapon, its damage figures, the target's
 *                  armour level, whether the target is a player and the player's damage setting, and the two arguments the original function takes (the damage modifier
 *                  and the velocity).  The host answers the capability (an integer, never negative).  DONE only: the forecast consumes NO sample of the shared random stream
 *                  (a NEED_DRAW or a NEED_LOOKUP is refused), as in the original.  The campaign keeps which weapons and targets are weighed, the salvo and rate-of-fire
 *                  multiplier and what the number is compared with; the REAL weapon's capability (its draw, its velocity, the aspect armour of the target and the multiplayer
 *                  multiplier) is not part of this request and stays in the campaign.  The damage figures are those of the weapon's own definition, or, for a
 *                  conventional-munitions weapon (which carries its damage in the munition it scatters), those of the submunition the database names for it: the
 *                  campaign resolves that definition relationship and sends the munition's figures; the host applies the conventional-munitions arithmetic.  The test fixture
 *                  build asks for an echo (one event) of the reference host's intermediate values, which it replays into its observer in the order the original produced.
 *
 *   DAMAGE         the damage of ONE HIT that reached a damage handler (EEDCS-46, real weapon): whether the player's invulnerability stops it, the weapon's capability against
 *                  the target (the original's weapon_damage_capability for a real weapon), the armour the weapon meets (the original's get_local_entity_armour_thickness: the
 *                  table level, and for a vehicle or an aircraft the aspect the weapon arrives at), the player's damage setting and the multiplayer multiplier, whether the capability
 *                  is applied to the health and the health it leaves (the subtraction and its clamp at zero), and the critical-state classification of the target before and after
 *                  (the original's vehicle_critically_damaged and aircraft_critically_damaged; a person is never critical and a fixed structure has none).  The campaign sends
 *                  everything by value (nothing of the campaign's entities, no identity, index, pointer or seed) and keeps the sounds a hit makes, the setting of the health and
 *                  its side effects, the credit and the kill, the player-setting table, the multiplayer multiplier's record and the debug display.  The arithmetic takes ONE
 *                  sample of the campaign's shared random stream (the original's frand1 ()): the host answers NEED_DRAW once, after the invulnerability test (an invulnerable
 *                  player's hit takes none), and is asked again with the raw integer in [0, 32767] from which it forms the double with the macro's own expression.  A
 *                  second request for a sample, a lookup, an answer that contradicts the request (an invulnerable target hit, health written under the cheat guard, health
 *                  raised, a classification the target's model cannot have) is refused.  A host with a model of its own answers from it and needs none of EECH's arithmetic.
 *
 *   HEALTH         (see campaign_health_request below.)
 *
 *   IMPACT         (EEDCS-46) what a weapon's death does to the world: the dispatch on its kill code.  The campaign sends the kill code and the facts the dispatch reads (whether the
 *                  weapon has a target and it is alive; the impact position).  The host answers the PLAN in the original's order: whether a ground crater is made (the host
 *                  asks for the terrain fact it needs, NEED_LOOKUP, and is asked again with it: the original read it for a land hit only), whether the target is first sent a direct
 *                  hit, how many area blasts follow (0, 1, or the land hit's deliberate two) and whether they leave the target out; or that the code is not one the dispatch knows
 *                  (the campaign then stops with the original's message).  The campaign keeps the effects (the crater entity, the explosion and collision effects chosen by the
 *                  kill code) and sends the notifications.
 *
 *   BLAST          (EEDCS-46) one area blast: which objects it reaches, and with what.  The HOST owns the damage radius, the sector rectangle, the order of the walk, which objects
 *                  are candidates, the range and the strict in-range test, the exclusion of the target, the damage modifier and the rocket interception (the warhead's threshold).  The
 *                  campaign sends the explosion (position, the weapon's damage capability and warhead type) and the map's sector figures, and supplies the facts of the world the host
 *                  asks for as WINDOWS: up to CAMPAIGN_BLAST_WINDOW_MAX objects of one sector's list, in list order, from the start of a sector or after the object last offered (a
 *                  declared transitional mirror: a host that owns its world answers from its own objects).  Every object in a window is named by a HANDLE the campaign gives it for
 *                  this blast only (an opaque, positive, per-blast number: no pointer, no index, no list layout, no persistent identity), which is how a fixed structure, which the
 *                  roster does not cover, is reported.  The host answers: that it needs a window (NEED_LOOKUP: sector and after-handle), or DONE with the NEXT object the blast reaches
 *                  and what the campaign must do (send it a collision with the modifier, or destroy a rocket), or DONE with nothing more.  The campaign applies each answer BEFORE the
 *                  walk continues and asks again after the object it applied, from the list as it is then (the original read the successor of an object after notifying it, and a
 *                  notification can kill objects and alter lists): the campaign never freezes the candidate set.  A blast takes no sample.  The campaign keeps what a collision does to
 *                  the object (accounting, credit, the D1 consequences), setting a destroyed rocket's kill code, and the effects.
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
	CAMPAIGN_AIM_WEAPON_REACQUIRE,
	/* the damage a weapon is capable of against a target, asked with no weapon in flight (EEDCS-46): a pure question, DONE only */
	CAMPAIGN_AIM_FORECAST,
	/* the damage of one hit that reached a damage handler (EEDCS-46): NEED_DRAW once, then DONE */
	CAMPAIGN_AIM_DAMAGE,
	/* the physical health facts of a target that are not a hit (EEDCS-46): its critical class, the thresholds a change of health crosses, the timed fire and bleeding rolls
	   and the eligibility of a hit (see campaign_health_request) */
	CAMPAIGN_AIM_HEALTH,
	/* what a weapon's death does to the world: the plan of the dispatch on its kill code (EEDCS-46); NEED_LOOKUP asks for the terrain fact once */
	CAMPAIGN_AIM_IMPACT,
	/* one area blast (EEDCS-46): the host walks the world through windows the campaign supplies and names the next object it reaches; NEED_LOOKUP asks for a window */
	CAMPAIGN_AIM_BLAST
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
	/* FORECAST echo (the test fixture build only): the original's observer hooks of the forecast in one event.  a the arithmetic path that ran (0 none, 1 the weapon pierces the
	   armour, 2 high-explosive splash), b the capability before the player's damage setting, c the warhead type the arithmetic used, d the final capability; f[0] the damage
	   modifier as it stood at the end (the splash path rewrites it), f[1] the penetration capability, f[2] the armour thickness, f[3] the random modifier (1: a forecast has none) */
	CAMPAIGN_AIM_EVENT_FORECAST,
	/* DAMAGE echo (the test fixture build only): a path in a, the capability before the player's setting in b, the warhead in c and the final capability in d, as the forecast's; a hit
	   stopped by the player's invulnerability echoes none.  The four intermediate floats the original's observer saw (the damage modifier as it stood at the end, the penetration
	   capability, the armour thickness and the random modifier) are NOT always finite: a weapon at the target's own position has no direction and its aspect armour is not a
	   number (the hit then does nothing).  The exchange carries finite floats only, so each travels as its raw 32-bit pattern, losslessly, as two exact 16-bit halves (high then
	   low) in f[4..11], value by value in the order above; f[0..3] repeat the finite ones (0 for the others) for a reader.  campaign_aim_damage_echo_float rebuilds them. */
	CAMPAIGN_AIM_EVENT_DAMAGE,
	/* BLAST echo (the test fixture build only): one event per object the walk examined as a candidate (alive, with a position, of a kind it examines), in the order examined, in the
	   window the request carried: a the object's index in the window, b the flags (bit 0 the range is strictly inside the radius, bit 1 the object is a rocket the blast destroys),
	   f[0] the range, f[1] the radius, f[2] the damage modifier (0 when not in range).  The fixture replays them into its observer in the original's order. */
	CAMPAIGN_AIM_EVENT_BLAST,
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

/* FORECAST: the damage figures of the weapon the forecast is about, by value.  `weapon_warhead` is the warhead type of the weapon's own definition.  The other four are the
   figures the forecast is made from: those of the weapon's own definition, or, when the weapon is a conventional-munitions weapon, those of the munition the database names
   as its submunition (the campaign resolves that relationship and the host applies the conventional-munitions arithmetic: the munition's damage scaled by its burst duration
   over three).  `burst_duration` is the database's float, unchanged. */
typedef struct campaign_damage_figures
{
	int weapon_warhead;
	int warhead;
	int damage_capability;
	int armor_penetration_capability;
	float burst_duration;
} campaign_damage_figures;

/* DAMAGE: how the armour of a target is modelled, and how its critical state is classified */
enum campaign_damage_armour
{
	CAMPAIGN_DAMAGE_ARMOUR_PLAIN = 0,		/* the table level only (a fixed structure) */
	CAMPAIGN_DAMAGE_ARMOUR_VEHICLE,			/* the level scaled by the aspect, from the vehicle definition's front, rear and side armour */
	CAMPAIGN_DAMAGE_ARMOUR_AIRCRAFT			/* the level scaled by the aspect, from the aircraft armour figures */
};

enum campaign_damage_critical
{
	CAMPAIGN_DAMAGE_CRITICAL_NONE = 0,		/* no classification (a fixed structure): both classes are 0 */
	CAMPAIGN_DAMAGE_CRITICAL_VEHICLE,		/* 0, or 1 (critical) below 15 percent of the initial level */
	CAMPAIGN_DAMAGE_CRITICAL_PERSON,		/* never critical: both classes are 0 */
	CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT		/* 0, 1 (mortal) below 30 percent, or 2 (critical) below 5 percent */
};

/* DAMAGE: one hit, by value.  `figures` are the weapon's damage figures as the forecast's; `weapon_speed` is the weapon's velocity and `weapon_cruise_velocity` its own
   definition's.  The armour: the target's table level, whether the launcher is a player at the easiest difficulty (the armour is then 0.1), the model, and for a vehicle
   the definition's front, rear and side armour; the two positions and the attitude matrix (row by row) are read only for a vehicle or an aircraft.  The settings: whether the
   target is not an AI and, then, the player's damage setting (`player_modifier_valid` is 0 when the campaign found none for the difficulty, which the original stopped on
   only when it needed it); whether the weapon has a launcher and the multiplayer multiplier then; the player's invulnerability (read only for a player); the cheat guard
   (the debug option and the session's cheats, which stop any health from being written).  The health: the level, the initial level and the classification. */
typedef struct campaign_damage_request
{
	campaign_damage_figures figures;
	float weapon_speed;
	float weapon_cruise_velocity;
	float damage_modifier;
	int armour_level;
	int armour_easy;
	int armour_model;
	float armour_front;
	float armour_rear;
	float armour_side;
	float weapon_position[3];
	float target_position[3];
	float target_attitude[9];
	int target_not_ai;
	int player_modifier_valid;
	float player_modifier;
	int has_launcher;
	int multiplier;
	int invulnerable;
	int cheat_guard;
	int damage_level;
	int initial_damage_level;
	int critical_model;
} campaign_damage_request;

/* DAMAGE answer: `exited` the invulnerability stopped the hit (nothing else is meaningful but the unchanged health); `capability` the hit's capability (any integer: a hit of zero
   or less does nothing); `written` the health is set to `new_level` (and is not otherwise); the classes of the target's critical state before and after; `bad_modifier` the
   target is a player and the host needed a damage setting the campaign could not resolve. */
typedef struct campaign_damage_result
{
	int exited;
	int capability;
	int written;
	int new_level;
	int critical_before;
	int critical_after;
	int bad_modifier;
} campaign_damage_result;

/* HEALTH (EEDCS-46): the physical facts about a target's health that are not a hit, by value.  `op` says which.
 *
 *   CLASSIFY      the critical class of a health: the original's vehicle_critically_damaged (0 or 1; a person never) and aircraft_critically_damaged (0, 1 mortal, 2 critical), from the
 *                 level and the initial level (critical_model as for DAMAGE).  Every reader of the class asks: the damage handlers, the updates' mobility gates, the aircraft's
 *                 damaged smoke and speech.  A pure question.
 *   ASSESS        what a CHANGE of health (old_damage_level to damage_level) crosses, from the original's float percentages: for a vehicle that can burn (explosive or flammable) whether
 *                 it fell below 15 percent (`crossed`: the campaign then lights the fire, stops the vehicle and sleeps it); for an aircraft whether it fell below 5 or 30 percent (`crossed`),
 *                 whether it recovered above 30 (`recovered`) and whether the health is within the range that ejects an AI crew (`eject`: below 3.75 percent and above ten points; the
 *                 campaign adds that the aircraft is airborne and an AI).
 *   TIMED         the frame's timed health roll, for the six updates (a fixed structure has no such update, so none is asked for one).  The host owns the physical eligibility, the roll
 *                 and the health: it answers whether the target is eligible, asks for the campaign's ONE sample (NEED_DRAW) if its model rolls, and answers the roll, the health a hit
 *                 leaves and whether it is fatal.  The campaign sends the facts (the health, the initial level, whether the vehicle can burn, the aircraft's class from the classification
 *                 asked earlier in the frame, the frame time) and validates only the shape: one sample at most, and an answer that is consistent and safe to apply.  The REFERENCE host
 *                 is eligible for a vehicle with health that can burn in the critical class, a person never (a person is never critical) and an aircraft in a class above none, takes
 *                 no sample for any other, and takes it whatever the odds for an eligible one; a host with a model of its own may roll for any of them.  The answer also carries the class of the health after the roll (`critical_class`), which the update's
 *                 next gate uses.  The campaign keeps the setting of the health, the observer's hooks, the kill and what follows.
 *   HIT_ELIGIBLE  whether a hit that is the target's own (`self_hit`) may hurt it: an AI helicopter ignores its own hits (the original's fix against helicopters killing themselves with
 *                 missiles fired into the ground).  Asked only for a self hit; the campaign returns from the handler when the answer is no.
 */
enum campaign_health_op
{
	CAMPAIGN_HEALTH_CLASSIFY = 1,
	CAMPAIGN_HEALTH_ASSESS,
	CAMPAIGN_HEALTH_TIMED,
	CAMPAIGN_HEALTH_HIT_ELIGIBLE
};

typedef struct campaign_health_request
{
	int op;
	int critical_model;
	int damage_level;
	int initial_damage_level;
	int old_damage_level;
	int explosive;
	int flammable;
	float delta_time;
	int aircraft_class;
	int self_hit;
	int target_not_ai;
} campaign_health_request;

typedef struct campaign_health_result
{
	int critical_class;
	int crossed;
	int recovered;
	int eject;
	int eligible;
	int rolled;
	int hit;
	int new_level;
	int fatal;
} campaign_health_result;

/* IMPACT (EEDCS-46): the dispatch on a weapon's kill code, by value.  `kill_code` is the weapon's (the campaign's enum, weapon.h); `has_target` and `target_alive` are the campaign's
   facts about the weapon's target link (`target_alive` is 0 without one); `terrain_known` says the campaign has read the terrain at the impact position, and then
   `terrain_draws_crater` is whether that terrain takes a crater.  The position is the weapon's. */
typedef struct campaign_impact_request
{
	int kill_code;
	int has_target;
	int target_alive;
	int terrain_known;
	int terrain_draws_crater;
	float position[3];
} campaign_impact_request;

/* IMPACT answer (state DONE): the plan in the original's order, crater first, then the direct hit, then `blasts` blasts.  `need_terrain` (state NEED_LOOKUP): the host needs the terrain
   fact at the impact position and is asked again with it.  `invalid`: the kill code is not one the dispatch knows (nothing else is planned). */
typedef struct campaign_impact_result
{
	int need_terrain;
	int invalid;
	int crater;
	int direct_hit;
	int blasts;
	int exclude_target;
} campaign_impact_result;

#define CAMPAIGN_BLAST_WINDOW_MAX 16

/* BLAST: one object of a sector's list, offered to the host.  `handle` names it for this blast only (positive, unique within the blast).  `type` is the campaign's entity type
   (entity_types, en_types.h); `readable` says the campaign can read the object's state (the kinds a blast examines): when it is 0 `alive`, `has_position`, `rocket` and the position
   are 0.  `rocket` is the weapon's rocket flag (read for a weapon only), `excluded` says the object is the blast's excluded target (the campaign's identity fact). */
typedef struct campaign_blast_entry
{
	int handle;
	int type;
	int readable;
	int alive;
	int has_position;
	int rocket;
	int excluded;
	float position[3];
} campaign_blast_entry;

/* BLAST: one area blast, by value.  The explosion: its position, the weapon's damage capability and warhead type, and whether the blast leaves a target out.  The map's sector
   figures (the side length of a sector and the sector rectangle the map holds).  The window the host last asked for, once it has been supplied: the sector it is in, the handle it
   follows (0 for the start of the sector's list), how many objects it holds and whether the sector's list ends with it.  The first request carries no window. */
typedef struct campaign_blast_request
{
	float position[3];
	int damage_capability;
	int warhead_type;
	int has_target;
	int sector_side_length;
	int min_x_sector;
	int max_x_sector;
	int min_z_sector;
	int max_z_sector;
	int window_valid;
	int window_sx;
	int window_sz;
	int window_after;
	int window_count;
	int window_end;
	campaign_blast_entry window[CAMPAIGN_BLAST_WINDOW_MAX];
} campaign_blast_request;

enum campaign_blast_action
{
	CAMPAIGN_BLAST_DONE = 0,		/* the walk is over */
	CAMPAIGN_BLAST_COLLISION,		/* send the object a collision with `modifier` */
	CAMPAIGN_BLAST_INTERCEPT		/* destroy a rocket: set its kill code to overshot-target */
};

/* BLAST answer.  `radius` is the damage radius the host computed (reported with every answer; it must not change within a blast).  State NEED_LOOKUP: the host needs the window at
   (need_sx, need_sz) following `need_after` (0: from the start of the sector's list).  State DONE: `action`, and for an action the `handle` of the object in the request's window and
   the `modifier` (a fraction of the radius, in [0, 1]). */
typedef struct campaign_blast_result
{
	float radius;
	int need_sx;
	int need_sz;
	int need_after;
	int action;
	int handle;
	float modifier;
} campaign_blast_result;

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

	/* FORECAST (`weapon` is the weapon sub type): its damage figures, the target's armour level (a table value, never negative), whether the target is not an AI and, then, the
	   player's damage setting (1 and unused for an AI), and the two arguments of the original function: the damage modifier and the velocity */
	campaign_damage_figures forecast_figures;
	int forecast_armour_level;
	int forecast_target_not_ai;
	float forecast_player_modifier;
	float forecast_damage_modifier;
	float forecast_velocity;

	/* DAMAGE (`weapon` is the weapon sub type; the sample is has_draw and draw_sample above) */
	campaign_damage_request damage;

	/* HEALTH (the sample of a TIMED roll is has_draw and draw_sample above) */
	campaign_health_request health;

	/* IMPACT, BLAST (no sample is taken for either) */
	campaign_impact_request impact;
	campaign_blast_request blast;
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
	/* FORECAST: the capability, never negative */
	int forecast_capability;
	/* DAMAGE */
	campaign_damage_result damage;
	/* HEALTH */
	campaign_health_result health;
	/* IMPACT, BLAST */
	campaign_impact_result impact;
	campaign_blast_result blast;
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

/* The damage forecast (CAMPAIGN_AIM_FORECAST): the capability of `weapon` (figures as campaign_damage_figures says) against a target of this armour level, a player or not (with
   the player's damage setting), for these two arguments of the original function.  A pure question: no draw is taken, none may be asked for.  `echo` asks the reference host for
   the observer's echo (the test fixture build); the result's echo is valid until the next request. */
campaign_aim_result campaign_aim_forecast (int weapon, const campaign_damage_figures *figures, int armour_level, int target_not_ai, float player_modifier, float damage_modifier, float velocity, int echo);

/* The damage of one hit (CAMPAIGN_AIM_DAMAGE) of `weapon` (a sub type): the campaign takes the one sample of its stream the host asks for.  `echo` asks the reference host for the
   observer's echo (the test fixture build); the result's echo is valid until the next request. */
campaign_aim_result campaign_aim_damage (int weapon, const campaign_damage_request *request, int echo);

/* A health fact (CAMPAIGN_AIM_HEALTH): the campaign takes the one sample of its stream that a TIMED roll asks for. */
campaign_aim_result campaign_aim_health (const campaign_health_request *request);

/* The dispatch of a weapon's death (CAMPAIGN_AIM_IMPACT): the plan, with the terrain fact supplied when the host asks for it.  `terrain` is called at most once, with the impact
   position, and returns whether the terrain there takes a crater; the original read it only for a land hit.  A pure question: no sample is taken. */
typedef int (*campaign_impact_terrain_fn) (void *context);

campaign_impact_result campaign_aim_impact (int kill_code, int has_target, int target_alive, const float *position, campaign_impact_terrain_fn terrain, void *context);

/* One area blast (CAMPAIGN_AIM_BLAST), driven by the campaign for the host: `window` fills the request's window as the host asks (the sector, the handle it follows or 0, and the
   room), returning the objects in list order and whether the list ends with them; `apply` is given each object the host reaches, in order, and acts on it BEFORE the next question.
   `begin(radius)` is called once, with the radius the host computed, before the first object. */
struct campaign_blast_walk
{
	/* the map's sector figures, given to the host as values (the original read them from the world map) */
	int sector_side_length;
	int min_x_sector;
	int max_x_sector;
	int min_z_sector;
	int max_z_sector;
	/* the test fixture build asks the reference host for its echo of the candidates it examined (the shipping module never does) */
	int echo;
	/* fills window[0..max) with the objects of sector (sx, sz) following the object `after` (NULL: from the start of the list); returns how many, and sets *end when the list ends with them */
	int (*window) (void *context, int sx, int sz, const void *after, campaign_blast_entry *entries, void **objects, int max, int *end);
	/* the object the host reached: act on it (a collision with the modifier, or a rocket's destruction); the walk then continues after that object, from the list as it is */
	void (*apply) (void *context, void *object, int action, float modifier);
	/* the radius the host computed, once, before the first object (the observer's blast begins here) */
	void (*begin) (void *context, float radius);
	/* the echo of the candidates the host examined in the window it was given (the test fixture build only); `objects` are the window's objects, in order */
	void (*examined) (void *context, const campaign_aim_result *result, void *const *objects, int count);
	/* the walk is over */
	void (*end) (void *context);
	void *context;
};

void campaign_aim_blast (const float *position, int damage_capability, int warhead_type, int has_target, const struct campaign_blast_walk *walk);

/* The float `index` (0 the damage modifier, 1 the penetration capability, 2 the armour thickness, 3 the random modifier) of a DAMAGE echo event, rebuilt from its raw bits. */
float campaign_aim_damage_echo_float (const campaign_aim_event *event, int index);

/* One frame of a falling weapon's parachute (CAMPAIGN_AIM_ANIMATE). */
campaign_aim_result campaign_aim_animate (int weapon, int status, int on_ground, float lifetime, float motion_y);

/* Runs one weapon's step, birth, decoy update or seduction (the request's mode says which) to its end: asks the host, answers the lookups it needs and takes the draws it asks for, and asks again, until it is done (at most
   CAMPAIGN_STEP_ROUNDS_MAX rounds).  `step` carries the weapon and is the campaign's to fill; the result holds the actions (and the echo the fixture build asked for). */
campaign_aim_result campaign_aim_weapon_step (campaign_aim_step *step, campaign_step_answer_fn answer, void *context);

#endif
